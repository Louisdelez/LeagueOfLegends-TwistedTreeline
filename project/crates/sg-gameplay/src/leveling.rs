use sg_core::constants::*;

pub fn xp_for_level(level: u8) -> f32 {
    if level == 0 || level > 18 { return 0.0; }
    XP_PER_LEVEL[(level - 1) as usize]
}

pub fn level_from_xp(total_xp: f32) -> u8 {
    for (i, &required) in XP_PER_LEVEL.iter().enumerate().rev() {
        if total_xp >= required {
            return (i + 1) as u8;
        }
    }
    1
}

pub fn kill_xp(victim_level: u8, killer_level: u8) -> f32 {
    let base = if victim_level >= 1 && victim_level <= 18 {
        KILL_XP_PER_LEVEL[(victim_level - 1) as usize]
    } else {
        KILL_XP_PER_LEVEL[17]
    };
    let level_diff = victim_level as f32 - killer_level as f32;
    let multiplier = 1.0 + level_diff * LEVEL_DIFF_XP_MULTIPLIER;
    base * multiplier.max(0.2)
}

pub fn shared_xp(base_xp: f32, nearby_allies: usize) -> f32 {
    let idx = nearby_allies.min(5).max(1) - 1;
    base_xp * SHARED_XP_PCT[idx]
}

pub fn death_timer(level: u8, game_time: f32) -> f32 {
    let base = if level >= 1 && level <= 18 {
        DEATH_TIMER_PER_LEVEL[(level - 1) as usize]
    } else {
        DEATH_TIMER_PER_LEVEL[17]
    };

    if game_time < DEATH_TIMER_SCALE_START {
        return base;
    }

    let intervals = ((game_time - DEATH_TIMER_SCALE_START) / DEATH_TIMER_SCALE_INTERVAL).floor();
    let scale = (1.0 + intervals * DEATH_TIMER_SCALE_PCT).min(DEATH_TIMER_SCALE_CAP);
    base * scale
}
