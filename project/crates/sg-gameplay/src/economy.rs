use sg_core::constants::*;

// Bounty constants from LS4-3x3 bounty.json Global section
const KILL_STREAK_BONUS: f32 = 0.165;
const MAX_KILL_STREAK_BONUS: f32 = 1.66667;
const MIN_KILLS_FOR_STREAK: u32 = 2;
const DEATH_STREAK_PENALTY: f32 = 0.2;
const MIN_DEATH_STREAK_PENALTY: f32 = 0.1666667;
const MIN_DEATHS_FOR_STREAK: u32 = 2;
const GOLD_POOL_FOR_ASSIST: f32 = 0.5;

/// Calculate kill gold based on killer and victim streaks.
/// Returns (kill_gold, assist_pool_gold)
pub fn kill_gold(victim_kills: u32, victim_deaths: u32) -> (f32, f32) {
    let base = BASE_KILL_GOLD; // 300

    // Kill streak bonus: victim had a streak, killer gets extra gold
    let streak_bonus = if victim_kills >= MIN_KILLS_FOR_STREAK {
        let streak = (victim_kills - MIN_KILLS_FOR_STREAK + 1) as f32;
        (base * KILL_STREAK_BONUS * streak).min(base * MAX_KILL_STREAK_BONUS)
    } else {
        0.0
    };

    // Death streak penalty: victim died a lot, less gold for killing them
    let death_penalty = if victim_deaths >= MIN_DEATHS_FOR_STREAK {
        let deaths = (victim_deaths - MIN_DEATHS_FOR_STREAK + 1) as f32;
        (1.0 - DEATH_STREAK_PENALTY * deaths).max(MIN_DEATH_STREAK_PENALTY)
    } else {
        1.0
    };

    let total_kill_gold = (base + streak_bonus) * death_penalty;
    let assist_pool = total_kill_gold * GOLD_POOL_FOR_ASSIST;

    (total_kill_gold, assist_pool)
}

pub fn minion_gold(minion_type: sg_core::types::MinionType, game_time: f32) -> f32 {
    let intervals = (game_time / 90.0).floor();
    match minion_type {
        sg_core::types::MinionType::Melee => MELEE_MINION_GOLD + MINION_GOLD_GROWTH_PER_90S * intervals,
        sg_core::types::MinionType::Caster => CASTER_MINION_GOLD + MINION_GOLD_GROWTH_PER_90S * intervals,
        sg_core::types::MinionType::Siege => SIEGE_MINION_GOLD + SIEGE_GOLD_GROWTH_PER_90S * intervals,
        sg_core::types::MinionType::Super => SIEGE_MINION_GOLD + SIEGE_GOLD_GROWTH_PER_90S * intervals,
    }
}

pub fn ambient_gold(game_time: f32) -> f32 {
    let ticks = (game_time / AMBIENT_GOLD_INTERVAL).floor();
    ticks * AMBIENT_GOLD_PER_TICK
}
