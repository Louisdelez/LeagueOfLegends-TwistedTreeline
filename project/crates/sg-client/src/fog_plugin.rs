use bevy::prelude::*;
use sg_core::components::*;
use sg_core::types::*;
use crate::menu::AppState;

pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_visibility.run_if(in_state(AppState::InGame)));
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
