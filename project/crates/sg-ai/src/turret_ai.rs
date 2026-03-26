use bevy::prelude::*;
use sg_core::types::*;

const TURRET_RANGE: f32 = 800.0;

/// Turret target priority:
/// 1. Enemy champion attacking allied champion (aggro swap)
/// 2. Closest enemy minion
/// 3. Closest enemy champion
pub fn turret_target_priority(
    is_champion: bool,
    is_attacking_ally_champion: bool,
    distance: f32,
) -> (u32, u32) {
    let priority = if is_champion && is_attacking_ally_champion {
        0
    } else if !is_champion {
        1
    } else {
        2
    };
    (priority, distance as u32)
}
