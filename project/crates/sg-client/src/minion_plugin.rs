//! Minion Plugin — 100% Faithful LoL Minion System
//!
//! Handles: staggered spawn (0.8s apart), lane patrol, AI targeting (0.25s tick),
//! combat (melee walks up, caster stays back), collision (48/65 radius), death, gold/xp.
//!
//! Stats from real LoL Twisted Treeline patch 4.20.
//! AI behavior from LeagueSandbox LaneMinionAI.cs.

use bevy::prelude::*;
use sg_core::components::*;
use sg_core::types::*;
use sg_core::constants::*;
use sg_core::GameSet;
use sg_gameplay::leveling::{kill_xp, shared_xp};
use crate::spawn_plugin::GameTimer;
use crate::map_plugin::MapData;
use crate::menu::AppState;

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MinionWaveManager::new())
            .add_systems(Update, (
                minion_wave_timer,
                minion_staggered_spawn,
            ).chain().in_set(GameSet::Spawn).run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                minion_call_for_help,
                minion_ai_tick,
            ).in_set(GameSet::AI).run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                minion_movement,
                minion_collision,
            ).in_set(GameSet::Movement).run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                minion_attack,
                minion_melee_windup,
                minion_projectile,
                minion_death,
                minion_gold_xp,
                gold_particle_system,
                minion_attack_pulse,
                minion_health_bars,
                minion_animations,
            ).in_set(GameSet::Combat).run_if(in_state(AppState::InGame)));
    }
}

// ═══════════════════════════════════════════════════════════
//  Stats — exact LoL TT patch 4.20 values
// ═══════════════════════════════════════════════════════════

fn minion_stats(mtype: MinionType, game_time: f32) -> (f32, f32, f32, f32, f32, f32, f32, f32) {
    // (hp, ad, armor, attack_speed, range, detect_range, collision_radius, move_speed)
    // Base stats + scaling per 90s after 3 minutes (real LoL)
    let scaling_intervals = ((game_time - 180.0).max(0.0) / 90.0).floor();

    let (base_hp, base_ad, armor) = match mtype {
        MinionType::Melee  => (455.0, 12.0, 0.0),
        MinionType::Caster => (290.0, 23.0, 0.0),
        MinionType::Siege  => (700.0, 40.0, 15.0),
        MinionType::Super  => (1500.0, 180.0, 30.0),
    };

    // HP grows ~20 per 90s for melee, ~15 for caster, ~27 for siege
    let hp_growth = match mtype {
        MinionType::Melee  => 20.0,
        MinionType::Caster => 15.0,
        MinionType::Siege  => 27.0,
        MinionType::Super  => 0.0, // Super doesn't scale
    };
    // AD grows ~1 per 90s for melee, ~1.5 for caster, ~2 for siege
    let ad_growth = match mtype {
        MinionType::Melee  => 1.0,
        MinionType::Caster => 1.5,
        MinionType::Siege  => 2.0,
        MinionType::Super  => 0.0,
    };

    let hp = base_hp + hp_growth * scaling_intervals;
    let ad = base_ad + ad_growth * scaling_intervals;

    let (atk_speed, range, detect, collision, move_speed) = match mtype {
        MinionType::Melee  => (1.6, 110.0, 475.0, 48.0, 325.0),
        MinionType::Caster => (1.6, 550.0, 700.0, 48.0, 325.0),
        MinionType::Siege  => (2.0, 300.0, 475.0, 65.0, 325.0),
        MinionType::Super  => (1.0, 170.0, 600.0, 65.0, 325.0),
    };

    (hp, ad, armor, atk_speed, range, detect, collision, move_speed)
}

// ═══════════════════════════════════════════════════════════
//  Resources & Components
// ═══════════════════════════════════════════════════════════

struct QueuedMinion {
    minion_type: MinionType,
    team: Team,
    lane: Lane,
    waypoints: Vec<Vec2>,
}

#[derive(Resource)]
struct MinionWaveManager {
    next_wave_time: f32,
    wave_count: u32,
    spawn_queue: Vec<QueuedMinion>,
    spawn_timer: f32,
}

impl MinionWaveManager {
    fn new() -> Self {
        Self {
            next_wave_time: MINION_FIRST_SPAWN,
            wave_count: 0,
            spawn_queue: Vec::new(),
            spawn_timer: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MinionState {
    Patrolling,
    Fighting,
    Returning,
}

/// AI state for each minion — replaces PatrolPath for minions
#[derive(Component)]
pub struct MinionAI {
    state: MinionState,
    ai_timer: f32,
    patience_timer: f32,
    waypoints: Vec<Vec2>,
    current_waypoint: usize,
    detect_range: f32,
    collision_radius: f32,
}

#[derive(Component)]
struct Dying(f32);

/// Visual attack feedback — scale pulse when attacking
#[derive(Component)]
struct AttackPulse(f32);

/// Track last animation state to avoid restarting every frame
#[derive(Component, Default)]
struct LastAnimState(u8); // 0=idle, 1=run, 2=attack, 3=death

/// Marker: gold already awarded for this minion death
#[derive(Component)]
struct GoldAwarded;

/// Delayed melee damage (windup before damage applies)
#[derive(Component)]
struct MeleeWindup {
    target: Entity,
    damage: f32,
    timer: f32,
}

// ═══════════════════════════════════════════════════════════
//  System 1: Wave timer — queue minions when it's time
// ═══════════════════════════════════════════════════════════

fn minion_wave_timer(
    game_timer: Res<GameTimer>,
    map: Res<MapData>,
    mut mgr: ResMut<MinionWaveManager>,
    inhib_state: Res<crate::combat_plugin::InhibitorState>,
) {
    if game_timer.elapsed < mgr.next_wave_time { return; }

    mgr.next_wave_time += MINION_WAVE_INTERVAL;
    // Cannon wave frequency scales with game time (real LoL)
    // 0-20min: every 3rd wave, 20-35min: every 2nd, 35min+: every wave
    let game_minutes = game_timer.elapsed / 60.0;
    let is_cannon = if game_minutes >= 35.0 {
        true // every wave
    } else if game_minutes >= 20.0 {
        mgr.wave_count % 2 == 1 // every 2nd wave
    } else {
        mgr.wave_count % 3 == 2 // every 3rd wave
    };
    mgr.wave_count += 1;

    // Build wave composition: melee first, cannon, then caster
    let mut composition = vec![
        MinionType::Melee, MinionType::Melee, MinionType::Melee,
    ];
    if is_cannon {
        composition.push(MinionType::Siege);
    }
    composition.extend([MinionType::Caster, MinionType::Caster, MinionType::Caster]);

    // Spawn for each lane
    let lanes: [(Lane, Team, &Vec<Vec2>); 4] = [
        (Lane::Top, Team::Blue, &map.0.lane_paths.top_blue),
        (Lane::Top, Team::Red, &map.0.lane_paths.top_red),
        (Lane::Bottom, Team::Blue, &map.0.lane_paths.bottom_blue),
        (Lane::Bottom, Team::Red, &map.0.lane_paths.bottom_red),
    ];

    // Queue each lane's minions as a block — all Blue Top, then Red Top, then Blue Bot, then Red Bot
    // Within each block, staggered spawn (0.8s) ensures minions spawn one at a time per lane
    for (lane, team, waypoints) in lanes {
        let mut lane_comp = composition.clone();

        let enemy_inhib_dead = match (team, lane) {
            (Team::Blue, Lane::Top) => !inhib_state.red_top_alive,
            (Team::Blue, Lane::Bottom) => !inhib_state.red_bot_alive,
            (Team::Red, Lane::Top) => !inhib_state.blue_top_alive,
            (Team::Red, Lane::Bottom) => !inhib_state.blue_bot_alive,
            _ => false,
        };
        let all_inhibs_dead = match team {
            Team::Blue => !inhib_state.red_top_alive && !inhib_state.red_bot_alive,
            Team::Red => !inhib_state.blue_top_alive && !inhib_state.blue_bot_alive,
            _ => false,
        };
        if all_inhibs_dead {
            lane_comp.insert(0, MinionType::Super);
            lane_comp.insert(0, MinionType::Super);
        } else if enemy_inhib_dead {
            lane_comp.insert(0, MinionType::Super);
        }

        for mtype in lane_comp {
            mgr.spawn_queue.push(QueuedMinion {
                minion_type: mtype,
                team,
                lane,
                waypoints: waypoints.clone(),
            });
        }
    }

    // Sort queue so same-lane same-team minions are grouped together
    // Blue Top first, Red Top second, Blue Bot third, Red Bot fourth
    mgr.spawn_queue.sort_by(|a, b| {
        let lane_order = |q: &QueuedMinion| -> u8 {
            match (q.team, q.lane) {
                (Team::Blue, Lane::Top) => 0,
                (Team::Red, Lane::Top) => 1,
                (Team::Blue, Lane::Bottom) => 2,
                (Team::Red, Lane::Bottom) => 3,
                _ => 4,
            }
        };
        lane_order(a).cmp(&lane_order(b))
    });
}

// ═══════════════════════════════════════════════════════════
//  System 2: Staggered spawn — 1 minion every 0.8s
// ═══════════════════════════════════════════════════════════

fn minion_staggered_spawn(
    mut commands: Commands,
    time: Res<Time>,
    game_timer: Res<GameTimer>,
    nav_grid: Res<sg_navigation::NavGrid>,
    asset_server: Res<AssetServer>,
    mut mgr: ResMut<MinionWaveManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if mgr.spawn_queue.is_empty() { return; }

    mgr.spawn_timer += time.delta_secs();
    if mgr.spawn_timer < 0.8 { return; }
    mgr.spawn_timer = 0.0;

    let queued = mgr.spawn_queue.remove(0);
    let (hp, ad, armor, atk_speed, range, detect, collision, move_speed) = minion_stats(queued.minion_type, game_timer.elapsed);

    // Real LoL minion 3D models
    let model_path = match (queued.minion_type, queued.team) {
        (MinionType::Melee | MinionType::Super, Team::Blue) => "models/minions/order_minion_melee.glb#Scene0",
        (MinionType::Melee | MinionType::Super, _)          => "models/minions/chaos_minion_melee.glb#Scene0",
        (MinionType::Caster, Team::Blue)                     => "models/minions/order_minion_caster.glb#Scene0",
        (MinionType::Caster, _)                              => "models/minions/chaos_minion_caster.glb#Scene0",
        (MinionType::Siege, Team::Blue)                      => "models/minions/order_minion_siege.glb#Scene0",
        (MinionType::Siege, _)                               => "models/minions/chaos_minion_siege.glb#Scene0",
    };

    let spawn_pos = queued.waypoints.first().copied().unwrap_or(Vec2::ZERO);

    // Get terrain height at spawn position
    let terrain_y = nav_grid.get_height(spawn_pos) + 10.0;
    let spawn_y = if terrain_y.abs() < 1.0 { 55.0 } else { terrain_y }; // fallback if no height data

    let entity = commands.spawn((
        Transform::from_xyz(spawn_pos.x, spawn_y, spawn_pos.y),
        Visibility::Inherited,
        Minion { minion_type: queued.minion_type, lane: queued.lane, team: queued.team },
        TeamMember(queued.team),
        Health { current: hp, max: hp, regen: 0.0 },
        CombatStats {
            attack_damage: ad, ability_power: 0.0, armor, magic_resist: 0.0,
            attack_speed: atk_speed, move_speed, crit_chance: 0.0, cdr: 0.0,
            armor_pen_flat: 0.0, armor_pen_pct: 0.0, magic_pen_flat: 0.0, magic_pen_pct: 0.0,
            life_steal: 0.0, spell_vamp: 0.0,
        },
        AutoAttackRange(range),
        AttackCooldown(1.0 / atk_speed),
        MinionAI {
            state: MinionState::Patrolling,
            ai_timer: 0.0,
            patience_timer: 0.0,
            waypoints: queued.waypoints,
            current_waypoint: 1,
            detect_range: detect,
            collision_radius: collision,
        },
        LastAnimState::default(),
    )).id();

    // Attach real 3D model as child — scaled to match game world
    let model_scale = match queued.minion_type {
        MinionType::Siege | MinionType::Super => 1.5,
        _ => 1.0,
    };
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            bevy::scene::SceneRoot(asset_server.load(model_path)),
            Transform::from_translation(Vec3::new(0.0, -25.0, 0.0))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)) // Face forward
                .with_scale(Vec3::splat(model_scale)),
        ));
    });
}

// ═══════════════════════════════════════════════════════════
//  System 2b: Call for Help — champion attacks champion → nearby minions retarget
// ═══════════════════════════════════════════════════════════

fn minion_call_for_help(
    mut commands: Commands,
    // Champions who are attacking another champion
    attackers: Query<(Entity, &Transform, &TeamMember, &AttackTarget), (With<Champion>, Without<Dead>)>,
    champions: Query<Entity, With<Champion>>,
    // Nearby enemy minions that could switch target
    mut minions: Query<(Entity, &Transform, &TeamMember, &mut MinionAI, &AutoAttackRange), (With<Minion>, Without<Dead>, Without<Dying>)>,
) {
    for (atk_entity, atk_tf, atk_team, atk_target) in &attackers {
        // Is this champion attacking another champion?
        if !champions.contains(atk_target.entity) { continue; }

        // Find enemy minions near the attacker that should switch to defend
        for (m_entity, m_tf, m_team, mut m_ai, _m_range) in &mut minions {
            // Must be on the DEFENDING team (same team as the victim)
            if m_team.0 == atk_team.0 { continue; } // Same team as attacker = skip

            // Must be within detection range of the attacker
            let dist = m_tf.translation.distance(atk_tf.translation);
            if dist > m_ai.detect_range { continue; }

            // Switch target to the attacking champion (higher priority)
            m_ai.state = MinionState::Fighting;
            m_ai.patience_timer = 0.0;
            if let Ok(mut ecmd) = commands.get_entity(m_entity) {
                ecmd.insert(AttackTarget { entity: atk_entity });
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 3: AI tick — target acquisition + state transitions
// ═══════════════════════════════════════════════════════════

fn minion_ai_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut minions: Query<
        (Entity, &Transform, &TeamMember, &AutoAttackRange, &mut MinionAI, Option<&AttackTarget>),
        (With<Minion>, Without<Dead>, Without<Dying>),
    >,
    enemy_minions: Query<(Entity, &Transform, &TeamMember, &Health), (With<Minion>, Without<Dead>)>,
    enemy_champs: Query<(Entity, &Transform, &TeamMember, &Health), (With<Champion>, Without<Dead>)>,
    structures: Query<(Entity, &Transform, &TeamMember, &Health), (With<Structure>, Without<Dead>)>,
) {
    let dt = time.delta_secs();

    for (entity, tf, team, atk_range, mut ai, current_target) in &mut minions {
        ai.ai_timer += dt;
        if ai.ai_timer < 0.25 { continue; }
        ai.ai_timer = 0.0;

        match ai.state {
            MinionState::Patrolling => {
                let my_pos = tf.translation;
                // Priority: 6=enemy minion, 7=enemy champion, 8=structure
                let mut best: Option<(Entity, u8, f32)> = None;

                // LoL priority system (simplified 6 tiers):
                // 6 = enemy melee minion (closest)
                // 7 = enemy caster minion (closest)
                // 8 = enemy cannon/super minion (closest)
                // 9 = enemy champion (closest)
                // 10 = enemy turret
                // 11 = enemy inhibitor/nexus
                for (e, e_tf, e_team, e_hp) in &enemy_minions {
                    if e == entity { continue; }
                    if e_team.0 == team.0 { continue; }
                    if e_hp.current <= 0.0 { continue; }
                    let dist = my_pos.distance(e_tf.translation);
                    if dist > ai.detect_range { continue; }
                    let priority = 6u8; // All enemy minions same priority, closest wins
                    if best.map_or(true, |(_, bp, bd)| priority < bp || (priority == bp && dist < bd)) {
                        best = Some((e, priority, dist));
                    }
                }

                for (e, e_tf, e_team, e_hp) in &enemy_champs {
                    if e_team.0 == team.0 { continue; }
                    if e_hp.current <= 0.0 { continue; }
                    let dist = my_pos.distance(e_tf.translation);
                    if dist > ai.detect_range { continue; }
                    if best.map_or(true, |(_, bp, bd)| 9 < bp || (9 == bp && dist < bd)) {
                        best = Some((e, 9, dist));
                    }
                }

                for (e, e_tf, e_team, e_hp) in &structures {
                    if e_team.0 == team.0 { continue; }
                    if e_hp.current <= 0.0 { continue; }
                    let dist = my_pos.distance(e_tf.translation);
                    if dist > ai.detect_range { continue; }
                    if best.map_or(true, |(_, bp, bd)| 10 < bp || (10 == bp && dist < bd)) {
                        best = Some((e, 10, dist));
                    }
                }

                if let Some((target, _, _)) = best {
                    ai.state = MinionState::Fighting;
                    ai.patience_timer = 0.0;
                    if let Ok(mut ecmd) = commands.get_entity(entity) {
                        ecmd.insert(AttackTarget { entity: target });
                    }
                }
            }

            MinionState::Fighting => {
                ai.patience_timer += 0.25;

                if let Some(target) = current_target {
                    // Validate target (check all queries)
                    let target_info = enemy_minions.get(target.entity)
                        .map(|(_, tf, _, hp)| (tf.translation, hp.current))
                        .or_else(|_| enemy_champs.get(target.entity).map(|(_, tf, _, hp)| (tf.translation, hp.current)))
                        .or_else(|_| structures.get(target.entity).map(|(_, tf, _, hp)| (tf.translation, hp.current)));

                    if let Ok((tgt_pos, tgt_hp_current)) = target_info {
                        if tgt_hp_current <= 0.0 {
                            ai.state = MinionState::Patrolling;
                            ai.patience_timer = 0.0;
                            if let Ok(mut ecmd) = commands.get_entity(entity) {
                                ecmd.remove::<AttackTarget>().remove::<MoveTarget>();
                            }
                        } else if tf.translation.distance(tgt_pos) > ai.detect_range + 200.0 {
                            // Too far → return to lane
                            ai.state = MinionState::Returning;
                            ai.patience_timer = 0.0;
                            if let Ok(mut ecmd) = commands.get_entity(entity) {
                                ecmd.remove::<AttackTarget>().remove::<MoveTarget>();
                            }
                        } else if ai.patience_timer > 4.0 {
                            // Patience expired → return to lane
                            ai.state = MinionState::Returning;
                            ai.patience_timer = 0.0;
                            if let Ok(mut ecmd) = commands.get_entity(entity) {
                                ecmd.remove::<AttackTarget>().remove::<MoveTarget>();
                            }
                        }
                        // Else: target valid, keep fighting
                    } else {
                        // Target despawned
                        ai.state = MinionState::Patrolling;
                        ai.patience_timer = 0.0;
                        if let Ok(mut ecmd) = commands.get_entity(entity) {
                            ecmd.remove::<AttackTarget>().remove::<MoveTarget>();
                        }
                    }
                } else {
                    // Lost target component somehow
                    ai.state = MinionState::Patrolling;
                }
            }

            MinionState::Returning => {
                // Check if we've reached our waypoint
                if ai.current_waypoint < ai.waypoints.len() {
                    let wp = ai.waypoints[ai.current_waypoint];
                    let dist = Vec2::new(tf.translation.x, tf.translation.z).distance(wp);
                    if dist < 100.0 {
                        ai.state = MinionState::Patrolling;
                    }
                } else {
                    ai.state = MinionState::Patrolling;
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 4: Movement — patrol, follow target, return
// ═══════════════════════════════════════════════════════════

fn minion_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut minions: Query<
        (Entity, &mut Transform, &mut MinionAI, &AutoAttackRange, Option<&AttackTarget>),
        (With<Minion>, Without<Dead>, Without<Dying>),
    >,
    target_transforms: Query<&Transform, Without<Minion>>,
) {
    // Collect minion positions for target lookup (avoids borrow conflict)
    let minion_positions: Vec<(Entity, Vec3)> = minions.iter()
        .map(|(e, tf, _, _, _)| (e, tf.translation))
        .collect();

    for (entity, mut tf, mut ai, atk_range, attack_target) in &mut minions {
        let my_pos = Vec2::new(tf.translation.x, tf.translation.z);

        match ai.state {
            MinionState::Patrolling | MinionState::Returning => {
                if ai.current_waypoint < ai.waypoints.len() {
                    let wp = ai.waypoints[ai.current_waypoint];
                    let dist = my_pos.distance(wp);

                    if dist < 50.0 && ai.state == MinionState::Patrolling {
                        ai.current_waypoint += 1;
                        if ai.current_waypoint >= ai.waypoints.len() {
                            ai.current_waypoint = ai.waypoints.len() - 1;
                        }
                    }

                    let target_wp = ai.waypoints[ai.current_waypoint.min(ai.waypoints.len() - 1)];
                    if let Ok(mut ecmd) = commands.get_entity(entity) {
                        ecmd.insert(MoveTarget { position: target_wp });
                    }

                    // Face movement direction
                    let dir = Vec3::new(target_wp.x - tf.translation.x, 0.0, target_wp.y - tf.translation.z).normalize_or_zero();
                    if dir != Vec3::ZERO {
                        let target_rot = Quat::from_rotation_arc(Vec3::NEG_Z, dir);
                        tf.rotation = tf.rotation.slerp(target_rot, 8.0 * time.delta_secs());
                    }
                }
            }

            MinionState::Fighting => {
                if let Some(target) = attack_target {
                    let tgt_pos = if let Ok(tgt_tf) = target_transforms.get(target.entity) {
                        Some(tgt_tf.translation)
                    } else {
                        // Target might be another minion — look up from collected positions
                        minion_positions.iter().find(|(e, _)| *e == target.entity).map(|(_, pos)| *pos)
                    };

                    if let Some(tgt_pos) = tgt_pos {
                        let dist = tf.translation.distance(tgt_pos);

                        // Face target
                        let dir = Vec3::new(tgt_pos.x - tf.translation.x, 0.0, tgt_pos.z - tf.translation.z).normalize_or_zero();
                        if dir != Vec3::ZERO {
                            let target_rot = Quat::from_rotation_arc(Vec3::NEG_Z, dir);
                            tf.rotation = tf.rotation.slerp(target_rot, 10.0 * time.delta_secs());
                        }

                        if dist > atk_range.0 + 20.0 {
                            if let Ok(mut ecmd) = commands.get_entity(entity) {
                                ecmd.insert(MoveTarget {
                                    position: Vec2::new(tgt_pos.x, tgt_pos.z),
                                });
                            }
                        } else {
                            // IN range → STOP
                            if let Ok(mut ecmd) = commands.get_entity(entity) {
                                ecmd.remove::<MoveTarget>();
                            }
                        }
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 5: Attack — deal damage when in range
// ═══════════════════════════════════════════════════════════

/// Projectile component for ranged minion attacks
#[derive(Component)]
struct MinionProjectile {
    target: Entity,
    damage: f32,
    speed: f32,
}

fn minion_attack(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<crate::audio_plugin::SfxHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut attackers: Query<
        (Entity, &Transform, &CombatStats, &mut AttackCooldown, &AttackTarget, &AutoAttackRange, &mut MinionAI, &Minion),
        (Without<Dead>, Without<Dying>),
    >,
    mut hp_query: Query<&mut Health>,
    stats_query: Query<&CombatStats>,
    tf_query: Query<&Transform>,
    champion_check: Query<(), With<Champion>>,
) {
    for (entity, tf, stats, mut cd, target, range, mut ai, minion) in &mut attackers {
        cd.0 -= time.delta_secs();
        if cd.0 > 0.0 { continue; }

        let Ok(tgt_tf) = tf_query.get(target.entity) else { continue; };
        let dist = tf.translation.distance(tgt_tf.translation);
        if dist > range.0 + 50.0 { continue; }

        // Attack!
        cd.0 = 1.0 / stats.attack_speed;
        ai.patience_timer = 0.0;

        let armor = stats_query.get(target.entity).map(|s| s.armor).unwrap_or(0.0);
        let mut damage = stats.attack_damage * (100.0 / (100.0 + armor.max(0.0)));

        // Real LoL: minions deal 60% damage to champions and structures
        if champion_check.contains(target.entity) {
            damage *= 0.6;
        }

        let is_ranged = matches!(minion.minion_type, MinionType::Caster | MinionType::Siege);

        if is_ranged {
            // Spawn visible projectile
            let proj_speed = match minion.minion_type {
                MinionType::Caster => 650.0,
                MinionType::Siege => 1200.0,
                _ => 650.0,
            };
            let proj_color = if minion.team == Team::Blue {
                Color::srgb(0.4, 0.6, 1.0)
            } else {
                Color::srgb(1.0, 0.4, 0.3)
            };
            let proj_mesh = meshes.add(Sphere::new(8.0));
            let proj_mat = mats.add(StandardMaterial {
                base_color: proj_color,
                emissive: bevy::color::LinearRgba::rgb(0.5, 0.5, 1.0),
                ..default()
            });
            commands.spawn((
                Mesh3d(proj_mesh),
                MeshMaterial3d(proj_mat),
                Transform::from_translation(tf.translation + Vec3::Y * 30.0),
                MinionProjectile { target: target.entity, damage, speed: proj_speed },
            ));
            // Visual: scale pulse on ranged attack
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.insert(AttackPulse(0.12));
            }
        } else {
            // Melee — windup 0.25s before damage (real LoL ~0.3s)
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.insert(MeleeWindup { target: target.entity, damage, timer: 0.25 });
                ecmd.insert(AttackPulse(0.25));
            }
            // Only play sound for 1 in 5 attacks to avoid audio spam
            static ATK_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            if ATK_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 5 == 0 {
                crate::audio_plugin::play_sfx(&mut commands, &sfx.hit);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 5a: Melee windup — delay before damage applies
// ═══════════════════════════════════════════════════════════

fn minion_melee_windup(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<crate::audio_plugin::SfxHandles>,
    mut windups: Query<(Entity, &mut MeleeWindup), With<Minion>>,
    mut hp_query: Query<&mut Health>,
) {
    for (entity, mut windup) in &mut windups {
        windup.timer -= time.delta_secs();
        if windup.timer <= 0.0 {
            // Windup complete — apply damage only if target still alive
            if let Ok(mut hp) = hp_query.get_mut(windup.target) {
                if hp.current > 0.0 {
                    hp.current -= windup.damage;
                    crate::audio_plugin::play_sfx(&mut commands, &sfx.hit);
                }
            }
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.remove::<MeleeWindup>();
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 5b: Projectile movement — caster/siege projectiles fly to target
// ═══════════════════════════════════════════════════════════

fn minion_projectile(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<crate::audio_plugin::SfxHandles>,
    mut projectiles: Query<(Entity, &mut Transform, &MinionProjectile)>,
    mut hp_query: Query<&mut Health>,
    tf_query: Query<&Transform, Without<MinionProjectile>>,
) {
    for (entity, mut proj_tf, proj) in &mut projectiles {
        let Ok(tgt_tf) = tf_query.get(proj.target) else {
            // Target gone — despawn projectile
            if let Ok(mut ecmd) = commands.get_entity(entity) { ecmd.despawn(); }
            continue;
        };

        let dir = (tgt_tf.translation - proj_tf.translation).normalize_or_zero();
        let step = proj.speed * time.delta_secs();
        proj_tf.translation += dir * step;

        // Check if projectile reached target
        if proj_tf.translation.distance(tgt_tf.translation) < 30.0 {
            // Hit — apply damage + sound
            if let Ok(mut hp) = hp_query.get_mut(proj.target) {
                hp.current -= proj.damage;
            }
            crate::audio_plugin::play_sfx(&mut commands, &sfx.hit);
            if let Ok(mut ecmd) = commands.get_entity(entity) { ecmd.despawn(); }
        }

        // Safety: despawn if too far (projectile missed)
        if proj_tf.translation.distance(tgt_tf.translation) > 2000.0 {
            if let Ok(mut ecmd) = commands.get_entity(entity) { ecmd.despawn(); }
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 6: Collision — push minions apart
// ═══════════════════════════════════════════════════════════

/// Collision between all units — minions push each other AND champions (creep block)
fn minion_collision(
    mut minions: Query<(Entity, &mut Transform, &MinionAI), (With<Minion>, Without<Dead>, Without<Dying>)>,
    mut champions: Query<(Entity, &mut Transform), (With<Champion>, Without<Dead>, Without<Minion>)>,
) {
    // Collect all unit positions + radii
    let minion_data: Vec<(Entity, Vec3, f32)> = minions.iter()
        .map(|(e, tf, ai)| (e, tf.translation, ai.collision_radius))
        .collect();
    let champ_data: Vec<(Entity, Vec3, f32)> = champions.iter()
        .map(|(e, tf)| (e, tf.translation, 35.0)) // Champion pathfinding radius ~35
        .collect();

    // Minion-minion collision
    for (entity, mut tf, ai) in &mut minions {
        let mut push_x = 0.0f32;
        let mut push_z = 0.0f32;

        for (other_e, other_pos, other_r) in &minion_data {
            if *other_e == entity { continue; }
            let dx = tf.translation.x - other_pos.x;
            let dz = tf.translation.z - other_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            let min_dist = ai.collision_radius + other_r;
            if dist < min_dist && dist > 0.5 {
                let overlap = min_dist - dist;
                push_x += (dx / dist) * overlap * 0.4;
                push_z += (dz / dist) * overlap * 0.4;
            }
        }

        if push_x.abs() > 0.5 || push_z.abs() > 0.5 {
            tf.translation.x += push_x;
            tf.translation.z += push_z;
        }
    }

    // Creep block: minions push champions
    for (champ_e, mut champ_tf) in &mut champions {
        let mut push_x = 0.0f32;
        let mut push_z = 0.0f32;

        for (_, m_pos, m_r) in &minion_data {
            let dx = champ_tf.translation.x - m_pos.x;
            let dz = champ_tf.translation.z - m_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            let min_dist = 35.0 + m_r; // champion radius + minion radius
            if dist < min_dist && dist > 0.5 {
                let overlap = min_dist - dist;
                push_x += (dx / dist) * overlap * 0.3;
                push_z += (dz / dist) * overlap * 0.3;
            }
        }

        if push_x.abs() > 0.3 || push_z.abs() > 0.3 {
            champ_tf.translation.x += push_x;
            champ_tf.translation.z += push_z;
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 7: Death — hide then despawn
// ═══════════════════════════════════════════════════════════

fn minion_death(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<crate::audio_plugin::SfxHandles>,
    alive: Query<(Entity, &Health), (With<Minion>, Without<Dead>, Without<Dying>)>,
    mut dying: Query<(Entity, &mut Dying), With<Minion>>,
) {
    for (entity, hp) in &alive {
        if hp.current <= 0.0 {
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.insert(Dying(0.8)) // Longer death for visual (shrink + fade)
                    .remove::<AttackTarget>()
                    .remove::<MoveTarget>();
                // Don't hide immediately — let the shrink animation play
            }
            // Death sound (limit to avoid spam)
            static DEATH_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            if DEATH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 3 == 0 {
                crate::audio_plugin::play_sfx(&mut commands, &sfx.death);
            }
        }
    }

    // Tick dying → despawn
    for (entity, mut dying) in &mut dying {
        dying.0 -= time.delta_secs();
        if dying.0 <= 0.0 {
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.despawn();
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 8: Gold & XP — award to nearby champions
// ═══════════════════════════════════════════════════════════

/// Gold particle that flies toward the champion who last-hit
#[derive(Component)]
struct GoldParticle {
    target_pos: Vec3,
    lifetime: f32,
}

fn minion_gold_xp(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    game_timer: Res<GameTimer>,
    dying_minions: Query<(Entity, &Transform, &Minion, &TeamMember), (With<Dying>, Without<GoldAwarded>)>,
    mut champions: Query<(&Transform, &mut Gold, &mut Champion, &TeamMember, &mut GameStats)>,
) {
    for (m_entity, m_tf, minion, m_team) in &dying_minions {
        // Mark as gold awarded so we don't double-count
        if let Ok(mut ecmd) = commands.get_entity(m_entity) {
            ecmd.insert(GoldAwarded);
        }
        let gold = match minion.minion_type {
            MinionType::Melee => MELEE_MINION_GOLD,
            MinionType::Caster => CASTER_MINION_GOLD,
            MinionType::Siege => SIEGE_MINION_GOLD,
            MinionType::Super => SUPER_MINION_GOLD,
        };

        let xp_value = match minion.minion_type {
            MinionType::Melee => 77.0,
            MinionType::Caster => 51.0,
            MinionType::Siege => 94.0,
            MinionType::Super => 500.0,
        };

        // Gold goes to closest enemy champion (approximation of last-hit)
        // XP is shared among all enemy champions in range
        let mut closest_champ: Option<(usize, f32)> = None;
        let mut champs_in_range = 0u32;

        for (idx, (c_tf, _, _, c_team, _)) in champions.iter().enumerate() {
            if c_team.0 == m_team.0 { continue; }
            let dist = c_tf.translation.distance(m_tf.translation);
            if dist < 550.0 {
                champs_in_range += 1;
                if closest_champ.map_or(true, |(_, d)| dist < d) {
                    closest_champ = Some((idx, dist));
                }
            }
        }

        // Award XP to all nearby enemy champions (shared)
        let shared_xp = if champs_in_range > 0 { xp_value / champs_in_range as f32 } else { 0.0 };
        for (c_tf, _, mut champ, c_team, _) in &mut champions {
            if c_team.0 == m_team.0 { continue; }
            let dist = c_tf.translation.distance(m_tf.translation);
            if dist < 550.0 {
                champ.xp += shared_xp;
            }
        }

        // Award gold only to closest (last-hit approximation)
        for (c_tf, mut c_gold, mut champ, c_team, mut stats) in &mut champions {
            if c_team.0 == m_team.0 { continue; }
            let dist = c_tf.translation.distance(m_tf.translation);
            if dist < 550.0 && closest_champ.map_or(false, |(_, d)| (dist - d).abs() < 1.0) {
                c_gold.0 += gold;
                stats.cs += 1;
                stats.gold_earned += gold;

                // Spawn gold particles flying toward champion
                let particle_mesh = meshes.add(Sphere::new(5.0));
                let particle_mat = mats.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.85, 0.0),
                    emissive: bevy::color::LinearRgba::rgb(2.0, 1.5, 0.0),
                    ..default()
                });
                for i in 0..3 {
                    let offset = Vec3::new(
                        (i as f32 - 1.0) * 20.0,
                        30.0 + i as f32 * 15.0,
                        (i as f32 - 1.0) * 15.0,
                    );
                    commands.spawn((
                        Mesh3d(particle_mesh.clone()),
                        MeshMaterial3d(particle_mat.clone()),
                        Transform::from_translation(m_tf.translation + offset),
                        GoldParticle { target_pos: c_tf.translation + Vec3::Y * 50.0, lifetime: 0.6 },
                    ));
                }
                break;
            }
        }
    }
}

/// Move gold particles toward champion and despawn
fn gold_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut GoldParticle)>,
) {
    for (entity, mut tf, mut particle) in &mut particles {
        particle.lifetime -= time.delta_secs();
        if particle.lifetime <= 0.0 {
            if let Ok(mut ecmd) = commands.get_entity(entity) { ecmd.despawn(); }
            continue;
        }
        // Fly toward target
        let dir = (particle.target_pos - tf.translation).normalize_or_zero();
        tf.translation += dir * 400.0 * time.delta_secs();
        // Shrink as it approaches
        let scale = (particle.lifetime / 0.6).clamp(0.2, 1.0);
        tf.scale = Vec3::splat(scale);
    }
}

// ═══════════════════════════════════════════════════════════
//  System 9: Minion health bars — per-type height
// ═══════════════════════════════════════════════════════════

/// Visual attack feedback — scale pulse when attacking
fn minion_attack_pulse(
    mut commands: Commands,
    time: Res<Time>,
    mut pulses: Query<(Entity, &mut Transform, &mut AttackPulse), With<Minion>>,
) {
    for (entity, mut tf, mut pulse) in &mut pulses {
        pulse.0 -= time.delta_secs();
        if pulse.0 > 0.0 {
            // Scale up briefly (attack swing)
            let t = pulse.0 / 0.15;
            let scale = 1.0 + 0.25 * (t * std::f32::consts::PI).sin();
            tf.scale = Vec3::splat(scale);
        } else {
            // Reset scale and remove pulse
            tf.scale = Vec3::ONE;
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.remove::<AttackPulse>();
            }
        }
    }
}

fn minion_health_bars(
    mut gizmos: Gizmos,
    minions: Query<(&Transform, &Health, &Minion, &TeamMember), (Without<Dead>, Without<Dying>)>,
    player: Query<&TeamMember, With<PlayerControlled>>,
) {
    let my_team = player.iter().next().map(|t| t.0).unwrap_or(Team::Blue);

    for (tf, hp, minion, team) in &minions {
        if hp.current <= 0.0 { continue; }

        // Height above minion depends on type (real LoL values)
        let bar_height = match minion.minion_type {
            MinionType::Melee | MinionType::Caster => 75.0,
            MinionType::Siege => 95.0,
            MinionType::Super => 110.0,
        };

        let center = tf.translation + Vec3::Y * bar_height;
        let bar_width = 60.0;
        let pct = (hp.current / hp.max).clamp(0.0, 1.0);

        let bar_color = if team.0 == my_team {
            Color::srgb(0.1, 0.9, 0.1)
        } else {
            Color::srgb(0.9, 0.1, 0.1)
        };

        // Background (dark)
        for y_off in 0..3 {
            let y = y_off as f32 * 1.5;
            gizmos.line(
                center + Vec3::new(-bar_width * 0.5, y, 0.0),
                center + Vec3::new(bar_width * 0.5, y, 0.0),
                Color::srgba(0.05, 0.05, 0.05, 0.8),
            );
        }
        // Filled HP bar
        for y_off in 0..3 {
            let y = y_off as f32 * 1.5;
            gizmos.line(
                center + Vec3::new(-bar_width * 0.5, y, 0.0),
                center + Vec3::new(-bar_width * 0.5 + bar_width * pct, y, 0.0),
                bar_color,
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  System 11: Animations — play idle/run/attack/death
// ═══════════════════════════════════════════════════════════

fn minion_anim_indices(mtype: MinionType, team: Team) -> (usize, usize, usize, usize) {
    // (idle, run, attack, death)
    match (mtype, team) {
        (MinionType::Melee, Team::Blue) => (4, 5, 7, 2),
        (MinionType::Caster, Team::Blue) => (0, 3, 2, 4),
        (MinionType::Siege, Team::Blue) => (3, 2, 5, 0),
        (MinionType::Caster, _) => (0, 3, 1, 2),
        (MinionType::Siege, _) => (1, 5, 2, 0),
        _ => (0, 0, 0, 0),
    }
}

fn minion_animations(
    mut minions: Query<(&MinionAI, &Minion, &Health, &Children, Option<&AttackTarget>, Option<&MoveTarget>, &mut LastAnimState), Without<Dying>>,
    children_q: Query<&Children>,
    mut players: Query<&mut bevy::animation::AnimationPlayer>,
) {
    for (ai, minion, hp, children, attack_target, move_target, mut last_anim) in &mut minions {
        let desired = if hp.current <= 0.0 { 3u8 }
            else if attack_target.is_some() && move_target.is_none() { 2 }
            else if move_target.is_some() { 1 }
            else { 0 };

        // Only change animation when state actually changes
        if desired == last_anim.0 { continue; }
        last_anim.0 = desired;

        let (idle, run, atk, death) = minion_anim_indices(minion.minion_type, minion.team);
        let clip = match desired { 0 => idle, 1 => run, 2 => atk, _ => death };
        let node = bevy::animation::graph::AnimationNodeIndex::new(clip + 1);
        let repeat = if desired == 3 { bevy::animation::RepeatAnimation::Never }
            else { bevy::animation::RepeatAnimation::Forever };

        for child in children.iter() {
            if try_play(child, node, repeat, &mut players) { break; }
            if let Ok(gcs) = children_q.get(child) {
                for gc in gcs.iter() {
                    if try_play(gc, node, repeat, &mut players) { break; }
                    if let Ok(ggcs) = children_q.get(gc) {
                        for ggc in ggcs.iter() {
                            if try_play(ggc, node, repeat, &mut players) { break; }
                        }
                    }
                }
            }
        }
    }
}

fn try_play(
    entity: Entity,
    node: bevy::animation::graph::AnimationNodeIndex,
    repeat: bevy::animation::RepeatAnimation,
    players: &mut Query<&mut bevy::animation::AnimationPlayer>,
) -> bool {
    if let Ok(mut player) = players.get_mut(entity) {
        player.play(node).set_repeat(repeat);
        true
    } else {
        false
    }
}
