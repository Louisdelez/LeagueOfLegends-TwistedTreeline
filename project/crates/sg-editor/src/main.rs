mod state;

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, AccumulatedMouseScroll};
use bevy::ecs::message::MessageReader;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiTextureHandle, EguiGlobalSettings};
use lucide_icons::{self, Icon};
use rand::Rng;
use noise::{NoiseFn, Perlin};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use state::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.07, 0.07, 0.09)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "League of Legends Studio".into(),
                resolution: bevy::window::WindowResolution::new(1920, 1080),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        })
        .set(bevy::render::RenderPlugin {
            synchronous_pipeline_compilation: false,
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(EguiGlobalSettings { enable_absorb_bevy_input_system: false, ..default() })
        .insert_resource(EditorState::default())
        .insert_resource(BrushSettings::default())
        .insert_resource(TexturePalette::default())
        .insert_resource(UndoHistory::default())
        .insert_resource(FpsTracker::default())
        .insert_resource(TextureEguiCache::default())
        .insert_resource(UiClickZones::default())
        .insert_resource(MaterialCache::default())
        .insert_resource(TerrainIndex::default())
        // MinimapCache removed (unused)
        .insert_resource(PointerOverUi::default())
        .insert_resource(LayerStack::default())
        .insert_resource(Selection::default())
        .insert_resource(BrushPresets::default())
        .insert_resource(AssetBrowser::default())
        .add_systems(Startup, (setup_camera, setup_terrain, load_map_glb))
        .add_systems(Update, (editor_ui, update_pointer_over_ui, hover_tracking, camera_controls))
        .add_systems(Update, (handle_painting, handle_ui_clicks, update_brush_preview, handle_shortcuts))
        .add_systems(Update, (draw_grid, read_mcp_commands, track_fps, load_terrain_on_start))
        .add_systems(Update, (autosave_system, build_terrain_index, handle_selection, handle_flood_fill))
        .add_systems(Update, (apply_heights, handle_clone_source, apply_shading_mode, refresh_all_terrain))
        .add_systems(Update, (handle_search_input, load_asset_browser, set_window_icon))
        .run();
}

// ─── Setup ───

fn set_window_icon(mut done: Local<bool>, time: Res<Time>) {
    if *done || time.elapsed_secs() < 2.0 { return; }
    *done = true;
    // Set X11 window icon via python-xlib (most reliable on Linux/X11)
    std::thread::spawn(|| {
        let _ = std::process::Command::new("python3").arg("-c").arg(r#"
from Xlib import display, Xatom
from PIL import Image
import subprocess
d = display.Display()
r = subprocess.run(["xdotool","search","--name","League of Legends"],capture_output=True,text=True)
wid = r.stdout.strip().split("\n")[0] if r.stdout.strip() else ""
if not wid: exit()
win = d.create_resource_object("window", int(wid))
img = Image.open("assets/icon.png").resize((64,64)).convert("RGBA")
w, h = img.size
data = [w, h]
for y in range(h):
    for x in range(w):
        r,g,b,a = img.getpixel((x,y))
        data.append((a<<24)|(r<<16)|(g<<8)|b)
win.change_property(d.intern_atom("_NET_WM_ICON"), Xatom.CARDINAL, 32, data)
d.flush()
"#).output();
    });
}

fn setup_camera(mut cmd: Commands) {
    cmd.spawn((
        Camera3d::default(),
        Transform::from_xyz(MAP_SIZE / 2.0, 5000.0, MAP_SIZE / 2.0 + 2500.0)
            .looking_at(Vec3::new(MAP_SIZE / 2.0, 0.0, MAP_SIZE / 2.0), Vec3::Y),
        EditorCamera,
    ));
    cmd.spawn((
        DirectionalLight { illuminance: 12000.0, shadows_enabled: false, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.3, 0.0)),
    ));
    cmd.spawn(DirectionalLight { illuminance: 4000.0, shadows_enabled: false, ..default() });
}

fn load_map_glb(mut cmd: Commands, srv: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(srv.load(bevy::gltf::GltfAssetLabel::Scene(0).from_asset("maps/twisted_treeline_patched.glb"))),
        Transform::default(), MapModel,
    ));
}

fn setup_terrain(
    mut cmd: Commands, mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>, srv: Res<AssetServer>,
    mut pal: ResMut<TexturePalette>,
) {
    // Curated descriptions for known textures
    let known: HashMap<&str, (&str, &str)> = HashMap::from([
        ("tile_lanetile_crackedstone_01", ("Pierre fissuree", "Lane")),
        ("tile_lanetile_crackedrubble_01", ("Debris pierre", "Lane")),
        ("structure_damge_tile_01", ("Dalle endommagee", "Lane")),
        ("decal_mud_path_01", ("Chemin boue", "Lane")),
        ("structure_pebbles", ("Cailloux", "Lane")),
        ("tile_mud_cracked_01", ("Boue craquelee", "Transition")),
        ("nature_dirt_skirt", ("Bordure terre", "Transition")),
        ("tile_roots_01", ("Racines sombres", "Jungle")),
        ("tile_vegetation_deadmossy_02", ("Mousse morte", "General")),
        ("decal_grass_tufts_02", ("Touffes herbe", "General")),
        ("tile_mud_and_wall_03", ("Boue et mur", "Jungle")),
        ("tile_roots_nastycurling_01", ("Racines tordues", "Jungle")),
        ("nature_spider_den_floor", ("Sol araignee", "Vilemaw")),
        ("structure_base_platform_01", ("Plateforme bleue", "Base")),
        ("structure_base_platform_02", ("Plateforme rouge", "Base")),
        ("structure_base_nexus_grnd_04", ("Sol nexus", "Base")),
        ("structure_base_inhibs_grnd_05", ("Sol inhibiteur", "Base")),
        ("structure_shrine_base_02", ("Base autel", "Autel")),
        ("decal_shrine_base", ("Decal autel", "Autel")),
        ("nature_spider_den_webs", ("Toiles araignee", "Vilemaw")),
        ("structure_walls_broken", ("Murs casses", "Structure")),
        ("structure_ground_steps", ("Marches", "Structure")),
        ("tile_vertical_dirt_02", ("Terre verticale", "Transition")),
        ("structure_lanetrim_01", ("Bordure lane", "Lane")),
    ]);

    // Auto-categorize by filename prefix
    fn auto_category(name: &str) -> &'static str {
        if name.starts_with("nature_spider") { return "Vilemaw"; }
        if name.starts_with("nature_") { return "Jungle"; }
        if name.starts_with("structure_base") || name.starts_with("nexus_") || name.starts_with("inhibitor_") || name.starts_with("chaos_") { return "Base"; }
        if name.starts_with("structure_shrine") || name.starts_with("decal_shrine") { return "Autel"; }
        if name.starts_with("structure_") { return "Structure"; }
        if name.contains("mud") || name.contains("dirt") || name.contains("vertical") { return "Transition"; }
        if name.starts_with("tile_lane") { return "Lane"; }
        if name.starts_with("tile_root") || name.starts_with("tile_vegetation") { return "Jungle"; }
        if name.starts_with("tile_") { return "Transition"; }
        if name.starts_with("decal_") { return "General"; }
        if name.starts_with("prop_") { return "Structure"; }
        "General"
    }

    fn prettify(name: &str) -> String {
        name.replace('_', " ").chars().enumerate()
            .map(|(i, c)| if i == 0 { c.to_uppercase().next().unwrap() } else { c })
            .collect()
    }

    // Scan texture directory for all PNGs
    let tex_dir = "assets/maps/textures";
    let mut tex_names: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(tex_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("png") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    tex_names.push(stem.to_string());
                }
            }
        }
    }
    tex_names.sort();

    for n in &tex_names {
        let (desc, cat) = if let Some(&(d, c)) = known.get(n.as_str()) {
            (d.to_string(), c.to_string())
        } else {
            (prettify(n), auto_category(n).to_string())
        };
        pal.entries.push(TexEntry {
            name: n.to_string(), desc, cat,
            image: srv.load(format!("maps/textures/{n}.png")),
        });
    }
    let cm = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(TILE_SIZE / 2.0)));
    let dark = mats.add(StandardMaterial { base_color: Color::srgb(0.14, 0.14, 0.16), perceptual_roughness: 1.0, ..default() });
    let light = mats.add(StandardMaterial { base_color: Color::srgb(0.18, 0.18, 0.20), perceptual_roughness: 1.0, ..default() });
    for gz in 0..TERRAIN_RES {
        for gx in 0..TERRAIN_RES {
            let m = if (gx + gz) % 2 == 0 { dark.clone() } else { light.clone() };
            cmd.spawn((
                Mesh3d(cm.clone()), MeshMaterial3d(m),
                Transform::from_xyz(gx as f32 * TILE_SIZE + TILE_SIZE / 2.0, -108.0, gz as f32 * TILE_SIZE + TILE_SIZE / 2.0),
                TerrainChunk { gx, gz },
            ));
        }
    }
    let bm = meshes.add(Torus::new(40.0, 50.0));
    let bmat = mats.add(StandardMaterial {
        base_color: Color::srgba(0.39, 0.52, 1.0, 0.5),
        emissive: bevy::color::LinearRgba::rgb(0.6, 0.8, 2.5),
        alpha_mode: AlphaMode::Blend, ..default()
    });
    cmd.spawn((Mesh3d(bm), MeshMaterial3d(bmat), Transform::from_xyz(0.0, -100.0, 0.0), BrushPreview));
}

// ─── UI ───

fn editor_ui(
    mut ctx: EguiContexts, mut st: ResMut<EditorState>, mut br: ResMut<BrushSettings>,
    pal: Res<TexturePalette>, mut map_vis: Query<&mut Visibility, With<MapModel>>,
    time: Res<Time>, fps: Res<FpsTracker>, undo: Res<UndoHistory>,
    mut tex_cache: ResMut<TextureEguiCache>,
    mut windows: Query<&mut Window>,
    mut zones: ResMut<UiClickZones>,
    layers: Res<LayerStack>, mut browser: ResMut<AssetBrowser>,
    _srv: Res<AssetServer>,
    presets: Res<BrushPresets>,
) {
    // Clear zones each frame (reuse allocations)
    zones.tool_buttons.clear();
    zones.texture_rows.clear();
    zones.category_pills.clear();
    zones.shape_circle = None;
    zones.shape_square = None;
    zones.size_minus = None;
    zones.size_plus = None;
    zones.opacity_minus = None;
    zones.opacity_plus = None;
    zones.rp_size_minus = None;
    zones.rp_size_plus = None;
    zones.rp_opacity_minus = None;
    zones.rp_opacity_plus = None;
    zones.rp_grid_toggle = None;
    zones.rp_model_toggle = None;
    zones.rp_overlay_toggle = None;
    zones.rp_sym_x = None;
    zones.rp_sym_z = None;
    zones.rp_falloff_btns.clear();
    zones.nav_gizmo_btns.clear();
    zones.search_bar = None;
    zones.layer_rows.clear();
    zones.layer_eye.clear();
    zones.layer_add = None;
    zones.preset_btns.clear();
    zones.rp_shading_btns.clear();
    zones.browser_items.clear();
    zones.browser_back = None;
    zones.browser_search = None;
    zones.browser_breadcrumbs.clear();
    zones.noise_scale_minus = None;
    zones.noise_scale_plus = None;
    zones.noise_tex2_btn = None;
    zones.toggle_left = None;
    zones.toggle_right = None;
    zones.toggle_browser = None;
    zones.menu_buttons.clear();
    zones.menu_items.clear();
    zones.any_panel_rect.clear();
    // Reserve capacity to avoid reallocation
    zones.tool_buttons.reserve(4);
    zones.texture_rows.reserve(24);
    zones.any_panel_rect.reserve(5);
    if time.elapsed_secs() < 0.5 { return; }

    // Show current tool in window title
    if let Ok(mut win) = windows.single_mut() {
        let tool = match st.tool { Tool::Paint => "Paint", Tool::Erase => "Erase", Tool::Pick => "Pick", Tool::Fill => "Fill", Tool::Smooth => "Smooth", Tool::Hand => "Hand", Tool::Select => "Select", Tool::Clone => "Clone", Tool::FloodFill => "Flood", Tool::Raise => "Height" };
        win.title = format!("League of Legends Studio — {tool}");
    }

    // Register texture handles in egui (once)
    if !tex_cache.initialized && !pal.entries.is_empty() {
        for (i, e) in pal.entries.iter().enumerate() {
            let id = ctx.add_image(EguiTextureHandle::Weak(e.image.id()));
            tex_cache.map.insert(i, id);
        }
        tex_cache.initialized = true;
    }

    let Ok(c) = ctx.ctx_mut() else { return };

    // Load ALL asset thumbnails directly into egui context
    if !browser.thumbs_loaded {
        browser.thumbs_loaded = true;
        if let Ok(entries) = std::fs::read_dir("assets/models/thumbnails") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("png") { continue; }
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue; };
                let Ok(img) = image::open(&path) else { continue; };
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                let tex_handle = c.load_texture(stem, color_image, egui::TextureOptions::LINEAR);
                browser.thumbnails.insert(stem.to_string(), tex_handle.id());
                browser.thumb_handles.push(tex_handle); // keep handle alive!
            }
        }
    }

    // Load Lucide icon font once
    if !tex_cache.fonts_loaded {
        tex_cache.fonts_loaded = true;
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "lucide".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(lucide_icons::LUCIDE_FONT_BYTES.to_vec())),
        );
        // Add lucide as fallback for proportional font
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().push("lucide".to_owned());
        c.set_fonts(fonts);
    }

    // Load logo once
    if browser.logo.is_none() {
        if let Ok(img) = image::open("assets/icon.png") {
            let rgba = img.resize(64, 64, image::imageops::FilterType::Lanczos3).to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
            browser.logo = Some(c.load_texture("logo", color_image, egui::TextureOptions::LINEAR));
        }
    }

    // ── Modern dark style (only set once) ──
    if !tex_cache.initialized {
    let mut sty = (*c.style()).clone();
    sty.visuals = egui::Visuals::dark();
    sty.visuals.panel_fill = rgb(BG_SURFACE);
    sty.visuals.window_fill = rgb(BG_ELEVATED);
    sty.visuals.window_stroke = egui::Stroke::new(1.0, rgb(BORDER));
    sty.visuals.widgets.inactive.bg_fill = rgb(BG_ELEVATED);
    sty.visuals.widgets.inactive.weak_bg_fill = rgb(BG_ELEVATED);
    sty.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, rgb(TEXT_SECONDARY));
    sty.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(0.5, rgb(BORDER));
    sty.visuals.widgets.hovered.bg_fill = rgb(BG_HOVER);
    sty.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, rgb(TEXT_PRIMARY));
    sty.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, rgb(ACCENT));
    sty.visuals.widgets.active.bg_fill = rgb(ACCENT);
    sty.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    sty.visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(ACCENT.0, ACCENT.1, ACCENT.2, 60);
    sty.visuals.selection.stroke = egui::Stroke::new(1.0, rgb(ACCENT));
    sty.visuals.extreme_bg_color = rgb(BG_BASE);
    sty.visuals.faint_bg_color = rgb(BG_ELEVATED);
    sty.visuals.widgets.noninteractive.bg_fill = rgb(BG_SURFACE);
    sty.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, rgb(TEXT_SECONDARY));
    sty.spacing.item_spacing = egui::vec2(8.0, 5.0);
    sty.spacing.button_padding = egui::vec2(10.0, 4.0);
    sty.visuals.window_corner_radius = 8.0.into();
    c.set_style(sty);
    } // end style init

    // ── Header bar ──
    let header_resp = egui::TopBottomPanel::top("header")
        .frame(egui::Frame::NONE.fill(rgb(BG_BASE)).inner_margin(egui::Margin { left: 12, right: 12, top: 6, bottom: 6 }))
        .show(c, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            // Logo
            // Logo image + title
            if let Some(ref logo) = browser.logo {
                let img = egui::Image::new(egui::load::SizedTexture::new(logo.id(), [20.0, 20.0]));
                ui.add(img);
            }
            ui.label(egui::RichText::new("LoL Studio").size(14.0).color(rgb(ACCENT)).strong());
            ui.colored_label(rgb(TEXT_MUTED), "|");

            // Menu labels (clicks handled by handle_ui_clicks)
            for name in ["File", "Edit", "View", "Help"] {
                let is_open = st.open_menu == Some(name);
                let color = if is_open { rgb(ACCENT) } else { rgb(TEXT_SECONDARY) };
                let r = ui.add(egui::Button::new(egui::RichText::new(name).size(12.0).color(color))
                    .fill(if is_open { rgb(BG_HOVER) } else { egui::Color32::TRANSPARENT })
                    .frame(false));
                zones.menu_buttons.push((r.rect, name));
            }

            ui.colored_label(rgb(TEXT_MUTED), "|");

            // Undo / Redo buttons
            let undo_col = if undo.undo_stack.is_empty() { rgb(TEXT_MUTED) } else { rgb(TEXT_SECONDARY) };
            let undo_r = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Undo2)).size(14.0).color(undo_col))
                .fill(egui::Color32::TRANSPARENT).frame(false));
            zones.nav_gizmo_btns.push((undo_r.rect, "undo_btn"));

            let redo_col = if undo.redo_stack.is_empty() { rgb(TEXT_MUTED) } else { rgb(TEXT_SECONDARY) };
            let redo_r = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Redo2)).size(14.0).color(redo_col))
                .fill(egui::Color32::TRANSPARENT).frame(false));
            zones.nav_gizmo_btns.push((redo_r.rect, "redo_btn"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                // FPS
                let fps_col = if fps.fps > 55.0 { rgb(TEXT_MUTED) } else { rgb(WARNING) };
                ui.label(egui::RichText::new(format!("{:.0} fps", fps.fps)).size(11.0).color(fps_col));

                // Coordinates
                ui.label(egui::RichText::new(format!("X:{:.0}  Z:{:.0}", st.cursor.x, st.cursor.z))
                    .size(11.0).color(rgb(TEXT_MUTED)).family(egui::FontFamily::Monospace));

                // Progress
                let pct = st.painted as f32 / (TERRAIN_RES * TERRAIN_RES) as f32;
                let pct_color = if pct > 0.5 { rgb(SUCCESS) } else if pct > 0.1 { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                ui.label(egui::RichText::new(format!("{:.1}%", pct * 100.0)).size(11.0).color(pct_color));
            });
        });
    });
    zones.any_panel_rect.push(header_resp.response.rect);

    // ── Toolbar ──
    let toolbar_resp = egui::TopBottomPanel::top("toolbar")
        .frame(egui::Frame::NONE
            .fill(rgb(BG_SURFACE))
            .inner_margin(egui::Margin { left: 8, right: 8, top: 4, bottom: 4 })
            .stroke(egui::Stroke::new(1.0, rgb(BORDER))))
        .show(c, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;

            // Tool buttons with icons
            let tools = [
                (Icon::Paintbrush, "Paint", "B", Tool::Paint),
                (Icon::Eraser, "Erase", "X", Tool::Erase),
                (Icon::Pipette, "Pick", "I", Tool::Pick),
                (Icon::PaintBucket, "Fill", "F", Tool::Fill),
                (Icon::Blend, "Smooth", "T", Tool::Smooth),
                (Icon::Hand, "Hand", "H", Tool::Hand),
                (Icon::SquareDashed, "Select", "M", Tool::Select),
                (Icon::Copy, "Clone", "C", Tool::Clone),
                (Icon::Droplets, "Flood", "L", Tool::FloodFill),
                (Icon::Mountain, "Height", "R", Tool::Raise),
            ];
            for (icon, label, key, tool) in tools {
                let icon = li(icon);
                let active = tool == st.tool;
                let (bg, fg) = if active {
                    (rgb(ACCENT), egui::Color32::WHITE)
                } else {
                    (rgb(BG_ELEVATED), rgb(TEXT_SECONDARY))
                };
                let btn = egui::Button::new(
                    egui::RichText::new(format!("{icon} {label}")).size(11.5).color(fg)
                ).fill(bg).corner_radius(6.0).min_size(egui::vec2(68.0, 28.0));
                let resp = ui.add(btn);
                zones.tool_buttons.push((resp.rect, tool));
                if resp.clicked() { st.tool = tool; st.status = format!("{label}"); }
                if resp.hovered() {
                    resp.on_hover_text(format!("{label} ({key})"));
                }
            }

            ui.add_space(6.0);
            separator_v(ui);
            ui.add_space(6.0);

            // Brush size with +/- buttons
            ui.label(egui::RichText::new(format!("{} Size", li(Icon::Maximize2))).size(11.0).color(rgb(TEXT_MUTED)));
            let sm = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Minus)).size(12.0).color(rgb(TEXT_PRIMARY)))
                .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(26.0, 26.0)));
            zones.size_minus = Some(sm.rect);
            ui.label(egui::RichText::new(format!("{:.0}", br.size)).size(13.0).color(egui::Color32::WHITE).strong().family(egui::FontFamily::Monospace));
            let sp = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Plus)).size(12.0).color(rgb(TEXT_PRIMARY)))
                .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(26.0, 26.0)));
            zones.size_plus = Some(sp.rect);

            ui.add_space(4.0);
            separator_v(ui);
            ui.add_space(4.0);

            // Opacity with +/- buttons
            ui.label(egui::RichText::new(format!("{} Opacity", li(Icon::Droplets))).size(11.0).color(rgb(TEXT_MUTED)));
            let om = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Minus)).size(12.0).color(rgb(TEXT_PRIMARY)))
                .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(26.0, 26.0)));
            zones.opacity_minus = Some(om.rect);
            ui.label(egui::RichText::new(format!("{:.1}", br.opacity)).size(13.0).color(egui::Color32::WHITE).strong().family(egui::FontFamily::Monospace));
            let op = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Plus)).size(12.0).color(rgb(TEXT_PRIMARY)))
                .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(26.0, 26.0)));
            zones.opacity_plus = Some(op.rect);

            ui.add_space(4.0);
            separator_v(ui);
            ui.add_space(4.0);

            // Shape toggle
            let circ = br.shape == BrushShape::Circle;
            let cr = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Circle)).size(14.0)).selected(circ));
            zones.shape_circle = Some(cr.rect);
            if cr.clicked() { br.shape = BrushShape::Circle; }
            let sr = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Square)).size(14.0)).selected(!circ));
            zones.shape_square = Some(sr.rect);
            if sr.clicked() { br.shape = BrushShape::Square; }

            ui.add_space(4.0);
            separator_v(ui);
            ui.add_space(4.0);

            // Selected texture name
            if st.tex < pal.entries.len() {
                ui.label(egui::RichText::new(&pal.entries[st.tex].name).size(11.0).color(rgb(ACCENT)));
            }
        });
    });
    zones.any_panel_rect.push(toolbar_resp.response.rect);

    // ── Status bar (must be declared BEFORE browser so it renders at the very bottom) ──
    let status_resp = egui::TopBottomPanel::bottom("status")
        .frame(egui::Frame::NONE
            .fill(rgb(BG_BASE))
            .inner_margin(egui::Margin { left: 12, right: 12, top: 3, bottom: 3 })
            .stroke(egui::Stroke::new(1.0, rgb(BORDER))))
        .min_height(22.0).show(c, |ui| {
        ui.horizontal(|ui| {
            let tool_label = match st.tool { Tool::Paint => "Paint", Tool::Erase => "Erase", Tool::Pick => "Pick", Tool::Fill => "Fill", Tool::Smooth => "Smooth", Tool::Hand => "Hand", Tool::Select => "Select", Tool::Clone => "Clone", Tool::FloodFill => "Flood", Tool::Raise => "Height" };
            ui.label(egui::RichText::new(tool_label).size(10.5).color(rgb(TEXT_MUTED)).family(egui::FontFamily::Monospace));
            ui.colored_label(rgb(BORDER), "\u{2022}");
            ui.label(egui::RichText::new(&st.status).size(10.5).color(rgb(TEXT_SECONDARY)));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Panel toggle buttons
                let tl_col = if st.show_left_panel { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                let tl = ui.add(egui::Button::new(egui::RichText::new(li(Icon::PanelLeft)).size(12.0).color(tl_col))
                    .fill(egui::Color32::TRANSPARENT).frame(false));
                zones.toggle_left = Some(tl.rect);

                let tr_col = if st.show_right_panel { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                let tr = ui.add(egui::Button::new(egui::RichText::new(li(Icon::PanelRight)).size(12.0).color(tr_col))
                    .fill(egui::Color32::TRANSPARENT).frame(false));
                zones.toggle_right = Some(tr.rect);

                let tb_col = if browser.show { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                let tb = ui.add(egui::Button::new(egui::RichText::new(li(Icon::PanelBottom)).size(12.0).color(tb_col))
                    .fill(egui::Color32::TRANSPARENT).frame(false));
                zones.toggle_browser = Some(tb.rect);

                ui.colored_label(rgb(BORDER), "\u{2022}");
                ui.label(egui::RichText::new("v0.5").size(10.0).color(rgb(TEXT_MUTED)));
                ui.colored_label(rgb(BORDER), "\u{2022}");
                ui.label(egui::RichText::new(format!("{}/{}", undo.undo_stack.len(), undo.redo_stack.len()))
                    .size(10.0).color(rgb(TEXT_MUTED)).family(egui::FontFamily::Monospace));
            });
        });
    });
    zones.any_panel_rect.push(status_resp.response.rect);

    // ── Asset Browser (bottom panel, Unity/Unreal style) ──
    if browser.show {
        let browser_resp = egui::TopBottomPanel::bottom("asset_browser")
            .frame(egui::Frame::NONE.fill(rgb(BG_SURFACE))
                .stroke(egui::Stroke::new(1.0, rgb(BORDER)))
                .inner_margin(egui::Margin { left: 10, right: 10, top: 8, bottom: 8 }))
            .resizable(true)
            .default_height(280.0)
            .min_height(180.0)
            .show(c, |ui| {
            // Header row: path + search
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{} Assets", li(Icon::FolderOpen))).size(12.0).color(rgb(TEXT_PRIMARY)).strong());
                ui.separator();

                // Category filter tabs or breadcrumb
                if browser.current_path == "ALL" {
                    let cats = ["All", "Champions", "Minions", "Props", "Turrets", "Maps", "Particles", "Animations", "Other"];
                    for cat in cats {
                        let active = browser.search.is_empty() && cat == "All"
                            || !browser.search.is_empty() && browser.search == cat;
                        let col = if active { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                        let r = ui.add(egui::Button::new(egui::RichText::new(cat).size(10.0).color(col))
                            .fill(if active { rgb(BG_ACTIVE) } else { egui::Color32::TRANSPARENT })
                            .corner_radius(8.0));
                        zones.browser_breadcrumbs.push((r.rect,
                            if cat == "All" { String::new() } else { cat.to_string() }));
                    }
                } else {
                    // Breadcrumb navigation
                    let r = ui.add(egui::Button::new(egui::RichText::new("All Models").size(11.0).color(rgb(TEXT_SECONDARY)))
                        .frame(false));
                    zones.browser_breadcrumbs.push((r.rect, "ALL".to_string()));
                    ui.colored_label(rgb(TEXT_MUTED), "\u{203A}");
                    let parts: Vec<&str> = browser.current_path.split('/').filter(|s| !s.is_empty()).collect();
                    for (i, part) in parts.iter().enumerate() {
                        if i > 0 { ui.colored_label(rgb(TEXT_MUTED), "\u{203A}"); }
                        let is_last = i == parts.len() - 1;
                        let col = if is_last { rgb(ACCENT) } else { rgb(TEXT_SECONDARY) };
                        let r = ui.add(egui::Button::new(egui::RichText::new(*part).size(11.0).color(col))
                            .frame(false));
                        let path_to: String = parts[..=i].join("/");
                        zones.browser_breadcrumbs.push((r.rect, path_to));
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Search
                    let search_col = if browser.search_focused { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                    let text = if browser.search.is_empty() { "Search...".to_string() } else { format!("{}|", browser.search) };
                    let sr = ui.add(egui::Button::new(
                        egui::RichText::new(format!("{} {}", li(Icon::Search), text)).size(10.5).color(search_col)
                    ).fill(rgb(BG_BASE)).corner_radius(4.0).min_size(egui::vec2(150.0, 20.0)));
                    zones.browser_search = Some(sr.rect);

                    // Back button
                    let br_btn = ui.add(egui::Button::new(egui::RichText::new(li(Icon::ArrowLeft)).size(12.0).color(rgb(TEXT_SECONDARY)))
                        .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(24.0, 20.0)));
                    zones.browser_back = Some(br_btn.rect);

                    // Item count
                    ui.label(egui::RichText::new(format!("{} items", browser.entries.len())).size(10.0).color(rgb(TEXT_MUTED)));
                });
            });
            ui.add_space(4.0);

            // Grid of asset items
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                    let query = browser.search.to_lowercase();
                    let is_cat_filter = ["champions", "minions", "props", "turrets", "maps", "particles", "animations", "other"].contains(&query.as_str());

                    for (i, entry) in browser.entries.iter().enumerate() {
                        if !query.is_empty() {
                            if is_cat_filter {
                                if entry.category.to_lowercase() != query { continue; }
                            } else if !entry.name.to_lowercase().contains(&query) { continue; }
                        }

                        let selected = browser.selected == Some(i);
                        let item_w = 120.0;
                        let item_h = 130.0;

                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(item_w, item_h), egui::Sense::click());

                        // Background
                        let bg = if selected {
                            egui::Color32::from_rgba_unmultiplied(ACCENT.0, ACCENT.1, ACCENT.2, 40)
                        } else if resp.hovered() {
                            rgb(BG_HOVER)
                        } else {
                            rgb(BG_ELEVATED)
                        };
                        ui.painter().rect_filled(rect, 8.0, bg);
                        if selected {
                            ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(2.0, rgb(ACCENT)), egui::StrokeKind::Outside);
                        }

                        // Thumbnail area (dark preview zone)
                        let thumb_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
                            egui::vec2(item_w - 12.0, 75.0),
                        );
                        ui.painter().rect_filled(thumb_rect, 6.0, rgb(BG_BASE));

                        // Thumbnail image or fallback icon
                        if !entry.is_dir && !entry.thumb_stem.is_empty() {
                            if let Some(&tex_id) = browser.thumbnails.get(&entry.thumb_stem) {
                                // Real thumbnail — use egui shape mesh
                                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                                let mut mesh = egui::Mesh::with_texture(tex_id);
                                mesh.add_rect_with_uv(thumb_rect, uv, egui::Color32::WHITE);
                                ui.painter().add(egui::Shape::mesh(mesh));
                            } else {
                                let col = match entry.category.as_str() {
                                    "Champions" => egui::Color32::from_rgb(100, 200, 255),
                                    "Minions" => egui::Color32::from_rgb(210, 170, 90),
                                    "Props" => egui::Color32::from_rgb(150, 175, 210),
                                    "Maps" => egui::Color32::from_rgb(90, 210, 130),
                                    _ => rgb(TEXT_SECONDARY),
                                };
                                ui.painter().text(thumb_rect.center(), egui::Align2::CENTER_CENTER, li(Icon::Box),
                                    egui::FontId::proportional(28.0), col);
                            }
                        } else if entry.is_dir {
                            ui.painter().text(thumb_rect.center(), egui::Align2::CENTER_CENTER, li(Icon::Folder),
                                egui::FontId::proportional(32.0), egui::Color32::from_rgb(230, 190, 55));
                        } else {
                            ui.painter().text(thumb_rect.center(), egui::Align2::CENTER_CENTER, li(Icon::Box),
                                egui::FontId::proportional(28.0), rgb(TEXT_SECONDARY));
                        }

                        // Category badge (top-right of thumbnail)
                        if !entry.is_dir {
                            let badge = &entry.category;
                            let badge_pos = egui::pos2(thumb_rect.right() - 4.0, thumb_rect.top() + 4.0);
                            ui.painter().text(badge_pos, egui::Align2::RIGHT_TOP, badge,
                                egui::FontId::proportional(7.5), rgb(TEXT_MUTED));
                        }

                        // File extension badge
                        if !entry.is_dir {
                            let ext = entry.name.rsplit('.').next().unwrap_or("");
                            let ext_rect = egui::Rect::from_min_size(
                                egui::pos2(thumb_rect.left() + 3.0, thumb_rect.bottom() - 14.0),
                                egui::vec2(26.0, 12.0),
                            );
                            ui.painter().rect_filled(ext_rect, 3.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150));
                            ui.painter().text(ext_rect.center(), egui::Align2::CENTER_CENTER,
                                ext.to_uppercase(), egui::FontId::proportional(7.0), egui::Color32::from_rgb(180, 180, 190));
                        }

                        // Name (below thumbnail)
                        let display_name = if entry.name.len() > 16 {
                            format!("{}...", &entry.name[..14])
                        } else {
                            entry.name.clone()
                        };
                        ui.painter().text(
                            egui::pos2(rect.center().x, rect.bottom() - 26.0),
                            egui::Align2::CENTER_CENTER, &display_name,
                            egui::FontId::proportional(9.5),
                            if selected { egui::Color32::WHITE } else { rgb(TEXT_PRIMARY) });

                        // Size
                        let info = if entry.is_dir { "folder".to_string() }
                            else if entry.size > 1_000_000 { format!("{:.1} MB", entry.size as f64 / 1_000_000.0) }
                            else if entry.size > 1_000 { format!("{:.0} KB", entry.size as f64 / 1_000.0) }
                            else { format!("{} B", entry.size) };
                        ui.painter().text(
                            egui::pos2(rect.center().x, rect.bottom() - 12.0),
                            egui::Align2::CENTER_CENTER, &info,
                            egui::FontId::proportional(8.5), rgb(TEXT_MUTED));

                        zones.browser_items.push((rect, i));
                    }
                });
            });
        });
        zones.any_panel_rect.push(browser_resp.response.rect);
    }

    let mut left_panel_rect = egui::Rect::NOTHING;
    let mut right_panel_rect = egui::Rect::NOTHING;

    // ── Left panel: Textures ──
    if st.show_left_panel {
    let left_resp = egui::SidePanel::left("textures")
        .default_width(280.0).resizable(true)
        .frame(egui::Frame::NONE.fill(rgb(BG_SURFACE)).stroke(egui::Stroke::new(1.0, rgb(BORDER)))
            .inner_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 0 }))
        .show(c, |ui| {
        // Panel header
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.label(egui::RichText::new(format!("{} Textures", li(Icon::Layers))).size(13.0).color(rgb(TEXT_PRIMARY)).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0);
                ui.label(egui::RichText::new(format!("{}", pal.entries.len())).size(11.0).color(rgb(TEXT_MUTED)));
            });
        });
        ui.add_space(8.0);

        // Search bar (full width)
        let border_col = if st.search_focused { rgb(ACCENT) } else { rgb(BORDER) };
        let search_frame = egui::Frame::NONE
            .fill(rgb(BG_BASE))
            .corner_radius(8.0)
            .inner_margin(egui::Margin { left: 10, right: 10, top: 8, bottom: 8 })
            .stroke(egui::Stroke::new(if st.search_focused { 1.5 } else { 0.5 }, border_col));
        ui.add_space(4.0);
        let sr = ui.horizontal(|ui| {
            ui.add_space(8.0);
            let r = search_frame.show(ui, |ui| {
                ui.set_min_width(ui.available_width() - 20.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(li(Icon::Search)).size(14.0).color(if st.search_focused { rgb(ACCENT) } else { rgb(TEXT_MUTED) }));
                    ui.add_space(6.0);
                    let text = if st.filter.is_empty() && !st.search_focused {
                        "Search textures...".to_string()
                    } else {
                        format!("{}{}", st.filter, if st.search_focused { "|" } else { "" })
                    };
                    let col = if st.filter.is_empty() && !st.search_focused { rgb(TEXT_MUTED) } else { rgb(TEXT_PRIMARY) };
                    ui.label(egui::RichText::new(text).size(13.0).color(col));
                });
            });
            r
        });
        zones.search_bar = Some(sr.inner.response.rect);
        ui.add_space(4.0);
        ui.add_space(6.0);

        // Category pills
        ui.horizontal_wrapped(|ui| {
            ui.add_space(10.0);
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
            for (ci, cat) in CATEGORIES.iter().enumerate() {
                let active = st.cat_filter == ci;
                let (bg, fg) = if active {
                    (rgb(ACCENT), egui::Color32::WHITE)
                } else {
                    (rgb(BG_ELEVATED), rgb(TEXT_MUTED))
                };
                let count = if ci == 0 { pal.entries.len() } else {
                    pal.entries.iter().filter(|e| e.cat == *cat).count()
                };
                let label = if ci == 0 { format!("All ({})", count) } else { format!("{} ({})", cat, count) };
                let btn = egui::Button::new(egui::RichText::new(label).size(10.0).color(fg))
                    .fill(bg).corner_radius(12.0);
                let pill_resp = ui.add(btn);
                zones.category_pills.push((pill_resp.rect, ci));
                if pill_resp.clicked() {
                    st.cat_filter = ci;
                    if ci == 0 { st.filter.clear(); }
                }
            }
        });
        ui.add_space(4.0);

        // Thin divider
        let rect = ui.available_rect_before_wrap();
        ui.painter().line_segment(
            [egui::pos2(rect.left(), rect.top()), egui::pos2(rect.right(), rect.top())],
            egui::Stroke::new(0.5, rgb(BORDER)),
        );
        ui.add_space(2.0);

        // Texture list
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            ui.add_space(4.0);
            let f = st.filter.to_lowercase();
            let cat_name = if st.cat_filter > 0 { CATEGORIES[st.cat_filter] } else { "" };

            for (i, e) in pal.entries.iter().enumerate() {
                // Filter by search text
                if !f.is_empty() && !e.name.to_lowercase().contains(&f) && !e.desc.to_lowercase().contains(&f) { continue; }
                // Filter by category
                if !cat_name.is_empty() && e.cat != cat_name { continue; }

                let sel = i == st.tex;

                // Allocate interactive rect for the whole row
                let _row_id = i; // index used for identification
                let desired = egui::vec2(ui.available_width(), 48.0);
                let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());

                // Background (selected, hovered, or transparent)
                let hovered = st.hovered_tex == Some(i);
                let bg = if sel {
                    egui::Color32::from_rgba_unmultiplied(ACCENT.0, ACCENT.1, ACCENT.2, 35)
                } else if hovered {
                    rgb(BG_HOVER)
                } else if resp.hovered() {
                    rgb(BG_HOVER)
                } else {
                    egui::Color32::TRANSPARENT
                };
                ui.painter().rect_filled(rect, 4.0, bg);

                // Selection indicator on left
                if sel {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height())),
                        1.0, rgb(ACCENT),
                    );
                }

                // Thumbnail
                let thumb_size = 36.0;
                let thumb_rect = egui::Rect::from_min_size(
                    rect.left_top() + egui::vec2(10.0, (rect.height() - thumb_size) / 2.0),
                    egui::vec2(thumb_size, thumb_size),
                );
                if let Some(&tex_id) = tex_cache.map.get(&i) {
                    let _ = _row_id;
                    ui.painter().image(tex_id, thumb_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                } else {
                    let (r, g, b) = cat_color(&e.cat);
                    ui.painter().rect_filled(thumb_rect, 3.0, egui::Color32::from_rgb(r, g, b));
                }
                ui.painter().rect_stroke(thumb_rect, 3.0,
                    egui::Stroke::new(if sel { 1.5 } else { 0.5 }, if sel { rgb(ACCENT) } else { rgb(BORDER) }),
                    egui::StrokeKind::Outside);

                // Text
                let text_x = thumb_rect.right() + 8.0;
                let name_col = if sel { egui::Color32::WHITE } else { rgb(TEXT_PRIMARY) };
                ui.painter().text(egui::pos2(text_x, rect.top() + 8.0), egui::Align2::LEFT_TOP,
                    &e.name, egui::FontId::proportional(11.0), name_col);
                ui.painter().text(egui::pos2(text_x, rect.top() + 22.0), egui::Align2::LEFT_TOP,
                    &e.desc, egui::FontId::proportional(9.5), rgb(TEXT_SECONDARY));
                let (cr, cg, cb) = cat_color(&e.cat);
                ui.painter().text(egui::pos2(text_x, rect.top() + 34.0), egui::Align2::LEFT_TOP,
                    &e.cat, egui::FontId::proportional(8.5),
                    egui::Color32::from_rgb(cr.saturating_add(50), cg.saturating_add(50), cb.saturating_add(50)));

                zones.texture_rows.push((rect, i));
                if resp.clicked() {
                    st.tex = i;
                    st.status = format!("Selected: {}", e.name);
                }
            }
            ui.add_space(8.0);
        });
    });
    left_panel_rect = left_resp.response.rect;
    zones.any_panel_rect.push(left_panel_rect);
    } // end show_left_panel

    // ── Right panel: Properties ──
    if st.show_right_panel {
    let right_resp =
    egui::SidePanel::right("props")
        .default_width(250.0).resizable(true)
        .frame(egui::Frame::NONE.fill(rgb(BG_SURFACE)).stroke(egui::Stroke::new(1.0, rgb(BORDER)))
            .inner_margin(egui::Margin { left: 12, right: 12, top: 12, bottom: 12 }))
        .show(c, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {

        // ── Brush ──
        section_header(ui, &format!("{} Brush", li(Icon::Paintbrush)));
        ui.add_space(4.0);
        // Size row with -/+
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Size").size(11.0).color(rgb(TEXT_SECONDARY)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let p = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Plus)).size(11.0).color(rgb(TEXT_PRIMARY)))
                    .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(24.0, 22.0)));
                zones.rp_size_plus = Some(p.rect);
                ui.label(egui::RichText::new(format!("{:.0}", br.size)).size(12.0).color(egui::Color32::WHITE).strong().family(egui::FontFamily::Monospace));
                let m = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Minus)).size(11.0).color(rgb(TEXT_PRIMARY)))
                    .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(24.0, 22.0)));
                zones.rp_size_minus = Some(m.rect);
            });
        });
        ui.add_space(2.0);
        // Opacity row with -/+
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Opacity").size(11.0).color(rgb(TEXT_SECONDARY)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let p = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Plus)).size(11.0).color(rgb(TEXT_PRIMARY)))
                    .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(24.0, 22.0)));
                zones.rp_opacity_plus = Some(p.rect);
                ui.label(egui::RichText::new(format!("{:.1}", br.opacity)).size(12.0).color(egui::Color32::WHITE).strong().family(egui::FontFamily::Monospace));
                let m = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Minus)).size(11.0).color(rgb(TEXT_PRIMARY)))
                    .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(24.0, 22.0)));
                zones.rp_opacity_minus = Some(m.rect);
            });
        });
        ui.add_space(6.0);

        // Falloff row
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{} Falloff", li(Icon::Circle))).size(11.0).color(rgb(TEXT_SECONDARY)));
        });
        ui.horizontal(|ui| {
            for (label, fo) in [("Smooth", Falloff::Smooth), ("Linear", Falloff::Linear), ("Sharp", Falloff::Sharp), ("Flat", Falloff::Constant)] {
                let active = br.falloff == fo;
                let col = if active { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                let bg = if active { rgb(BG_ACTIVE) } else { rgb(BG_ELEVATED) };
                let r = ui.add(egui::Button::new(egui::RichText::new(label).size(9.5).color(col))
                    .fill(bg).corner_radius(3.0).min_size(egui::vec2(0.0, 20.0)));
                zones.rp_falloff_btns.push((r.rect, fo));
            }
        });
        ui.add_space(4.0);

        // Symmetry row
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{} Symmetry", li(Icon::ArrowLeftRight))).size(11.0).color(rgb(TEXT_SECONDARY)));
        });
        ui.horizontal(|ui| {
            let sx_col = if br.sym_x { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
            let sx_bg = if br.sym_x { rgb(BG_ACTIVE) } else { rgb(BG_ELEVATED) };
            let sx = ui.add(egui::Button::new(egui::RichText::new("X").size(11.0).color(sx_col))
                .fill(sx_bg).corner_radius(3.0).min_size(egui::vec2(30.0, 22.0)));
            zones.rp_sym_x = Some(sx.rect);

            let sz_col = if br.sym_z { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
            let sz_bg = if br.sym_z { rgb(BG_ACTIVE) } else { rgb(BG_ELEVATED) };
            let sz = ui.add(egui::Button::new(egui::RichText::new("Z").size(11.0).color(sz_col))
                .fill(sz_bg).corner_radius(3.0).min_size(egui::vec2(30.0, 22.0)));
            zones.rp_sym_z = Some(sz.rect);
        });

        ui.add_space(8.0);

        // ── Noise Settings ──
        section_header(ui, &format!("{} Noise", li(Icon::Sparkles)));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Scale").size(11.0).color(rgb(TEXT_SECONDARY)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let sp = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Plus)).size(11.0).color(rgb(TEXT_PRIMARY)))
                    .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(24.0, 20.0)));
                zones.noise_scale_plus = Some(sp.rect);
                ui.label(egui::RichText::new(format!("{:.3}", st.noise_scale)).size(11.0).color(rgb(TEXT_PRIMARY)));
                let sm = ui.add(egui::Button::new(egui::RichText::new(li(Icon::Minus)).size(11.0).color(rgb(TEXT_PRIMARY)))
                    .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(24.0, 20.0)));
                zones.noise_scale_minus = Some(sm.rect);
            });
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Tex 2").size(11.0).color(rgb(TEXT_SECONDARY)));
            let name2 = if st.noise_tex2 < pal.entries.len() { pal.entries[st.noise_tex2].name.clone() } else { "none".into() };
            let tb = ui.add(egui::Button::new(egui::RichText::new(&name2).size(10.0).color(rgb(ACCENT)))
                .fill(rgb(BG_ELEVATED)).corner_radius(4.0));
            zones.noise_tex2_btn = Some(tb.rect);
        });
        ui.label(egui::RichText::new("Tip: select tex1, set tex2 via PgUp/Dn, then Edit > Noise Fill").size(8.5).color(rgb(TEXT_MUTED)));
        ui.add_space(8.0);

        // ── Presets ──
        section_header(ui, &format!("{} Presets", li(Icon::Bookmark)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (pi, preset) in presets.0.iter().enumerate() {
                let r = ui.add(egui::Button::new(egui::RichText::new(&preset.name).size(9.5).color(rgb(TEXT_SECONDARY)))
                    .fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(0.0, 20.0)));
                zones.preset_btns.push((r.rect, pi));
            }
        });
        ui.add_space(8.0);

        // ── Layers ──
        section_header(ui, &format!("{} Layers", li(Icon::Layers)));
        ui.add_space(4.0);
        for (li_idx, layer) in layers.layers.iter().enumerate() {
            ui.horizontal(|ui| {
                // Eye toggle
                let eye_icon = if layer.visible { Icon::Eye } else { Icon::EyeOff };
                let eye_col = if layer.visible { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                let er = ui.add(egui::Button::new(egui::RichText::new(li(eye_icon)).size(11.0).color(eye_col))
                    .fill(egui::Color32::TRANSPARENT).frame(false).min_size(egui::vec2(20.0, 20.0)));
                zones.layer_eye.push((er.rect, li_idx));

                // Layer name (active = highlighted)
                let active = li_idx == layers.active;
                let name_col = if active { egui::Color32::WHITE } else { rgb(TEXT_SECONDARY) };
                let bg = if active { egui::Color32::from_rgba_unmultiplied(ACCENT.0, ACCENT.1, ACCENT.2, 30) } else { egui::Color32::TRANSPARENT };
                let lr = ui.add(egui::Button::new(egui::RichText::new(&layer.name).size(11.0).color(name_col))
                    .fill(bg).corner_radius(3.0).min_size(egui::vec2(ui.available_width() - 4.0, 20.0)));
                zones.layer_rows.push((lr.rect, li_idx));

                if layer.locked {
                    ui.label(egui::RichText::new(li(Icon::Lock)).size(10.0).color(rgb(TEXT_MUTED)));
                }
            });
        }
        let add_r = ui.add(egui::Button::new(egui::RichText::new(format!("{} Add Layer", li(Icon::Plus))).size(10.0).color(rgb(TEXT_MUTED)))
            .fill(rgb(BG_ELEVATED)).corner_radius(3.0).min_size(egui::vec2(ui.available_width(), 20.0)));
        zones.layer_add = Some(add_r.rect);
        ui.add_space(8.0);

        // ── Selected Texture ──
        section_header(ui, &format!("{} Selected", li(Icon::Image)));
        ui.add_space(6.0);
        if st.tex < pal.entries.len() {
            let e = &pal.entries[st.tex];
            ui.label(egui::RichText::new(&e.name).size(12.0).color(rgb(TEXT_PRIMARY)).strong());
            ui.label(egui::RichText::new(format!("{} \u{2022} {}", e.desc, e.cat)).size(10.0).color(rgb(TEXT_SECONDARY)));
            ui.add_space(6.0);

            // Large texture preview
            let preview_w = (ui.available_width() - 4.0).min(220.0);
            if let Some(&tex_id) = tex_cache.map.get(&st.tex) {
                let img = egui::Image::new(egui::load::SizedTexture::new(tex_id, [preview_w, preview_w]))
                    .corner_radius(6.0);
                let resp = ui.add(img);
                // Accent border
                ui.painter().rect_stroke(resp.rect, 6.0, egui::Stroke::new(1.5, rgb(ACCENT)), egui::StrokeKind::Outside);
            } else {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(preview_w, preview_w), egui::Sense::hover());
                let (r, g, b) = cat_color(&e.cat);
                ui.painter().rect_filled(rect, 6.0, egui::Color32::from_rgb(r, g, b));
                ui.painter().rect_stroke(rect, 6.0, egui::Stroke::new(1.0, rgb(BORDER)), egui::StrokeKind::Outside);
            }
        }
        ui.add_space(12.0);

        // ── Map Info ──
        section_header(ui, &format!("{} Map", li(Icon::Map)));
        ui.add_space(4.0);
        info_row(ui, "Size", "15398");
        info_row(ui, "Grid", &format!("{0}x{0}", TERRAIN_RES));
        info_row(ui, "Tile", &format!("{:.1}", TILE_SIZE));
        info_row(ui, "Painted", &format!("{}", st.painted));

        // Mini progress bar
        ui.add_space(4.0);
        let pct = st.painted as f32 / (TERRAIN_RES * TERRAIN_RES) as f32;
        let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.0), egui::Sense::hover());
        ui.painter().rect_filled(bar_rect, 2.0, rgb(BG_BASE));
        let mut filled = bar_rect;
        filled.set_width(bar_rect.width() * pct);
        let bar_color = if pct > 0.5 { rgb(SUCCESS) } else { rgb(ACCENT) };
        ui.painter().rect_filled(filled, 2.0, bar_color);

        // Minimap: only draw painted cells (skip empty ones), max 1 rect per 4x4 block for perf
        ui.add_space(6.0);
        let minimap_size = (ui.available_width() - 4.0).min(160.0);
        let (mm_rect, _) = ui.allocate_exact_size(egui::vec2(minimap_size, minimap_size), egui::Sense::hover());
        ui.painter().rect_filled(mm_rect, 2.0, rgb(BG_BASE));
        // Downscale: render 32x32 blocks (each = 4x4 terrain cells)
        let block = 4;
        let blocks = TERRAIN_RES / block;
        let block_px = minimap_size / blocks as f32;
        for bz in 0..blocks {
            for bx in 0..blocks {
                // Sample center cell of block
                let gx = bx * block + block / 2;
                let gz = bz * block + block / 2;
                let v = st.cells[gz * TERRAIN_RES + gx];
                if v > 0 {
                    let tex_idx = (v - 1) as usize;
                    let (r, g, b) = if tex_idx < pal.entries.len() { cat_color(&pal.entries[tex_idx].cat) } else { (80, 80, 80) };
                    let px = mm_rect.left() + bx as f32 * block_px;
                    let py = mm_rect.top() + bz as f32 * block_px;
                    let cr = egui::Rect::from_min_size(egui::pos2(px, py), egui::vec2(block_px, block_px));
                    ui.painter().rect_filled(cr, 0.0, egui::Color32::from_rgb(r.saturating_add(40), g.saturating_add(40), b.saturating_add(40)));
                }
            }
        }
        ui.painter().rect_stroke(mm_rect, 2.0, egui::Stroke::new(0.5, rgb(BORDER)), egui::StrokeKind::Outside);

        ui.add_space(12.0);

        // ── View ──
        section_header(ui, &format!("{} View", li(Icon::Eye)));
        ui.add_space(4.0);
        // Grid toggle button
        let grid_icon = if st.grid { Icon::Eye } else { Icon::EyeOff };
        let grid_col = if st.grid { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
        let gr = ui.add(egui::Button::new(
            egui::RichText::new(format!("{} {} Grid", li(grid_icon), if st.grid { "ON" } else { "OFF" })).size(11.0).color(grid_col)
        ).fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(ui.available_width(), 24.0)));
        zones.rp_grid_toggle = Some(gr.rect);

        ui.add_space(2.0);

        // 3D Model toggle button
        let model_icon = if st.show_map { Icon::Eye } else { Icon::EyeOff };
        let model_col = if st.show_map { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
        let mr = ui.add(egui::Button::new(
            egui::RichText::new(format!("{} {} 3D Model", li(model_icon), if st.show_map { "ON" } else { "OFF" })).size(11.0).color(model_col)
        ).fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(ui.available_width(), 24.0)));
        zones.rp_model_toggle = Some(mr.rect);
        for mut v in &mut map_vis {
            *v = if st.show_map { Visibility::Inherited } else { Visibility::Hidden };
        }

        ui.add_space(2.0);

        // Overlay toggle
        let ov_icon = if st.show_overlay { Icon::Eye } else { Icon::EyeOff };
        let ov_col = if st.show_overlay { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
        let ov = ui.add(egui::Button::new(
            egui::RichText::new(format!("{} {} Overlay", li(ov_icon), if st.show_overlay { "ON" } else { "OFF" })).size(11.0).color(ov_col)
        ).fill(rgb(BG_ELEVATED)).corner_radius(4.0).min_size(egui::vec2(ui.available_width(), 24.0)));
        zones.rp_overlay_toggle = Some(ov.rect);

        ui.add_space(4.0);
        // Shading mode buttons
        ui.horizontal(|ui| {
            for (label, mode) in [("Textured", ShadingMode::Textured), ("Category", ShadingMode::CategoryColor)] {
                let active = st.shading_mode == mode;
                let col = if active { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
                let bg = if active { rgb(BG_ACTIVE) } else { rgb(BG_ELEVATED) };
                let r = ui.add(egui::Button::new(egui::RichText::new(label).size(9.5).color(col))
                    .fill(bg).corner_radius(3.0).min_size(egui::vec2(0.0, 20.0)));
                zones.rp_shading_btns.push((r.rect, mode));
            }
        });

        ui.add_space(12.0);

        // ── Undo History ──
        section_header(ui, &format!("{} History", li(Icon::History)));
        ui.add_space(4.0);
        if undo.undo_stack.is_empty() {
            ui.label(egui::RichText::new("No actions yet").size(10.0).color(rgb(TEXT_MUTED)));
        } else {
            // Show last 8 undo actions
            let start = undo.undo_stack.len().saturating_sub(8);
            for (i, action) in undo.undo_stack[start..].iter().enumerate() {
                let num = start + i + 1;
                ui.label(egui::RichText::new(format!("{}. {}", num, action.desc)).size(10.0).color(rgb(TEXT_SECONDARY)));
            }
        }

        ui.add_space(12.0);

        // ── Shortcuts ──
        section_header(ui, &format!("{} Shortcuts", li(Icon::Keyboard)));
        ui.add_space(4.0);
        let shortcuts = [
            ("B", "Paint"), ("X", "Erase"), ("I", "Pick"), ("F", "Fill"), ("T", "Smooth"),
            ("+/-", "Size"), ("[/]", "Opacity"), ("Ctrl+Z", "Undo"),
            ("Ctrl+S", "Save"), ("G", "Grid"), ("MMB", "Pan"), ("Home", "Reset Cam"),
        ];
        egui::Grid::new("shortcuts_grid").num_columns(2).spacing([10.0, 2.0]).show(ui, |ui| {
            for (k, a) in shortcuts {
                ui.label(egui::RichText::new(k).size(10.0).color(rgb(TEXT_MUTED)).family(egui::FontFamily::Monospace));
                ui.label(egui::RichText::new(a).size(10.0).color(rgb(TEXT_SECONDARY)));
                ui.end_row();
            }
        });
        }); // ScrollArea
    });
    right_panel_rect = right_resp.response.rect;
    zones.any_panel_rect.push(right_panel_rect);
    } // end show_right_panel

    // ── 3D Orientation Gizmo (top-right of VIEWPORT, not panel) ──
    {
        let arm = 26.0;
        // Use egui's central area rect — this is the remaining space after all panels
        let central = c.available_rect();
        let center = egui::pos2(
            central.right() - 55.0,
            central.top() + 55.0,
        );
        let p = c.layer_painter(egui::LayerId::new(egui::Order::Foreground, "gizmo".into()));

        // Background
        p.circle_filled(center, 44.0, egui::Color32::from_rgba_unmultiplied(14, 14, 18, 190));
        p.circle_stroke(center, 44.0, egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(55, 55, 70, 120)));

        // Axes: X→right(red), Y→up(blue), Z→down-left(green)
        let axes: [(&str, egui::Vec2, [u8; 3]); 3] = [
            ("X", egui::vec2(1.0, 0.0),       [230, 60, 60]),
            ("Y", egui::vec2(0.0, -1.0),      [55, 115, 245]),
            ("Z", egui::vec2(-0.71, 0.71),    [50, 200, 80]),
        ];
        // Draw back-to-front: Z, X, Y
        for idx in [2usize, 0, 1] {
            let (label, dir, [r, g, b]) = &axes[idx];
            let d = dir.normalized();
            let tip = center + d * arm;
            let base = center + d * (arm - 8.0);
            let perp = egui::vec2(-d.y, d.x);

            // Shaft
            p.line_segment([center, base], egui::Stroke::new(3.0, egui::Color32::from_rgb(*r, *g, *b)));
            // Arrowhead
            p.add(egui::Shape::convex_polygon(
                vec![tip, base + perp * 5.5, base - perp * 5.5],
                egui::Color32::from_rgb(*r, *g, *b), egui::Stroke::NONE,
            ));
            // Label
            let lp = center + d * (arm + 10.0);
            p.circle_filled(lp, 7.0, egui::Color32::from_rgb(*r, *g, *b));
            p.text(lp, egui::Align2::CENTER_CENTER, *label,
                egui::FontId::proportional(8.0), egui::Color32::WHITE);
            zones.nav_gizmo_btns.push((egui::Rect::from_center_size(lp, egui::vec2(16.0, 16.0)), label));
        }
        // Center dot
        p.circle_filled(center, 3.0, egui::Color32::from_rgb(170, 170, 185));

        // ── Viewport tool buttons (vertical strip below gizmo) ──
        let strip_x = center.x;
        let strip_top = center.y + 50.0;
        let btn_size = 32.0;
        let btn_gap = 2.0;

        let vp_tools: [(&str, Icon); 5] = [
            ("hand", Icon::Hand),
            ("zoom_in", Icon::ZoomIn),
            ("zoom_out", Icon::ZoomOut),
            ("camera", Icon::Camera),
            ("grid", Icon::Grid3x3),
        ];

        let strip_h = (btn_size + btn_gap) * vp_tools.len() as f32 + 8.0;
        let strip_w = btn_size + 12.0;
        let strip_bg = egui::Rect::from_min_size(
            egui::pos2(strip_x - strip_w / 2.0, strip_top),
            egui::vec2(strip_w, strip_h),
        );
        p.rect_filled(strip_bg, 6.0, egui::Color32::from_rgba_unmultiplied(14, 14, 18, 200));
        p.rect_stroke(strip_bg, 6.0, egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(55, 55, 70, 120)), egui::StrokeKind::Outside);

        for (i, (name, icon)) in vp_tools.iter().enumerate() {
            let by = strip_top + 4.0 + i as f32 * (btn_size + btn_gap);
            let btn_center = egui::pos2(strip_x, by + btn_size / 2.0);
            let btn_rect = egui::Rect::from_center_size(btn_center, egui::vec2(btn_size, btn_size));

            let is_active = (*name == "hand" && st.tool == Tool::Hand)
                || (*name == "grid" && st.grid);
            let bg = if is_active { rgb(BG_ACTIVE) } else { egui::Color32::TRANSPARENT };
            p.rect_filled(btn_rect, 4.0, bg);

            let icon_col = if is_active { rgb(ACCENT) } else { rgb(TEXT_MUTED) };
            p.text(btn_center, egui::Align2::CENTER_CENTER, li(*icon),
                egui::FontId::proportional(16.0), icon_col);

            // Large click zone
            zones.nav_gizmo_btns.push((btn_rect, name));
        }

        // Register the ENTIRE gizmo + strip area as one big blocking rect
        let full_gizmo_rect = egui::Rect::from_min_max(
            egui::pos2(center.x - 48.0, center.y - 48.0),
            egui::pos2(strip_bg.right(), strip_bg.bottom()),
        );
        zones.any_panel_rect.push(full_gizmo_rect);
    }

    // ── Viewport Overlay HUD (top-left of viewport) ──
    if st.show_overlay {
        let vp_left = left_panel_rect.right() + 10.0;
        let vp_top = toolbar_resp.response.rect.bottom() + 8.0;
        let painter = c.layer_painter(egui::LayerId::new(egui::Order::Foreground, "overlay".into()));

        let lines = [
            format!("{} Perspective", li(Icon::Camera)),
            format!("{} Height: {:.0}", li(Icon::ArrowUpDown), st.cam_height),
            format!("{} Cells: {}/{}", li(Icon::Grid3x3), st.painted, TERRAIN_RES * TERRAIN_RES),
            format!("{} ({:.0}, {:.0})", li(Icon::Crosshair), st.cursor.x, st.cursor.z),
        ];
        for (i, line) in lines.iter().enumerate() {
            painter.text(
                egui::pos2(vp_left, vp_top + i as f32 * 16.0),
                egui::Align2::LEFT_TOP, line,
                egui::FontId::proportional(11.0),
                egui::Color32::from_rgba_unmultiplied(200, 200, 210, 160),
            );
        }
    }

    // ── Command Palette (Ctrl+P) ──
    if st.show_palette {
        let screen = c.content_rect();
        let pw = 400.0_f32.min(screen.width() - 100.0);
        let palette_rect = egui::Rect::from_min_size(
            egui::pos2((screen.width() - pw) / 2.0, 80.0),
            egui::vec2(pw, 320.0),
        );
        let painter = c.layer_painter(egui::LayerId::new(egui::Order::Foreground, "palette".into()));
        painter.rect_filled(palette_rect, 10.0, rgb(BG_ELEVATED));
        painter.rect_stroke(palette_rect, 10.0, egui::Stroke::new(1.0, rgb(ACCENT)), egui::StrokeKind::Outside);

        // Search bar
        painter.text(
            egui::pos2(palette_rect.left() + 12.0, palette_rect.top() + 16.0),
            egui::Align2::LEFT_CENTER,
            if st.palette_query.is_empty() { format!("{} Search actions...", li(Icon::Search)) }
                else { format!("{} {}|", li(Icon::Search), st.palette_query) },
            egui::FontId::proportional(13.0), rgb(TEXT_MUTED),
        );

        // Action list
        let actions: Vec<(&str, &str)> = vec![
            ("Paint tool", "B"), ("Erase tool", "X"), ("Pick tool", "I"), ("Fill tool", "F"),
            ("Smooth tool", "T"), ("Hand tool", "H"), ("Select tool", "M"),
            ("Clone tool", "C"), ("Height tool", "R"),
            ("Toggle Grid", "G"), ("Toggle Overlay", ""), ("Reset Camera", "Home"),
            ("Save", "Ctrl+S"), ("Save Project", ""), ("Load Project", ""),
            ("Export PNG", ""), ("Export BIN", ""), ("Noise Fill", ""),
            ("Undo", "Ctrl+Z"), ("Redo", "Ctrl+Shift+Z"), ("Clear All", ""),
            ("Help", "F1"),
        ];
        let query = st.palette_query.to_lowercase();
        let mut y = palette_rect.top() + 36.0;
        let item_h = 24.0;
        let mut visible_count = 0;
        for (name, key) in &actions {
            if !query.is_empty() && !name.to_lowercase().contains(&query) { continue; }
            if visible_count >= 10 { break; }
            let item_rect = egui::Rect::from_min_size(
                egui::pos2(palette_rect.left() + 4.0, y), egui::vec2(pw - 8.0, item_h));
            painter.text(egui::pos2(item_rect.left() + 10.0, y + item_h / 2.0),
                egui::Align2::LEFT_CENTER, *name, egui::FontId::proportional(12.0), rgb(TEXT_PRIMARY));
            if !key.is_empty() {
                painter.text(egui::pos2(item_rect.right() - 10.0, y + item_h / 2.0),
                    egui::Align2::RIGHT_CENTER, *key, egui::FontId::proportional(10.0), rgb(TEXT_MUTED));
            }
            zones.nav_gizmo_btns.push((item_rect, name)); // reuse nav_gizmo for click handling
            y += item_h;
            visible_count += 1;
        }
        zones.any_panel_rect.push(palette_rect);
    }

    // ── Help overlay ──
    if st.show_help {
        egui::Window::new(egui::RichText::new("Shortcuts").size(14.0).color(rgb(TEXT_PRIMARY)))
            .collapsible(false).resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::NONE.fill(rgb(BG_ELEVATED)).corner_radius(12.0)
                .stroke(egui::Stroke::new(1.0, rgb(BORDER)))
                .inner_margin(egui::Margin { left: 24, right: 24, top: 20, bottom: 20 }))
            .show(c, |ui| {
                let groups = [
                    ("Tools", &[("B", "Paint"), ("X", "Erase"), ("I", "Pick"), ("F", "Fill")][..]),
                    ("Brush", &[("+/-", "Size"), ("[/]", "Opacity"), ("1-9", "Quick select")]),
                    ("Camera", &[("WASD", "Pan"), ("Scroll", "Zoom"), ("MMB drag", "Pan")]),
                    ("Edit", &[("Ctrl+Z", "Undo"), ("Ctrl+Shift+Z", "Redo"), ("Ctrl+S", "Save")]),
                    ("View", &[("G", "Grid"), ("F1", "Help")]),
                ];
                for (section, keys) in groups {
                    ui.label(egui::RichText::new(section).size(12.0).color(rgb(ACCENT)).strong());
                    ui.add_space(2.0);
                    egui::Grid::new(section).num_columns(2).spacing([16.0, 3.0]).show(ui, |ui| {
                        for (k, a) in keys.iter() {
                            // Key badge
                            ui.label(egui::RichText::new(*k).size(11.0).color(rgb(TEXT_PRIMARY))
                                .family(egui::FontFamily::Monospace).background_color(rgb(BG_BASE)));
                            ui.label(egui::RichText::new(*a).size(11.0).color(rgb(TEXT_SECONDARY)));
                            ui.end_row();
                        }
                    });
                    ui.add_space(6.0);
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new(
                    egui::RichText::new("Close").size(11.0).color(egui::Color32::WHITE)
                ).fill(rgb(ACCENT)).corner_radius(6.0)).clicked() {
                    st.show_help = false;
                }
            });
    }

    // ── Dropdown menus (rendered as egui::Window, clicks via handle_ui_clicks) ──
    if let Some(menu) = st.open_menu {
        // Find the menu button rect to position the dropdown
        let menu_rect = zones.menu_buttons.iter().find(|(_, n)| *n == menu).map(|(r, _)| *r);
        if let Some(mr) = menu_rect {
            let items: Vec<String> = match menu {
                "File" => vec![
                    format!("{} Save  Ctrl+S", li(Icon::Save)),
                    format!("{} Save Project", li(Icon::FolderDown)),
                    format!("{} Load Project", li(Icon::FolderUp)),
                    format!("{} Export PNG", li(Icon::ImageDown)),
                    format!("{} Export BIN", li(Icon::FileDown)),
                    format!("{} Export Splat", li(Icon::Palette)),
                    format!("{} Quit", li(Icon::LogOut)),
                ],
                "Edit" => vec![format!("{} Undo  Ctrl+Z", li(Icon::Undo2)), format!("{} Redo  Ctrl+Shift+Z", li(Icon::Redo2)), format!("{} Noise Fill", li(Icon::Sparkles)), format!("{} Clear All", li(Icon::Trash2))],
                "View" => vec![format!("{} Toggle Grid  G", li(Icon::Grid3x3)), format!("{} Toggle 3D Model", li(Icon::Package))],
                "Help" => vec![format!("{} Shortcuts  F1", li(Icon::Keyboard))],
                _ => vec![],
            };
            let dropdown_w = 180.0;
            let item_h = 28.0;
            let dropdown_h = items.len() as f32 * item_h + 8.0;
            let dropdown_rect = egui::Rect::from_min_size(
                egui::pos2(mr.left(), mr.bottom() + 2.0),
                egui::vec2(dropdown_w, dropdown_h),
            );
            // Background
            c.layer_painter(egui::LayerId::new(egui::Order::Foreground, "menu_dropdown".into()))
                .rect_filled(dropdown_rect, 6.0, rgb(BG_ELEVATED));
            c.layer_painter(egui::LayerId::new(egui::Order::Foreground, "menu_dropdown".into()))
                .rect_stroke(dropdown_rect, 6.0, egui::Stroke::new(1.0, rgb(BORDER)), egui::StrokeKind::Outside);

            // Render items and store their rects
            for (idx, item) in items.iter().enumerate() {
                let item_rect = egui::Rect::from_min_size(
                    egui::pos2(dropdown_rect.left() + 4.0, dropdown_rect.top() + 4.0 + idx as f32 * item_h),
                    egui::vec2(dropdown_w - 8.0, item_h - 2.0),
                );
                c.layer_painter(egui::LayerId::new(egui::Order::Foreground, "menu_dropdown".into()))
                    .text(
                        egui::pos2(item_rect.left() + 8.0, item_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        item, egui::FontId::proportional(12.0), rgb(TEXT_PRIMARY),
                    );
                zones.menu_items.push((item_rect, item.clone()));
            }
            zones.any_panel_rect.push(dropdown_rect);
        }
    }
}

// ─── Camera ───

fn camera_controls(
    time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>, mut motion: MessageReader<MouseMotion>,
    mut cam: Query<&mut Transform, With<EditorCamera>>,
    pointer_ui: Res<PointerOverUi>, mut st: ResMut<EditorState>,
    wins: Query<&Window, With<PrimaryWindow>>, zones: Res<UiClickZones>,
) {
    let Ok(mut tf) = cam.single_mut() else { return };

    let over_ui = pointer_ui.0 || is_over_ui(&wins, &zones);

    // Home = reset camera
    if keys.just_pressed(KeyCode::Home) {
        tf.translation = Vec3::new(MAP_SIZE / 2.0, 5000.0, MAP_SIZE / 2.0 + 2500.0);
        *tf = tf.looking_at(Vec3::new(MAP_SIZE / 2.0, 0.0, MAP_SIZE / 2.0), Vec3::Y);
    }

    let hand_drag = st.tool == Tool::Hand && mouse.pressed(MouseButton::Left) && !over_ui;
    if (mouse.pressed(MouseButton::Middle) && !over_ui) || hand_drag {
        for ev in motion.read() {
            let scale = tf.translation.y * 0.002;
            tf.translation.x -= ev.delta.x * scale;
            tf.translation.z -= ev.delta.y * scale;
        }
    } else {
        motion.read();
    }

    {
        let s = (tf.translation.y * 0.8) * time.delta_secs();
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) { tf.translation.z -= s; }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) { tf.translation.z += s; }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) { tf.translation.x -= s; }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { tf.translation.x += s; }
    }

    // Apply UI button zoom/reset (cam_height changed externally)
    if (st.cam_height - tf.translation.y).abs() > 100.0 {
        tf.translation.y = st.cam_height;
    }

    // Zoom with mouse wheel
    if !over_ui && scroll.delta.y.abs() > 0.001 {
        tf.translation.y = (tf.translation.y - scroll.delta.y * tf.translation.y * 0.1).clamp(300.0, 12000.0);
    }

    st.cam_height = tf.translation.y;
}

// ─── Painting ───

fn handle_painting(
    mouse: Res<ButtonInput<MouseButton>>, keys: Res<ButtonInput<KeyCode>>,
    wins: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut st: ResMut<EditorState>, br: Res<BrushSettings>, pal: Res<TexturePalette>,
    mut terr: Query<&mut MeshMaterial3d<StandardMaterial>, With<TerrainChunk>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut undo: ResMut<UndoHistory>,
    mut mat_cache: ResMut<MaterialCache>, terrain_idx: Res<TerrainIndex>,
    zones: Res<UiClickZones>,
    selection: Res<Selection>, mut layers: ResMut<LayerStack>,
) {
    if pal.entries.is_empty() || matches!(st.tool, Tool::Hand | Tool::Select | Tool::FloodFill) { return; }
    if is_over_ui(&wins, &zones) { return; }
    // Check layer lock
    if layers.active < layers.layers.len() && layers.layers[layers.active].locked { return; }

    if mouse.just_pressed(MouseButton::Left) {
        st.is_painting_stroke = true;
        st.stroke_cells.clear();
        st.stroke_heights.clear();
    }
    if mouse.just_released(MouseButton::Left) && st.is_painting_stroke {
        st.is_painting_stroke = false;
        let has_cells = !st.stroke_cells.is_empty();
        let has_heights = !st.stroke_heights.is_empty();
        if has_cells || has_heights {
            let drained: Vec<(usize, u8)> = st.stroke_cells.drain(..).collect();
            let changes: Vec<(usize, u8, u8)> = drained.into_iter().map(|(idx, old)| (idx, old, st.cells[idx])).collect();
            let drained_h: Vec<(usize, f32)> = st.stroke_heights.drain(..).collect();
            let height_changes: Vec<(usize, f32, f32)> = drained_h.into_iter().map(|(idx, old_h)| (idx, old_h, st.heights[idx])).collect();
            let desc = match st.tool {
                Tool::Paint => format!("Paint {} cells", changes.len()),
                Tool::Erase => format!("Erase {} cells", changes.len()),
                Tool::Fill => format!("Fill {} cells", changes.len()),
                Tool::Smooth => format!("Smooth {} cells", changes.len()),
                Tool::Clone => format!("Clone {} cells", changes.len()),
                Tool::Raise => format!("Height {} cells", changes.len()),
                _ => format!("{} cells", changes.len()),
            };
            if undo.undo_stack.len() >= MAX_UNDO { undo.undo_stack.remove(0); }
            undo.undo_stack.push(UndoAction { changes, height_changes, desc });
            undo.redo_stack.clear();
        }
    }

    if !mouse.pressed(MouseButton::Left) { return; }

    let Ok(w) = wins.single() else { return };
    let Some(cur) = w.cursor_position() else { return };
    let Ok((cam, ctf)) = cam.single() else { return };
    let Ok(ray) = cam.viewport_to_world(ctf, cur) else { return };
    let Some(d) = ray.intersect_plane(Vec3::new(0.0, -108.0, 0.0), InfinitePlane3d::new(Vec3::Y)) else { return };
    let hit = ray.get_point(d);
    let cx = (hit.x / TILE_SIZE) as i32;
    let cz = (hit.z / TILE_SIZE) as i32;
    let r = br.size as i32;

    let coords: Vec<(usize, usize)> = match br.shape {
        BrushShape::Circle => {
            let mut v = Vec::new();
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dz * dz <= r * r {
                        v.push(((cx + dx) as usize, (cz + dz) as usize));
                    }
                }
            }
            v
        }
        BrushShape::Square => {
            let mut v = Vec::new();
            for dz in -r..=r {
                for dx in -r..=r {
                    v.push(((cx + dx) as usize, (cz + dz) as usize));
                }
            }
            v
        }
    };

    // Add symmetry mirrors
    let mid = (TERRAIN_RES as i32) / 2;
    let mut sym_coords = Vec::new();
    if br.sym_x {
        for &(gx, gz) in &coords {
            let mx = (2 * mid - gx as i32) as usize;
            if mx < TERRAIN_RES { sym_coords.push((mx, gz)); }
        }
    }
    if br.sym_z {
        for &(gx, gz) in &coords {
            let mz = (2 * mid - gz as i32) as usize;
            if mz < TERRAIN_RES { sym_coords.push((gx, mz)); }
        }
    }
    if br.sym_x && br.sym_z {
        for &(gx, gz) in &coords {
            let mx = (2 * mid - gx as i32) as usize;
            let mz = (2 * mid - gz as i32) as usize;
            if mx < TERRAIN_RES && mz < TERRAIN_RES { sym_coords.push((mx, mz)); }
        }
    }
    let mut all_coords = coords;
    all_coords.extend(sym_coords);
    let coords = all_coords;

    // Get or create cached material for current texture
    let tex_mat = mat_cache.texture_mats.entry(st.tex).or_insert_with(|| {
        mats.add(StandardMaterial {
            base_color_texture: Some(pal.entries[st.tex].image.clone()),
            perceptual_roughness: 1.0, ..default()
        })
    }).clone();

    // Get or create checker materials
    let checker_d = mat_cache.checker_dark.get_or_insert_with(|| {
        mats.add(StandardMaterial { base_color: Color::srgb(0.14, 0.14, 0.16), perceptual_roughness: 1.0, ..default() })
    }).clone();
    let checker_l = mat_cache.checker_light.get_or_insert_with(|| {
        mats.add(StandardMaterial { base_color: Color::srgb(0.18, 0.18, 0.20), perceptual_roughness: 1.0, ..default() })
    }).clone();

    let mut changed = false;
    let mut rng = rand::rng();
    let cx_f = cx as f32;
    let cz_f = cz as f32;
    for (gx, gz) in coords {
        if gx >= TERRAIN_RES || gz >= TERRAIN_RES { continue; }
        // Respect selection
        if let Some((sx, sz, ex, ez)) = selection.rect {
            if gx < sx || gx > ex || gz < sz || gz > ez { continue; }
        }

        // Apply falloff + opacity: compute paint probability
        let dx = gx as f32 - cx_f;
        let dz = gz as f32 - cz_f;
        let dist = (dx * dx + dz * dz).sqrt();
        let norm_d = (dist / br.size).min(1.0);
        let falloff_mul = match br.falloff {
            Falloff::Constant => 1.0,
            Falloff::Linear => 1.0 - norm_d,
            Falloff::Smooth => 1.0 - norm_d * norm_d,
            Falloff::Sharp => if norm_d < 0.5 { 1.0 } else { 2.0 * (1.0 - norm_d) },
        };
        let probability = br.opacity * falloff_mul;
        if probability < 1.0 && rng.random::<f32>() > probability { continue; }

        let idx = gz * TERRAIN_RES + gx;
        match st.tool {
            Tool::Paint | Tool::Fill => {
                let new_val = (st.tex as u8) + 1;
                let old = st.cells[idx];
                if old != new_val {
                    st.stroke_cells.push((idx, old));
                    st.cells[idx] = new_val;
                    // Also write to active layer
                    let al = layers.active;
                    if al < layers.layers.len() {
                        layers.layers[al].cells[idx] = new_val;
                    }
                    if let Some(&entity) = terrain_idx.0.get(&(gx, gz)) {
                        if let Ok(mut m) = terr.get_mut(entity) {
                            *m = MeshMaterial3d(tex_mat.clone());
                        }
                    }
                    changed = true;
                }
            }
            Tool::Erase => {
                let old = st.cells[idx];
                if old != 0 {
                    st.stroke_cells.push((idx, old));
                    st.cells[idx] = 0;
                    { let al = layers.active; if al < layers.layers.len() { layers.layers[al].cells[idx] = 0; } }
                    if let Some(&entity) = terrain_idx.0.get(&(gx, gz)) {
                        if let Ok(mut m) = terr.get_mut(entity) {
                            *m = MeshMaterial3d(if (gx + gz) % 2 == 0 { checker_d.clone() } else { checker_l.clone() });
                        }
                    }
                    changed = true;
                }
            }
            Tool::Pick => {
                let v = st.cells[idx];
                if v > 0 {
                    st.tex = (v - 1) as usize;
                    st.status = format!("Picked: {}", pal.entries[st.tex].name);
                }
            }
            Tool::Smooth => {
                // Smooth: replace cell with most common neighbor texture
                let mut counts: HashMap<u8, u8> = HashMap::new();
                for dz2 in -1i32..=1 { for dx2 in -1i32..=1 {
                    let nx = gx as i32 + dx2; let nz = gz as i32 + dz2;
                    if nx >= 0 && nx < TERRAIN_RES as i32 && nz >= 0 && nz < TERRAIN_RES as i32 {
                        let nv = st.cells[nz as usize * TERRAIN_RES + nx as usize];
                        if nv > 0 { *counts.entry(nv).or_insert(0) += 1; }
                    }
                }}
                if let Some((&best, _)) = counts.iter().max_by_key(|(_, c)| **c) {
                    let old = st.cells[idx];
                    if old != best {
                        st.stroke_cells.push((idx, old));
                        st.cells[idx] = best;
                        let tex_i = (best - 1) as usize;
                        let smooth_mat = mat_cache.texture_mats.entry(tex_i).or_insert_with(|| {
                            mats.add(StandardMaterial {
                                base_color_texture: Some(pal.entries[tex_i].image.clone()),
                                perceptual_roughness: 1.0, ..default()
                            })
                        }).clone();
                        if let Some(&entity) = terrain_idx.0.get(&(gx, gz)) {
                            if let Ok(mut m) = terr.get_mut(entity) {
                                *m = MeshMaterial3d(smooth_mat);
                            }
                        }
                        changed = true;
                    }
                }
            }
            Tool::Clone => {
                if let Some((src_x, src_z)) = st.clone_source {
                    let offset_x = gx as i32 - cx;
                    let offset_z = gz as i32 - cz;
                    let sgx = (src_x + offset_x) as usize;
                    let sgz = (src_z + offset_z) as usize;
                    if sgx < TERRAIN_RES && sgz < TERRAIN_RES {
                        let src_val = st.cells[sgz * TERRAIN_RES + sgx];
                        if src_val > 0 {
                            let old = st.cells[idx];
                            if old != src_val {
                                st.stroke_cells.push((idx, old));
                                st.cells[idx] = src_val;
                                let ti = (src_val - 1) as usize;
                                let clone_mat = mat_cache.texture_mats.entry(ti).or_insert_with(|| {
                                    mats.add(StandardMaterial {
                                        base_color_texture: Some(pal.entries[ti].image.clone()),
                                        perceptual_roughness: 1.0, ..default()
                                    })
                                }).clone();
                                if let Some(&entity) = terrain_idx.0.get(&(gx, gz)) {
                                    if let Ok(mut m) = terr.get_mut(entity) { *m = MeshMaterial3d(clone_mat); }
                                }
                                changed = true;
                            }
                        }
                    }
                }
            }
            Tool::Raise => {
                let raise_amount = if keys.pressed(KeyCode::ShiftLeft) { -2.0 } else { 2.0 };
                let strength = br.opacity * match br.falloff {
                    Falloff::Constant => 1.0, Falloff::Linear => 1.0 - norm_d,
                    Falloff::Smooth => 1.0 - norm_d * norm_d, Falloff::Sharp => if norm_d < 0.5 { 1.0 } else { 2.0 * (1.0 - norm_d) },
                };
                let old_h = st.heights[idx];
                st.heights[idx] += raise_amount * strength;
                st.stroke_heights.push((idx, old_h));
                // Height is applied in apply_heights system
                changed = true;
            }
            Tool::Hand | Tool::Select | Tool::FloodFill => {} // handled elsewhere
        }
    }
    if changed {
        st.painted = st.cells.iter().filter(|&&v| v > 0).count();
        // minimap refresh handled by needs_refresh
    }
}

fn update_brush_preview(
    wins: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut bq: Query<(&mut Transform, &mut Visibility), With<BrushPreview>>,
    br: Res<BrushSettings>, mut st: ResMut<EditorState>,
    zones: Res<UiClickZones>,
) {
    let Ok((mut bt, mut vis)) = bq.single_mut() else { return };
    // Hide brush when over UI or in Hand mode
    if is_over_ui(&wins, &zones) || st.tool == Tool::Hand {
        *vis = Visibility::Hidden;
        return;
    }
    *vis = Visibility::Inherited;

    let Ok(w) = wins.single() else { return };
    let Some(cur) = w.cursor_position() else { return };
    let Ok((cam, ctf)) = cam.single() else { return };
    if let Ok(ray) = cam.viewport_to_world(ctf, cur) {
        if let Some(d) = ray.intersect_plane(Vec3::new(0.0, -108.0, 0.0), InfinitePlane3d::new(Vec3::Y)) {
            let h = ray.get_point(d);
            bt.translation = Vec3::new(h.x, -104.0, h.z);
            bt.scale = Vec3::splat(br.size * TILE_SIZE / 50.0);
            st.cursor = h;
        }
    }
}

// ─── Shortcuts ───

fn handle_shortcuts(
    keys: Res<ButtonInput<KeyCode>>, mut st: ResMut<EditorState>, mut br: ResMut<BrushSettings>,
    pal: Res<TexturePalette>, mut undo: ResMut<UndoHistory>,
    mut terr: Query<&mut MeshMaterial3d<StandardMaterial>, With<TerrainChunk>>,
    mut mats: ResMut<Assets<StandardMaterial>>, terrain_idx: Res<TerrainIndex>,
    mut selection: ResMut<Selection>, mut mat_cache: ResMut<MaterialCache>,
) {
    // Block shortcuts when search is focused or palette is open
    if st.search_focused || st.show_palette { return; }

    // Home = reset camera
    if keys.just_pressed(KeyCode::KeyB) { st.tool = Tool::Paint; st.status = "Paint".into(); }
    if keys.just_pressed(KeyCode::KeyX) { st.tool = Tool::Erase; st.status = "Erase".into(); }
    if keys.just_pressed(KeyCode::KeyI) { st.tool = Tool::Pick; st.status = "Pick".into(); }
    if keys.just_pressed(KeyCode::KeyF) { st.tool = Tool::Fill; st.status = "Fill".into(); }
    if keys.just_pressed(KeyCode::KeyT) { st.tool = Tool::Smooth; st.status = "Smooth".into(); }
    if keys.just_pressed(KeyCode::KeyH) { st.tool = Tool::Hand; st.status = "Hand".into(); }
    if keys.just_pressed(KeyCode::KeyM) { st.tool = Tool::Select; st.status = "Select".into(); }
    if keys.just_pressed(KeyCode::KeyC) { st.tool = Tool::Clone; st.status = "Clone (Alt+click to set source)".into(); }
    if keys.just_pressed(KeyCode::KeyR) { st.tool = Tool::Raise; st.status = "Height (Shift=lower)".into(); }
    if keys.just_pressed(KeyCode::KeyL) { st.tool = Tool::FloodFill; st.status = "Flood Fill".into(); }
    if keys.just_pressed(KeyCode::Escape) { selection.rect = None; selection.dragging = None; st.status = "Selection cleared".into(); }
    if keys.just_pressed(KeyCode::KeyG) { st.grid = !st.grid; }
    if keys.just_pressed(KeyCode::F1) { st.show_help = !st.show_help; }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) { br.size = (br.size + 1.0).min(20.0); }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) { br.size = (br.size - 1.0).max(1.0); }
    if keys.just_pressed(KeyCode::BracketLeft) { br.opacity = (br.opacity - 0.1).max(0.1); }
    if keys.just_pressed(KeyCode::BracketRight) { br.opacity = (br.opacity + 0.1).min(1.0); }
    if keys.just_pressed(KeyCode::PageUp) && st.tex > 0 { st.tex -= 1; }
    if keys.just_pressed(KeyCode::PageDown) && st.tex + 1 < pal.entries.len() { st.tex += 1; }
    for (k, i) in [
        (KeyCode::Digit1, 0), (KeyCode::Digit2, 1), (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3), (KeyCode::Digit5, 4), (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6), (KeyCode::Digit8, 7), (KeyCode::Digit9, 8),
    ] {
        if keys.just_pressed(k) && i < pal.entries.len() { st.tex = i; }
    }

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if ctrl && keys.just_pressed(KeyCode::KeyP) { st.show_palette = !st.show_palette; st.palette_query.clear(); }

    if ctrl && keys.just_pressed(KeyCode::KeyZ) && !keys.pressed(KeyCode::ShiftLeft) {
        if let Some(action) = undo.undo_stack.pop() {
            for &(idx, old, _new) in &action.changes {
                st.cells[idx] = old;
                apply_cell_visual(idx % TERRAIN_RES, idx / TERRAIN_RES, old, &pal, &mut terr, &mut mats, &terrain_idx, &mut mat_cache);
            }
            for &(idx, old_h, _new_h) in &action.height_changes {
                st.heights[idx] = old_h;
            }
            st.status = format!("Undo: {}", action.desc);
            undo.redo_stack.push(action);
            st.painted = st.cells.iter().filter(|&&v| v > 0).count();
        }
    }

    if (ctrl && keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::KeyZ))
        || (ctrl && keys.just_pressed(KeyCode::KeyY))
    {
        if let Some(action) = undo.redo_stack.pop() {
            for &(idx, _old, new) in &action.changes {
                st.cells[idx] = new;
                apply_cell_visual(idx % TERRAIN_RES, idx / TERRAIN_RES, new, &pal, &mut terr, &mut mats, &terrain_idx, &mut mat_cache);
            }
            for &(idx, _old_h, new_h) in &action.height_changes {
                st.heights[idx] = new_h;
            }
            st.status = format!("Redo: {}", action.desc);
            undo.undo_stack.push(action);
            st.painted = st.cells.iter().filter(|&&v| v > 0).count();
        }
    }

    if ctrl && keys.just_pressed(KeyCode::KeyS) {
        save_terrain(&st, &pal);
        st.status = format!("Saved ({} cells)", st.painted);
    }
}

fn apply_cell_visual(
    gx: usize, gz: usize, val: u8, pal: &TexturePalette,
    terr: &mut Query<&mut MeshMaterial3d<StandardMaterial>, With<TerrainChunk>>,
    mats: &mut Assets<StandardMaterial>,
    terrain_idx: &TerrainIndex,
    mat_cache: &mut MaterialCache,
) {
    let Some(&entity) = terrain_idx.0.get(&(gx, gz)) else { return; };
    let Ok(mut m) = terr.get_mut(entity) else { return; };
    if val > 0 && (val as usize - 1) < pal.entries.len() {
        let ti = (val - 1) as usize;
        let mat = mat_cache.texture_mats.entry(ti).or_insert_with(|| {
            mats.add(StandardMaterial { base_color_texture: Some(pal.entries[ti].image.clone()), perceptual_roughness: 1.0, ..default() })
        }).clone();
        *m = MeshMaterial3d(mat);
    } else {
        let ck = mat_cache.checker_dark.get_or_insert_with(|| {
            mats.add(StandardMaterial { base_color: Color::srgb(0.14, 0.14, 0.16), perceptual_roughness: 1.0, ..default() })
        }).clone();
        let cl = mat_cache.checker_light.get_or_insert_with(|| {
            mats.add(StandardMaterial { base_color: Color::srgb(0.18, 0.18, 0.20), perceptual_roughness: 1.0, ..default() })
        }).clone();
        *m = MeshMaterial3d(if (gx + gz) % 2 == 0 { ck } else { cl });
    }
}

// ─── Save / Load / Grid / MCP ───

fn save_terrain(st: &EditorState, pal: &TexturePalette) {
    let d: Vec<String> = st.cells.iter().map(|&v| {
        if v > 0 && (v as usize - 1) < pal.entries.len() {
            pal.entries[v as usize - 1].name.clone()
        } else {
            "empty".into()
        }
    }).collect();
    let j = serde_json::to_string_pretty(&serde_json::json!({"resolution": TERRAIN_RES, "cells": d}))
        .unwrap_or_default();
    std::fs::write("assets/maps/terrain_editor_data.json", &j).ok();
}

fn export_terrain_png(st: &EditorState, pal: &TexturePalette) {
    let mut img = image::RgbImage::new(TERRAIN_RES as u32, TERRAIN_RES as u32);
    for gz in 0..TERRAIN_RES {
        for gx in 0..TERRAIN_RES {
            let v = st.cells[gz * TERRAIN_RES + gx];
            let (r, g, b) = if v > 0 && (v as usize - 1) < pal.entries.len() {
                let (cr, cg, cb) = cat_color(&pal.entries[(v - 1) as usize].cat);
                (cr.saturating_add(40), cg.saturating_add(40), cb.saturating_add(40))
            } else {
                (30, 30, 35)
            };
            img.put_pixel(gx as u32, gz as u32, image::Rgb([r, g, b]));
        }
    }
    img.save("assets/maps/terrain_export.png").ok();
}

/// Export splat map: R=layer0, G=layer1, B=layer2
fn export_splat_map(layers: &LayerStack) {
    let mut img = image::RgbImage::new(TERRAIN_RES as u32, TERRAIN_RES as u32);
    for gz in 0..TERRAIN_RES {
        for gx in 0..TERRAIN_RES {
            let idx = gz * TERRAIN_RES + gx;
            let r = if layers.layers.len() > 0 { layers.layers[0].cells[idx] } else { 0 };
            let g = if layers.layers.len() > 1 { layers.layers[1].cells[idx] } else { 0 };
            let b = if layers.layers.len() > 2 { layers.layers[2].cells[idx] } else { 0 };
            img.put_pixel(gx as u32, gz as u32, image::Rgb([r, g, b]));
        }
    }
    img.save("assets/maps/terrain_splat.png").ok();
}

/// Export as compact binary: 4-byte header "SGTM" + raw cells
fn export_terrain_bin(st: &EditorState) {
    let mut data = Vec::with_capacity(4 + st.cells.len() + st.heights.len() * 4);
    data.extend_from_slice(b"SGTM"); // magic
    data.push(TERRAIN_RES as u8); // resolution
    data.push(1); // version
    data.push(0); data.push(0); // reserved
    data.extend_from_slice(&st.cells);
    // Append heights as f32 LE bytes
    for &h in &st.heights {
        data.extend_from_slice(&h.to_le_bytes());
    }
    std::fs::write("assets/maps/terrain.bin", &data).ok();
}

/// Save entire project state as JSON
fn save_project(st: &EditorState, br: &BrushSettings, pal: &TexturePalette, layers: &LayerStack) {
    let cells_json: Vec<String> = st.cells.iter().map(|&v| {
        if v > 0 && (v as usize - 1) < pal.entries.len() { pal.entries[v as usize - 1].name.clone() } else { "empty".into() }
    }).collect();

    let layers_json: Vec<serde_json::Value> = layers.layers.iter().map(|l| {
        let lcells: Vec<String> = l.cells.iter().map(|&v| {
            if v > 0 && (v as usize - 1) < pal.entries.len() { pal.entries[v as usize - 1].name.clone() } else { "empty".into() }
        }).collect();
        serde_json::json!({
            "name": l.name, "visible": l.visible, "locked": l.locked, "cells": lcells
        })
    }).collect();

    let project = serde_json::json!({
        "version": 2,
        "resolution": TERRAIN_RES,
        "cells": cells_json,
        "layers": layers_json,
        "active_layer": layers.active,
        "heights": st.heights,
        "brush": {
            "size": br.size, "opacity": br.opacity,
            "shape": if br.shape == BrushShape::Circle { "circle" } else { "square" },
            "falloff": match br.falloff { Falloff::Smooth => "smooth", Falloff::Linear => "linear", Falloff::Sharp => "sharp", Falloff::Constant => "constant" },
            "sym_x": br.sym_x, "sym_z": br.sym_z
        },
        "camera_height": st.cam_height,
        "grid": st.grid, "show_map": st.show_map, "show_overlay": st.show_overlay,
    });
    let j = serde_json::to_string_pretty(&project).unwrap_or_default();
    std::fs::write("assets/maps/project.sgproj", &j).ok();
}

/// Load project from .sgproj
fn load_project(st: &mut EditorState, br: &mut BrushSettings, pal: &TexturePalette, layers: &mut LayerStack) -> bool {
    let Ok(content) = std::fs::read_to_string("assets/maps/project.sgproj") else { return false; };
    let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) else { return false; };

    let name_to_idx: HashMap<&str, u8> = pal.entries.iter().enumerate()
        .map(|(i, e)| (e.name.as_str(), (i as u8) + 1)).collect();

    // Load cells
    if let Some(cells) = data.get("cells").and_then(|v| v.as_array()) {
        for (i, cell) in cells.iter().enumerate() {
            if i >= TERRAIN_RES * TERRAIN_RES { break; }
            if let Some(name) = cell.as_str() {
                st.cells[i] = if name == "empty" { 0 } else { *name_to_idx.get(name).unwrap_or(&0) };
            }
        }
    }

    // Load heights
    if let Some(heights) = data.get("heights").and_then(|v| v.as_array()) {
        for (i, h) in heights.iter().enumerate() {
            if i >= st.heights.len() { break; }
            st.heights[i] = h.as_f64().unwrap_or(0.0) as f32;
        }
    }

    // Load layers
    if let Some(layer_arr) = data.get("layers").and_then(|v| v.as_array()) {
        layers.layers.clear();
        for lj in layer_arr {
            let name = lj.get("name").and_then(|v| v.as_str()).unwrap_or("Layer").to_string();
            let visible = lj.get("visible").and_then(|v| v.as_bool()).unwrap_or(true);
            let locked = lj.get("locked").and_then(|v| v.as_bool()).unwrap_or(false);
            let mut layer = Layer::new(&name);
            layer.visible = visible;
            layer.locked = locked;
            if let Some(lcells) = lj.get("cells").and_then(|v| v.as_array()) {
                for (i, cell) in lcells.iter().enumerate() {
                    if i >= layer.cells.len() { break; }
                    if let Some(n) = cell.as_str() {
                        layer.cells[i] = if n == "empty" { 0 } else { *name_to_idx.get(n).unwrap_or(&0) };
                    }
                }
            }
            layers.layers.push(layer);
        }
        layers.active = data.get("active_layer").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    }

    // Load brush
    if let Some(b) = data.get("brush") {
        br.size = b.get("size").and_then(|v| v.as_f64()).unwrap_or(3.0) as f32;
        br.opacity = b.get("opacity").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        br.shape = if b.get("shape").and_then(|v| v.as_str()) == Some("square") { BrushShape::Square } else { BrushShape::Circle };
        br.falloff = match b.get("falloff").and_then(|v| v.as_str()).unwrap_or("smooth") {
            "linear" => Falloff::Linear, "sharp" => Falloff::Sharp, "constant" => Falloff::Constant, _ => Falloff::Smooth,
        };
        br.sym_x = b.get("sym_x").and_then(|v| v.as_bool()).unwrap_or(false);
        br.sym_z = b.get("sym_z").and_then(|v| v.as_bool()).unwrap_or(false);
    }

    st.cam_height = data.get("camera_height").and_then(|v| v.as_f64()).unwrap_or(5000.0) as f32;
    st.grid = data.get("grid").and_then(|v| v.as_bool()).unwrap_or(true);
    st.show_map = data.get("show_map").and_then(|v| v.as_bool()).unwrap_or(true);
    st.show_overlay = data.get("show_overlay").and_then(|v| v.as_bool()).unwrap_or(true);
    st.painted = st.cells.iter().filter(|&&v| v > 0).count();
    st.needs_refresh = true;
    true
}

fn track_fps(mut fps: ResMut<FpsTracker>) {
    let now = Instant::now();
    fps.frames += 1;
    if let Some(last) = fps.last_update {
        let elapsed = now.duration_since(last).as_secs_f32();
        if elapsed >= 0.5 {
            fps.fps = fps.frames as f32 / elapsed;
            fps.frames = 0;
            fps.last_update = Some(now);
        }
    } else {
        fps.last_update = Some(now);
    }
}

fn load_terrain_on_start(
    mut st: ResMut<EditorState>, pal: Res<TexturePalette>,
    mut terr: Query<&mut MeshMaterial3d<StandardMaterial>, With<TerrainChunk>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    terrain_idx: Res<TerrainIndex>,
) {
    if st.loaded || pal.entries.is_empty() || terrain_idx.0.is_empty() { return; }
    st.loaded = true;

    let Ok(content) = std::fs::read_to_string("assets/maps/terrain_editor_data.json") else { return; };
    let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) else { return; };
    let Some(cells) = data.get("cells").and_then(|v| v.as_array()) else { return; };

    let name_to_idx: HashMap<&str, u8> = pal.entries.iter().enumerate()
        .map(|(i, e)| (e.name.as_str(), (i as u8) + 1)).collect();

    let mut count = 0;
    for (i, cell) in cells.iter().enumerate() {
        if i >= TERRAIN_RES * TERRAIN_RES { break; }
        let Some(name) = cell.as_str() else { continue; };
        if name == "empty" { continue; }
        if let Some(&idx) = name_to_idx.get(name) {
            st.cells[i] = idx;
            let gx = i % TERRAIN_RES;
            let gz = i / TERRAIN_RES;
            if let Some(&entity) = terrain_idx.0.get(&(gx, gz)) {
                if let Ok(mut m) = terr.get_mut(entity) {
                    let th = pal.entries[(idx - 1) as usize].image.clone();
                    *m = MeshMaterial3d(mats.add(StandardMaterial {
                        base_color_texture: Some(th), perceptual_roughness: 1.0, ..default()
                    }));
                }
            }
            count += 1;
        }
    }
    st.painted = count;
    if count > 0 { st.status = format!("Loaded {count} cells"); }
}

fn draw_grid(mut gz: Gizmos, st: Res<EditorState>, selection: Res<Selection>) {
    if !st.grid { return; }
    let y = -107.5;
    let step = TILE_SIZE * 8.0;
    let major = Color::srgba(0.3, 0.3, 0.38, 0.3);
    let mut x = 0.0;
    while x <= MAP_SIZE {
        gz.line(Vec3::new(x, y, 0.0), Vec3::new(x, y, MAP_SIZE), major);
        x += step;
    }
    let mut z = 0.0;
    while z <= MAP_SIZE {
        gz.line(Vec3::new(0.0, y, z), Vec3::new(MAP_SIZE, y, z), major);
        z += step;
    }
    // Center crosshair
    let c = MAP_SIZE / 2.0;
    gz.line(Vec3::new(c, y, 0.0), Vec3::new(c, y, MAP_SIZE), Color::srgba(0.25, 0.45, 0.25, 0.35));
    gz.line(Vec3::new(0.0, y, c), Vec3::new(MAP_SIZE, y, c), Color::srgba(0.45, 0.25, 0.25, 0.35));
    // Border
    let b = Color::srgba(0.5, 0.35, 0.15, 0.4);
    gz.line(Vec3::new(0.0, y, 0.0), Vec3::new(MAP_SIZE, y, 0.0), b);
    gz.line(Vec3::new(MAP_SIZE, y, 0.0), Vec3::new(MAP_SIZE, y, MAP_SIZE), b);
    gz.line(Vec3::new(MAP_SIZE, y, MAP_SIZE), Vec3::new(0.0, y, MAP_SIZE), b);
    gz.line(Vec3::new(0.0, y, MAP_SIZE), Vec3::new(0.0, y, 0.0), b);

    // Draw selection rectangle
    if let Some((sx, sz, ex, ez)) = selection.rect {
        let sel_y = -106.0;
        let sel_c = Color::srgba(0.4, 0.7, 1.0, 0.8);
        let x0 = sx as f32 * TILE_SIZE; let z0 = sz as f32 * TILE_SIZE;
        let x1 = (ex + 1) as f32 * TILE_SIZE; let z1 = (ez + 1) as f32 * TILE_SIZE;
        gz.line(Vec3::new(x0, sel_y, z0), Vec3::new(x1, sel_y, z0), sel_c);
        gz.line(Vec3::new(x1, sel_y, z0), Vec3::new(x1, sel_y, z1), sel_c);
        gz.line(Vec3::new(x1, sel_y, z1), Vec3::new(x0, sel_y, z1), sel_c);
        gz.line(Vec3::new(x0, sel_y, z1), Vec3::new(x0, sel_y, z0), sel_c);
    }
}

/// Handles UI clicks via Bevy's mouse input since bevy_egui doesn't receive click events.
/// Uses real egui rects stored in UiClickZones.
fn handle_ui_clicks(
    mouse: Res<ButtonInput<MouseButton>>, wins: Query<&Window, With<PrimaryWindow>>,
    mut st: ResMut<EditorState>, mut br: ResMut<BrushSettings>, pal: Res<TexturePalette>,
    zones: Res<UiClickZones>, mut layers: ResMut<LayerStack>, presets: Res<BrushPresets>,
    selection: Res<Selection>, mut browser: ResMut<AssetBrowser>,
    mut undo: ResMut<UndoHistory>, mut terr: Query<&mut MeshMaterial3d<StandardMaterial>, With<TerrainChunk>>,
    mut mats: ResMut<Assets<StandardMaterial>>, terrain_idx: Res<TerrainIndex>,
    mut mat_cache: ResMut<MaterialCache>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    let Ok(w) = wins.single() else { return; };
    let Some(cursor) = w.cursor_position() else { return; };
    let pos = egui::pos2(cursor.x, cursor.y);

    // Navigation gizmo + viewport tool buttons (check FIRST since they overlay the map)
    for (rect, name) in &zones.nav_gizmo_btns {
        if rect.contains(pos) {
            match *name {
                "hand" | "Hand tool" => { st.tool = Tool::Hand; st.status = "Hand".into(); st.show_palette = false; }
                "zoom_in" => {
                    st.cam_height = (st.cam_height * 0.7).max(300.0);
                    st.status = format!("Zoom: {:.0}", st.cam_height);
                }
                "zoom_out" => {
                    st.cam_height = (st.cam_height * 1.4).min(12000.0);
                    st.status = format!("Zoom: {:.0}", st.cam_height);
                }
                "camera" => {
                    // Reset camera to default view
                    st.cam_height = 5000.0;
                    st.status = "Camera reset".into();
                }
                "grid" | "Toggle Grid" => { st.grid = !st.grid; st.status = format!("Grid: {}", if st.grid { "ON" } else { "OFF" }); st.show_palette = false; }
                "Paint tool" => { st.tool = Tool::Paint; st.status = "Paint".into(); st.show_palette = false; }
                "Erase tool" => { st.tool = Tool::Erase; st.status = "Erase".into(); st.show_palette = false; }
                "Pick tool" => { st.tool = Tool::Pick; st.status = "Pick".into(); st.show_palette = false; }
                "Fill tool" => { st.tool = Tool::Fill; st.status = "Fill".into(); st.show_palette = false; }
                "Smooth tool" => { st.tool = Tool::Smooth; st.status = "Smooth".into(); st.show_palette = false; }
                "Select tool" => { st.tool = Tool::Select; st.status = "Select".into(); st.show_palette = false; }
                "Clone tool" => { st.tool = Tool::Clone; st.status = "Clone".into(); st.show_palette = false; }
                "Height tool" => { st.tool = Tool::Raise; st.status = "Height".into(); st.show_palette = false; }
                "Toggle Overlay" => { st.show_overlay = !st.show_overlay; st.show_palette = false; }
                "Reset Camera" => { st.cam_height = 5000.0; st.show_palette = false; }
                "Help" => { st.show_help = !st.show_help; st.show_palette = false; }
                "Save" => { save_terrain(&st, &pal); st.status = "Saved".into(); st.show_palette = false; }
                "Export PNG" => { export_terrain_png(&st, &pal); st.status = "Exported PNG".into(); st.show_palette = false; }
                "Export BIN" => { export_terrain_bin(&st); st.status = "Exported BIN".into(); st.show_palette = false; }
                "Save Project" => { save_project(&st, &br, &pal, &layers); st.status = "Project saved".into(); st.show_palette = false; }
                "Load Project" => { if load_project(&mut st, &mut br, &pal, &mut layers) { st.status = "Loaded".into(); } st.show_palette = false; }
                "Noise Fill" => { noise_fill(&mut st, &pal, &selection); st.status = "Noise filled".into(); st.show_palette = false; }
                "Undo" | "undo_btn" => {
                    st.show_palette = false;
                    if let Some(action) = undo.undo_stack.pop() {
                        for &(idx, old, _new) in &action.changes {
                            st.cells[idx] = old;
                            apply_cell_visual(idx % TERRAIN_RES, idx / TERRAIN_RES, old, &pal, &mut terr, &mut mats, &terrain_idx, &mut mat_cache);
                        }
                        for &(idx, old_h, _) in &action.height_changes { st.heights[idx] = old_h; }
                        st.status = format!("Undo: {}", action.desc);
                        undo.redo_stack.push(action);
                        st.painted = st.cells.iter().filter(|&&v| v > 0).count();
                    }
                }
                "Redo" | "redo_btn" => {
                    st.show_palette = false;
                    if let Some(action) = undo.redo_stack.pop() {
                        for &(idx, _old, new) in &action.changes {
                            st.cells[idx] = new;
                            apply_cell_visual(idx % TERRAIN_RES, idx / TERRAIN_RES, new, &pal, &mut terr, &mut mats, &terrain_idx, &mut mat_cache);
                        }
                        for &(idx, _, new_h) in &action.height_changes { st.heights[idx] = new_h; }
                        st.status = format!("Redo: {}", action.desc);
                        undo.undo_stack.push(action);
                        st.painted = st.cells.iter().filter(|&&v| v > 0).count();
                    }
                }
                "Clear All" => { st.cells = vec![0; TERRAIN_RES*TERRAIN_RES]; st.painted = 0; st.needs_refresh = true; st.show_palette = false; }
                _ => { st.status = format!("{name}"); st.show_palette = false; }
            }
            return;
        }
    }

    // Menu dropdown items (check first since they overlay everything)
    if st.open_menu.is_some() {
        for (rect, action) in &zones.menu_items {
            if rect.contains(pos) {
                if action.contains("Save") && action.contains("Ctrl") { save_terrain(&st, &pal); st.status = format!("Saved ({} cells)", st.painted); }
                else if action.contains("Save Project") { save_project(&st, &br, &pal, &layers); st.status = "Project saved".into(); }
                else if action.contains("Load Project") {
                    if load_project(&mut st, &mut br, &pal, &mut layers) { st.status = "Project loaded".into(); }
                    else { st.status = "No project found".into(); }
                }
                else if action.contains("Export PNG") { export_terrain_png(&st, &pal); st.status = "Exported terrain_export.png".into(); }
                else if action.contains("Export BIN") { export_terrain_bin(&st); st.status = "Exported terrain.bin".into(); }
                else if action.contains("Export Splat") { export_splat_map(&layers); st.status = "Exported terrain_splat.png".into(); }
                else if action.contains("Quit") { std::process::exit(0); }
                else if action.contains("Grid") { st.grid = !st.grid; }
                else if action.contains("3D Model") { st.show_map = !st.show_map; }
                else if action.contains("Shortcuts") { st.show_help = !st.show_help; }
                else if action.contains("Noise") {
                    noise_fill(&mut st, &pal, &selection);
                    st.status = "Noise fill applied".into();
                }
                else if action.contains("Clear") {
                    st.cells = vec![0; TERRAIN_RES * TERRAIN_RES];
                    st.painted = 0;
                    st.status = "Cleared all cells".into();
                }
                st.open_menu = None;
                return;
            }
        }
        // Click outside dropdown = close it
        st.open_menu = None;
        return;
    }

    // Menu buttons (File/Edit/View/Help)
    for (rect, name) in &zones.menu_buttons {
        if rect.contains(pos) {
            st.open_menu = if st.open_menu == Some(name) { None } else { Some(name) };
            return;
        }
    }

    // Tool buttons
    for (rect, tool) in &zones.tool_buttons {
        if rect.contains(pos) {
            st.tool = *tool;
            st.status = match tool {
                Tool::Paint => "Paint", Tool::Erase => "Erase",
                Tool::Pick => "Pick", Tool::Fill => "Fill", Tool::Smooth => "Smooth", Tool::Hand => "Hand", Tool::Select => "Select", Tool::Clone => "Clone", Tool::FloodFill => "Flood", Tool::Raise => "Height",
            }.into();
            return;
        }
    }

    // Size +/- (toolbar)
    if let Some(r) = zones.size_minus { if r.contains(pos) { br.size = (br.size - 1.0).max(1.0); return; } }
    if let Some(r) = zones.size_plus { if r.contains(pos) { br.size = (br.size + 1.0).min(20.0); return; } }

    // Opacity +/- (toolbar)
    if let Some(r) = zones.opacity_minus { if r.contains(pos) { br.opacity = (br.opacity - 0.1).max(0.1); return; } }
    if let Some(r) = zones.opacity_plus { if r.contains(pos) { br.opacity = (br.opacity + 0.1).min(1.0); return; } }

    // Right panel: Size +/-
    if let Some(r) = zones.rp_size_minus { if r.contains(pos) { br.size = (br.size - 1.0).max(1.0); return; } }
    if let Some(r) = zones.rp_size_plus { if r.contains(pos) { br.size = (br.size + 1.0).min(20.0); return; } }

    // Right panel: Opacity +/-
    if let Some(r) = zones.rp_opacity_minus { if r.contains(pos) { br.opacity = (br.opacity - 0.1).max(0.1); return; } }
    if let Some(r) = zones.rp_opacity_plus { if r.contains(pos) { br.opacity = (br.opacity + 0.1).min(1.0); return; } }

    // Right panel: Grid toggle
    if let Some(r) = zones.rp_grid_toggle { if r.contains(pos) { st.grid = !st.grid; return; } }

    // Right panel: 3D Model toggle
    if let Some(r) = zones.rp_model_toggle { if r.contains(pos) { st.show_map = !st.show_map; return; } }

    // Right panel: Overlay toggle
    if let Some(r) = zones.rp_overlay_toggle { if r.contains(pos) { st.show_overlay = !st.show_overlay; return; } }

    // Right panel: Symmetry toggles
    if let Some(r) = zones.rp_sym_x { if r.contains(pos) { br.sym_x = !br.sym_x; return; } }
    if let Some(r) = zones.rp_sym_z { if r.contains(pos) { br.sym_z = !br.sym_z; return; } }

    // Right panel: Falloff buttons
    for (rect, fo) in &zones.rp_falloff_btns {
        if rect.contains(pos) { br.falloff = *fo; return; }
    }

    // Layers
    for (rect, li) in &zones.layer_rows {
        if rect.contains(pos) { layers.active = *li; st.status = format!("Layer: {}", layers.layers[*li].name); return; }
    }
    for (rect, li) in &zones.layer_eye {
        if rect.contains(pos) {
            layers.layers[*li].visible = !layers.layers[*li].visible;
            // Recomposite layers into st.cells
            for i in 0..TERRAIN_RES * TERRAIN_RES {
                let mut val = 0u8;
                for layer in layers.layers.iter().rev() {
                    if layer.visible && layer.cells[i] > 0 { val = layer.cells[i]; break; }
                }
                st.cells[i] = val;
            }
            st.painted = st.cells.iter().filter(|&&v| v > 0).count();
            st.needs_refresh = true;
            return;
        }
    }
    if let Some(r) = zones.layer_add {
        if r.contains(pos) {
            let n = layers.layers.len();
            layers.layers.push(Layer::new(&format!("Layer {}", n + 1)));
            layers.active = n;
            st.status = format!("Added layer {}", n + 1);
            return;
        }
    }

    // Presets
    for (rect, pi) in &zones.preset_btns {
        if rect.contains(pos) {
            if let Some(preset) = presets.0.get(*pi) {
                br.size = preset.size; br.opacity = preset.opacity; br.shape = preset.shape;
                br.falloff = preset.falloff; br.sym_x = preset.sym_x; br.sym_z = preset.sym_z;
                st.status = format!("Preset: {}", preset.name);
            }
            return;
        }
    }

    // Noise settings
    if let Some(r) = zones.noise_scale_minus { if r.contains(pos) { st.noise_scale = (st.noise_scale - 0.005).max(0.005); return; } }
    if let Some(r) = zones.noise_scale_plus { if r.contains(pos) { st.noise_scale = (st.noise_scale + 0.005).min(0.5); return; } }
    if let Some(r) = zones.noise_tex2_btn { if r.contains(pos) { st.noise_tex2 = (st.noise_tex2 + 1) % pal.entries.len().max(1); return; } }

    // Shading mode
    for (rect, mode) in &zones.rp_shading_btns {
        if rect.contains(pos) { st.shading_mode = *mode; return; }
    }

    // Panel toggles (status bar)
    if let Some(r) = zones.toggle_left { if r.contains(pos) { st.show_left_panel = !st.show_left_panel; return; } }
    if let Some(r) = zones.toggle_right { if r.contains(pos) { st.show_right_panel = !st.show_right_panel; return; } }
    if let Some(r) = zones.toggle_browser { if r.contains(pos) { browser.show = !browser.show; return; } }

    // Asset browser breadcrumbs / category filters
    for (rect, path) in &zones.browser_breadcrumbs {
        if rect.contains(pos) {
            if path.is_empty() || path == "ALL" || ["Champions", "Minions", "Props", "Turrets", "Maps", "Particles", "Animations", "Other"].contains(&path.as_str()) {
                // Category filter (not navigation)
                browser.search = if path == "ALL" || path.is_empty() { String::new() } else { path.clone() };
                browser.current_path = "ALL".into();
                browser.loaded = false;
            } else {
                browser.current_path = path.clone();
                browser.loaded = false;
            }
            return;
        }
    }

    // Asset browser
    if let Some(r) = zones.browser_search {
        if r.contains(pos) { browser.search_focused = !browser.search_focused; return; }
    }
    if let Some(r) = zones.browser_back {
        if r.contains(pos) {
            // Go up one directory
            if let Some(parent) = std::path::Path::new(&browser.current_path).parent() {
                browser.current_path = parent.to_string_lossy().to_string();
                browser.loaded = false; // trigger rescan
            }
            return;
        }
    }
    for (rect, idx) in &zones.browser_items {
        if rect.contains(pos) {
            if *idx < browser.entries.len() {
                let is_dir = browser.entries[*idx].is_dir;
                let path = browser.entries[*idx].path.clone();
                let name = browser.entries[*idx].name.clone();
                if is_dir {
                    browser.current_path = path;
                    browser.loaded = false;
                } else {
                    browser.selected = Some(*idx);
                    st.status = format!("Selected: {}", name);
                }
            }
            return;
        }
    }

    // Shape toggles
    if let Some(r) = zones.shape_circle { if r.contains(pos) { br.shape = BrushShape::Circle; return; } }
    if let Some(r) = zones.shape_square { if r.contains(pos) { br.shape = BrushShape::Square; return; } }

    // Search bar focus
    if let Some(r) = zones.search_bar {
        if r.contains(pos) { st.search_focused = true; return; }
    }

    // Click outside search = unfocus
    if st.search_focused { st.search_focused = false; }

    // Category pills
    for (rect, ci) in &zones.category_pills {
        if rect.contains(pos) {
            st.cat_filter = *ci;
            if *ci == 0 { st.filter.clear(); }
            return;
        }
    }

    // Texture rows
    for (rect, tex_idx) in &zones.texture_rows {
        if rect.contains(pos) {
            st.tex = *tex_idx;
            if *tex_idx < pal.entries.len() {
                st.status = format!("Selected: {}", pal.entries[*tex_idx].name);
            }
            return;
        }
    }
}

/// Only check MCP commands every 30 frames to reduce filesystem polling
fn read_mcp_commands(
    mut st: ResMut<EditorState>, pal: Res<TexturePalette>,
    mut terr: Query<&mut MeshMaterial3d<StandardMaterial>, With<TerrainChunk>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    terrain_idx: Res<TerrainIndex>, mut mat_cache: ResMut<MaterialCache>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;
    if *frame_count % 30 != 0 { return; } // only check every 30 frames
    let Ok(content) = std::fs::read_to_string("assets/maps/editor_commands.json") else { return; };
    let _ = std::fs::remove_file("assets/maps/editor_commands.json");
    let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&content) else { return; };
    if cmd.get("action").and_then(|v| v.as_str()) != Some("paint") { return; }
    let cells = cmd.get("cells").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let mut p = 0;
    for cell in &cells {
        let gx = cell.get("gx").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let gz_val = cell.get("gz").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let tex = cell.get("texture").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        if gx >= TERRAIN_RES || gz_val >= TERRAIN_RES || tex >= pal.entries.len() { continue; }
        st.cells[gz_val * TERRAIN_RES + gx] = (tex + 1) as u8;
        let mat = mat_cache.texture_mats.entry(tex).or_insert_with(|| {
            mats.add(StandardMaterial { base_color_texture: Some(pal.entries[tex].image.clone()), perceptual_roughness: 1.0, ..default() })
        }).clone();
        if let Some(&entity) = terrain_idx.0.get(&(gx, gz_val)) {
            if let Ok(mut m) = terr.get_mut(entity) { *m = MeshMaterial3d(mat); }
        }
        p += 1;
    }
    st.painted = st.cells.iter().filter(|&&v| v > 0).count();
    st.status = format!("MCP: {p} cells");
}

// ─── Autosave ───

/// Flood fill: BFS from clicked cell, replaces all connected same-texture cells
fn handle_flood_fill(
    mouse: Res<ButtonInput<MouseButton>>, wins: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut st: ResMut<EditorState>, pal: Res<TexturePalette>,
    mut terr: Query<&mut MeshMaterial3d<StandardMaterial>, With<TerrainChunk>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut undo: ResMut<UndoHistory>, pointer_ui: Res<PointerOverUi>,
    terrain_idx: Res<TerrainIndex>, zones: Res<UiClickZones>,
    mut mat_cache: ResMut<MaterialCache>,
) {
    if st.tool != Tool::FloodFill || !mouse.just_pressed(MouseButton::Left) { return; }
    if pointer_ui.0 || is_over_ui(&wins, &zones) || pal.entries.is_empty() { return; }
    let Ok(w) = wins.single() else { return };
    let Some(cur) = w.cursor_position() else { return };
    let Ok((cam, ctf)) = cam.single() else { return };
    let Ok(ray) = cam.viewport_to_world(ctf, cur) else { return };
    let Some(d) = ray.intersect_plane(Vec3::new(0.0, -108.0, 0.0), InfinitePlane3d::new(Vec3::Y)) else { return };
    let hit = ray.get_point(d);
    let start_gx = (hit.x / TILE_SIZE) as usize;
    let start_gz = (hit.z / TILE_SIZE) as usize;
    if start_gx >= TERRAIN_RES || start_gz >= TERRAIN_RES { return; }

    let target_tex = st.cells[start_gz * TERRAIN_RES + start_gx];
    let new_tex = (st.tex as u8) + 1;
    if target_tex == new_tex { return; } // same texture, nothing to do

    // BFS
    let mut queue = VecDeque::new();
    let mut visited = vec![false; TERRAIN_RES * TERRAIN_RES];
    let mut changes = Vec::new();
    queue.push_back((start_gx, start_gz));
    visited[start_gz * TERRAIN_RES + start_gx] = true;

    while let Some((gx, gz)) = queue.pop_front() {
        if changes.len() >= 4096 { break; } // safety limit
        let idx = gz * TERRAIN_RES + gx;
        let old = st.cells[idx];

        // Match: same texture, or same category if tolerance
        let matches = old == target_tex;
        if !matches { continue; }

        changes.push((idx, old, new_tex));
        st.cells[idx] = new_tex;

        // Update visual
        let fill_mat = mat_cache.texture_mats.entry(st.tex).or_insert_with(|| {
            mats.add(StandardMaterial {
                base_color_texture: Some(pal.entries[st.tex].image.clone()),
                perceptual_roughness: 1.0, ..default()
            })
        }).clone();
        if let Some(&entity) = terrain_idx.0.get(&(gx, gz)) {
            if let Ok(mut m) = terr.get_mut(entity) { *m = MeshMaterial3d(fill_mat); }
        }

        // Expand to neighbors
        for (dx, dz) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = gx as i32 + dx;
            let nz = gz as i32 + dz;
            if nx >= 0 && nx < TERRAIN_RES as i32 && nz >= 0 && nz < TERRAIN_RES as i32 {
                let ni = nz as usize * TERRAIN_RES + nx as usize;
                if !visited[ni] {
                    visited[ni] = true;
                    queue.push_back((nx as usize, nz as usize));
                }
            }
        }
    }

    if !changes.is_empty() {
        let desc = format!("Flood fill {} cells", changes.len());
        if undo.undo_stack.len() >= MAX_UNDO { undo.undo_stack.remove(0); }
        undo.undo_stack.push(UndoAction { changes: changes.iter().map(|&(i, o, n)| (i, o, n)).collect(), height_changes: Vec::new(), desc });
        undo.redo_stack.clear();
        st.painted = st.cells.iter().filter(|&&v| v > 0).count();
        st.status = format!("Filled {} cells", changes.len());
    }
}

/// Noise fill: distribute textures using Perlin noise over selection or whole map
fn noise_fill(st: &mut EditorState, _pal: &TexturePalette, selection: &Selection) {
    let perlin = Perlin::new(42);
    let scale = st.noise_scale as f64;
    let tex1 = (st.tex as u8) + 1;
    let tex2 = (st.noise_tex2 as u8) + 1;

    let (sx, sz, ex, ez) = selection.rect.unwrap_or((0, 0, TERRAIN_RES - 1, TERRAIN_RES - 1));

    for gz in sz..=ez {
        for gx in sx..=ex {
            let n = perlin.get([gx as f64 * scale, gz as f64 * scale]);
            let idx = gz * TERRAIN_RES + gx;
            st.cells[idx] = if n > 0.0 { tex1 } else { tex2 };
        }
    }
    st.painted = st.cells.iter().filter(|&&v| v > 0).count();
    st.needs_refresh = true;
}

/// Sync active layer cells into st.cells when painting writes to layers
#[allow(dead_code)]
fn sync_layer_to_cells(
    mut st: ResMut<EditorState>, layers: Res<LayerStack>,
) {
    // Composite: top-down, first non-zero from visible layers wins
    for i in 0..TERRAIN_RES * TERRAIN_RES {
        let mut val = 0u8;
        for layer in layers.layers.iter().rev() {
            if layer.visible && layer.cells[i] > 0 {
                val = layer.cells[i];
                break;
            }
        }
        st.cells[i] = val;
    }
    st.painted = st.cells.iter().filter(|&&v| v > 0).count();
}

/// Refresh ALL terrain chunk visuals from st.cells when needs_refresh is set
fn refresh_all_terrain(
    mut st: ResMut<EditorState>, pal: Res<TexturePalette>,
    mut terr: Query<(&TerrainChunk, &mut MeshMaterial3d<StandardMaterial>)>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut mat_cache: ResMut<MaterialCache>,
) {
    if !st.needs_refresh { return; }
    st.needs_refresh = false;
    for (chunk, mut m) in &mut terr {
        let idx = chunk.gz * TERRAIN_RES + chunk.gx;
        let v = st.cells[idx];
        if v > 0 && (v as usize - 1) < pal.entries.len() {
            let ti = (v - 1) as usize;
            let mat = mat_cache.texture_mats.entry(ti).or_insert_with(|| {
                mats.add(StandardMaterial { base_color_texture: Some(pal.entries[ti].image.clone()), perceptual_roughness: 1.0, ..default() })
            }).clone();
            *m = MeshMaterial3d(mat);
        } else {
            let ck = mat_cache.checker_dark.get_or_insert_with(|| {
                mats.add(StandardMaterial { base_color: Color::srgb(0.14, 0.14, 0.16), perceptual_roughness: 1.0, ..default() })
            }).clone();
            let cl = mat_cache.checker_light.get_or_insert_with(|| {
                mats.add(StandardMaterial { base_color: Color::srgb(0.18, 0.18, 0.20), perceptual_roughness: 1.0, ..default() })
            }).clone();
            *m = MeshMaterial3d(if (chunk.gx + chunk.gz) % 2 == 0 { ck } else { cl });
        }
    }
}

/// System to apply CategoryColor shading when mode changes
fn apply_shading_mode(
    st: Res<EditorState>, pal: Res<TexturePalette>,
    mut terr: Query<(&TerrainChunk, &mut MeshMaterial3d<StandardMaterial>)>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut mat_cache: ResMut<MaterialCache>,
    mut last_mode: Local<Option<ShadingMode>>,
) {
    if *last_mode == Some(st.shading_mode) { return; }
    *last_mode = Some(st.shading_mode);

    for (chunk, mut m) in &mut terr {
        let idx = chunk.gz * TERRAIN_RES + chunk.gx;
        let v = st.cells[idx];
        match st.shading_mode {
            ShadingMode::Textured => {
                if v > 0 && (v as usize - 1) < pal.entries.len() {
                    let ti = (v - 1) as usize;
                    let mat = mat_cache.texture_mats.entry(ti).or_insert_with(|| {
                        mats.add(StandardMaterial { base_color_texture: Some(pal.entries[ti].image.clone()), perceptual_roughness: 1.0, ..default() })
                    }).clone();
                    *m = MeshMaterial3d(mat);
                } else {
                    let c = if (chunk.gx + chunk.gz) % 2 == 0 { 0.14 } else { 0.18 };
                    *m = MeshMaterial3d(mats.add(StandardMaterial { base_color: Color::srgb(c, c, c + 0.02), perceptual_roughness: 1.0, ..default() }));
                }
            }
            ShadingMode::CategoryColor => {
                if v > 0 && (v as usize - 1) < pal.entries.len() {
                    let (r, g, b) = cat_color(&pal.entries[(v - 1) as usize].cat);
                    *m = MeshMaterial3d(mats.add(StandardMaterial {
                        base_color: Color::srgb(r as f32 / 255.0 + 0.15, g as f32 / 255.0 + 0.15, b as f32 / 255.0 + 0.15),
                        perceptual_roughness: 1.0, ..default()
                    }));
                } else {
                    let c = if (chunk.gx + chunk.gz) % 2 == 0 { 0.10 } else { 0.14 };
                    *m = MeshMaterial3d(mats.add(StandardMaterial { base_color: Color::srgb(c, c, c), perceptual_roughness: 1.0, ..default() }));
                }
            }
        }
    }
}

fn apply_heights(
    st: Res<EditorState>,
    mut chunks: Query<(&TerrainChunk, &mut Transform)>,
) {
    // Only run when heights have been modified (tool == Raise or needs_refresh)
    if st.tool != Tool::Raise && !st.needs_refresh { return; }
    for (chunk, mut tf) in &mut chunks {
        let idx = chunk.gz * TERRAIN_RES + chunk.gx;
        let h = st.heights[idx];
        let target_y = -108.0 + h;
        if (tf.translation.y - target_y).abs() > 0.01 {
            tf.translation.y = target_y;
        }
    }
}

/// Handle Alt+click to set clone source
fn handle_clone_source(
    mouse: Res<ButtonInput<MouseButton>>, keys: Res<ButtonInput<KeyCode>>,
    wins: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut st: ResMut<EditorState>, pointer_ui: Res<PointerOverUi>, zones: Res<UiClickZones>,
) {
    if st.tool != Tool::Clone || !keys.pressed(KeyCode::AltLeft) || !mouse.just_pressed(MouseButton::Left) { return; }
    if pointer_ui.0 || is_over_ui(&wins, &zones) { return; }
    let Ok(w) = wins.single() else { return };
    let Some(cur) = w.cursor_position() else { return };
    let Ok((cam, ctf)) = cam.single() else { return };
    let Ok(ray) = cam.viewport_to_world(ctf, cur) else { return };
    let Some(d) = ray.intersect_plane(Vec3::new(0.0, -108.0, 0.0), InfinitePlane3d::new(Vec3::Y)) else { return };
    let hit = ray.get_point(d);
    st.clone_source = Some(((hit.x / TILE_SIZE) as i32, (hit.z / TILE_SIZE) as i32));
    st.status = format!("Clone source set at ({:.0}, {:.0})", hit.x, hit.z);
}

fn handle_selection(
    mouse: Res<ButtonInput<MouseButton>>, wins: Query<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    st: Res<EditorState>, mut selection: ResMut<Selection>,
    pointer_ui: Res<PointerOverUi>, zones: Res<UiClickZones>,
) {
    if st.tool != Tool::Select || pointer_ui.0 || is_over_ui(&wins, &zones) { return; }
    let Ok(w) = wins.single() else { return };
    let Some(cur) = w.cursor_position() else { return };
    let Ok((cam, ctf)) = cam.single() else { return };
    let Ok(ray) = cam.viewport_to_world(ctf, cur) else { return };
    let Some(d) = ray.intersect_plane(Vec3::new(0.0, -108.0, 0.0), InfinitePlane3d::new(Vec3::Y)) else { return };
    let hit = ray.get_point(d);
    let gx = (hit.x / TILE_SIZE).clamp(0.0, (TERRAIN_RES - 1) as f32) as usize;
    let gz = (hit.z / TILE_SIZE).clamp(0.0, (TERRAIN_RES - 1) as f32) as usize;

    if mouse.just_pressed(MouseButton::Left) {
        selection.dragging = Some((gx, gz));
    }
    if mouse.pressed(MouseButton::Left) {
        if let Some((sx, sz)) = selection.dragging {
            selection.rect = Some((sx.min(gx), sz.min(gz), sx.max(gx), sz.max(gz)));
        }
    }
    if mouse.just_released(MouseButton::Left) {
        selection.dragging = None;
    }
}

/// Handle keyboard input for search bar when focused
fn handle_search_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut char_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut st: ResMut<EditorState>, mut browser: ResMut<AssetBrowser>,
) {
    let active = st.search_focused || browser.search_focused || st.show_palette;
    if !active { return; }

    if keys.just_pressed(KeyCode::Escape) {
        st.search_focused = false;
        browser.search_focused = false;
        st.show_palette = false;
        return;
    }

    // Determine which search field to edit
    let target = if st.show_palette { &mut st.palette_query }
        else if browser.search_focused { &mut browser.search }
        else { &mut st.filter };

    if keys.just_pressed(KeyCode::Backspace) { target.pop(); return; }

    for ev in char_events.read() {
        if ev.state != bevy::input::ButtonState::Pressed { continue; }
        if let bevy::input::keyboard::Key::Character(ref ch) = ev.logical_key {
            if ch.len() == 1 {
                let c = ch.chars().next().unwrap();
                if c.is_alphanumeric() || c == '_' || c == ' ' || c == '.' {
                    target.push(c);
                }
            }
        }
    }
}

fn autosave_system(mut st: ResMut<EditorState>, pal: Res<TexturePalette>, time: Res<Time>) {
    st.autosave_timer += time.delta_secs();
    // Autosave every 60 seconds if there are painted cells
    if st.autosave_timer > 60.0 && st.painted > 0 {
        st.autosave_timer = 0.0;
        save_terrain(&st, &pal);
        st.last_save_time = time.elapsed_secs();
        st.status = format!("{} Auto-saved ({} cells)", li(Icon::Save), st.painted);
    }
}

// ─── Hover tracking for texture list ───

fn hover_tracking(
    wins: Query<&Window, With<PrimaryWindow>>,
    mut st: ResMut<EditorState>, zones: Res<UiClickZones>,
) {
    let Ok(w) = wins.single() else { return; };
    let Some(cursor) = w.cursor_position() else { st.hovered_tex = None; return; };
    let pos = egui::pos2(cursor.x, cursor.y);
    st.hovered_tex = None;
    for (rect, tex_idx) in &zones.texture_rows {
        if rect.contains(pos) {
            st.hovered_tex = Some(*tex_idx);
            break;
        }
    }
}

/// Build spatial index once when terrain chunks are spawned
/// Scan asset directories and populate the asset browser
fn load_asset_browser(mut browser: ResMut<AssetBrowser>) {
    if browser.loaded { return; }
    browser.loaded = true;

    fn scan_dir(path: &str, entries: &mut Vec<AssetEntry>) {
        let Ok(dir) = std::fs::read_dir(path) else { return; };
        let mut items: Vec<AssetEntry> = Vec::new();
        for entry in dir.flatten() {
            let meta = entry.metadata().ok();
            let name = entry.file_name().to_string_lossy().to_string();
            let full_path = entry.path().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

            if is_dir {
                items.push(AssetEntry {
                    name: name.clone(), path: full_path, is_dir: true,
                    size: 0, category: "Folder".into(), thumb_stem: String::new(),
                });
            } else if name.ends_with(".glb") || name.ends_with(".gltf") {
                let size = meta.map(|m| m.len()).unwrap_or(0);
                let cat = if full_path.contains("champion") { "Champions" }
                    else if full_path.contains("minion") { "Minions" }
                    else if full_path.contains("prop") { "Props" }
                    else if full_path.contains("map") { "Maps" }
                    else { "Other" };
                let stem = name.strip_suffix(".glb").or(name.strip_suffix(".gltf")).unwrap_or(&name).to_string();
                items.push(AssetEntry {
                    name, path: full_path, is_dir: false,
                    size, category: cat.into(), thumb_stem: stem,
                });
            }
        }
        items.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir) // dirs first
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        entries.extend(items);
    }

    let path = browser.current_path.clone();
    browser.entries.clear();

    if path == "ALL" {
        // Scan ALL 3D model files recursively from the entire project tree
        fn scan_recursive(path: &str, entries: &mut Vec<AssetEntry>) {
            let Ok(dir) = std::fs::read_dir(path) else { return; };
            for entry in dir.flatten() {
                let meta = entry.metadata().ok();
                let name = entry.file_name().to_string_lossy().to_string();
                let full_path = entry.path().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

                if is_dir {
                    scan_recursive(&full_path, entries);
                } else {
                    let is_3d = name.ends_with(".glb") || name.ends_with(".gltf")
                        || name.ends_with(".skn") || name.ends_with(".scb") || name.ends_with(".sco")
                        || name.ends_with(".wgeo") || name.ends_with(".mapgeo")
                        || name.ends_with(".nvr") || name.ends_with(".anm") || name.ends_with(".blend");
                    if !is_3d { continue; }
                    let size = meta.map(|m| m.len()).unwrap_or(0);
                    if size < 50 { continue; }
                    let cat = if full_path.contains("champion") || full_path.contains("characters") { "Champions" }
                        else if full_path.contains("minion") { "Minions" }
                        else if full_path.contains("prop") || full_path.contains("levelprop") { "Props" }
                        else if full_path.contains("map") || full_path.contains("terrain") { "Maps" }
                        else if full_path.contains("particle") || full_path.contains("shared/particle") { "Particles" }
                        else if full_path.contains("turret") { "Turrets" }
                        else if full_path.contains("anm") || name.ends_with(".anm") { "Animations" }
                        else { "Other" };
                    let stem = name.rsplit('.').skip(1).next().unwrap_or(&name).to_string();
                    entries.push(AssetEntry {
                        name, path: full_path, is_dir: false,
                        size, category: cat.into(), thumb_stem: stem,
                    });
                }
            }
        }

        // Scan everything
        scan_recursive("/media/louisdelez/SSD500/Workflows/LeagueOfLegends/project", &mut browser.entries);
        scan_recursive("/media/louisdelez/SSD500/Workflows/LeagueOfLegends/assets", &mut browser.entries);

        // Deduplicate by name (keep largest)
        let mut best: HashMap<String, AssetEntry> = HashMap::new();
        for e in browser.entries.drain(..) {
            let existing = best.get(&e.name);
            if existing.is_none() || existing.unwrap().size < e.size {
                best.insert(e.name.clone(), e);
            }
        }
        browser.entries = best.into_values().collect();
    } else {
        scan_dir(&path, &mut browser.entries);
    }

    browser.entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir)
            .then(a.category.cmp(&b.category))
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}

fn build_terrain_index(
    chunks: Query<(Entity, &TerrainChunk), Added<TerrainChunk>>,
    mut idx: ResMut<TerrainIndex>,
) {
    for (entity, chunk) in &chunks {
        idx.0.insert((chunk.gx, chunk.gz), entity);
    }
}

/// Compute pointer-over-UI once per frame, used by painting + camera
fn update_pointer_over_ui(
    wins: Query<&Window, With<PrimaryWindow>>,
    zones: Res<UiClickZones>, mut pui: ResMut<PointerOverUi>,
) {
    pui.0 = is_over_ui(&wins, &zones);
}
