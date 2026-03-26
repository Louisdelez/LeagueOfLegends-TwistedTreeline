use bevy::prelude::*;
use sg_core::{components::*, types::DamageType};

/// Calculate effective damage after armor/MR reduction
pub fn calculate_damage(
    raw_damage: f32,
    damage_type: DamageType,
    attacker: &CombatStats,
    target: &CombatStats,
) -> f32 {
    match damage_type {
        DamageType::Physical => {
            let effective_armor = target.armor * (1.0 - attacker.armor_pen_pct) - attacker.armor_pen_flat;
            apply_resistance(raw_damage, effective_armor)
        }
        DamageType::Magical => {
            let effective_mr = target.magic_resist * (1.0 - attacker.magic_pen_pct) - attacker.magic_pen_flat;
            apply_resistance(raw_damage, effective_mr)
        }
        DamageType::True => raw_damage,
    }
}

fn apply_resistance(damage: f32, resistance: f32) -> f32 {
    if resistance >= 0.0 {
        damage * 100.0 / (100.0 + resistance)
    } else {
        damage * (2.0 - 100.0 / (100.0 - resistance))
    }
}
