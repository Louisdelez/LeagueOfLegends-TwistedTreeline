use serde::{Deserialize, Serialize};
use sg_core::components::CombatStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    pub id: u32,
    pub name: String,
    pub total_cost: u32,
    pub sell_value: u32,
    pub components: Vec<u32>,
    pub stat_bonuses: StatBonuses,
    pub max_stacks: u8,
    pub unique: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatBonuses {
    pub health: f32,
    pub mana: f32,
    pub health_regen: f32,
    pub mana_regen: f32,
    pub attack_damage: f32,
    pub ability_power: f32,
    pub armor: f32,
    pub magic_resist: f32,
    pub attack_speed: f32,
    pub crit_chance: f32,
    pub cdr: f32,
    pub move_speed_flat: f32,
    pub move_speed_pct: f32,
    pub life_steal: f32,
    pub spell_vamp: f32,
    pub armor_pen: f32,
    pub magic_pen: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: [Option<ItemInstance>; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInstance {
    pub definition_id: u32,
    pub stacks: u8,
    pub cooldown_remaining: f32,
}

impl Inventory {
    pub fn new() -> Self {
        Self { slots: Default::default() }
    }

    pub fn first_empty_slot(&self) -> Option<usize> {
        self.slots.iter().position(|s| s.is_none())
    }
}
