pub mod components;
pub mod types;
pub mod constants;
pub mod runes;
pub mod spells;

pub use types::*;

use bevy::prelude::*;

/// Buff data stored in ActiveBuffs component
#[derive(Debug, Clone)]
pub struct BuffData {
    pub buff_type: BuffType,
    pub duration: f32,
    pub remaining: f32,
    pub source: Option<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuffType {
    Stun,
    Slow { percent: f32 },
    Root,
    Silence,
    SpeedShrine { bonus: f32 },
    VilemawBuff,
    AltarMoveSpeed,
    AltarHpRestore,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    AI,
    Movement,
    Combat,
    Spawn,
}
