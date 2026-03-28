use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::ecs::message::MessageReader;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

const MAP_SIZE: f32 = 15398.0;
const TERRAIN_RES: usize = 128;
const TILE_SIZE: f32 = MAP_SIZE / TERRAIN_RES as f32;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.17)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shadow Grove — Map Editor".into(),
                resolution: bevy::window::WindowResolution::new(1920, 1080),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(EditorState::default())
        .insert_resource(BrushSettings::default())
        .insert_resource(TexturePalette::default())
        .add_systems(Startup, (setup_camera, setup_terrain, load_map_glb))
        .add_systems(Update, (
            editor_ui, camera_controls, handle_painting,
            update_brush_preview, handle_shortcuts, draw_grid, read_mcp_commands,
        ))
        .run();
}

#[derive(Resource)] struct EditorState { tex: usize, cells: Vec<u8>, tool: Tool, grid: bool, show_map: bool, cursor: Vec3, painted: usize, status: String, filter: String }
#[derive(Clone, Copy, PartialEq)] enum Tool { Paint, Erase, Pick }
impl Default for EditorState { fn default() -> Self { Self { tex: 0, cells: vec![0; TERRAIN_RES*TERRAIN_RES], tool: Tool::Paint, grid: true, show_map: true, cursor: Vec3::ZERO, painted: 0, status: "Ready".into(), filter: String::new() } } }
#[derive(Resource)] struct BrushSettings { size: f32, opacity: f32 }
impl Default for BrushSettings { fn default() -> Self { Self { size: 3.0, opacity: 1.0 } } }
#[derive(Resource)] struct TexturePalette { entries: Vec<TexEntry> }
struct TexEntry { name: String, desc: String, cat: String, image: Handle<Image> }
impl Default for TexturePalette { fn default() -> Self { Self { entries: vec![] } } }
#[derive(Component)] struct EditorCamera;
#[derive(Component)] struct TerrainChunk { gx: usize, gz: usize }
#[derive(Component)] struct BrushPreview;
#[derive(Component)] struct MapModel;

fn setup_camera(mut cmd: Commands) {
    cmd.spawn((Camera3d::default(), Transform::from_xyz(MAP_SIZE/2.0, 5000.0, MAP_SIZE/2.0+2500.0).looking_at(Vec3::new(MAP_SIZE/2.0, 0.0, MAP_SIZE/2.0), Vec3::Y), EditorCamera));
    cmd.spawn((DirectionalLight { illuminance: 12000.0, shadows_enabled: false, ..default() }, Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.3, 0.0))));
    cmd.spawn(DirectionalLight { illuminance: 4000.0, shadows_enabled: false, ..default() });
}

fn load_map_glb(mut cmd: Commands, srv: Res<AssetServer>) {
    cmd.spawn((SceneRoot(srv.load(bevy::gltf::GltfAssetLabel::Scene(0).from_asset("maps/twisted_treeline_patched.glb"))), Transform::default(), MapModel));
}

fn setup_terrain(mut cmd: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>, srv: Res<AssetServer>, mut pal: ResMut<TexturePalette>) {
    let texs: Vec<(&str,&str,&str)> = vec![
        ("tile_lanetile_crackedstone_01","Pierre fissuree","Lane"),("tile_lanetile_crackedrubble_01","Debris pierre","Lane"),
        ("structure_damge_tile_01","Dalle endommagee","Lane"),("decal_mud_path_01","Chemin boue","Lane"),
        ("structure_pebbles","Cailloux","Lane"),("tile_mud_cracked_01","Boue craquelee","Transition"),
        ("nature_dirt_skirt","Bordure terre","Transition"),("tile_roots_01","Racines sombres","Jungle"),
        ("tile_vegetation_deadmossy_02","Mousse morte","General"),("decal_grass_tufts_02","Touffes herbe","General"),
        ("tile_mud_and_wall_03","Boue et mur","Jungle"),("tile_roots_nastycurling_01","Racines tordues","Jungle"),
        ("nature_spider_den_floor","Sol araignee","Vilemaw"),("structure_base_platform_01","Plateforme bleue","Base"),
        ("structure_base_platform_02","Plateforme rouge","Base"),("structure_base_nexus_grnd_04","Sol nexus","Base"),
        ("structure_base_inhibs_grnd_05","Sol inhibiteur","Base"),("structure_shrine_base_02","Base autel","Autel"),
        ("decal_shrine_base","Decal autel","Autel"),("nature_spider_den_webs","Toiles araignee","Vilemaw"),
        ("structure_walls_broken","Murs casses","Structure"),("structure_ground_steps","Marches","Structure"),
        ("tile_vertical_dirt_02","Terre verticale","Transition"),("structure_lanetrim_01","Bordure lane","Lane"),
    ];
    for (n,d,c) in &texs { pal.entries.push(TexEntry { name: n.to_string(), desc: d.to_string(), cat: c.to_string(), image: srv.load(format!("maps/textures/{}.png",n)) }); }
    let cm = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(TILE_SIZE/2.0)));
    let d = mats.add(StandardMaterial { base_color: Color::srgb(0.16,0.16,0.18), perceptual_roughness: 1.0, ..default() });
    let l = mats.add(StandardMaterial { base_color: Color::srgb(0.20,0.20,0.22), perceptual_roughness: 1.0, ..default() });
    for gz in 0..TERRAIN_RES { for gx in 0..TERRAIN_RES {
        let m = if (gx+gz)%2==0 { d.clone() } else { l.clone() };
        cmd.spawn((Mesh3d(cm.clone()), MeshMaterial3d(m), Transform::from_xyz(gx as f32*TILE_SIZE+TILE_SIZE/2.0, -108.0, gz as f32*TILE_SIZE+TILE_SIZE/2.0), TerrainChunk{gx,gz}));
    }}
    let bm = meshes.add(Torus::new(40.0,50.0));
    let bmat = mats.add(StandardMaterial { base_color: Color::srgba(1.0,0.9,0.2,0.6), emissive: bevy::color::LinearRgba::rgb(2.0,1.8,0.2), alpha_mode: AlphaMode::Blend, ..default() });
    cmd.spawn((Mesh3d(bm), MeshMaterial3d(bmat), Transform::from_xyz(0.0,-100.0,0.0), BrushPreview));
}

fn editor_ui(mut ctx: EguiContexts, mut st: ResMut<EditorState>, mut br: ResMut<BrushSettings>, pal: Res<TexturePalette>, mut map_vis: Query<&mut Visibility, With<MapModel>>, time: Res<Time>) {
    // Skip first frames while egui initializes
    if time.elapsed_secs() < 0.5 { return; }
    let Ok(c) = ctx.ctx_mut() else { return };
    let mut sty = (*c.style()).clone();
    sty.visuals = egui::Visuals::dark();
    sty.visuals.panel_fill = egui::Color32::from_rgb(30,30,34);
    sty.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40,40,45);
    sty.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(55,55,65);
    sty.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(45,80,140);
    sty.visuals.selection.bg_fill = egui::Color32::from_rgb(40,80,150);
    c.set_style(sty);

    egui::TopBottomPanel::top("menu").show(c, |ui| { egui::menu::bar(ui, |ui| {
        ui.colored_label(egui::Color32::from_rgb(200,170,50), "\u{2b22} SHADOW GROVE");
        ui.separator();
        ui.menu_button("File", |ui| { if ui.button("\u{1f4be} Save").clicked() {} if ui.button("\u{1f4e4} Export").clicked() {} ui.separator(); if ui.button("Quit").clicked() { std::process::exit(0); } });
        ui.menu_button("Edit", |ui| { ui.button("Undo"); ui.button("Redo"); });
        ui.menu_button("View", |ui| { ui.checkbox(&mut st.grid, "Grid"); ui.checkbox(&mut st.show_map, "Map Model"); });
        ui.menu_button("Tools", |ui| { if ui.button("\u{1f58c} Paint (B)").clicked(){st.tool=Tool::Paint;} if ui.button("\u{1f6ab} Erase (X)").clicked(){st.tool=Tool::Erase;} if ui.button("\u{1f4a7} Pick (I)").clicked(){st.tool=Tool::Pick;} });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("{:.0}%", st.painted as f32/(TERRAIN_RES*TERRAIN_RES) as f32*100.0));
            ui.separator(); ui.label(format!("({:.0}, {:.0})", st.cursor.x, st.cursor.z));
        });
    }); });

    egui::TopBottomPanel::top("toolbar").min_height(34.0).show(c, |ui| { ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (label, tool) in [("\u{1f58c} Paint", Tool::Paint), ("\u{1f6ab} Erase", Tool::Erase), ("\u{1f4a7} Pick", Tool::Pick)] {
            if ui.add(egui::Button::new(label).fill(if tool==st.tool { egui::Color32::from_rgb(40,80,150) } else { egui::Color32::from_rgb(45,45,50) }).min_size(egui::vec2(75.0,26.0))).clicked() { st.tool = tool; }
        }
        ui.separator();
        ui.label("Size:"); ui.add(egui::Slider::new(&mut br.size, 1.0..=15.0).integer());
        ui.separator();
        ui.label("Opacity:"); ui.add(egui::Slider::new(&mut br.opacity, 0.1..=1.0).fixed_decimals(1));
        ui.separator();
        if st.tex < pal.entries.len() { ui.colored_label(egui::Color32::from_rgb(200,170,50), &pal.entries[st.tex].name); }
    }); });

    egui::TopBottomPanel::bottom("status").min_height(22.0).show(c, |ui| { ui.horizontal(|ui| {
        ui.label(format!("{} | Brush: {:.0} | {}x{}", match st.tool { Tool::Paint=>"Paint", Tool::Erase=>"Erase", Tool::Pick=>"Pick" }, br.size, TERRAIN_RES, TERRAIN_RES));
        ui.separator(); ui.label(&st.status);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label("Map Editor v0.3"); });
    }); });

    egui::SidePanel::left("textures").default_width(250.0).show(c, |ui| {
        ui.heading("\u{1f3a8} Textures");
        ui.separator();
        ui.horizontal(|ui| { ui.label("\u{1f50d}"); ui.text_edit_singleline(&mut st.filter); });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            for cat in ["All","Lane","Jungle","Vilemaw","Base","Autel","Transition","Structure","General"] {
                if ui.selectable_label(st.filter==cat||(st.filter.is_empty()&&cat=="All"), cat).clicked() {
                    st.filter = if cat=="All" { String::new() } else { cat.into() };
                }
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let f = st.filter.to_lowercase();
            for (i, e) in pal.entries.iter().enumerate() {
                if !f.is_empty() && !e.name.to_lowercase().contains(&f) && !e.cat.to_lowercase().contains(&f) { continue; }
                let sel = i == st.tex;
                let fr = egui::Frame::none().fill(if sel { egui::Color32::from_rgb(40,80,150) } else { egui::Color32::TRANSPARENT }).inner_margin(4.0).rounding(3.0);
                let resp = fr.show(ui, |ui| { ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(90,90,100), format!("{:2}", i+1));
                    let (r,g,b) = match e.cat.as_str() { "Lane"=>(120,125,140),"Jungle"=>(50,55,60),"Vilemaw"=>(40,45,40),"Base"=>(60,70,90),"Autel"=>(80,70,60),"Transition"=>(70,72,80),"Structure"=>(90,85,100),_=>(80,80,85) };
                    let (rect,_) = ui.allocate_exact_size(egui::vec2(20.0,20.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, egui::Color32::from_rgb(r,g,b));
                    ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(55,55,60)), egui::StrokeKind::Outside);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&e.name).size(11.0).color(if sel { egui::Color32::WHITE } else { egui::Color32::from_rgb(195,195,200) }));
                        ui.label(egui::RichText::new(format!("{} \u{2022} {}", e.desc, e.cat)).size(9.0).color(egui::Color32::from_rgb(110,110,120)));
                    });
                }); });
                if resp.response.interact(egui::Sense::click()).clicked() { st.tex = i; st.status = format!("Selected: {}", e.name); }
            }
        });
    });

    egui::SidePanel::right("props").default_width(230.0).show(c, |ui| {
        ui.heading("\u{1f58c} Brush");
        ui.separator();
        egui::Grid::new("bg").num_columns(2).spacing([8.0,4.0]).show(ui, |ui| {
            ui.label("Size"); ui.add(egui::DragValue::new(&mut br.size).range(1.0..=20.0)); ui.end_row();
            ui.label("Opacity"); ui.add(egui::DragValue::new(&mut br.opacity).range(0.1..=1.0).speed(0.01).fixed_decimals(2)); ui.end_row();
        });
        ui.add_space(10.0);
        ui.heading("\u{25a3} Selected");
        ui.separator();
        if st.tex < pal.entries.len() {
            let e = &pal.entries[st.tex];
            ui.colored_label(egui::Color32::from_rgb(200,170,50), &e.name);
            ui.label(format!("Cat: {}", e.cat)); ui.label(&e.desc);
            ui.add_space(6.0);
            let (r,g,b) = match e.cat.as_str() { "Lane"=>(120,125,140),"Jungle"=>(50,55,60),"Vilemaw"=>(40,45,40),"Base"=>(60,70,90),"Autel"=>(80,70,60),"Transition"=>(70,72,80),"Structure"=>(90,85,100),_=>(80,80,85) };
            let (rect,_) = ui.allocate_exact_size(egui::vec2(190.0,190.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(r,g,b));
            ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(70,70,80)), egui::StrokeKind::Outside);
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, &e.name, egui::FontId::proportional(11.0), egui::Color32::WHITE);
        }
        ui.add_space(10.0);
        ui.heading("\u{1f5fa} Map");
        ui.separator();
        egui::Grid::new("mi").num_columns(2).spacing([8.0,2.0]).show(ui, |ui| {
            ui.label("Size"); ui.label("15398"); ui.end_row();
            ui.label("Grid"); ui.label(format!("{}x{}", TERRAIN_RES, TERRAIN_RES)); ui.end_row();
            ui.label("Painted"); ui.label(format!("{}", st.painted)); ui.end_row();
        });
        ui.add_space(10.0);
        ui.heading("\u{1f441} View");
        ui.separator();
        ui.checkbox(&mut st.grid, "Grid");
        let mut sm = st.show_map;
        if ui.checkbox(&mut sm, "Map Model").changed() { st.show_map = sm; }
        for mut v in &mut map_vis { *v = if st.show_map { Visibility::Inherited } else { Visibility::Hidden }; }
        ui.add_space(10.0);
        ui.heading("\u{2328} Keys");
        ui.separator();
        for (k,a) in [("B","Paint"),("X","Erase"),("I","Pick"),("+/-","Size"),("1-9","Texture"),("WASD","Pan"),("Scroll","Zoom"),("Ctrl+S","Save"),("G","Grid")] {
            ui.horizontal(|ui| { ui.colored_label(egui::Color32::from_rgb(140,140,150), k); ui.label(a); });
        }
    });
}

fn camera_controls(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mut scroll: MessageReader<MouseWheel>, mut cam: Query<&mut Transform, With<EditorCamera>>, mut ctx: EguiContexts) {
    let Ok(ectx) = ctx.ctx_mut() else { return }; if ectx.wants_keyboard_input() { return; }
    let Ok(mut tf) = cam.single_mut() else { return };
    let s = 3000.0 * time.delta_secs();
    if keys.pressed(KeyCode::KeyW)||keys.pressed(KeyCode::ArrowUp) { tf.translation.z -= s; }
    if keys.pressed(KeyCode::KeyS)||keys.pressed(KeyCode::ArrowDown) { tf.translation.z += s; }
    if keys.pressed(KeyCode::KeyA)||keys.pressed(KeyCode::ArrowLeft) { tf.translation.x -= s; }
    if keys.pressed(KeyCode::KeyD)||keys.pressed(KeyCode::ArrowRight) { tf.translation.x += s; }
    for ev in scroll.read() { tf.translation.y = (tf.translation.y - ev.y*200.0).clamp(500.0, 10000.0); }
}

fn handle_painting(mouse: Res<ButtonInput<MouseButton>>, wins: Query<&Window, With<PrimaryWindow>>, cam: Query<(&Camera, &GlobalTransform), With<EditorCamera>>, mut st: ResMut<EditorState>, br: Res<BrushSettings>, pal: Res<TexturePalette>, mut terr: Query<(&TerrainChunk, &mut MeshMaterial3d<StandardMaterial>)>, mut mats: ResMut<Assets<StandardMaterial>>, mut ctx: EguiContexts) {
    let Ok(ectx) = ctx.ctx_mut() else { return }; if !mouse.pressed(MouseButton::Left) || ectx.wants_pointer_input() || pal.entries.is_empty() { return; }
    let Ok(w) = wins.single() else { return };
    let Some(cur) = w.cursor_position() else { return };
    let Ok((cam, ctf)) = cam.single() else { return };
    let Ok(ray) = cam.viewport_to_world(ctf, cur) else { return };
    let Some(d) = ray.intersect_plane(Vec3::new(0.0,-108.0,0.0), InfinitePlane3d::new(Vec3::Y)) else { return };
    let hit = ray.get_point(d);
    let cx = (hit.x/TILE_SIZE) as i32; let cz = (hit.z/TILE_SIZE) as i32; let r = br.size as i32;
    let th = pal.entries[st.tex].image.clone();
    for dz in -r..=r { for dx in -r..=r {
        if dx*dx+dz*dz > r*r { continue; }
        let gx=(cx+dx) as usize; let gz=(cz+dz) as usize;
        if gx>=TERRAIN_RES||gz>=TERRAIN_RES { continue; }
        let idx=gz*TERRAIN_RES+gx;
        match st.tool {
            Tool::Paint => { st.cells[idx]=(st.tex as u8)+1; for (ch,mut m) in &mut terr { if ch.gx==gx&&ch.gz==gz { *m=MeshMaterial3d(mats.add(StandardMaterial{base_color_texture:Some(th.clone()),perceptual_roughness:1.0,..default()})); break; } } }
            Tool::Erase => { st.cells[idx]=0; for (ch,mut m) in &mut terr { if ch.gx==gx&&ch.gz==gz { let c=if(gx+gz)%2==0{0.16}else{0.20}; *m=MeshMaterial3d(mats.add(StandardMaterial{base_color:Color::srgb(c,c,c+0.02),perceptual_roughness:1.0,..default()})); break; } } }
            Tool::Pick => { let v=st.cells[idx]; if v>0 { st.tex=(v-1) as usize; } }
        }
    }}
    st.painted = st.cells.iter().filter(|&&v|v>0).count();
}

fn update_brush_preview(wins: Query<&Window, With<PrimaryWindow>>, cam: Query<(&Camera, &GlobalTransform), With<EditorCamera>>, mut bq: Query<&mut Transform, With<BrushPreview>>, br: Res<BrushSettings>, mut st: ResMut<EditorState>, mut ctx: EguiContexts) {
    let Ok(ectx2) = ctx.ctx_mut() else { return }; if ectx2.wants_pointer_input() { return; }
    let Ok(w) = wins.single() else { return };
    let Some(cur) = w.cursor_position() else { return };
    let Ok((cam,ctf)) = cam.single() else { return };
    let Ok(mut bt) = bq.single_mut() else { return };
    if let Ok(ray) = cam.viewport_to_world(ctf, cur) {
        if let Some(d) = ray.intersect_plane(Vec3::new(0.0,-108.0,0.0), InfinitePlane3d::new(Vec3::Y)) {
            let h = ray.get_point(d); bt.translation = Vec3::new(h.x,-104.0,h.z); bt.scale = Vec3::splat(br.size*TILE_SIZE/50.0); st.cursor = h;
        }
    }
}

fn handle_shortcuts(keys: Res<ButtonInput<KeyCode>>, mut st: ResMut<EditorState>, mut br: ResMut<BrushSettings>, pal: Res<TexturePalette>, mut ctx: EguiContexts) {
    let Ok(ectx) = ctx.ctx_mut() else { return }; if ectx.wants_keyboard_input() { return; }
    if keys.just_pressed(KeyCode::KeyB) { st.tool=Tool::Paint; }
    if keys.just_pressed(KeyCode::KeyX) { st.tool=Tool::Erase; }
    if keys.just_pressed(KeyCode::KeyI) { st.tool=Tool::Pick; }
    if keys.just_pressed(KeyCode::KeyG) { st.grid=!st.grid; }
    if keys.just_pressed(KeyCode::Equal)||keys.just_pressed(KeyCode::NumpadAdd) { br.size=(br.size+1.0).min(20.0); }
    if keys.just_pressed(KeyCode::Minus)||keys.just_pressed(KeyCode::NumpadSubtract) { br.size=(br.size-1.0).max(1.0); }
    if keys.just_pressed(KeyCode::PageUp)&&st.tex>0 { st.tex-=1; }
    if keys.just_pressed(KeyCode::PageDown)&&st.tex+1<pal.entries.len() { st.tex+=1; }
    for (k,i) in [(KeyCode::Digit1,0),(KeyCode::Digit2,1),(KeyCode::Digit3,2),(KeyCode::Digit4,3),(KeyCode::Digit5,4),(KeyCode::Digit6,5),(KeyCode::Digit7,6),(KeyCode::Digit8,7),(KeyCode::Digit9,8)] {
        if keys.just_pressed(k)&&i<pal.entries.len() { st.tex=i; }
    }
    if keys.pressed(KeyCode::ControlLeft)&&keys.just_pressed(KeyCode::KeyS) {
        let d: Vec<String> = st.cells.iter().map(|&v| if v>0&&(v as usize-1)<pal.entries.len() { pal.entries[v as usize-1].name.clone() } else { "empty".into() }).collect();
        let j = serde_json::to_string_pretty(&serde_json::json!({"resolution":TERRAIN_RES,"cells":d})).unwrap_or_default();
        std::fs::write("assets/maps/terrain_editor_data.json", &j).ok();
        st.status = "Saved!".into();
    }
}

fn draw_grid(mut gz: Gizmos, st: Res<EditorState>) {
    if !st.grid { return; }
    let y=-107.5; let s=TILE_SIZE*8.0; let mc=Color::srgba(0.35,0.35,0.40,0.4);
    let mut x=0.0; while x<=MAP_SIZE { gz.line(Vec3::new(x,y,0.0),Vec3::new(x,y,MAP_SIZE),mc); x+=s; }
    let mut z=0.0; while z<=MAP_SIZE { gz.line(Vec3::new(0.0,y,z),Vec3::new(MAP_SIZE,y,z),mc); z+=s; }
    let c=MAP_SIZE/2.0;
    gz.line(Vec3::new(c,y,0.0),Vec3::new(c,y,MAP_SIZE),Color::srgba(0.3,0.5,0.3,0.4));
    gz.line(Vec3::new(0.0,y,c),Vec3::new(MAP_SIZE,y,c),Color::srgba(0.5,0.3,0.3,0.4));
    let b=Color::srgba(0.6,0.4,0.2,0.5);
    gz.line(Vec3::new(0.0,y,0.0),Vec3::new(MAP_SIZE,y,0.0),b); gz.line(Vec3::new(MAP_SIZE,y,0.0),Vec3::new(MAP_SIZE,y,MAP_SIZE),b);
    gz.line(Vec3::new(MAP_SIZE,y,MAP_SIZE),Vec3::new(0.0,y,MAP_SIZE),b); gz.line(Vec3::new(0.0,y,MAP_SIZE),Vec3::new(0.0,y,0.0),b);
}

fn read_mcp_commands(mut st: ResMut<EditorState>, pal: Res<TexturePalette>, mut terr: Query<(&TerrainChunk, &mut MeshMaterial3d<StandardMaterial>)>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let Ok(content) = std::fs::read_to_string("assets/maps/editor_commands.json") else { return; };
    let _ = std::fs::remove_file("assets/maps/editor_commands.json");
    let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&content) else { return; };
    if cmd.get("action").and_then(|v|v.as_str()) != Some("paint") { return; }
    let cells = cmd.get("cells").and_then(|v|v.as_array()).cloned().unwrap_or_default();
    let mut p=0;
    for cell in &cells {
        let gx=cell.get("gx").and_then(|v|v.as_u64()).unwrap_or(0) as usize;
        let gz_val=cell.get("gz").and_then(|v|v.as_u64()).unwrap_or(0) as usize;
        let tex=cell.get("texture").and_then(|v|v.as_u64()).unwrap_or(0) as usize;
        if gx>=TERRAIN_RES||gz_val>=TERRAIN_RES||tex>=pal.entries.len() { continue; }
        st.cells[gz_val*TERRAIN_RES+gx]=(tex+1) as u8;
        let th=pal.entries[tex].image.clone();
        for (ch,mut m) in &mut terr { if ch.gx==gx&&ch.gz==gz_val { *m=MeshMaterial3d(mats.add(StandardMaterial{base_color_texture:Some(th.clone()),perceptual_roughness:1.0,..default()})); break; } }
        p+=1;
    }
    st.painted=st.cells.iter().filter(|&&v|v>0).count();
    st.status=format!("MCP: {} cells",p);
}
