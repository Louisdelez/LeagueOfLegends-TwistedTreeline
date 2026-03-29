use bevy::prelude::*;
use sg_core::components::*;
use sg_core::types::*;
use crate::menu::AppState;

pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_visibility, draw_fog_overlay).run_if(in_state(AppState::InGame)));
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
