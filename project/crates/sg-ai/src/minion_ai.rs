use bevy::prelude::*;
use sg_core::components::*;
use sg_core::types::*;

const DETECT_RANGE: f32 = 475.0;
const ATTACK_RANGE: f32 = 110.0;
const CASTER_ATTACK_RANGE: f32 = 550.0;

/// Target priority for lane minions (lower = higher priority)
pub fn classify_target(target_team: Team, my_team: Team, is_champion: bool, is_turret: bool) -> u32 {
    if target_team == my_team { return u32::MAX; }
    if is_champion { return 0; }
    if is_turret { return 5; }
    10 // minion
}

/// Get attack range based on minion type
pub fn attack_range(minion_type: MinionType) -> f32 {
    match minion_type {
        MinionType::Melee | MinionType::Super => ATTACK_RANGE,
        MinionType::Caster | MinionType::Siege => CASTER_ATTACK_RANGE,
    }
}
