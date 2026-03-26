use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SummonerSpell {
    Flash,
    Ignite,
    Heal,
    Barrier,
    Exhaust,
    Ghost,
    Cleanse,
    Teleport,
    Smite,
}

impl SummonerSpell {
    pub fn all() -> &'static [SummonerSpell] {
        &[Self::Flash, Self::Ignite, Self::Heal, Self::Barrier, Self::Exhaust, Self::Ghost, Self::Cleanse, Self::Teleport, Self::Smite]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Flash => "Flash", Self::Ignite => "Ignite", Self::Heal => "Heal",
            Self::Barrier => "Barrier", Self::Exhaust => "Exhaust", Self::Ghost => "Ghost",
            Self::Cleanse => "Cleanse", Self::Teleport => "Teleport", Self::Smite => "Smite",
        }
    }

    pub fn icon_path(&self) -> &'static str {
        match self {
            Self::Flash => "ui/spells/summoner_flash.png",
            Self::Ignite => "ui/spells/summoner_ignite.png",
            Self::Heal => "ui/spells/summoner_heal.png",
            Self::Barrier => "ui/spells/summoner_barrier.png",
            Self::Exhaust => "ui/spells/summoner_exhaust.png",
            Self::Ghost => "ui/spells/summoner_flash.png", // fallback
            Self::Cleanse => "ui/spells/summoner_flash.png", // fallback
            Self::Teleport => "ui/spells/summoner_teleport.png",
            Self::Smite => "ui/spells/summoner_smite.png",
        }
    }

    pub fn cooldown(&self) -> f32 {
        match self {
            Self::Flash => 300.0, Self::Ignite => 180.0, Self::Heal => 240.0,
            Self::Barrier => 180.0, Self::Exhaust => 210.0, Self::Ghost => 180.0,
            Self::Cleanse => 210.0, Self::Teleport => 360.0, Self::Smite => 90.0,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Flash => "Blink 400 units in target direction",
            Self::Ignite => "Deal 80-505 true damage over 5s, apply Grievous Wounds",
            Self::Heal => "Restore 90-345 HP to self + nearest ally, +30% MS for 1s",
            Self::Barrier => "Shield for 115-455 for 2.5s",
            Self::Exhaust => "Slow 30%, reduce damage dealt by 40% for 2.5s",
            Self::Ghost => "+28-45% movement speed for 10s",
            Self::Cleanse => "Remove all CC, +65% tenacity for 3s",
            Self::Teleport => "Channel 4.5s to teleport to allied structure/minion",
            Self::Smite => "Deal 390-1000 true damage to monster/minion",
        }
    }

    pub fn unlock_level(&self) -> u8 {
        match self {
            Self::Heal | Self::Ghost => 1,
            Self::Barrier | Self::Exhaust => 4,
            Self::Flash | Self::Teleport => 7,
            Self::Ignite | Self::Cleanse | Self::Smite => 9,
        }
    }
}
