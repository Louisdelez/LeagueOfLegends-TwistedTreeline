use bevy::prelude::*;
use sg_core::components::*;
use sg_core::constants::*;
use sg_core::types::*;
use sg_core::GameSet;
use crate::spawn_plugin::GameTimer;
use crate::map_plugin::MapData;
use crate::menu::AppState;

pub struct ObjectivesPlugin;

impl Plugin for ObjectivesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AltarState::default())
            .insert_resource(VilemawState::default())
            .add_systems(Update, (
                spawn_vilemaw,
                vilemaw_ai,
                vilemaw_death_buff,
                altar_capture_system,
                apply_altar_buffs,
                health_relic_pickup,
                speed_shrine_pickup,
                tick_buffs,
                place_ward,
                tick_wards,
            ).in_set(GameSet::Combat).run_if(in_state(AppState::InGame)));
    }
}

// === Buff components ===

#[derive(Component)]
pub struct VilemawBuff {
    pub remaining: f32,
}

#[derive(Component)]
pub struct AltarSpeedBuff;

#[derive(Component)]
pub struct AltarHpRestoreBuff;

#[derive(Component)]
pub struct SpeedShrineBuff {
    pub remaining: f32,
}

// === Altar System ===

#[derive(Resource, Default)]
pub struct AltarState {
    pub left_owner: Option<Team>,
    pub right_owner: Option<Team>,
    pub left_lockout: f32,
    pub right_lockout: f32,
}

impl AltarState {
    pub fn altars_owned_by(&self, team: Team) -> u8 {
        let mut count = 0;
        if self.left_owner == Some(team) { count += 1; }
        if self.right_owner == Some(team) { count += 1; }
        count
    }
}

fn altar_capture_system(
    time: Res<Time>,
    game_timer: Res<GameTimer>,
    mut altar_state: ResMut<AltarState>,
    mut altars: Query<(&Transform, &mut Altar)>,
    champions: Query<(&Transform, &TeamMember), (With<Champion>, Without<Dead>)>,
) {
    let dt = time.delta_secs();
    if game_timer.elapsed < ALTAR_UNLOCK_TIME { return; }

    altar_state.left_lockout = (altar_state.left_lockout - dt).max(0.0);
    altar_state.right_lockout = (altar_state.right_lockout - dt).max(0.0);

    for (altar_tf, mut altar) in &mut altars {
        let lockout = match altar.side {
            AltarSide::Left => altar_state.left_lockout,
            AltarSide::Right => altar_state.right_lockout,
        };
        if lockout > 0.0 { continue; }

        let mut capturing_team: Option<Team> = None;
        for (champ_tf, team) in &champions {
            if altar_tf.translation.distance(champ_tf.translation) < 200.0 {
                capturing_team = Some(team.0);
                break;
            }
        }

        if let Some(team) = capturing_team {
            if altar.captured_by != Some(team) {
                altar.capture_progress += dt;
                if altar.capture_progress >= ALTAR_CAPTURE_TIME {
                    altar.captured_by = Some(team);
                    altar.capture_progress = 0.0;
                    match altar.side {
                        AltarSide::Left => {
                            altar_state.left_owner = Some(team);
                            altar_state.left_lockout = ALTAR_LOCKOUT_TIME;
                        }
                        AltarSide::Right => {
                            altar_state.right_owner = Some(team);
                            altar_state.right_lockout = ALTAR_LOCKOUT_TIME;
                        }
                    }
                }
            }
        } else {
            altar.capture_progress = (altar.capture_progress - dt * 2.0).max(0.0);
        }
    }
}

/// Apply altar buffs to champions based on how many altars their team owns
fn apply_altar_buffs(
    mut commands: Commands,
    altar_state: Res<AltarState>,
    mut champions: Query<(Entity, &TeamMember, &mut CombatStats, Has<AltarSpeedBuff>, Has<AltarHpRestoreBuff>), With<Champion>>,
) {
    for (entity, team, mut stats, has_speed, has_hp_restore) in &mut champions {
        let owned = altar_state.altars_owned_by(team.0);

        // 1 altar: +10% bonus movement speed
        if owned >= 1 && !has_speed {
            stats.move_speed *= 1.0 + ALTAR_1_MOVE_SPEED_BONUS;
            commands.entity(entity).insert(AltarSpeedBuff);
        } else if owned < 1 && has_speed {
            stats.move_speed /= 1.0 + ALTAR_1_MOVE_SPEED_BONUS;
            commands.entity(entity).remove::<AltarSpeedBuff>();
        }

        // 2 altars: HP restore on minion kill (tracked by component)
        if owned >= 2 && !has_hp_restore {
            commands.entity(entity).insert(AltarHpRestoreBuff);
        } else if owned < 2 && has_hp_restore {
            commands.entity(entity).remove::<AltarHpRestoreBuff>();
        }
    }
}

// === Vilemaw Boss ===

#[derive(Component)]
pub struct Vilemaw {
    pub armor_shred_stacks: u32,
}

#[derive(Resource)]
pub struct VilemawState {
    pub alive: bool,
    pub next_spawn: f32,
}

impl Default for VilemawState {
    fn default() -> Self {
        Self {
            alive: false,
            next_spawn: VILEMAW_FIRST_SPAWN,
        }
    }
}

fn spawn_vilemaw(
    mut commands: Commands,
    mut state: ResMut<VilemawState>,
    game_timer: Res<GameTimer>,
    map: Res<MapData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<Entity, With<Vilemaw>>,
) {
    if state.alive || game_timer.elapsed < state.next_spawn { return; }
    if !existing.is_empty() { return; }

    let pos = map.0.vilemaw_spawn;
    let mesh = meshes.add(Sphere::new(80.0));
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.0, 0.5),
        emissive: bevy::color::LinearRgba::rgb(1.0, 0.0, 2.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_xyz(pos.x, 80.0, pos.y),
        Vilemaw { armor_shred_stacks: 0 },
        TeamMember(Team::Neutral),
        Health { current: VILEMAW_HP, max: VILEMAW_HP, regen: 0.0 },
        CombatStats {
            attack_damage: VILEMAW_AD,
            ability_power: 0.0,
            armor: 40.0,
            magic_resist: 40.0,
            attack_speed: 0.5,
            move_speed: 0.0,
            crit_chance: 0.0,
            cdr: 0.0,
            armor_pen_flat: 0.0,
            armor_pen_pct: 0.0,
            magic_pen_flat: 0.0,
            magic_pen_pct: 0.0,
            life_steal: 0.0,
            spell_vamp: 0.0,
        },
        AutoAttackRange(800.0),
    ));

    state.alive = true;
}

fn vilemaw_ai(
    mut commands: Commands,
    vilemaw_q: Query<(Entity, &Transform, &Health, &Vilemaw)>,
    champions: Query<(Entity, &Transform, &TeamMember), (With<Champion>, Without<Dead>)>,
) {
    for (vim_entity, vim_tf, vim_health, _vilemaw) in &vilemaw_q {
        if vim_health.current <= 0.0 { continue; }

        // Target closest champion in range
        let mut closest: Option<(Entity, f32)> = None;
        for (champ_entity, champ_tf, _team) in &champions {
            let dist = vim_tf.translation.distance(champ_tf.translation);
            if dist < 800.0 {
                if closest.map_or(true, |(_, d)| dist < d) {
                    closest = Some((champ_entity, dist));
                }
            }
        }

        if let Some((target, _)) = closest {
            commands.entity(vim_entity)
                .insert(AttackTarget { entity: target })
                .insert(AttackCooldown(0.0));
        }
    }
}

/// When Vilemaw dies, give buff to the team that killed it
fn vilemaw_death_buff(
    mut commands: Commands,
    mut state: ResMut<VilemawState>,
    game_timer: Res<GameTimer>,
    vilemaw_q: Query<(Entity, &Health), With<Vilemaw>>,
    player_q: Query<&TeamMember, With<PlayerControlled>>,
    mut champions: Query<(Entity, &TeamMember, &mut CombatStats, &mut Gold), With<Champion>>,
) {
    for (vim_entity, vim_health) in &vilemaw_q {
        if vim_health.current <= 0.0 {
            commands.entity(vim_entity).despawn();
            state.alive = false;
            state.next_spawn = game_timer.elapsed + VILEMAW_RESPAWN;

            // Give buff to the player's team (simplified: local player's team gets it)
            let my_team = match player_q.iter().next() {
                Some(t) => t.0,
                None => return,
            };

            for (entity, team, mut stats, mut gold) in &mut champions {
                if team.0 == my_team {
                    // Vilemaw buff: +40 AD, +40 AP for 180s
                    stats.attack_damage += 40.0;
                    stats.ability_power += 40.0;
                    gold.0 += 150.0;
                    commands.entity(entity).insert(VilemawBuff { remaining: VILEMAW_BUFF_DURATION });
                }
            }
        }
    }
}

// === Pickups ===

fn health_relic_pickup(
    map: Res<MapData>,
    game_timer: Res<GameTimer>,
    mut champions: Query<(&Transform, &mut Health), (With<Champion>, Without<Dead>)>,
) {
    if game_timer.elapsed < HEALTH_RELIC_UNLOCK { return; }

    for relic_pos in &map.0.health_relics {
        for (champ_tf, mut health) in &mut champions {
            let dist_xz = Vec2::new(champ_tf.translation.x, champ_tf.translation.z)
                .distance(*relic_pos);
            if dist_xz < 100.0 {
                let heal = health.max * 0.1;
                health.current = (health.current + heal).min(health.max);
            }
        }
    }
}

fn speed_shrine_pickup(
    mut commands: Commands,
    map: Res<MapData>,
    game_timer: Res<GameTimer>,
    mut champions: Query<(Entity, &Transform, &mut CombatStats, Has<SpeedShrineBuff>), (With<Champion>, Without<Dead>)>,
) {
    if game_timer.elapsed < SPEED_SHRINE_UNLOCK { return; }

    let shrine = map.0.speed_shrine;
    for (entity, champ_tf, mut stats, has_buff) in &mut champions {
        if has_buff { continue; }
        let dist = Vec2::new(champ_tf.translation.x, champ_tf.translation.z)
            .distance(shrine);
        if dist < 100.0 {
            // +80 move speed for 4 seconds
            stats.move_speed += 80.0;
            commands.entity(entity).insert(SpeedShrineBuff { remaining: 4.0 });
        }
    }
}

/// Tick buff durations and remove expired buffs
fn tick_buffs(
    mut commands: Commands,
    time: Res<Time>,
    mut vilemaw_buffs: Query<(Entity, &mut VilemawBuff, &mut CombatStats)>,
    mut shrine_buffs: Query<(Entity, &mut SpeedShrineBuff, &mut CombatStats), Without<VilemawBuff>>,
) {
    let dt = time.delta_secs();

    for (entity, mut buff, mut stats) in &mut vilemaw_buffs {
        buff.remaining -= dt;
        if buff.remaining <= 0.0 {
            stats.attack_damage -= 40.0;
            stats.ability_power -= 40.0;
            commands.entity(entity).remove::<VilemawBuff>();
        }
    }

    for (entity, mut buff, mut stats) in &mut shrine_buffs {
        buff.remaining -= dt;
        if buff.remaining <= 0.0 {
            stats.move_speed -= 80.0;
            commands.entity(entity).remove::<SpeedShrineBuff>();
        }
    }
}

// === Ward System ===

#[derive(Component)]
pub struct Ward {
    pub lifetime: f32,
}

/// Place ward: press 4 to place a ward at cursor position
fn place_ward(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    player_q: Query<(&TeamMember, &Transform), With<PlayerControlled>>,
    existing_wards: Query<Entity, With<Ward>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !keys.just_pressed(KeyCode::Digit4) { return; }

    // Max 2 wards
    if existing_wards.iter().count() >= 2 { return; }

    let Ok((team, _player_tf)) = player_q.single() else { return; };
    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok((camera, cam_tf)) = camera_q.single() else { return; };
    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else { return; };
    let Some(dist) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else { return; };
    let target_pos = ray.get_point(dist);

    let ward_color = if team.0 == Team::Blue { Color::srgb(0.2, 0.5, 1.0) } else { Color::srgb(0.8, 0.2, 0.2) };
    let ward_mesh = meshes.add(Cylinder::new(8.0, 30.0));
    let ward_mat = materials.add(StandardMaterial {
        base_color: ward_color,
        emissive: bevy::color::LinearRgba::rgb(0.3, 0.6, 1.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(ward_mesh), MeshMaterial3d(ward_mat),
        Transform::from_translation(target_pos + Vec3::Y * 15.0),
        Ward { lifetime: 180.0 },
        TeamMember(team.0),
        Health { current: 3.0, max: 3.0, regen: 0.0 },
        VisionRange(1100.0),
    ));
}

/// Tick ward lifetime and despawn expired wards
fn tick_wards(
    mut commands: Commands,
    time: Res<Time>,
    mut wards: Query<(Entity, &mut Ward, &Health)>,
) {
    let dt = time.delta_secs();
    for (entity, mut ward, health) in &mut wards {
        ward.lifetime -= dt;
        if ward.lifetime <= 0.0 || health.current <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
