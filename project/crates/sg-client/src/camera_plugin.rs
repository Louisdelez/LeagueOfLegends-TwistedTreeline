use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use sg_core::constants::*;
use sg_core::GameSet;
use crate::menu::AppState;

#[derive(Component)]
pub struct MobaCamera {
    pub speed: f32,
    pub zoom_speed: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub edge_margin: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_moba_camera)
            .add_systems(Update, camera_edge_scroll.in_set(GameSet::Input).run_if(in_state(AppState::InGame)))
            .add_systems(Update, camera_zoom.in_set(GameSet::Input).run_if(in_state(AppState::InGame)));
    }
}

fn setup_moba_camera(mut commands: Commands) {
    // NVR map center is approximately (8252, -190, 6847)
    // Camera positioned above center, looking down at ~55 degrees
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(8252.0, 3000.0, 8500.0)
            .looking_at(Vec3::new(8252.0, -190.0, 6847.0), Vec3::Y),
        MobaCamera {
            speed: 3000.0,
            zoom_speed: 300.0,
            min_height: 500.0,
            max_height: 8000.0,
            edge_margin: 20.0,
        },
    ));
}

fn camera_edge_scroll(
    windows: Query<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_q: Query<(&mut Transform, &MobaCamera)>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((mut tf, cam)) = camera_q.single_mut() else { return };
    let mut delta = Vec3::ZERO;
    let dt = time.delta_secs();

    // Keyboard
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyZ) {
        delta.z -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        delta.z += 1.0;
    }
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyQ) {
        delta.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        delta.x += 1.0;
    }

    // Mouse edge scroll
    if let Some(cursor) = window.cursor_position() {
        if cursor.x < cam.edge_margin {
            delta.x -= 1.0;
        }
        if cursor.x > window.width() - cam.edge_margin {
            delta.x += 1.0;
        }
        if cursor.y < cam.edge_margin {
            delta.z -= 1.0;
        }
        if cursor.y > window.height() - cam.edge_margin {
            delta.z += 1.0;
        }
    }

    if delta != Vec3::ZERO {
        delta = delta.normalize() * cam.speed * dt;
        tf.translation += delta;
        tf.translation.x = tf.translation.x.clamp(0.0, MAP_WIDTH);
        tf.translation.z = tf.translation.z.clamp(0.0, MAP_HEIGHT);
    }
}

fn camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_q: Query<(&mut Transform, &MobaCamera)>,
) {
    let Ok((mut tf, cam)) = camera_q.single_mut() else { return };

    for ev in scroll_events.read() {
        let zoom_delta = -ev.y * cam.zoom_speed;
        tf.translation.y = (tf.translation.y + zoom_delta).clamp(cam.min_height, cam.max_height);
    }
}
