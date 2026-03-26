use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::light::GlobalAmbientLight;
use bevy::image::{ImageSampler, ImageSamplerDescriptor, ImageAddressMode, ImageFilterMode};
use sg_core::components::*;
use sg_core::constants::*;
use sg_core::types::*;
use sg_map::layout::MapLayout;

#[derive(Resource)]
pub struct MapData(pub MapLayout);

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapData(MapLayout::twisted_treeline()))
            .insert_resource(MaterialFixApplied(false))
            .add_systems(OnEnter(crate::menu::AppState::InGame), (spawn_map_mesh, spawn_structures, spawn_camp_markers, spawn_props, spawn_lighting))
            // fix_map_materials disabled - textures now fixed in GLB directly
            ;
    }
}

#[derive(Resource)]
struct MaterialFixApplied(bool);

fn spawn_map_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load the real Twisted Treeline 3D map (converted from NVR)
    commands.spawn((
        SceneRoot(asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("maps/tt_blender.glb")
        )),
        Transform::default(),
    ));

    // Dark ground plane below map
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50000.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.02, 0.03),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(7700.0, -500.0, 7000.0),
    ));

    // Dark walls around the map edges (pseudo-skybox)
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.015, 0.02, 0.03),
        unlit: true,
        ..default()
    });
    let wall_mesh = meshes.add(Cuboid::new(40000.0, 6000.0, 10.0));
    // North wall
    commands.spawn((
        Mesh3d(wall_mesh.clone()), MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(7700.0, 0.0, -5000.0),
    ));
    // South wall
    commands.spawn((
        Mesh3d(wall_mesh.clone()), MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(7700.0, 0.0, 20000.0),
    ));
    let wall_mesh_z = meshes.add(Cuboid::new(10.0, 6000.0, 40000.0));
    // East wall
    commands.spawn((
        Mesh3d(wall_mesh_z.clone()), MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(22000.0, 0.0, 7000.0),
    ));
    // West wall
    commands.spawn((
        Mesh3d(wall_mesh_z.clone()), MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(-6000.0, 0.0, 7000.0),
    ));
}

fn spawn_structures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapData>,
) {
    let turret_mesh = meshes.add(Cylinder::new(50.0, 150.0));
    let nexus_mesh = meshes.add(Sphere::new(80.0));
    let inhib_mesh = meshes.add(Cuboid::new(60.0, 80.0, 60.0));

    let blue_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.4, 0.9),
        ..default()
    });
    let red_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.2, 0.2),
        ..default()
    });

    // Turrets — use real SCB models when available
    // Map turret positions to SCB filenames
    let turret_models: Vec<(&str, &str)> = vec![
        // (team_prefix, position_suffix) — Blue=t1, Red=t2
        // Indexes match layout.rs turret order:
        // 0: Blue NexusTurret, 1: Blue InnerTop, 2: Blue OuterTop, 3: Blue InnerBot, 4: Blue OuterBot
        // 5: Red NexusTurret, 6: Red InnerTop, 7: Red OuterTop, 8: Red InnerBot, 9: Red OuterBot
    ];
    let turret_scb_names = [
        "turret_t1_c_01", "turret_t1_c_06", "turret_t1_l_02", "turret_t1_c_07", "turret_t1_r_02",
        "turret_t2_c_01", "turret_t2_l_01", "turret_t2_l_02", "turret_t2_r_01", "turret_t2_r_02",
    ];

    for (i, turret) in map.0.turrets.iter().enumerate() {
        let hp = match turret.structure_type {
            StructureType::OuterTurret => OUTER_TURRET_HP,
            StructureType::InnerTurret => INNER_TURRET_HP,
            StructureType::InhibitorTurret => INHIB_TURRET_HP,
            StructureType::NexusTurret => NEXUS_TURRET_HP,
            _ => OUTER_TURRET_HP,
        };

        let structure_components = (
            Structure {
                structure_type: turret.structure_type,
                team: turret.team,
                lane: Some(turret.lane),
            },
            TeamMember(turret.team),
            Health { current: hp, max: hp, regen: 0.0 },
            CombatStats {
                attack_damage: 152.0, ability_power: 0.0,
                armor: 100.0, magic_resist: 100.0,
                attack_speed: 0.83, move_speed: 0.0,
                crit_chance: 0.0, cdr: 0.0,
                armor_pen_flat: 0.0, armor_pen_pct: 0.0,
                magic_pen_flat: 0.0, magic_pen_pct: 0.0,
                life_steal: 0.0, spell_vamp: 0.0,
            },
            AutoAttackRange(800.0),
        );

        if i < turret_scb_names.len() {
            // Load real SCB model
            let path = format!("models/props/{}.glb#Scene0", turret_scb_names[i]);
            commands.spawn((
                SceneRoot(asset_server.load(&path)),
                Transform::from_xyz(turret.position.x, 0.0, turret.position.y),
                structure_components,
            ));
        } else {
            // Fallback cylinder
            let mat = match turret.team {
                Team::Blue => blue_mat.clone(),
                _ => red_mat.clone(),
            };
            commands.spawn((
                Mesh3d(turret_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_xyz(turret.position.x, 75.0, turret.position.y),
                structure_components,
            ));
        }
    }

    // Nexuses
    for nexus in &map.0.nexuses {
        let mat = match nexus.team {
            Team::Blue => blue_mat.clone(),
            _ => red_mat.clone(),
        };
        commands.spawn((
            Mesh3d(nexus_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(nexus.position.x, 80.0, nexus.position.y),
            Structure {
                structure_type: StructureType::Nexus,
                team: nexus.team,
                lane: None,
            },
            TeamMember(nexus.team),
            Health { current: NEXUS_TURRET_HP, max: NEXUS_TURRET_HP, regen: 1.0 },
        ));
    }

    // Inhibitors
    for inhib in &map.0.inhibitors {
        let mat = match inhib.team {
            Team::Blue => blue_mat.clone(),
            _ => red_mat.clone(),
        };
        commands.spawn((
            Mesh3d(inhib_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(inhib.position.x, 40.0, inhib.position.y),
            Structure {
                structure_type: StructureType::Inhibitor,
                team: inhib.team,
                lane: None,
            },
            TeamMember(inhib.team),
            Health { current: 1500.0, max: 1500.0, regen: 0.0 },
        ));
    }
}

fn spawn_camp_markers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapData>,
) {
    let camp_mesh = meshes.add(Sphere::new(30.0));
    let neutral_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.6, 0.1),
        ..default()
    });
    let vilemaw_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.0, 0.6),
        ..default()
    });

    // Jungle camps
    for camp in &map.0.jungle_camps {
        commands.spawn((
            Mesh3d(camp_mesh.clone()),
            MeshMaterial3d(neutral_mat.clone()),
            Transform::from_xyz(camp.position.x, 30.0, camp.position.y),
        ));
    }

    // Vilemaw
    let vilemaw_mesh = meshes.add(Sphere::new(60.0));
    commands.spawn((
        Mesh3d(vilemaw_mesh),
        MeshMaterial3d(vilemaw_mat),
        Transform::from_xyz(map.0.vilemaw_spawn.x, 60.0, map.0.vilemaw_spawn.y),
    ));

    // Health relic
    let relic_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.9, 0.2),
        emissive: bevy::color::LinearRgba::rgb(0.0, 2.0, 0.0),
        ..default()
    });
    for relic in &map.0.health_relics {
        commands.spawn((
            Mesh3d(camp_mesh.clone()),
            MeshMaterial3d(relic_mat.clone()),
            Transform::from_xyz(relic.x, 20.0, relic.y),
        ));
    }

    // Speed shrine
    let shrine_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.7, 0.9),
        emissive: bevy::color::LinearRgba::rgb(0.0, 1.0, 2.0),
        ..default()
    });
    commands.spawn((
        Mesh3d(camp_mesh.clone()),
        MeshMaterial3d(shrine_mat),
        Transform::from_xyz(map.0.speed_shrine.x, 20.0, map.0.speed_shrine.y),
    ));

    // Altars — invisible entity for game logic, visuals come from the map GLB
    for altar_placement in &map.0.altars {
        commands.spawn((
            Transform::from_xyz(altar_placement.position.x, 5.0, altar_placement.position.y),
            Altar {
                side: altar_placement.side,
                captured_by: None,
                capture_progress: 0.0,
                lockout_timer: 0.0,
            },
        ));
    }
}

fn spawn_props(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load SCB props at their exact positions from LS4-3x3
    let props: &[(&str, [f32; 3], f32)] = &[
        // Nexus gears
        ("levelprop_tt_nexus_gears", [3000.0, 19.5, 7289.7], 0.0),
        ("levelprop_tt_nexus_gears1", [12392.0, -2.7, 7244.4], 180.0),
        // Speed shrine
        ("levelprop_tt_speedshrine_gears", [7706.3, -124.9, 6720.4], 0.0),
        // Chains
        ("levelprop_tt_chains_bot_lane", [3624.3, -100.4, 3731.0], 0.0),
        ("levelprop_tt_chains_order_base", [3778.4, -496.1, 7573.5], 0.0),
        ("levelprop_tt_chains_xaos_base", [11636.1, -551.6, 7618.7], 0.0),
        ("levelprop_tt_chains_order_periph", [759.2, 508.0, 4740.9], 0.0),
        // Braziers
        ("levelprop_tt_brazier1", [1372.0, 580.1, 5049.9], 134.0),
        ("levelprop_tt_brazier2", [390.2, 663.8, 6517.9], 0.0),
        ("levelprop_tt_brazier3", [399.4, 692.2, 8021.1], 0.0),
        ("levelprop_tt_brazier4", [1314.3, 582.8, 9495.6], 48.0),
        ("levelprop_tt_brazier5", [14091.1, 582.8, 9530.3], 120.0),
        ("levelprop_tt_brazier6", [14990.5, 675.8, 8053.9], 0.0),
        ("levelprop_tt_brazier7", [15016.4, 664.7, 6532.8], 0.0),
        ("levelprop_tt_brazier8", [14103.0, 580.5, 5098.4], 36.0),
        // Shops
        ("ordershop01", [1340.8, 126.3, 7996.9], 0.0),
        ("chaosshop01", [14152.8, 126.3, 7996.9], 0.0),
    ];

    for (name, pos, rotation_y) in props {
        // Skip props with no known position
        if pos[0] == 0.0 && pos[2] == 0.0 { continue; }

        let path = format!("models/props/{}.glb#Scene0", name);
        let mut transform = Transform::from_xyz(pos[0], pos[1], pos[2]);
        if *rotation_y != 0.0 {
            transform.rotate_y(rotation_y.to_radians());
        }

        commands.spawn((
            SceneRoot(asset_server.load(&path)),
            transform,
        ));
    }
}

fn spawn_lighting(mut commands: Commands) {
    // Strong directional light — LoL textures are pre-baked dark, need bright light
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_3, // ~60 degrees down
            0.3,
            0.0,
        )),
    ));

    // Secondary fill light from opposite direction
    commands.spawn((
        DirectionalLight {
            illuminance: 2500.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            2.5,
            0.0,
        )),
    ));

    // === Brazier fire point lights (warm orange glow) ===
    let brazier_positions: &[[f32; 3]] = &[
        [1372.0, 630.0, 5049.9],
        [390.2, 710.0, 6517.9],
        [399.4, 740.0, 8021.1],
        [1314.3, 630.0, 9495.6],
        [14091.1, 630.0, 9530.3],
        [14990.5, 720.0, 8053.9],
        [15016.4, 710.0, 6532.8],
        [14103.0, 630.0, 5098.4],
    ];
    for pos in brazier_positions {
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.6, 0.2),
                intensity: 800000.0,
                range: 1500.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(pos[0], pos[1], pos[2]),
        ));
    }

    // === Altar glow lights (blue-purple) ===
    for pos in [[5400.0, 50.0, 6400.0], [9900.0, 50.0, 6400.0]] {
        commands.spawn((
            PointLight {
                color: Color::srgb(0.3, 0.4, 1.0),
                intensity: 500000.0,
                range: 800.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(pos[0], pos[1], pos[2]),
        ));
    }

    // === Speed shrine glow (green-cyan) ===
    commands.spawn((
        PointLight {
            color: Color::srgb(0.2, 0.9, 0.6),
            intensity: 300000.0,
            range: 600.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(7706.3, 30.0, 6720.4),
    ));

    // === Nexus glow lights ===
    // Blue nexus
    commands.spawn((
        PointLight {
            color: Color::srgb(0.2, 0.5, 1.0),
            intensity: 1000000.0,
            range: 1200.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(2981.0, 50.0, 7283.0),
    ));
    // Red nexus
    commands.spawn((
        PointLight {
            color: Color::srgb(0.8, 0.2, 0.4),
            intensity: 1000000.0,
            range: 1200.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(12379.5, 50.0, 7289.9),
    ));
}

/// Fix map materials after GLB is loaded — hide untextured white meshes
fn fix_map_materials(
    mut applied: ResMut<MaterialFixApplied>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if applied.0 { return; }

    let mut fixed = 0;
    for (_handle, mat) in materials.iter_mut() {
        // Materials without a base color texture that are pure white = untextured mesh
        if mat.base_color_texture.is_none() {
            let c = mat.base_color.to_srgba();
            if c.red > 0.9 && c.green > 0.9 && c.blue > 0.9 {
                mat.base_color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                mat.alpha_mode = AlphaMode::Blend;
                fixed += 1;
            }
        }
    }

    if fixed > 0 {
        println!("Fixed {} untextured white materials", fixed);
        applied.0 = true;
    }

    // Mark as done after materials are loaded
    if materials.len() > 20 {
        applied.0 = true;
    }
}

#[derive(Resource)]
struct TerrainSharpenApplied(bool);

/// Set terrain texture sampler to linear (sharper) with anisotropic filtering
fn sharpen_terrain_texture(
    mut applied: Local<bool>,
    mut images: ResMut<Assets<Image>>,
) {
    if *applied { return; }

    let mut sharpened = 0;
    for (_handle, image) in images.iter_mut() {
        // Target large textures (terrain is 4096x4096)
        if image.width() >= 2048 && image.height() >= 2048 {
            image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                address_mode_u: ImageAddressMode::ClampToEdge,
                address_mode_v: ImageAddressMode::ClampToEdge,
                anisotropy_clamp: 16,
                ..default()
            });
            sharpened += 1;
        }
    }

    if sharpened > 0 || images.len() > 10 {
        *applied = true;
    }
}
