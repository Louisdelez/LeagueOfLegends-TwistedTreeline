/// Twisted Treeline game constants extracted from map10.bin.json (patch 9.22)
/// These are reference values for faithful recreation.

// === Map ===
pub const MAP_WIDTH: f32 = 15398.0;
pub const MAP_HEIGHT: f32 = 15398.0;
pub const TICK_RATE: f32 = 60.0;

// === Teams ===
pub const TEAM_SIZE: usize = 3;

// === Starting Values ===
pub const STARTING_GOLD: f32 = 850.0;
pub const AMBIENT_GOLD_PER_TICK: f32 = 0.95;
pub const AMBIENT_GOLD_INTERVAL: f32 = 5.0;

// === XP Required Per Level ===
pub const XP_PER_LEVEL: [f32; 18] = [
    0.0, 280.0, 660.0, 1140.0, 1720.0, 2400.0, 3180.0, 4060.0, 5040.0,
    6120.0, 7300.0, 8580.0, 9960.0, 11440.0, 13020.0, 14700.0, 16480.0, 18360.0,
];

// === XP Granted for Kill Per Level ===
pub const KILL_XP_PER_LEVEL: [f32; 18] = [
    35.0, 101.0, 167.0, 233.0, 275.0, 327.0, 389.0, 451.0, 513.0,
    590.0, 640.0, 690.0, 740.0, 790.0, 840.0, 890.0, 940.0, 990.0,
];

/// XP multiplier per level difference between killer and victim
pub const LEVEL_DIFF_XP_MULTIPLIER: f32 = 0.16;

/// Shared XP percentages by number of nearby allies [1..5]
pub const SHARED_XP_PCT: [f32; 5] = [0.92, 0.58, 0.35, 0.275, 0.22];

// === Death Timers (seconds, by level 1-18) ===
pub const DEATH_TIMER_PER_LEVEL: [f32; 18] = [
    13.0, 14.5, 16.0, 17.5, 19.0, 21.0, 23.0, 25.0, 27.0,
    29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0,
];

/// Death timer scaling starts at this game time (seconds)
pub const DEATH_TIMER_SCALE_START: f32 = 480.0;
/// Scaling increment interval (seconds)
pub const DEATH_TIMER_SCALE_INTERVAL: f32 = 30.0;
/// Percent increase per interval
pub const DEATH_TIMER_SCALE_PCT: f32 = 0.03;
/// Maximum scaling cap
pub const DEATH_TIMER_SCALE_CAP: f32 = 1.5;

// === Gold / Bounty ===
pub const BASE_KILL_GOLD: f32 = 300.0;
pub const FIRST_BLOOD_BONUS: f32 = 100.0;
pub const ASSIST_POOL_MIN: f32 = 0.25;
pub const ASSIST_POOL_MAX: f32 = 0.50;

// === Minion Gold ===
pub const MELEE_MINION_GOLD: f32 = 20.0;
pub const CASTER_MINION_GOLD: f32 = 17.0;
pub const SIEGE_MINION_GOLD: f32 = 45.0;
pub const MINION_GOLD_GROWTH_PER_90S: f32 = 0.125;
pub const SIEGE_GOLD_GROWTH_PER_90S: f32 = 0.35;

// === Minion Spawning ===
pub const MINION_FIRST_SPAWN: f32 = 45.0;
pub const MINION_WAVE_INTERVAL: f32 = 45.0;
pub const MINION_SPAWN_DELAY: f32 = 0.8; // seconds between each minion in wave

// === Jungle Camps ===
pub const JUNGLE_FIRST_SPAWN: f32 = 65.0; // 1:05
pub const JUNGLE_RESPAWN: f32 = 75.0;
// Golem (TT_NGolem from LS4-3x3)
pub const GOLEM_HP: f32 = 1250.0;
pub const GOLEM_AD: f32 = 60.0;
pub const GOLEM_ARMOR: f32 = 12.0;
pub const GOLEM_GOLD: f32 = 48.0;
pub const GOLEM_XP: f32 = 150.0;
// Wolf (TT_NWolf)
pub const WOLF_HP: f32 = 1150.0;
pub const WOLF_AD: f32 = 40.0;
pub const WOLF_ARMOR: f32 = 9.0;
pub const WOLF_GOLD: f32 = 28.0;
pub const WOLF_XP: f32 = 144.0;
// Wraith (TT_NWraith)
pub const WRAITH_HP: f32 = 1000.0;
pub const WRAITH_AD: f32 = 28.0;
pub const WRAITH_ARMOR: f32 = 15.0;
pub const WRAITH_GOLD: f32 = 33.0;
pub const WRAITH_XP: f32 = 120.0;

// === Vilemaw ===
pub const VILEMAW_FIRST_SPAWN: f32 = 600.0; // 10:00
pub const VILEMAW_RESPAWN: f32 = 360.0;     // 6:00
pub const VILEMAW_HP: f32 = 5500.0;
pub const VILEMAW_AD: f32 = 100.0;
pub const VILEMAW_ARMOR_SHRED_PER_STACK: f32 = 0.5;
pub const VILEMAW_MAX_SHRED_STACKS: u32 = 60;
pub const VILEMAW_BUFF_DURATION: f32 = 180.0;

// === Altars ===
pub const ALTAR_UNLOCK_TIME: f32 = 150.0;  // 2:30
pub const ALTAR_CAPTURE_TIME: f32 = 9.0;
pub const ALTAR_LOCKOUT_TIME: f32 = 90.0;
pub const ALTAR_CAPTURE_GOLD: f32 = 80.0;

// One altar: +10% bonus movement speed
pub const ALTAR_1_MOVE_SPEED_BONUS: f32 = 0.10;
// Two altars: +1% max HP restored on minion/monster kill
pub const ALTAR_2_HP_RESTORE_PCT: f32 = 0.01;

// === Health Relic ===
pub const HEALTH_RELIC_UNLOCK: f32 = 150.0; // 2:30

// === Speed Shrine ===
pub const SPEED_SHRINE_UNLOCK: f32 = 150.0;

// === Turret Stats ===
pub const OUTER_TURRET_HP: f32 = 900.0;
pub const INNER_TURRET_HP: f32 = 1100.0;
pub const INHIB_TURRET_HP: f32 = 1500.0;
pub const NEXUS_TURRET_HP: f32 = 1900.0;
pub const TURRET_HP_PER_CHAMP: f32 = 250.0;

// === Experience Range ===
pub const XP_RANGE: f32 = 1250.0;

// === Lethality ===
pub const LETHALITY_START_PCT: f32 = 0.60;
pub const LETHALITY_SCALE_LEVEL: u8 = 18;

// === Surrender ===
pub const SURRENDER_AVAILABLE_AT: f32 = 900.0; // 15:00

// === Perk Replacements (conceptual) ===
// Waterwalking -> Scorch
// ZombieWard -> EyeballCollection
// GhostPoro -> EyeballCollection
