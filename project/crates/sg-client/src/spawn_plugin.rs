use bevy::prelude::*;
use sg_core::components::*;
use sg_core::constants::*;
use sg_core::types::*;
use sg_core::GameSet;
use crate::map_plugin::MapData;

#[derive(Resource)]
pub struct GameTimer {
    pub elapsed: f32,
    pub next_wave: f32,
    pub wave_count: u32,
}

/// Tracks jungle camp alive/respawn state
#[derive(Resource)]
pub struct JungleCampState {
    pub camps: Vec<CampTracker>,
}

pub struct CampTracker {
    pub camp_type: CampType,
    pub position: Vec2,
    pub alive: bool,
    pub entity: Option<Entity>,
    pub next_spawn: f32,
}

impl Default for JungleCampState {
    fn default() -> Self {
        Self { camps: Vec::new() }
    }
}

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTimer {
            elapsed: 0.0,
            next_wave: MINION_FIRST_SPAWN,
            wave_count: 0,
        })
        .insert_resource(JungleCampState::default())
        // Minion spawning moved to MinionPlugin
        .add_systems(Update, (update_game_timer, init_jungle_camps, spawn_jungle_camps).chain().in_set(GameSet::Spawn).run_if(in_state(crate::menu::AppState::InGame)));
    }
}

fn update_game_timer(time: Res<Time>, mut timer: ResMut<GameTimer>) {
    timer.elapsed += time.delta_secs();
}

fn spawn_minion_waves(
    mut commands: Commands,
    mut timer: ResMut<GameTimer>,
    map: Res<MapData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    inhib_state: Res<crate::combat_plugin::InhibitorState>,
) {
    if timer.elapsed < timer.next_wave {
        return;
    }

    timer.next_wave += MINION_WAVE_INTERVAL;
    let is_cannon = timer.wave_count % 3 == 2;
    timer.wave_count += 1;

    // Minion meshes — visible at game scale
    let melee_mesh = meshes.add(Cuboid::new(35.0, 45.0, 35.0));
    let caster_mesh = meshes.add(Capsule3d::new(18.0, 35.0));
    let siege_mesh = meshes.add(Cuboid::new(50.0, 40.0, 60.0));

    let blue_melee_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.3, 0.8),
        emissive: bevy::color::LinearRgba::rgb(0.0, 0.1, 0.5),
        ..default()
    });
    let blue_caster_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 1.0),
        emissive: bevy::color::LinearRgba::rgb(0.1, 0.2, 0.8),
        ..default()
    });
    let red_melee_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.15, 0.15),
        emissive: bevy::color::LinearRgba::rgb(0.5, 0.0, 0.0),
        ..default()
    });
    let red_caster_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.3, 0.3),
        emissive: bevy::color::LinearRgba::rgb(0.8, 0.1, 0.1),
        ..default()
    });
    let blue_siege_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.2, 0.6),
        emissive: bevy::color::LinearRgba::rgb(0.0, 0.1, 0.3),
        ..default()
    });
    let red_siege_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.1, 0.1),
        emissive: bevy::color::LinearRgba::rgb(0.3, 0.0, 0.0),
        ..default()
    });

    let mut composition = vec![
        MinionType::Melee,
        MinionType::Melee,
        MinionType::Melee,
        MinionType::Caster,
        MinionType::Caster,
        MinionType::Caster,
    ];
    if is_cannon {
        composition.push(MinionType::Siege);
    }

    let lanes: [(Lane, Team, &Vec<Vec2>); 4] = [
        (Lane::Top, Team::Blue, &map.0.lane_paths.top_blue),
        (Lane::Top, Team::Red, &map.0.lane_paths.top_red),
        (Lane::Bottom, Team::Blue, &map.0.lane_paths.bottom_blue),
        (Lane::Bottom, Team::Red, &map.0.lane_paths.bottom_red),
    ];

    for (lane, team, waypoints) in lanes {
        let spawn_pos = waypoints[0];

        // Check if enemy inhibitor for this lane is destroyed → add super minion
        let mut lane_composition = composition.clone();
        let enemy_inhib_dead = match (team, lane) {
            // Blue team pushes toward Red side, check Red inhibitor
            (Team::Blue, Lane::Top) => !inhib_state.red_top_alive,
            (Team::Blue, Lane::Bottom) => !inhib_state.red_bot_alive,
            // Red team pushes toward Blue side, check Blue inhibitor
            (Team::Red, Lane::Top) => !inhib_state.blue_top_alive,
            (Team::Red, Lane::Bottom) => !inhib_state.blue_bot_alive,
            _ => false,
        };
        if enemy_inhib_dead {
            lane_composition.push(MinionType::Super);
        }

        for (i, &mtype) in lane_composition.iter().enumerate() {
            // Real LoL TT patch 4.20 minion stats
            let hp = match mtype {
                MinionType::Melee => 455.0,
                MinionType::Caster => 290.0,
                MinionType::Siege => 805.0,
                MinionType::Super => 1500.0,
            };
            let ad = match mtype {
                MinionType::Melee => 12.0,
                MinionType::Caster => 23.0,
                MinionType::Siege => 40.0,
                MinionType::Super => 180.0,
            };
            let atk_speed = match mtype {
                MinionType::Melee | MinionType::Caster => 1.6,
                MinionType::Siege => 2.0,
                MinionType::Super => 1.0,
            };
            let armor = match mtype {
                MinionType::Super => 30.0,
                _ => 0.0,
            };

            let (mesh, mat) = match (mtype, team) {
                (MinionType::Melee | MinionType::Super, Team::Blue) => (melee_mesh.clone(), blue_melee_mat.clone()),
                (MinionType::Melee | MinionType::Super, _) => (melee_mesh.clone(), red_melee_mat.clone()),
                (MinionType::Caster, Team::Blue) => (caster_mesh.clone(), blue_caster_mat.clone()),
                (MinionType::Caster, _) => (caster_mesh.clone(), red_caster_mat.clone()),
                (MinionType::Siege, Team::Blue) => (siege_mesh.clone(), blue_siege_mat.clone()),
                (MinionType::Siege, _) => (siege_mesh.clone(), red_siege_mat.clone()),
            };

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                // Each minion spawns at the lane start, spaced 60 units apart along the lane
                Transform::from_xyz(
                    spawn_pos.x,
                    15.0,
                    spawn_pos.y + (i as f32 * 30.0), // small Z offset to prevent stacking
                ),
                Minion {
                    minion_type: mtype,
                    lane,
                    team,
                },
                TeamMember(team),
                Health { current: hp, max: hp, regen: 0.0 },
                CombatStats {
                    attack_damage: ad,
                    ability_power: 0.0,
                    armor,
                    magic_resist: 0.0,
                    attack_speed: atk_speed,
                    move_speed: 325.0,
                    crit_chance: 0.0,
                    cdr: 0.0,
                    armor_pen_flat: 0.0,
                    armor_pen_pct: 0.0,
                    magic_pen_flat: 0.0,
                    magic_pen_pct: 0.0,
                    life_steal: 0.0,
                    spell_vamp: 0.0,
                },
                AutoAttackRange(match mtype {
                    MinionType::Melee => 110.0,
                    MinionType::Super => 170.0,
                    MinionType::Caster => 550.0,
                    MinionType::Siege => 300.0,
                }),
                AttackCooldown(0.0),
                PatrolPath {
                    waypoints: waypoints.clone(),
                    current_index: 0,
                },
            )).with_children(|parent| {
                // Attach real 3D minion model as child
                let model_name = match (mtype, team) {
                    (MinionType::Melee | MinionType::Super, Team::Blue) => "order_minion_melee",
                    (MinionType::Melee | MinionType::Super, _) => "chaos_minion_melee",
                    (MinionType::Caster, Team::Blue) => "order_minion_caster",
                    (MinionType::Caster, _) => "chaos_minion_caster",
                    (MinionType::Siege, Team::Blue) => "order_minion_siege",
                    (MinionType::Siege, _) => "chaos_minion_siege",
                };
                let path = format!("models/minions/{}.glb#Scene0", model_name);
                parent.spawn((
                    SceneRoot(asset_server.load(&path)),
                    Transform::from_translation(Vec3::new(0.0, -15.0, 0.0))
                        .with_scale(Vec3::splat(0.8)),
                ));
            });
        }
    }
}

/// Initialize jungle camp trackers from map data (runs once)
fn init_jungle_camps(
    map: Res<MapData>,
    mut camp_state: ResMut<JungleCampState>,
) {
    if !camp_state.camps.is_empty() { return; } // already initialized

    for camp in &map.0.jungle_camps {
        camp_state.camps.push(CampTracker {
            camp_type: camp.camp_type,
            position: camp.position,
            alive: false,
            entity: None,
            next_spawn: JUNGLE_FIRST_SPAWN,
        });
    }
}

/// Spawn jungle camps when their timer expires
fn spawn_jungle_camps(
    mut commands: Commands,
    timer: Res<GameTimer>,
    mut camp_state: ResMut<JungleCampState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Check if dead camps need despawn
    dead_camps: Query<(Entity, &Health), With<JungleCamp>>,
) {
    // First: check for dead camps and update state
    for tracker in camp_state.camps.iter_mut() {
        if let Some(entity) = tracker.entity {
            if let Ok((_, health)) = dead_camps.get(entity) {
                if health.current <= 0.0 {
                    commands.entity(entity).despawn();
                    tracker.alive = false;
                    tracker.entity = None;
                    tracker.next_spawn = timer.elapsed + JUNGLE_RESPAWN;
                }
            }
        }
    }

    // Then: spawn camps whose timer has expired
    for tracker in camp_state.camps.iter_mut() {
        if tracker.alive || timer.elapsed < tracker.next_spawn { continue; }

        let (hp, ad, armor, gold, xp, color, size) = match tracker.camp_type {
            CampType::Golem => (GOLEM_HP, GOLEM_AD, GOLEM_ARMOR, GOLEM_GOLD, GOLEM_XP,
                Color::srgb(0.5, 0.35, 0.2), 35.0),
            CampType::Wolf => (WOLF_HP, WOLF_AD, WOLF_ARMOR, WOLF_GOLD, WOLF_XP,
                Color::srgb(0.4, 0.4, 0.5), 25.0),
            CampType::Wraith => (WRAITH_HP, WRAITH_AD, WRAITH_ARMOR, WRAITH_GOLD, WRAITH_XP,
                Color::srgb(0.3, 0.5, 0.4), 22.0),
            CampType::Vilemaw => continue, // Vilemaw handled by objectives_plugin
        };

        let mesh = meshes.add(Sphere::new(size));
        let mat = materials.add(StandardMaterial {
            base_color: color,
            emissive: bevy::color::LinearRgba::rgb(0.2, 0.15, 0.1),
            ..default()
        });

        let entity = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_xyz(tracker.position.x, 30.0, tracker.position.y),
            JungleCamp {
                camp_type: tracker.camp_type,
                team: Team::Neutral,
                respawn_timer: JUNGLE_RESPAWN,
            },
            TeamMember(Team::Neutral),
            Health { current: hp, max: hp, regen: 0.0 },
            CombatStats {
                attack_damage: ad,
                ability_power: 0.0,
                armor,
                magic_resist: 0.0,
                attack_speed: 0.625,
                move_speed: 300.0,
                crit_chance: 0.0, cdr: 0.0,
                armor_pen_flat: 0.0, armor_pen_pct: 0.0,
                magic_pen_flat: 0.0, magic_pen_pct: 0.0,
                life_steal: 0.0, spell_vamp: 0.0,
            },
            AutoAttackRange(100.0),
        )).id();

        tracker.alive = true;
        tracker.entity = Some(entity);
    }
}
