use sg_core::constants::*;
use sg_core::types::*;

pub struct SpawnScheduler {
    pub game_time: f32,
    pub next_minion_wave: f32,
    pub next_jungle_spawn: f32,
    pub vilemaw_respawn_at: Option<f32>,
    pub altar_unlocked: bool,
}

impl SpawnScheduler {
    pub fn new() -> Self {
        Self {
            game_time: 0.0,
            next_minion_wave: MINION_FIRST_SPAWN,
            next_jungle_spawn: JUNGLE_FIRST_SPAWN,
            vilemaw_respawn_at: Some(VILEMAW_FIRST_SPAWN),
            altar_unlocked: false,
        }
    }

    pub fn update(&mut self, dt: f32) -> SpawnEvents {
        self.game_time += dt;
        let mut events = SpawnEvents::default();

        // Unlock altars and relics at 2:30
        if !self.altar_unlocked && self.game_time >= ALTAR_UNLOCK_TIME {
            self.altar_unlocked = true;
            events.unlock_altars = true;
            events.spawn_health_relics = true;
            events.spawn_speed_shrine = true;
        }

        // Minion waves
        if self.game_time >= self.next_minion_wave {
            events.spawn_minion_wave = true;
            // Determine wave type
            let wave_number = ((self.game_time - MINION_FIRST_SPAWN) / MINION_WAVE_INTERVAL) as u32;
            events.is_cannon_wave = wave_number % 3 == 2; // every 3rd wave
            self.next_minion_wave += MINION_WAVE_INTERVAL;
        }

        // Jungle camps
        if self.game_time >= self.next_jungle_spawn {
            events.spawn_jungle_camps = true;
            self.next_jungle_spawn += JUNGLE_RESPAWN;
        }

        // Vilemaw
        if let Some(spawn_at) = self.vilemaw_respawn_at {
            if self.game_time >= spawn_at {
                events.spawn_vilemaw = true;
                self.vilemaw_respawn_at = None; // will be set again on kill
            }
        }

        events
    }

    pub fn on_vilemaw_killed(&mut self) {
        self.vilemaw_respawn_at = Some(self.game_time + VILEMAW_RESPAWN);
    }

    pub fn minion_wave_composition(&self, is_cannon: bool, is_super: bool) -> Vec<MinionType> {
        let mut wave = vec![
            MinionType::Melee,
            MinionType::Melee,
            MinionType::Melee,
            MinionType::Caster,
            MinionType::Caster,
            MinionType::Caster,
        ];
        if is_super {
            wave.push(MinionType::Super);
        } else if is_cannon {
            wave.push(MinionType::Siege);
        }
        wave
    }
}

#[derive(Default)]
pub struct SpawnEvents {
    pub spawn_minion_wave: bool,
    pub is_cannon_wave: bool,
    pub spawn_jungle_camps: bool,
    pub spawn_vilemaw: bool,
    pub unlock_altars: bool,
    pub spawn_health_relics: bool,
    pub spawn_speed_shrine: bool,
}
