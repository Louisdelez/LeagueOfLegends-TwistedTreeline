use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use sg_core::runes::RunePage;
use sg_core::spells::SummonerSpell;
use sg_gameplay::champions::ChampionClass;

/// Ranked tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankedTier {
    Unranked, Iron, Bronze, Silver, Gold, Platinum, Diamond, Master, Grandmaster, Challenger,
}

impl RankedTier {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unranked => "Unranked", Self::Iron => "Iron", Self::Bronze => "Bronze",
            Self::Silver => "Silver", Self::Gold => "Gold", Self::Platinum => "Platinum",
            Self::Diamond => "Diamond", Self::Master => "Master",
            Self::Grandmaster => "Grandmaster", Self::Challenger => "Challenger",
        }
    }
    pub fn color(&self) -> [f32; 3] {
        match self {
            Self::Unranked => [0.5, 0.5, 0.5],
            Self::Iron => [0.4, 0.3, 0.3],
            Self::Bronze => [0.6, 0.4, 0.2],
            Self::Silver => [0.7, 0.7, 0.75],
            Self::Gold => [0.85, 0.68, 0.25],
            Self::Platinum => [0.0, 0.6, 0.6],
            Self::Diamond => [0.3, 0.5, 0.9],
            Self::Master => [0.5, 0.2, 0.7],
            Self::Grandmaster => [0.7, 0.15, 0.2],
            Self::Challenger => [0.9, 0.75, 0.3],
        }
    }
}

/// Ranked division (IV lowest, I highest)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Division { IV, III, II, I }

impl Division {
    pub fn name(&self) -> &'static str {
        match self { Self::IV => "IV", Self::III => "III", Self::II => "II", Self::I => "I" }
    }
}

/// Player rank info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankInfo {
    pub tier: RankedTier,
    pub division: Division,
    pub lp: u32,
    pub wins: u32,
    pub losses: u32,
}

impl Default for RankInfo {
    fn default() -> Self {
        Self { tier: RankedTier::Unranked, division: Division::IV, lp: 0, wins: 0, losses: 0 }
    }
}

/// Match result for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub won: bool,
    pub champion_class: u8,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub cs: u32,
    pub gold: u32,
    pub duration_secs: u32,
    pub lp_change: i32,
}

/// Full player profile (persisted)
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub summoner_name: String,
    pub level: u32,
    pub icon_id: u32,
    pub rank: RankInfo,
    pub honor_level: u8,
    pub rune_pages: Vec<RunePage>,
    pub selected_rune_page: usize,
    pub spell_d: SummonerSpell,
    pub spell_f: SummonerSpell,
    pub preferred_champion: Option<ChampionClass>,
    pub match_history: Vec<MatchResult>,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            summoner_name: "Summoner".into(),
            level: 1,
            icon_id: 0,
            rank: RankInfo::default(),
            honor_level: 2,
            rune_pages: vec![RunePage::default()],
            selected_rune_page: 0,
            spell_d: SummonerSpell::Flash,
            spell_f: SummonerSpell::Ignite,
            preferred_champion: None,
            match_history: vec![],
        }
    }
}

/// Queue type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    BlindPick,
    DraftPick,
    Ranked,
    Custom,
    Practice,
}

impl QueueType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BlindPick => "Normal (Blind Pick)",
            Self::DraftPick => "Normal (Draft Pick)",
            Self::Ranked => "Ranked 3v3",
            Self::Custom => "Custom Game",
            Self::Practice => "Practice Tool",
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            Self::BlindPick => "3v3 Twisted Treeline — Simultaneous champion selection, no bans",
            Self::DraftPick => "3v3 Twisted Treeline — Ban phase + turn-based picks",
            Self::Ranked => "3v3 Competitive — Draft pick with LP rewards",
            Self::Custom => "Create a custom lobby with friends",
            Self::Practice => "Solo practice mode — no opponents",
        }
    }
}

/// Champion select phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectPhase {
    BanPhase,
    PickPhase,
    Finalization,
}
