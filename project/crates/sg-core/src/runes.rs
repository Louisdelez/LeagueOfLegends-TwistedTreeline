use serde::{Deserialize, Serialize};

/// Rune path (primary or secondary tree)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RunePath {
    Precision,
    Domination,
    Sorcery,
    Resolve,
    Inspiration,
}

impl RunePath {
    pub fn all() -> &'static [RunePath] {
        &[RunePath::Precision, RunePath::Domination, RunePath::Sorcery, RunePath::Resolve, RunePath::Inspiration]
    }
    pub fn name(&self) -> &'static str {
        match self { Self::Precision => "Precision", Self::Domination => "Domination", Self::Sorcery => "Sorcery", Self::Resolve => "Resolve", Self::Inspiration => "Inspiration" }
    }
    pub fn color_hex(&self) -> &'static str {
        match self { Self::Precision => "#C8AA6E", Self::Domination => "#E84057", Self::Sorcery => "#9B59B6", Self::Resolve => "#2ECC71", Self::Inspiration => "#49B4BB" }
    }
    pub fn color_rgb(&self) -> [f32; 3] {
        match self {
            Self::Precision => [0.78, 0.67, 0.43],
            Self::Domination => [0.91, 0.25, 0.34],
            Self::Sorcery => [0.61, 0.35, 0.71],
            Self::Resolve => [0.18, 0.80, 0.44],
            Self::Inspiration => [0.29, 0.71, 0.73],
        }
    }
}

/// Keystone rune ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Keystone {
    // Precision
    PressTheAttack, LethalTempo, FleetFootwork, Conqueror,
    // Domination
    Electrocute, Predator, DarkHarvest, HailOfBlades,
    // Sorcery
    SummonAery, ArcaneComet, PhaseRush,
    // Resolve
    GraspOfTheUndying, Aftershock, Guardian,
    // Inspiration
    GlacialAugment, Kleptomancy, UnsealedSpellbook,
}

impl Keystone {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PressTheAttack => "Press the Attack", Self::LethalTempo => "Lethal Tempo",
            Self::FleetFootwork => "Fleet Footwork", Self::Conqueror => "Conqueror",
            Self::Electrocute => "Electrocute", Self::Predator => "Predator",
            Self::DarkHarvest => "Dark Harvest", Self::HailOfBlades => "Hail of Blades",
            Self::SummonAery => "Summon Aery", Self::ArcaneComet => "Arcane Comet",
            Self::PhaseRush => "Phase Rush",
            Self::GraspOfTheUndying => "Grasp of the Undying", Self::Aftershock => "Aftershock",
            Self::Guardian => "Guardian",
            Self::GlacialAugment => "Glacial Augment", Self::Kleptomancy => "Kleptomancy",
            Self::UnsealedSpellbook => "Unsealed Spellbook",
        }
    }
    pub fn path(&self) -> RunePath {
        match self {
            Self::PressTheAttack | Self::LethalTempo | Self::FleetFootwork | Self::Conqueror => RunePath::Precision,
            Self::Electrocute | Self::Predator | Self::DarkHarvest | Self::HailOfBlades => RunePath::Domination,
            Self::SummonAery | Self::ArcaneComet | Self::PhaseRush => RunePath::Sorcery,
            Self::GraspOfTheUndying | Self::Aftershock | Self::Guardian => RunePath::Resolve,
            Self::GlacialAugment | Self::Kleptomancy | Self::UnsealedSpellbook => RunePath::Inspiration,
        }
    }
    pub fn keystones_for(path: RunePath) -> Vec<Keystone> {
        match path {
            RunePath::Precision => vec![Self::PressTheAttack, Self::LethalTempo, Self::FleetFootwork, Self::Conqueror],
            RunePath::Domination => vec![Self::Electrocute, Self::Predator, Self::DarkHarvest, Self::HailOfBlades],
            RunePath::Sorcery => vec![Self::SummonAery, Self::ArcaneComet, Self::PhaseRush],
            RunePath::Resolve => vec![Self::GraspOfTheUndying, Self::Aftershock, Self::Guardian],
            RunePath::Inspiration => vec![Self::GlacialAugment, Self::Kleptomancy, Self::UnsealedSpellbook],
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            Self::PressTheAttack => "3 consecutive attacks deal bonus damage and expose the target",
            Self::LethalTempo => "Gain attack speed after damaging a champion",
            Self::FleetFootwork => "Energized attacks heal and grant movement speed",
            Self::Conqueror => "Stacking adaptive force in combat, max stacks heal",
            Self::Electrocute => "3 hits within 3s deal bonus adaptive damage",
            Self::Predator => "Enchant boots for a speed burst toward champions",
            Self::DarkHarvest => "Bonus damage on low-health targets, stacking souls",
            Self::HailOfBlades => "First 3 attacks gain massive attack speed",
            Self::SummonAery => "Abilities send Aery to damage enemies or shield allies",
            Self::ArcaneComet => "Abilities hurl a comet at the target",
            Self::PhaseRush => "3 hits grant a burst of movement speed",
            Self::GraspOfTheUndying => "Every 4s in combat, next attack heals and gains HP",
            Self::Aftershock => "After immobilizing, gain resists then explode",
            Self::Guardian => "Guard nearby allies with a shield on damage",
            Self::GlacialAugment => "Attacks slow, item actives create freeze rays",
            Self::Kleptomancy => "After abilities, attacks grant random items/gold",
            Self::UnsealedSpellbook => "Swap summoner spells during the game",
        }
    }
}

/// Minor rune ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MinorRune {
    // Precision Slot 1
    Overheal, Triumph, PresenceOfMind,
    // Precision Slot 2
    LegendAlacrity, LegendTenacity, LegendBloodline,
    // Precision Slot 3
    CoupDeGrace, CutDown, LastStand,
    // Domination Slot 1
    CheapShot, TasteOfBlood, SuddenImpact,
    // Domination Slot 2
    ZombieWard, GhostPoro, EyeballCollection,
    // Domination Slot 3
    RavenousHunter, IngeniousHunter, RelentlessHunter, UltimateHunter,
    // Sorcery Slot 1
    NullifyingOrb, ManaflowBand, NimbusCloak,
    // Sorcery Slot 2
    Transcendence, Celerity, AbsoluteFocus,
    // Sorcery Slot 3
    Scorch, Waterwalking, GatheringStorm,
    // Resolve Slot 1
    Demolish, FontOfLife, ShieldBash,
    // Resolve Slot 2
    Conditioning, SecondWind, BonePlating,
    // Resolve Slot 3
    Overgrowth, Revitalize, Unflinching,
    // Inspiration Slot 1
    HextechFlashtraption, MagicalFootwear, PerfectTiming,
    // Inspiration Slot 2
    FuturesMarket, MinionDematerializer, BiscuitDelivery,
    // Inspiration Slot 3
    CosmicInsight, ApproachVelocity, TimeWarpTonic,
}

impl MinorRune {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Overheal => "Overheal", Self::Triumph => "Triumph", Self::PresenceOfMind => "Presence of Mind",
            Self::LegendAlacrity => "Legend: Alacrity", Self::LegendTenacity => "Legend: Tenacity", Self::LegendBloodline => "Legend: Bloodline",
            Self::CoupDeGrace => "Coup de Grace", Self::CutDown => "Cut Down", Self::LastStand => "Last Stand",
            Self::CheapShot => "Cheap Shot", Self::TasteOfBlood => "Taste of Blood", Self::SuddenImpact => "Sudden Impact",
            Self::ZombieWard => "Zombie Ward", Self::GhostPoro => "Ghost Poro", Self::EyeballCollection => "Eyeball Collection",
            Self::RavenousHunter => "Ravenous Hunter", Self::IngeniousHunter => "Ingenious Hunter", Self::RelentlessHunter => "Relentless Hunter", Self::UltimateHunter => "Ultimate Hunter",
            Self::NullifyingOrb => "Nullifying Orb", Self::ManaflowBand => "Manaflow Band", Self::NimbusCloak => "Nimbus Cloak",
            Self::Transcendence => "Transcendence", Self::Celerity => "Celerity", Self::AbsoluteFocus => "Absolute Focus",
            Self::Scorch => "Scorch", Self::Waterwalking => "Waterwalking", Self::GatheringStorm => "Gathering Storm",
            Self::Demolish => "Demolish", Self::FontOfLife => "Font of Life", Self::ShieldBash => "Shield Bash",
            Self::Conditioning => "Conditioning", Self::SecondWind => "Second Wind", Self::BonePlating => "Bone Plating",
            Self::Overgrowth => "Overgrowth", Self::Revitalize => "Revitalize", Self::Unflinching => "Unflinching",
            Self::HextechFlashtraption => "Hextech Flashtraption", Self::MagicalFootwear => "Magical Footwear", Self::PerfectTiming => "Perfect Timing",
            Self::FuturesMarket => "Future's Market", Self::MinionDematerializer => "Minion Dematerializer", Self::BiscuitDelivery => "Biscuit Delivery",
            Self::CosmicInsight => "Cosmic Insight", Self::ApproachVelocity => "Approach Velocity", Self::TimeWarpTonic => "Time Warp Tonic",
        }
    }

    pub fn path(&self) -> RunePath {
        match self {
            Self::Overheal | Self::Triumph | Self::PresenceOfMind |
            Self::LegendAlacrity | Self::LegendTenacity | Self::LegendBloodline |
            Self::CoupDeGrace | Self::CutDown | Self::LastStand => RunePath::Precision,
            Self::CheapShot | Self::TasteOfBlood | Self::SuddenImpact |
            Self::ZombieWard | Self::GhostPoro | Self::EyeballCollection |
            Self::RavenousHunter | Self::IngeniousHunter | Self::RelentlessHunter | Self::UltimateHunter => RunePath::Domination,
            Self::NullifyingOrb | Self::ManaflowBand | Self::NimbusCloak |
            Self::Transcendence | Self::Celerity | Self::AbsoluteFocus |
            Self::Scorch | Self::Waterwalking | Self::GatheringStorm => RunePath::Sorcery,
            Self::Demolish | Self::FontOfLife | Self::ShieldBash |
            Self::Conditioning | Self::SecondWind | Self::BonePlating |
            Self::Overgrowth | Self::Revitalize | Self::Unflinching => RunePath::Resolve,
            Self::HextechFlashtraption | Self::MagicalFootwear | Self::PerfectTiming |
            Self::FuturesMarket | Self::MinionDematerializer | Self::BiscuitDelivery |
            Self::CosmicInsight | Self::ApproachVelocity | Self::TimeWarpTonic => RunePath::Inspiration,
        }
    }

    /// Slot index within the path (0, 1, or 2)
    pub fn slot(&self) -> usize {
        match self {
            Self::Overheal | Self::Triumph | Self::PresenceOfMind |
            Self::CheapShot | Self::TasteOfBlood | Self::SuddenImpact |
            Self::NullifyingOrb | Self::ManaflowBand | Self::NimbusCloak |
            Self::Demolish | Self::FontOfLife | Self::ShieldBash |
            Self::HextechFlashtraption | Self::MagicalFootwear | Self::PerfectTiming => 0,
            Self::LegendAlacrity | Self::LegendTenacity | Self::LegendBloodline |
            Self::ZombieWard | Self::GhostPoro | Self::EyeballCollection |
            Self::Transcendence | Self::Celerity | Self::AbsoluteFocus |
            Self::Conditioning | Self::SecondWind | Self::BonePlating |
            Self::FuturesMarket | Self::MinionDematerializer | Self::BiscuitDelivery => 1,
            _ => 2,
        }
    }

    /// Get all minor runes for a given path and slot
    pub fn for_path_slot(path: RunePath, slot: usize) -> Vec<MinorRune> {
        match (path, slot) {
            (RunePath::Precision, 0) => vec![Self::Overheal, Self::Triumph, Self::PresenceOfMind],
            (RunePath::Precision, 1) => vec![Self::LegendAlacrity, Self::LegendTenacity, Self::LegendBloodline],
            (RunePath::Precision, 2) => vec![Self::CoupDeGrace, Self::CutDown, Self::LastStand],
            (RunePath::Domination, 0) => vec![Self::CheapShot, Self::TasteOfBlood, Self::SuddenImpact],
            (RunePath::Domination, 1) => vec![Self::ZombieWard, Self::GhostPoro, Self::EyeballCollection],
            (RunePath::Domination, 2) => vec![Self::RavenousHunter, Self::IngeniousHunter, Self::RelentlessHunter, Self::UltimateHunter],
            (RunePath::Sorcery, 0) => vec![Self::NullifyingOrb, Self::ManaflowBand, Self::NimbusCloak],
            (RunePath::Sorcery, 1) => vec![Self::Transcendence, Self::Celerity, Self::AbsoluteFocus],
            (RunePath::Sorcery, 2) => vec![Self::Scorch, Self::Waterwalking, Self::GatheringStorm],
            (RunePath::Resolve, 0) => vec![Self::Demolish, Self::FontOfLife, Self::ShieldBash],
            (RunePath::Resolve, 1) => vec![Self::Conditioning, Self::SecondWind, Self::BonePlating],
            (RunePath::Resolve, 2) => vec![Self::Overgrowth, Self::Revitalize, Self::Unflinching],
            (RunePath::Inspiration, 0) => vec![Self::HextechFlashtraption, Self::MagicalFootwear, Self::PerfectTiming],
            (RunePath::Inspiration, 1) => vec![Self::FuturesMarket, Self::MinionDematerializer, Self::BiscuitDelivery],
            (RunePath::Inspiration, 2) => vec![Self::CosmicInsight, Self::ApproachVelocity, Self::TimeWarpTonic],
            _ => vec![],
        }
    }
}

/// Stat shard options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatShard {
    AttackSpeed,      // +10% AS
    AdaptiveForce,    // +9 Adaptive (5.4 AD or 9 AP)
    CDR,              // +1-10% CDR
    Armor,            // +6 Armor
    MagicResist,      // +8 MR
    Health,           // +15-90 HP
}

impl StatShard {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AttackSpeed => "+10% Attack Speed",
            Self::AdaptiveForce => "+9 Adaptive Force",
            Self::CDR => "+1-10% CDR",
            Self::Armor => "+6 Armor",
            Self::MagicResist => "+8 Magic Resist",
            Self::Health => "+15-90 Health",
        }
    }

    pub fn for_slot(slot: usize) -> Vec<StatShard> {
        match slot {
            0 => vec![Self::AttackSpeed, Self::AdaptiveForce, Self::CDR],
            1 => vec![Self::AdaptiveForce, Self::Armor, Self::MagicResist],
            2 => vec![Self::Health, Self::Armor, Self::MagicResist],
            _ => vec![],
        }
    }
}

/// A complete rune page configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunePage {
    pub name: String,
    pub primary_path: RunePath,
    pub keystone: Keystone,
    pub primary_slots: [MinorRune; 3],
    pub secondary_path: RunePath,
    pub secondary_picks: [MinorRune; 2],
    pub stat_shards: [StatShard; 3],
}

impl Default for RunePage {
    fn default() -> Self {
        Self {
            name: "Rune Page 1".into(),
            primary_path: RunePath::Precision,
            keystone: Keystone::PressTheAttack,
            primary_slots: [MinorRune::Triumph, MinorRune::LegendAlacrity, MinorRune::CoupDeGrace],
            secondary_path: RunePath::Domination,
            secondary_picks: [MinorRune::TasteOfBlood, MinorRune::RavenousHunter],
            stat_shards: [StatShard::AttackSpeed, StatShard::AdaptiveForce, StatShard::Armor],
        }
    }
}
