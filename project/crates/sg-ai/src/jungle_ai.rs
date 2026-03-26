use bevy::prelude::*;

const LEASH_RANGE: f32 = 800.0;
const PATIENCE_MAX: f32 = 10.0;

/// Jungle mob behavior: attack closest enemy in range, return to spawn if leashed
pub struct JungleMobState {
    pub spawn_position: Vec2,
    pub patience: f32,
    pub current_target: Option<Entity>,
}

impl JungleMobState {
    pub fn new(spawn_pos: Vec2) -> Self {
        Self {
            spawn_position: spawn_pos,
            patience: PATIENCE_MAX,
            current_target: None,
        }
    }

    pub fn should_reset(&self, current_pos: Vec2) -> bool {
        current_pos.distance(self.spawn_position) > LEASH_RANGE || self.patience <= 0.0
    }
}
