use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone)]
pub struct Buff {
    pub buff_type: BuffType,
    pub duration: f32,
    pub remaining: f32,
    pub stacks: u32,
    pub source: Entity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuffType {
    // Vilemaw buff
    CrestOfCrushingWrath,
    // Altar buffs
    AltarMovementSpeed,
    AltarHpRestore,
    // Speed shrine
    SpeedShrineBuff,
    // Health relic
    HealthRelicHeal,
    // CC
    Stun,
    Slow { percent: f32 },
    Root,
    Silence,
    // Combat
    ArmorShred { per_stack: f32 },
}
