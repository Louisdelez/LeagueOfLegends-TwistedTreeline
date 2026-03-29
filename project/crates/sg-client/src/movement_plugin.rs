use bevy::prelude::*;
use bevy::animation::AnimationPlayer;
use bevy::animation::RepeatAnimation;
use sg_core::components::*;
use sg_core::GameSet;
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

/// Play animations on champions — the GLTF loader auto-creates AnimationPlayer
/// and AnimationGraph. We just need to start playing the first clip.
fn play_champion_animations(
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    // When a new AnimationPlayer appears (GLTF scene loaded), auto-play first animation
    for mut player in &mut players {
        let first_clip = bevy::animation::graph::AnimationNodeIndex::new(1);
        player.play(first_clip).set_repeat(RepeatAnimation::Forever);
    }
}

fn move_to_target(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &MoveTarget, &CombatStats), (Without<Stunned>, Without<Rooted>)>,
) {
    for (entity, mut tf, target, stats) in &mut query {
        let current = Vec2::new(tf.translation.x, tf.translation.z);
        let direction = target.position - current;
        let distance = direction.length();

        if distance < 5.0 {
            commands.entity(entity).remove::<MoveTarget>();
            continue;
        }

        let step = stats.move_speed * time.delta_secs();
        if step >= distance {
            tf.translation.x = target.position.x;
            tf.translation.z = target.position.y;
            commands.entity(entity).remove::<MoveTarget>();
        } else {
            let delta = direction.normalize() * step;
            tf.translation.x += delta.x;
            tf.translation.z += delta.y;
        }

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
    mut query: Query<(Entity, &mut PatrolPath, &Transform), Without<MoveTarget>>,
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
