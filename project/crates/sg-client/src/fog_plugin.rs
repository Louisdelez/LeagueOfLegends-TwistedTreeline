use bevy::prelude::*;
use sg_core::components::*;
use sg_core::types::*;
use crate::menu::AppState;

pub struct FogPlugin;

#[derive(Component)]
pub struct FogTile { pub gx: usize, pub gz: usize }

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_fog_tiles)
            .add_systems(Update, (update_visibility, update_fog_tiles, draw_fog_overlay).run_if(in_state(AppState::InGame)));
    }
}

const FOG_RES: usize = 32; // 32x32 fog grid (each tile covers ~480 units)
const FOG_TILE_SIZE: f32 = 15398.0 / FOG_RES as f32;

/// Spawn fog overlay tiles covering the map
fn spawn_fog_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(FOG_TILE_SIZE / 2.0)));
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.05, 0.6),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    for gz in 0..FOG_RES {
        for gx in 0..FOG_RES {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(
                    gx as f32 * FOG_TILE_SIZE + FOG_TILE_SIZE / 2.0,
                    200.0,
                    gz as f32 * FOG_TILE_SIZE + FOG_TILE_SIZE / 2.0,
                ),
                FogTile { gx, gz },
            ));
        }
    }
}

/// Update fog tile visibility based on allied vision
fn update_fog_tiles(
    player_q: Query<&TeamMember, With<PlayerControlled>>,
    allies: Query<(&Transform, &TeamMember, Option<&VisionRange>), Without<Dead>>,
    mut fog_tiles: Query<(&FogTile, &mut Visibility)>,
) {
    let my_team = match player_q.iter().next() {
        Some(t) => t.0,
        None => return,
    };

    let vision_sources: Vec<(Vec2, f32)> = allies
        .iter()
        .filter(|(_, team, _)| team.0 == my_team)
        .map(|(tf, _, vr)| {
            let range = vr.map(|v| v.0).unwrap_or(1200.0);
            (Vec2::new(tf.translation.x, tf.translation.z), range)
        })
        .collect();

    for (tile, mut vis) in &mut fog_tiles {
        let tile_center = Vec2::new(
            tile.gx as f32 * FOG_TILE_SIZE + FOG_TILE_SIZE / 2.0,
            tile.gz as f32 * FOG_TILE_SIZE + FOG_TILE_SIZE / 2.0,
        );

        let in_vision = vision_sources.iter().any(|(pos, range)| {
            pos.distance(tile_center) < *range
        });

        *vis = if in_vision { Visibility::Hidden } else { Visibility::Inherited };
    }
}

/// Visual fog overlay: draw dark spheres where vision is absent
fn draw_fog_overlay(
    mut gizmos: Gizmos,
    player_q: Query<&TeamMember, With<PlayerControlled>>,
    allies: Query<(&Transform, &TeamMember, Option<&VisionRange>), Without<Dead>>,
) {
    let my_team = match player_q.iter().next() {
        Some(t) => t.0,
        None => return,
    };

    // Collect vision sources
    let vision: Vec<(Vec3, f32)> = allies
        .iter()
        .filter(|(_, team, _)| team.0 == my_team)
        .map(|(tf, _, vr)| (tf.translation, vr.map(|v| v.0).unwrap_or(1200.0)))
        .collect();

    // Draw fog circles at the edge of vision (subtle dark rings)
    for (pos, range) in &vision {
        // Draw vision boundary ring
        gizmos.circle(
            Isometry3d::from_translation(*pos + Vec3::Y * 5.0),
            *range,
            Color::srgba(0.1, 0.1, 0.2, 0.15),
        );
    }
}

/// Hide enemy entities that are outside vision range of any allied unit
fn update_visibility(
    player_q: Query<&TeamMember, With<PlayerControlled>>,
    allies: Query<(&Transform, &TeamMember, Option<&VisionRange>), Without<Dead>>,
    mut enemies: Query<
        (&Transform, &TeamMember, &mut Visibility),
        (Without<PlayerControlled>, Without<Structure>),
    >,
) {
    let my_team = match player_q.iter().next() {
        Some(t) => t.0,
        None => return,
    };

    // Collect allied vision sources
    let vision_sources: Vec<(Vec3, f32)> = allies
        .iter()
        .filter(|(_, team, _)| team.0 == my_team)
        .map(|(tf, _, vr)| {
            let range = vr.map(|v| v.0).unwrap_or(1200.0); // default 1200 for champions
            (tf.translation, range)
        })
        .collect();

    // Check each enemy entity
    for (enemy_tf, enemy_team, mut vis) in &mut enemies {
        if enemy_team.0 == my_team || enemy_team.0 == Team::Neutral {
            *vis = Visibility::Inherited;
            continue;
        }

        let enemy_pos = enemy_tf.translation;
        let visible = vision_sources.iter().any(|(pos, range)| {
            pos.distance(enemy_pos) < *range
        });

        *vis = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
