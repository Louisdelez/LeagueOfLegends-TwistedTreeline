use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use sg_core::types::DamageType;

#[derive(Component, Debug, Clone)]
pub struct AbilitySlots {
    pub q: Ability,
    pub w: Ability,
    pub e: Ability,
    pub r: Ability,
    pub passive: Option<PassiveAbility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub name: String,
    pub level: u8,
    pub max_level: u8,
    pub cooldown: [f32; 5],
    pub current_cooldown: f32,
    pub mana_cost: [f32; 5],
    pub cast_range: [f32; 5],
    pub damage_type: DamageType,
    pub ad_ratio: f32,
    pub ap_ratio: f32,
    pub base_damage: [f32; 5],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveAbility {
    pub name: String,
    pub description: String,
}

impl Ability {
    pub fn current_cooldown_time(&self) -> f32 {
        if self.level == 0 { return 0.0; }
        self.cooldown[(self.level - 1) as usize]
    }

    pub fn current_mana_cost(&self) -> f32 {
        if self.level == 0 { return 0.0; }
        self.mana_cost[(self.level - 1) as usize]
    }

    pub fn current_range(&self) -> f32 {
        if self.level == 0 { return 0.0; }
        self.cast_range[(self.level - 1) as usize]
    }

    pub fn current_base_damage(&self) -> f32 {
        if self.level == 0 { return 0.0; }
        self.base_damage[(self.level - 1) as usize]
    }
}
