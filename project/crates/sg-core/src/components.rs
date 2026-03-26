use bevy::prelude::*;
use crate::types::*;

// === Identity ===

#[derive(Component, Debug)]
pub struct Champion {
    pub name: String,
    pub level: u8,
    pub xp: f32,
    pub team: Team,
}

#[derive(Component, Debug)]
pub struct Minion {
    pub minion_type: MinionType,
    pub lane: Lane,
    pub team: Team,
}

#[derive(Component, Debug)]
pub struct JungleCamp {
    pub camp_type: CampType,
    pub team: Team,
    pub respawn_timer: f32,
}

#[derive(Component, Debug)]
pub struct Structure {
    pub structure_type: StructureType,
    pub team: Team,
    pub lane: Option<Lane>,
}

#[derive(Component, Debug)]
pub struct Altar {
    pub side: AltarSide,
    pub captured_by: Option<Team>,
    pub capture_progress: f32,
    pub lockout_timer: f32,
}

// === Stats ===

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub regen: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Mana {
    pub current: f32,
    pub max: f32,
    pub regen: f32,
}

#[derive(Component, Debug, Clone)]
pub struct CombatStats {
    pub attack_damage: f32,
    pub ability_power: f32,
    pub armor: f32,
    pub magic_resist: f32,
    pub attack_speed: f32,
    pub move_speed: f32,
    pub crit_chance: f32,
    pub cdr: f32,
    pub armor_pen_flat: f32,
    pub armor_pen_pct: f32,
    pub magic_pen_flat: f32,
    pub magic_pen_pct: f32,
    pub life_steal: f32,
    pub spell_vamp: f32,
}

#[derive(Component, Debug)]
pub struct Gold(pub f32);

#[derive(Component, Debug)]
pub struct Dead {
    pub respawn_timer: f32,
}

// === Movement ===

#[derive(Component, Debug)]
pub struct MoveTarget {
    pub position: Vec2,
}

#[derive(Component, Debug)]
pub struct AttackTarget {
    pub entity: Entity,
}

#[derive(Component, Debug)]
pub struct PatrolPath {
    pub waypoints: Vec<Vec2>,
    pub current_index: usize,
}

// === Team ===

#[derive(Component, Debug, Clone, Copy)]
pub struct TeamMember(pub Team);

// === Combat Timing ===

#[derive(Component, Debug)]
pub struct AttackCooldown(pub f32);

#[derive(Component, Debug)]
pub struct AutoAttackRange(pub f32);

#[derive(Component, Debug)]
pub struct PlayerControlled;

// === Vision ===

#[derive(Component, Debug)]
pub struct VisionRange(pub f32);

#[derive(Component, Debug)]
pub struct Visible {
    pub to_blue: bool,
    pub to_red: bool,
}

// === Game Result ===

#[derive(Resource, Debug)]
pub struct GameResult {
    pub victory: bool,
    pub game_duration: f32,
}

// === Bounty ===

#[derive(Component, Debug, Default)]
pub struct KillStreak {
    pub kills: u32,
    pub deaths: u32,
}

// === Stats Tracking ===

#[derive(Component, Debug, Default)]
pub struct GameStats {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub cs: u32,
    pub gold_earned: f32,
    pub damage_dealt: f32,
    pub damage_taken: f32,
}

// === CC Markers ===

#[derive(Component, Debug)]
pub struct Stunned;

#[derive(Component, Debug)]
pub struct Rooted;

#[derive(Component, Debug)]
pub struct Silenced;

#[derive(Component, Debug, Default)]
pub struct ActiveBuffs(pub Vec<crate::BuffData>);

// === Base Stats (for recalculation) ===

#[derive(Component, Debug, Clone)]
pub struct BaseStats {
    pub attack_damage: f32,
    pub ability_power: f32,
    pub armor: f32,
    pub magic_resist: f32,
    pub attack_speed: f32,
    pub move_speed: f32,
    pub ad_per_level: f32,
    pub armor_per_level: f32,
    pub mr_per_level: f32,
    pub hp_per_level: f32,
    pub mana_per_level: f32,
    pub base_hp: f32,
    pub base_mana: f32,
}
