use bevy::prelude::*;
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
            ).run_if(in_state(AppState::InGame)),
        );
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
