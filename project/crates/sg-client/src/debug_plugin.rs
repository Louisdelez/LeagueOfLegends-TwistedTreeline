use bevy::prelude::*;
use crate::map_plugin::MapData;
use sg_core::components::*;
use crate::menu::AppState;

#[derive(Resource)]
struct DebugVisible(bool);

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugVisible(true))
            .add_systems(Update, (toggle_debug, draw_lane_paths, draw_turret_ranges).run_if(in_state(AppState::InGame)));
    }
}

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut visible: ResMut<DebugVisible>) {
    if keys.just_pressed(KeyCode::F3) {
        visible.0 = !visible.0;
    }
}

fn draw_lane_paths(mut gizmos: Gizmos, map: Res<MapData>, visible: Res<DebugVisible>) {
    if !visible.0 {
        return;
    }

    let height = 5.0;

    // Top lane (green)
    let top = &map.0.lane_paths.top_blue;
    for i in 0..top.len().saturating_sub(1) {
        let a = Vec3::new(top[i].x, height, top[i].y);
        let b = Vec3::new(top[i + 1].x, height, top[i + 1].y);
        gizmos.line(a, b, Color::srgb(0.0, 0.8, 0.0));
    }

    // Bottom lane (orange)
    let bot = &map.0.lane_paths.bottom_blue;
    for i in 0..bot.len().saturating_sub(1) {
        let a = Vec3::new(bot[i].x, height, bot[i].y);
        let b = Vec3::new(bot[i + 1].x, height, bot[i + 1].y);
        gizmos.line(a, b, Color::srgb(0.9, 0.5, 0.0));
    }
}

fn draw_turret_ranges(
    mut gizmos: Gizmos,
    visible: Res<DebugVisible>,
    turrets: Query<(&Transform, &AutoAttackRange), With<Structure>>,
) {
    if !visible.0 {
        return;
    }

    for (tf, range) in &turrets {
        let center = Vec3::new(tf.translation.x, 2.0, tf.translation.z);
        gizmos.circle(
            Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            range.0,
            Color::srgba(1.0, 0.0, 0.0, 0.3),
        );
    }
}
