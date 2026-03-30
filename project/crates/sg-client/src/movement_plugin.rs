use bevy::prelude::*;
use bevy::animation::AnimationPlayer;
use bevy::animation::RepeatAnimation;
use sg_core::components::*;
use sg_core::GameSet;
use sg_gameplay::champions::ChampionId;
use crate::ability_plugin::ChampionIdentity;
use crate::menu::AppState;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                follow_patrol_path.in_set(GameSet::AI),
                move_to_target.in_set(GameSet::Movement),
                update_anim_state,
                play_champion_animations,
                switch_champion_animations,
            ).run_if(in_state(AppState::InGame)),
        );
    }
}

/// Tracks the current animation state of a champion
#[derive(Component, PartialEq, Clone, Copy)]
pub enum ChampionAnimState {
    Idle,
    Walking,
    Attacking,
    Dead,
}

impl Default for ChampionAnimState {
    fn default() -> Self { Self::Idle }
}

/// Stores animation clip indices for each action (resolved from GLTF names)
#[derive(Component, Default)]
pub struct AnimIndices {
    pub idle: u32,
    pub run: u32,
    pub attack: u32,
    pub death: u32,
    pub resolved: bool,
}

/// Update animation state based on movement/combat/death
fn update_anim_state(
    mut query: Query<(
        &mut ChampionAnimState,
        Option<&MoveTarget>,
        Option<&AttackTarget>,
        Option<&Dead>,
    ), With<Champion>>,
) {
    for (mut state, move_target, attack_target, dead) in &mut query {
        let new_state = if dead.is_some() {
            ChampionAnimState::Dead
        } else if attack_target.is_some() {
            ChampionAnimState::Attacking
        } else if move_target.is_some() {
            ChampionAnimState::Walking
        } else {
            ChampionAnimState::Idle
        };
        if *state != new_state {
            *state = new_state;
        }
    }
}

/// Auto-play idle animation when AnimationPlayer first appears
fn play_champion_animations(
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        // Play first animation as idle placeholder — will be overridden by switch_champion_animations
        let idle_clip = bevy::animation::graph::AnimationNodeIndex::new(7); // Annie idle = clip 6 → node 7
        player.play(idle_clip).set_repeat(RepeatAnimation::Forever);
    }
}

/// Switch animations based on ChampionAnimState changes
/// Searches for AnimationPlayer in children/grandchildren of champion entities
/// Per-champion animation clip indices (GLTF clip index → AnimationNodeIndex = clip + 1)
fn champion_anim_indices(id: ChampionId) -> [usize; 4] {
    // [idle, run, attack, death] — GLTF clip indices
    match id {
        ChampionId::Annie      => [6, 17, 13, 10],
        ChampionId::Garen      => [7, 8, 5, 22],
        ChampionId::Ashe       => [4, 1, 8, 12],
        ChampionId::Darius     => [1, 5, 9, 18],
        ChampionId::Lux        => [0, 1, 6, 16],
        ChampionId::Thresh     => [5, 19, 2, 36],
        ChampionId::Jinx       => [9, 8, 3, 24],
        ChampionId::Yasuo      => [24, 0, 23, 18],
        ChampionId::MasterYi   => [16, 5, 1, 3],
        ChampionId::Jax        => [4, 13, 9, 6],
        ChampionId::Teemo      => [5, 0, 13, 14],
        ChampionId::Singed     => [1, 1, 17, 6],
        ChampionId::Tryndamere => [3, 4, 6, 1],
        ChampionId::Mordekaiser=> [0, 15, 14, 31],
        ChampionId::Poppy      => [8, 0, 22, 20],
    }
}

fn switch_champion_animations(
    champions: Query<(&ChampionAnimState, &Children, Option<&ChampionIdentity>), (With<Champion>, Changed<ChampionAnimState>)>,
    children_q: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (state, children, champ_id_opt) in &champions {
        let indices = champ_id_opt
            .map(|id| champion_anim_indices(id.0))
            .unwrap_or([6, 17, 13, 10]); // default to Annie

        let clip_idx = match state {
            ChampionAnimState::Idle => indices[0],
            ChampionAnimState::Walking => indices[1],
            ChampionAnimState::Attacking => indices[2],
            ChampionAnimState::Dead => indices[3],
        };
        // AnimationNodeIndex is 1-indexed (0 = root node)
        let node = bevy::animation::graph::AnimationNodeIndex::new(clip_idx + 1);
        let repeat = match state {
            ChampionAnimState::Dead => RepeatAnimation::Never,
            _ => RepeatAnimation::Forever,
        };

        // Search recursively for AnimationPlayer
        for child in children.iter() {
            if try_play_anim(&mut players, child, node, repeat) { break; }
            if let Ok(grandchildren) = children_q.get(child) {
                for gc in grandchildren.iter() {
                    if try_play_anim(&mut players, gc, node, repeat) { break; }
                    if let Ok(ggchildren) = children_q.get(gc) {
                        for ggc in ggchildren.iter() {
                            if try_play_anim(&mut players, ggc, node, repeat) { break; }
                        }
                    }
                }
            }
        }
    }
}

fn try_play_anim(
    players: &mut Query<&mut AnimationPlayer>,
    entity: Entity,
    node: bevy::animation::graph::AnimationNodeIndex,
    repeat: RepeatAnimation,
) -> bool {
    if let Ok(mut player) = players.get_mut(entity) {
        player.play(node).set_repeat(repeat);
        true
    } else {
        false
    }
}

fn move_to_target(
    mut commands: Commands,
    time: Res<Time>,
    nav_grid: Res<sg_navigation::NavGrid>,
    mut query: Query<(Entity, &mut Transform, &MoveTarget, &CombatStats, Option<&Minion>), (Without<Stunned>, Without<Rooted>)>,
) {
    for (entity, mut tf, target, stats, is_minion) in &mut query {
        let current = Vec2::new(tf.translation.x, tf.translation.z);
        let direction = target.position - current;
        let distance = direction.length();

        if distance < 5.0 {
            commands.entity(entity).remove::<MoveTarget>();
            continue;
        }

        let step = stats.move_speed * time.delta_secs();
        let (next_x, next_z) = if step >= distance {
            (target.position.x, target.position.y)
        } else {
            let delta = direction.normalize() * step;
            (tf.translation.x + delta.x, tf.translation.z + delta.y)
        };

        // Check walkability — only for champions (minions follow patrol paths on lanes)
        let blocked = if is_minion.is_some() {
            false // Minions always move (their patrol waypoints are on walkable terrain)
        } else if nav_grid.loaded {
            !nav_grid.is_walkable_world(Vec2::new(next_x, next_z))
        } else {
            false
        };

        if !blocked {
            tf.translation.x = next_x;
            tf.translation.z = next_z;
            if step >= distance {
                commands.entity(entity).remove::<MoveTarget>();
            }
        } else {
            // Blocked by wall — remove MoveTarget
            commands.entity(entity).remove::<MoveTarget>();
        }

        // Clamp to map bounds
        tf.translation.x = tf.translation.x.clamp(500.0, 14900.0);
        tf.translation.z = tf.translation.z.clamp(3500.0, 11000.0);

        // Face movement direction
        let look_dir = Vec3::new(direction.x, 0.0, direction.y).normalize_or_zero();
        if look_dir != Vec3::ZERO {
            let target_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, look_dir);
            tf.rotation = tf.rotation.slerp(target_rotation, 10.0 * time.delta_secs());
        }
    }
}

fn follow_patrol_path(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PatrolPath, &Transform), (Without<MoveTarget>, Without<AttackTarget>)>,
) {
    for (entity, mut patrol, tf) in &mut query {
        if patrol.current_index >= patrol.waypoints.len() {
            continue;
        }

        let next_wp = patrol.waypoints[patrol.current_index];
        let current = Vec2::new(tf.translation.x, tf.translation.z);

        if current.distance(next_wp) < 50.0 {
            patrol.current_index += 1;
            if patrol.current_index >= patrol.waypoints.len() {
                continue;
            }
        }

        let target_wp = patrol.waypoints[patrol.current_index.min(patrol.waypoints.len() - 1)];
        commands.entity(entity).insert(MoveTarget { position: target_wp });
    }
}
