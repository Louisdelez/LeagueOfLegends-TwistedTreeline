use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::ecs::message::MessageReader;
use bevy::window::PrimaryWindow;
use std::path::PathBuf;

const MAP_SIZE: f32 = 15398.0;
const TERRAIN_RES: usize = 128;
const TILE_SIZE: f32 = MAP_SIZE / TERRAIN_RES as f32;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.22, 0.22, 0.24)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shadow Grove — Map Editor".into(),
                resolution: bevy::window::WindowResolution::new(1920, 1080),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(EditorState::default())
        .insert_resource(BrushSettings::default())
        .insert_resource(TexturePalette::default())
        .add_systems(Startup, (setup_camera, setup_terrain, setup_full_ui, load_map_glb))
        .add_systems(Update, (
            camera_controls,
            handle_painting,
            update_brush_preview,
            handle_keyboard_shortcuts,
            handle_save_export,
            update_status_bar,
            draw_grid,
            read_mcp_commands,
        ))
        .run();
}

// ─── Colors (dark theme like Blender/Unreal) ───
const BG_DARK: Color = Color::srgb(0.14, 0.14, 0.16);
const BG_PANEL: Color = Color::srgb(0.18, 0.18, 0.20);
const BG_HEADER: Color = Color::srgb(0.10, 0.10, 0.12);
const BG_BUTTON: Color = Color::srgb(0.22, 0.22, 0.25);
const BG_BUTTON_HOVER: Color = Color::srgb(0.30, 0.30, 0.35);
const BG_SELECTED: Color = Color::srgb(0.15, 0.35, 0.60);
const BORDER: Color = Color::srgb(0.08, 0.08, 0.10);
const TEXT_DIM: Color = Color::srgb(0.55, 0.55, 0.58);
const TEXT_NORMAL: Color = Color::srgb(0.78, 0.78, 0.80);
const TEXT_BRIGHT: Color = Color::srgb(0.95, 0.95, 0.97);
const ACCENT_BLUE: Color = Color::srgb(0.25, 0.55, 0.85);
const ACCENT_GOLD: Color = Color::srgb(0.85, 0.70, 0.30);

// ─── Resources ───

#[derive(Resource)]
struct EditorState {
    painting: bool,
    current_texture_idx: usize,
    terrain_textures: Vec<u8>,
    tool: EditorTool,
    show_grid: bool,
    cursor_world: Vec3,
    painted_count: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum EditorTool { Paint, Erase, Eyedrop }

impl Default for EditorState {
    fn default() -> Self {
        Self {
            painting: false,
            current_texture_idx: 0,
            terrain_textures: vec![0; TERRAIN_RES * TERRAIN_RES],
            tool: EditorTool::Paint,
            show_grid: true,
            cursor_world: Vec3::ZERO,
            painted_count: 0,
        }
    }
}

#[derive(Resource)]
struct BrushSettings { radius: f32 }
impl Default for BrushSettings { fn default() -> Self { Self { radius: 3.0 } } }

#[derive(Resource)]
struct TexturePalette { textures: Vec<PaletteEntry> }
struct PaletteEntry { name: String, image: Handle<Image> }
impl Default for TexturePalette { fn default() -> Self { Self { textures: vec![] } } }

// ─── Components ───
#[derive(Component)] struct EditorCamera;
#[derive(Component)] struct TerrainChunk { grid_x: usize, grid_z: usize }
#[derive(Component)] struct BrushPreview;
#[derive(Component)] struct MapModel;
#[derive(Component)] struct StatusBarText;
#[derive(Component)] struct ToolIndicator;
#[derive(Component)] struct TextureListItem(usize);
#[derive(Component)] struct TextureNameLabel;

// ─── Setup ───

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(MAP_SIZE / 2.0, 5000.0, MAP_SIZE / 2.0 + 2500.0)
            .looking_at(Vec3::new(MAP_SIZE / 2.0, 0.0, MAP_SIZE / 2.0), Vec3::Y),
        EditorCamera,
    ));
    commands.spawn((
        DirectionalLight { illuminance: 12000.0, shadows_enabled: false, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.3, 0.0)),
    ));
    commands.spawn(DirectionalLight { illuminance: 4000.0, shadows_enabled: false, ..default() });
}

fn load_map_glb(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneRoot(asset_server.load(bevy::gltf::GltfAssetLabel::Scene(0).from_asset("maps/twisted_treeline_patched.glb"))),
        Transform::default(),
        MapModel,
    ));
}

fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut palette: ResMut<TexturePalette>,
) {
    let texture_files = [
        "tile_lanetile_crackedstone_01.png", "tile_lanetile_crackedrubble_01.png",
        "structure_damge_tile_01.png", "decal_mud_path_01.png", "structure_pebbles.png",
        "tile_mud_cracked_01.png", "nature_dirt_skirt.png", "tile_roots_01.png",
        "tile_vegetation_deadmossy_02.png", "decal_grass_tufts_02.png",
        "tile_mud_and_wall_03.png", "tile_roots_nastycurling_01.png", "nature_spider_den_floor.png",
        "structure_base_platform_01.png", "structure_base_platform_02.png",
        "structure_base_nexus_grnd_04.png", "structure_base_inhibs_grnd_05.png",
        "structure_shrine_base_02.png", "decal_shrine_base.png", "nature_spider_den_webs.png",
        "structure_walls_broken.png", "structure_ground_steps.png",
        "tile_vertical_dirt_02.png", "structure_lanetrim_01.png",
    ];
    for file in &texture_files {
        let path = format!("maps/textures/{}", file);
        palette.textures.push(PaletteEntry {
            name: file.replace(".png", "").to_string(),
            image: asset_server.load(&path),
        });
    }

    let cell_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(TILE_SIZE / 2.0)));
    // Checkerboard pattern like Blender/Unreal
    let dark_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.18, 0.20),
        perceptual_roughness: 1.0, ..default()
    });
    let light_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.22, 0.24),
        perceptual_roughness: 1.0, ..default()
    });
    for gz in 0..TERRAIN_RES {
        for gx in 0..TERRAIN_RES {
            let checker = if (gx + gz) % 2 == 0 { dark_mat.clone() } else { light_mat.clone() };
            commands.spawn((
                Mesh3d(cell_mesh.clone()), MeshMaterial3d(checker),
                Transform::from_xyz(gx as f32 * TILE_SIZE + TILE_SIZE / 2.0, -115.0, gz as f32 * TILE_SIZE + TILE_SIZE / 2.0),
                TerrainChunk { grid_x: gx, grid_z: gz },
            ));
        }
    }

    let brush_mesh = meshes.add(Torus::new(40.0, 50.0));
    let brush_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.9, 0.2, 0.6),
        emissive: bevy::color::LinearRgba::rgb(2.0, 1.8, 0.2),
        alpha_mode: AlphaMode::Blend, ..default()
    });
    commands.spawn((Mesh3d(brush_mesh), MeshMaterial3d(brush_mat), Transform::from_xyz(0.0, -100.0, 0.0), BrushPreview));
}

// ─── Full UI Layout ───

fn setup_full_ui(mut commands: Commands) {
    // Root layout: left panel | center (top bar + viewport) | right panel
    commands.spawn(Node {
        width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column, ..default()
    }).with_children(|root| {
        // === TOP MENU BAR ===
        root.spawn((
            Node {
                width: Val::Percent(100.0), height: Val::Px(32.0),
                flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)), column_gap: Val::Px(4.0),
                border: UiRect::bottom(Val::Px(1.0)), ..default()
            },
            BackgroundColor(BG_HEADER), BorderColor::all(BORDER),
        )).with_children(|bar| {
            // Logo
            bar.spawn((Text::new("SHADOW GROVE"), TextFont { font_size: 13.0, ..default() }, TextColor(ACCENT_GOLD)));
            bar.spawn(Node { width: Val::Px(2.0), height: Val::Px(20.0), margin: UiRect::horizontal(Val::Px(8.0)), ..default() });
            // Menu items
            for label in ["File", "Edit", "View", "Tools", "Help"] {
                bar.spawn((
                    Node { padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)), ..default() },
                )).with_children(|btn| {
                    btn.spawn((Text::new(label.to_string()), TextFont { font_size: 12.0, ..default() }, TextColor(TEXT_NORMAL)));
                });
            }
            bar.spawn(Node { flex_grow: 1.0, ..default() });
            // Tool indicators
            for (label, shortcut) in [("Paint", "B"), ("Erase", "X"), ("Eyedrop", "I")] {
                bar.spawn((
                    Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)), border: UiRect::all(Val::Px(1.0)),
                           margin: UiRect::horizontal(Val::Px(2.0)), ..default() },
                    BackgroundColor(if label == "Paint" { BG_SELECTED } else { BG_BUTTON }),
                    BorderColor::all(BORDER),
                )).with_children(|btn| {
                    btn.spawn((Text::new(format!("{} [{}]", label, shortcut)), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_BRIGHT)));
                });
            }
        });

        // === MAIN AREA: Left Panel + Viewport + Right Panel ===
        root.spawn(Node {
            width: Val::Percent(100.0), flex_grow: 1.0,
            flex_direction: FlexDirection::Row, ..default()
        }).with_children(|main| {
            // === LEFT PANEL: Texture Palette ===
            main.spawn((
                Node {
                    width: Val::Px(220.0), height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column, border: UiRect::right(Val::Px(1.0)),
                    overflow: Overflow::scroll_y(), ..default()
                },
                BackgroundColor(BG_PANEL), BorderColor::all(BORDER),
            )).with_children(|panel| {
                // Panel header
                panel.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(28.0), padding: UiRect::horizontal(Val::Px(8.0)),
                           align_items: AlignItems::Center, border: UiRect::bottom(Val::Px(1.0)), ..default() },
                    BackgroundColor(BG_HEADER), BorderColor::all(BORDER),
                )).with_children(|h| {
                    h.spawn((Text::new("TEXTURES"), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                });

                // Texture list
                let tex_names = [
                    "crackedstone_01", "crackedrubble_01", "dmg_tile_01", "mud_path_01", "pebbles",
                    "mud_cracked_01", "dirt_skirt", "roots_01", "deadmossy_02", "grass_tufts_02",
                    "mud_wall_03", "roots_nasty_01", "spider_floor", "base_platform_01", "base_platform_02",
                    "base_nexus_04", "base_inhibs_05", "shrine_base_02", "shrine_decal", "spider_webs",
                    "walls_broken", "ground_steps", "vertical_dirt", "lanetrim_01",
                ];

                for (i, name) in tex_names.iter().enumerate() {
                    let is_selected = i == 0;
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0), height: Val::Px(26.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row, column_gap: Val::Px(6.0), ..default()
                        },
                        BackgroundColor(if is_selected { BG_SELECTED } else { Color::NONE }),
                        TextureListItem(i),
                    )).with_children(|row| {
                        // Number
                        row.spawn((
                            Text::new(format!("{:2}", i + 1)),
                            TextFont { font_size: 10.0, ..default() },
                            TextColor(TEXT_DIM),
                        ));
                        // Color preview square
                        row.spawn((
                            Node { width: Val::Px(14.0), height: Val::Px(14.0), border: UiRect::all(Val::Px(1.0)), ..default() },
                            BackgroundColor(Color::srgb(0.3 + (i as f32 * 0.02), 0.3, 0.3 + (i as f32 * 0.01))),
                            BorderColor::all(BORDER),
                        ));
                        // Name
                        row.spawn((
                            Text::new(name.to_string()),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(if is_selected { TEXT_BRIGHT } else { TEXT_NORMAL }),
                        ));
                    });
                }
            });

            // === CENTER: Viewport (empty node, 3D renders behind) ===
            main.spawn(Node { flex_grow: 1.0, ..default() });

            // === RIGHT PANEL: Properties ===
            main.spawn((
                Node {
                    width: Val::Px(240.0), height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column, border: UiRect::left(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(BG_PANEL), BorderColor::all(BORDER),
            )).with_children(|panel| {
                // Brush settings header
                panel.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(28.0), padding: UiRect::horizontal(Val::Px(8.0)),
                           align_items: AlignItems::Center, border: UiRect::bottom(Val::Px(1.0)), ..default() },
                    BackgroundColor(BG_HEADER), BorderColor::all(BORDER),
                )).with_children(|h| {
                    h.spawn((Text::new("BRUSH"), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                });

                // Brush size
                panel.spawn(Node { padding: UiRect::all(Val::Px(8.0)), flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), ..default() })
                .with_children(|props| {
                    for (label, value) in [("Size", "3"), ("Strength", "100%"), ("Shape", "Circle")] {
                        props.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, ..default() })
                        .with_children(|row| {
                            row.spawn((Text::new(label.to_string()), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                            row.spawn((
                                Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                                BackgroundColor(BG_BUTTON), BorderColor::all(BORDER),
                            )).with_children(|v| {
                                v.spawn((Text::new(value.to_string()), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_BRIGHT)));
                            });
                        });
                    }
                });

                // Separator
                panel.spawn((Node { width: Val::Percent(100.0), height: Val::Px(1.0), ..default() }, BackgroundColor(BORDER)));

                // Map info header
                panel.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(28.0), padding: UiRect::horizontal(Val::Px(8.0)),
                           align_items: AlignItems::Center, border: UiRect::bottom(Val::Px(1.0)), ..default() },
                    BackgroundColor(BG_HEADER), BorderColor::all(BORDER),
                )).with_children(|h| {
                    h.spawn((Text::new("MAP INFO"), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                });

                panel.spawn(Node { padding: UiRect::all(Val::Px(8.0)), flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() })
                .with_children(|info| {
                    for (label, value) in [("Map Size", "15398 x 15398"), ("Grid", "128 x 128"), ("Cell Size", "120.3"), ("Textures", "24")] {
                        info.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, ..default() })
                        .with_children(|row| {
                            row.spawn((Text::new(label.to_string()), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                            row.spawn((Text::new(value.to_string()), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_NORMAL)));
                        });
                    }
                });

                // Separator
                panel.spawn((Node { width: Val::Percent(100.0), height: Val::Px(1.0), ..default() }, BackgroundColor(BORDER)));

                // Selected texture preview
                panel.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(28.0), padding: UiRect::horizontal(Val::Px(8.0)),
                           align_items: AlignItems::Center, border: UiRect::bottom(Val::Px(1.0)), ..default() },
                    BackgroundColor(BG_HEADER), BorderColor::all(BORDER),
                )).with_children(|h| {
                    h.spawn((Text::new("SELECTED"), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                });

                panel.spawn(Node { padding: UiRect::all(Val::Px(8.0)), flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() })
                .with_children(|sel| {
                    sel.spawn((
                        Text::new("tile_lanetile_crackedstone_01"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(ACCENT_GOLD),
                        TextureNameLabel,
                    ));
                    sel.spawn((Text::new("Type: Sol / Lane"), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                    sel.spawn((Text::new("Tiling: 250 units"), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                });

                panel.spawn(Node { flex_grow: 1.0, ..default() });

                // Shortcuts help
                panel.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(28.0), padding: UiRect::horizontal(Val::Px(8.0)),
                           align_items: AlignItems::Center, border: UiRect::top(Val::Px(1.0)), ..default() },
                    BackgroundColor(BG_HEADER), BorderColor::all(BORDER),
                )).with_children(|h| {
                    h.spawn((Text::new("SHORTCUTS"), TextFont { font_size: 11.0, ..default() }, TextColor(TEXT_DIM)));
                });

                panel.spawn(Node { padding: UiRect::all(Val::Px(6.0)), flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), ..default() })
                .with_children(|help| {
                    for line in ["LMB = Paint", "+/- = Brush size", "PgUp/Dn = Texture", "WASD = Pan camera", "Scroll = Zoom", "Ctrl+S = Save", "E = Export stats"] {
                        help.spawn((Text::new(line.to_string()), TextFont { font_size: 10.0, ..default() }, TextColor(TEXT_DIM)));
                    }
                });
            });
        });

        // === BOTTOM STATUS BAR ===
        root.spawn((
            Node {
                width: Val::Percent(100.0), height: Val::Px(24.0),
                flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(10.0)), column_gap: Val::Px(20.0),
                border: UiRect::top(Val::Px(1.0)), ..default()
            },
            BackgroundColor(BG_HEADER), BorderColor::all(BORDER),
        )).with_children(|bar| {
            bar.spawn((
                Text::new("Ready | Painted: 0/16384 (0%) | Tool: Paint | Brush: 3"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(TEXT_DIM),
                StatusBarText,
            ));
        });
    });
}

// ─── Camera Controls ───

fn camera_controls(
    time: Res<Time>, keys: Res<ButtonInput<KeyCode>>,
    mut scroll: MessageReader<MouseWheel>,
    mut camera: Query<&mut Transform, With<EditorCamera>>,
) {
    let Ok(mut tf) = camera.single_mut() else { return };
    let dt = time.delta_secs();
    let speed = 3000.0 * dt;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) { tf.translation.z -= speed; }
    if keys.pressed(KeyCode::ArrowDown) { tf.translation.z += speed; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) { tf.translation.x -= speed; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { tf.translation.x += speed; }
    for ev in scroll.read() {
        tf.translation.y -= ev.y * 200.0;
        tf.translation.y = tf.translation.y.clamp(500.0, 10000.0);
    }
}

// ─── Painting ───

fn handle_painting(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut state: ResMut<EditorState>,
    brush: Res<BrushSettings>,
    palette: Res<TexturePalette>,
    mut terrain: Query<(&TerrainChunk, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse.pressed(MouseButton::Left) { return; }
    if palette.textures.is_empty() { return; }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };

    // Don't paint if cursor is over UI panels (left 220px or right 240px)
    if cursor.x < 220.0 || cursor.x > window.width() - 240.0 || cursor.y < 32.0 || cursor.y > window.height() - 24.0 { return; }

    let Ok((camera, cam_tf)) = camera.single() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor) else { return };
    let Some(dist) = ray.intersect_plane(Vec3::new(0.0, -115.0, 0.0), InfinitePlane3d::new(Vec3::Y)) else { return };
    let hit = ray.get_point(dist);

    let cx = (hit.x / TILE_SIZE) as i32;
    let cz = (hit.z / TILE_SIZE) as i32;
    let r = brush.radius as i32;
    let tex_idx = state.current_texture_idx;
    let tex_handle = palette.textures[tex_idx].image.clone();

    for dz in -r..=r {
        for dx in -r..=r {
            if dx*dx + dz*dz > r*r { continue; }
            let gx = cx + dx; let gz = cz + dz;
            if gx < 0 || gz < 0 || gx >= TERRAIN_RES as i32 || gz >= TERRAIN_RES as i32 { continue; }
            let idx = gz as usize * TERRAIN_RES + gx as usize;

            if state.tool == EditorTool::Paint {
                if state.terrain_textures[idx] != (tex_idx as u8 + 1) {
                    state.terrain_textures[idx] = tex_idx as u8 + 1;
                    state.painted_count = state.terrain_textures.iter().filter(|&&v| v > 0).count();
                }
                for (chunk, mut mat_handle) in &mut terrain {
                    if chunk.grid_x == gx as usize && chunk.grid_z == gz as usize {
                        *mat_handle = MeshMaterial3d(materials.add(StandardMaterial {
                            base_color_texture: Some(tex_handle.clone()),
                            perceptual_roughness: 1.0, ..default()
                        }));
                        break;
                    }
                }
            } else if state.tool == EditorTool::Erase {
                state.terrain_textures[idx] = 0;
                state.painted_count = state.terrain_textures.iter().filter(|&&v| v > 0).count();
                for (chunk, mut mat_handle) in &mut terrain {
                    if chunk.grid_x == gx as usize && chunk.grid_z == gz as usize {
                        *mat_handle = MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.06, 0.06, 0.08),
                            perceptual_roughness: 1.0, ..default()
                        }));
                        break;
                    }
                }
            }
        }
    }
}

fn update_brush_preview(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut brush_q: Query<&mut Transform, With<BrushPreview>>,
    brush: Res<BrushSettings>,
    mut state: ResMut<EditorState>,
) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_tf)) = camera.single() else { return };
    let Ok(mut brush_tf) = brush_q.single_mut() else { return };
    if let Ok(ray) = camera.viewport_to_world(cam_tf, cursor) {
        if let Some(dist) = ray.intersect_plane(Vec3::new(0.0, -115.0, 0.0), InfinitePlane3d::new(Vec3::Y)) {
            let hit = ray.get_point(dist);
            brush_tf.translation = Vec3::new(hit.x, -108.0, hit.z);
            brush_tf.scale = Vec3::splat(brush.radius * TILE_SIZE / 50.0);
            state.cursor_world = hit;
        }
    }
}

// ─── Keyboard ───

fn handle_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
    mut brush: ResMut<BrushSettings>,
    palette: Res<TexturePalette>,
) {
    // Tool selection
    if keys.just_pressed(KeyCode::KeyB) { state.tool = EditorTool::Paint; }
    if keys.just_pressed(KeyCode::KeyX) { state.tool = EditorTool::Erase; }
    if keys.just_pressed(KeyCode::KeyI) { state.tool = EditorTool::Eyedrop; }

    // Brush size
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        brush.radius = (brush.radius + 1.0).min(20.0);
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        brush.radius = (brush.radius - 1.0).max(1.0);
    }

    // Texture selection
    if keys.just_pressed(KeyCode::PageUp) && state.current_texture_idx > 0 {
        state.current_texture_idx -= 1;
    }
    if keys.just_pressed(KeyCode::PageDown) && state.current_texture_idx + 1 < palette.textures.len() {
        state.current_texture_idx += 1;
    }
    let key_map = [
        (KeyCode::Digit1, 0), (KeyCode::Digit2, 1), (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3), (KeyCode::Digit5, 4), (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6), (KeyCode::Digit8, 7), (KeyCode::Digit9, 8),
        (KeyCode::Digit0, 9),
    ];
    for (key, idx) in key_map {
        if keys.just_pressed(key) && idx < palette.textures.len() {
            state.current_texture_idx = idx;
        }
    }
}

fn handle_save_export(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<EditorState>,
    palette: Res<TexturePalette>,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyS) {
        let data: Vec<String> = state.terrain_textures.iter().map(|&idx| {
            if idx > 0 && (idx as usize - 1) < palette.textures.len() {
                palette.textures[idx as usize - 1].name.clone()
            } else { "empty".to_string() }
        }).collect();
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "resolution": TERRAIN_RES, "tile_size": TILE_SIZE, "map_size": MAP_SIZE, "cells": data,
        })).unwrap_or_default();
        std::fs::write("assets/maps/terrain_editor_data.json", &json).ok();
        println!("Saved terrain data");
    }
    if keys.just_pressed(KeyCode::KeyE) && !keys.pressed(KeyCode::ControlLeft) {
        println!("Painted: {}/{} ({:.0}%)", state.painted_count, TERRAIN_RES * TERRAIN_RES,
            state.painted_count as f32 / (TERRAIN_RES * TERRAIN_RES) as f32 * 100.0);
    }
}

/// Read commands from MCP server via shared JSON file
fn read_mcp_commands(
    mut state: ResMut<EditorState>,
    palette: Res<TexturePalette>,
    mut terrain: Query<(&TerrainChunk, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cmd_path = "assets/maps/editor_commands.json";
    let Ok(content) = std::fs::read_to_string(cmd_path) else { return; };
    // Delete the file immediately so we don't re-read it
    let _ = std::fs::remove_file(cmd_path);

    let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&content) else { return; };

    let action = cmd.get("action").and_then(|v| v.as_str()).unwrap_or("");

    if action == "paint" {
        let cells = cmd.get("cells").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let mut painted = 0;

        for cell in &cells {
            let gx = cell.get("gx").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let gz = cell.get("gz").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let tex_idx = cell.get("texture").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

            if gx >= TERRAIN_RES || gz >= TERRAIN_RES || tex_idx >= palette.textures.len() { continue; }

            let idx = gz * TERRAIN_RES + gx;
            state.terrain_textures[idx] = (tex_idx + 1) as u8;

            let tex_handle = palette.textures[tex_idx].image.clone();
            for (chunk, mut mat_handle) in &mut terrain {
                if chunk.grid_x == gx && chunk.grid_z == gz {
                    *mat_handle = MeshMaterial3d(materials.add(StandardMaterial {
                        base_color_texture: Some(tex_handle.clone()),
                        perceptual_roughness: 1.0, ..default()
                    }));
                    break;
                }
            }
            painted += 1;
        }
        state.painted_count = state.terrain_textures.iter().filter(|&&v| v > 0).count();
        println!("MCP: Painted {} cells", painted);
    } else if action == "save" {
        // Trigger save
        println!("MCP: Save requested");
    }
}

fn draw_grid(
    mut gizmos: Gizmos,
    state: Res<EditorState>,
) {
    if !state.show_grid { return; }

    let grid_color = Color::srgba(0.35, 0.35, 0.38, 0.3);
    let major_color = Color::srgba(0.45, 0.45, 0.50, 0.5);
    let y = -114.5;

    // Major grid lines every 8 cells
    let major_step = TILE_SIZE * 8.0;
    let mut x = 0.0;
    while x <= MAP_SIZE {
        gizmos.line(Vec3::new(x, y, 0.0), Vec3::new(x, y, MAP_SIZE), major_color);
        x += major_step;
    }
    let mut z = 0.0;
    while z <= MAP_SIZE {
        gizmos.line(Vec3::new(0.0, y, z), Vec3::new(MAP_SIZE, y, z), major_color);
        z += major_step;
    }

    // Center axes
    let center = MAP_SIZE / 2.0;
    gizmos.line(Vec3::new(center, y, 0.0), Vec3::new(center, y, MAP_SIZE), Color::srgba(0.3, 0.5, 0.3, 0.4));
    gizmos.line(Vec3::new(0.0, y, center), Vec3::new(MAP_SIZE, y, center), Color::srgba(0.5, 0.3, 0.3, 0.4));

    // Map border
    let border_color = Color::srgba(0.6, 0.4, 0.2, 0.6);
    gizmos.line(Vec3::new(0.0, y, 0.0), Vec3::new(MAP_SIZE, y, 0.0), border_color);
    gizmos.line(Vec3::new(MAP_SIZE, y, 0.0), Vec3::new(MAP_SIZE, y, MAP_SIZE), border_color);
    gizmos.line(Vec3::new(MAP_SIZE, y, MAP_SIZE), Vec3::new(0.0, y, MAP_SIZE), border_color);
    gizmos.line(Vec3::new(0.0, y, MAP_SIZE), Vec3::new(0.0, y, 0.0), border_color);
}

fn update_status_bar(
    state: Res<EditorState>, brush: Res<BrushSettings>, palette: Res<TexturePalette>,
    mut text_q: Query<&mut Text, With<StatusBarText>>,
    mut name_q: Query<&mut Text, (With<TextureNameLabel>, Without<StatusBarText>)>,
) {
    let total = TERRAIN_RES * TERRAIN_RES;
    let pct = state.painted_count as f32 / total as f32 * 100.0;
    let tool = match state.tool {
        EditorTool::Paint => "Paint",
        EditorTool::Erase => "Erase",
        EditorTool::Eyedrop => "Eyedrop",
    };
    let tex_name = if state.current_texture_idx < palette.textures.len() {
        &palette.textures[state.current_texture_idx].name
    } else { "none" };

    if let Ok(mut text) = text_q.single_mut() {
        **text = format!("Painted: {}/{} ({:.0}%) | Tool: {} | Brush: {:.0} | Texture: {} | Pos: ({:.0}, {:.0})",
            state.painted_count, total, pct, tool, brush.radius, tex_name, state.cursor_world.x, state.cursor_world.z);
    }
    if let Ok(mut text) = name_q.single_mut() {
        **text = tex_name.to_string();
    }
}
