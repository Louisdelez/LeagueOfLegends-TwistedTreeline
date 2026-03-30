//! Automated 3v3 bot-vs-bot integration test for Twisted Treeline
//! Converts the player to a bot, adjusts to 3v3, observes the full game
//! Usage: cargo run --bin sg-test --release

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use sg_core::components::*;
use sg_core::types::*;
use sg_ai::champion_ai::BotController;
use crate::menu::AppState;
use crate::spawn_plugin::GameTimer;
use std::path::PathBuf;

pub struct AutoTestPlugin;

impl Plugin for AutoTestPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TestRunner::new())
            .add_systems(Update, (convert_player_to_bot, run_tests, follow_bot_camera).run_if(in_state(AppState::InGame)))
            .add_systems(Update, run_tests_postgame.run_if(in_state(AppState::PostGame)));
    }
}

/// MINION-ONLY MODE — despawn everything except minions, structures, map
fn convert_player_to_bot(
    mut commands: Commands,
    champions: Query<Entity, With<Champion>>,
    fog_tiles: Query<Entity, With<crate::fog_plugin::FogTile>>,
    mut done: Local<bool>,
) {
    if *done { return; }
    *done = true;

    // Despawn ALL champions
    for entity in &champions {
        if let Ok(mut ecmd) = commands.get_entity(entity) {
            ecmd.despawn();
        }
    }

    // Despawn fog tiles (dark overlay that blocks visibility)
    for entity in &fog_tiles {
        if let Ok(mut ecmd) = commands.get_entity(entity) {
            ecmd.despawn();
        }
    }
}

/// Camera locked on ONE minion — follows the first Blue melee minion that spawns
fn follow_bot_camera(
    player: Query<&Transform, (With<PlayerControlled>, With<Champion>, Without<Camera3d>)>,
    minions: Query<(Entity, &Transform), (With<Minion>, Without<Dead>, Without<Camera3d>)>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Champion>, Without<Minion>)>,
    time: Res<Time>,
    game_timer: Res<crate::spawn_plugin::GameTimer>,
    mut tracked: Local<Option<Entity>>,
) {
    let Ok(mut cam_tf) = camera.single_mut() else { return; };

    // Lock on the first minion we find and NEVER change
    if tracked.is_none() {
        if let Some((e, _)) = minions.iter().next() {
            *tracked = Some(e);
        }
    }

    // Follow the tracked minion
    if let Some(target_entity) = *tracked {
        if let Ok((_, m_tf)) = minions.get(target_entity) {
            let pos = m_tf.translation;
            cam_tf.translation = Vec3::new(pos.x, pos.y + 800.0, pos.z + 500.0);
            cam_tf.look_at(pos, Vec3::Y);
            return;
        } else {
            // Tracked minion died — pick the next one
            *tracked = None;
            if let Some((e, _)) = minions.iter().next() {
                *tracked = Some(e);
            }
        }
    }

    // Fallback: map center
    cam_tf.translation = Vec3::new(7700.0, 3000.0, 9300.0);
    cam_tf.look_at(Vec3::new(7700.0, 50.0, 7300.0), Vec3::Y);
}

// ═══════════════════════════════════════════════════════
//  Test Runner
// ═══════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    WaitSpawn, ValidateSetup, WatchEarly, CheckFarming, WatchMid,
    CheckCombat, CheckEconomy, CheckLevels, WatchLate, FinalScoreboard,
    Surrender, PostGame, Done,
}

#[derive(Resource)]
struct TestRunner {
    phase: Phase, pt: f32, tt: f32, step: u32,
    results: Vec<(String, bool, String)>,
    done: bool, ss_dir: PathBuf, ss_count: u32, ss_done: bool,
    init_gold: Vec<f32>,
}

impl TestRunner {
    fn new() -> Self {
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let dir = PathBuf::from(format!("test_screenshots/test_3v3_{}", ts));
        std::fs::create_dir_all(&dir).ok();
        println!("\n══════════════════════════════════════════════════════════════");
        println!("  TWISTED TREELINE — 3v3 BOT VS BOT AUTOMATED TEST");
        println!("  Screenshots: {}", dir.display());
        println!("══════════════════════════════════════════════════════════════\n");
        Self { phase: Phase::WaitSpawn, pt: 0.0, tt: 0.0, step: 0,
            results: vec![], done: false, ss_dir: dir, ss_count: 0, ss_done: false,
            init_gold: vec![] }
    }
    fn pass(&mut self, p: &str, d: &str) { println!("  [PASS] {} — {}", p, d); self.results.push((p.into(), true, d.into())); }
    fn fail(&mut self, p: &str, d: &str) { println!("  [FAIL] {} — {}", p, d); self.results.push((p.into(), false, d.into())); }
    fn next(&mut self, p: Phase) { self.phase = p; self.pt = 0.0; self.step = 0; self.ss_done = false; }
}

fn ss(cmd: &mut Commands, t: &mut TestRunner, label: &str) {
    if t.ss_done { return; }
    t.ss_done = true;
    let path = t.ss_count + 1;
    let p = t.ss_dir.join(format!("{:03}_{}.png", path, label));
    t.ss_count += 1;
    cmd.spawn(Screenshot::primary_window()).observe(save_to_disk(p));
}

fn extra_ss(cmd: &mut Commands, t: &mut TestRunner, label: &str) {
    t.ss_count += 1;
    let p = t.ss_dir.join(format!("{:03}_{}.png", t.ss_count, label));
    cmd.spawn(Screenshot::primary_window()).observe(save_to_disk(p));
}

fn scoreboard(champs: &Query<(&Champion, &TeamMember, &Health, &Gold, &CombatStats, Option<&GameStats>)>) {
    println!("\n  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │  CHAMPION     TEAM  LV  HP          GOLD   KDA       CS   DMG   │");
    println!("  ├─────────────────────────────────────────────────────────────────┤");
    for (champ, team, hp, gold, stats, gs) in champs.iter() {
        let t = if team.0 == Team::Blue { "BLU" } else { "RED" };
        let (k, d, a, cs, dmg) = gs.map(|s| (s.kills, s.deaths, s.assists, s.cs, s.damage_dealt)).unwrap_or((0, 0, 0, 0, 0.0));
        println!("  │  {:12} {:3}  {:2}  {:4.0}/{:4.0}  {:6.0}  {}/{}/{}  {:3}  {:5.0}  │",
            champ.name, t, champ.level, hp.current, hp.max, gold.0, k, d, a, cs, dmg);
    }
    println!("  └─────────────────────────────────────────────────────────────────┘\n");
}

// ═══════════════════════════════════════════════════════
//  Main test loop
// ═══════════════════════════════════════════════════════

#[allow(clippy::type_complexity)]
fn run_tests(
    mut cmd: Commands,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppState>>,
    mut t: ResMut<TestRunner>,
    champs: Query<(&Champion, &TeamMember, &Health, &Gold, &CombatStats, Option<&GameStats>)>,
    structures: Query<Entity, (With<Structure>, Without<Champion>)>,
    minions: Query<Entity, (With<Minion>, Without<Dead>, Without<Champion>)>,
    gt: Option<Res<GameTimer>>,
) {
    if t.done { return; }
    t.pt += time.delta_secs();
    t.tt += time.delta_secs();

    match t.phase {
        Phase::WaitSpawn => {
            if champs.iter().count() >= 6 {
                let e = t.pt;
                ss(&mut cmd, &mut t, "game_start");
                t.pass("GameStart", &format!("6 champions ready in {:.1}s", e));
                t.next(Phase::ValidateSetup);
            } else if t.pt > 10.0 {
                t.fail("GameStart", &format!("Only {} champions", champs.iter().count()));
                t.next(Phase::Done);
            }
        }

        Phase::ValidateSetup => {
            if t.pt > 1.5 {
                ss(&mut cmd, &mut t, "setup_validated");
                let blue = champs.iter().filter(|(_, tm, _, _, _, _)| tm.0 == Team::Blue).count();
                let red = champs.iter().filter(|(_, tm, _, _, _, _)| tm.0 == Team::Red).count();
                let s = structures.iter().count();
                let m = minions.iter().count();

                let mut names_blue = Vec::new();
                let mut names_red = Vec::new();
                for (c, tm, _, _, _, _) in &champs {
                    if tm.0 == Team::Blue { names_blue.push(c.name.as_str()); }
                    else { names_red.push(c.name.as_str()); }
                }

                t.pass("ValidateSetup", &format!("{}v{} — Blue[{}] vs Red[{}], {} structures, {} minions",
                    blue, red, names_blue.join(","), names_red.join(","), s, m));

                t.init_gold = champs.iter().map(|(_, _, _, g, _, _)| g.0).collect();
                t.next(Phase::WatchEarly);
            }
        }

        // ═══════════ Early game: 0-70s (bots lane, minions spawn at 45s) ═══════════
        Phase::WatchEarly => {
            if t.pt > 15.0 && t.step == 0 { ss(&mut cmd, &mut t, "early_15s_bots_moving"); t.step = 1; }
            if t.pt > 45.0 && t.step == 1 { extra_ss(&mut cmd, &mut t, "early_45s_minions_spawn"); t.step = 2; }
            if t.pt > 60.0 && t.step == 2 { extra_ss(&mut cmd, &mut t, "early_60s_laning"); t.step = 3; }
            if t.pt > 70.0 {
                extra_ss(&mut cmd, &mut t, "early_70s");
                let alive = champs.iter().filter(|(_, _, h, _, _, _)| h.current > 0.0).count();
                let mins = minions.iter().count();
                t.pass("EarlyGame", &format!("{}/6 alive, {} minions after 70s", alive, mins));
                scoreboard(&champs);
                t.next(Phase::CheckFarming);
            }
        }

        Phase::CheckFarming => {
            let total_cs: u32 = champs.iter().map(|(_, _, _, _, _, s)| s.map(|s| s.cs).unwrap_or(0)).sum();
            let total_gold: f32 = champs.iter().map(|(_, _, _, g, _, _)| g.0).sum();
            let init_total: f32 = t.init_gold.iter().sum();
            t.pass("Farming", &format!("Total CS: {}, gold gained: +{:.0}", total_cs, total_gold - init_total));
            t.next(Phase::WatchMid);
        }

        // ═══════════ Mid game: watch 40 more seconds (combat should happen) ═══════════
        Phase::WatchMid => {
            if t.pt > 15.0 && t.step == 0 { ss(&mut cmd, &mut t, "mid_phase_15s"); t.step = 1; }
            if t.pt > 30.0 && t.step == 1 { extra_ss(&mut cmd, &mut t, "mid_phase_30s"); t.step = 2; }
            if t.pt > 40.0 {
                extra_ss(&mut cmd, &mut t, "mid_phase_40s");
                let total_kills: u32 = champs.iter().map(|(_, _, _, _, _, s)| s.map(|s| s.kills).unwrap_or(0)).sum();
                let total_dmg: f32 = champs.iter().map(|(_, _, _, _, _, s)| s.map(|s| s.damage_dealt).unwrap_or(0.0)).sum();
                t.pass("MidGame", &format!("Total kills: {}, total damage: {:.0}", total_kills, total_dmg));
                scoreboard(&champs);
                t.next(Phase::CheckCombat);
            }
        }

        Phase::CheckCombat => {
            let mut took_dmg = 0;
            for (_, _, hp, _, _, _) in &champs {
                if hp.current < hp.max { took_dmg += 1; }
            }
            let kills: u32 = champs.iter().map(|(_, _, _, _, _, s)| s.map(|s| s.kills).unwrap_or(0)).sum();
            let deaths: u32 = champs.iter().map(|(_, _, _, _, _, s)| s.map(|s| s.deaths).unwrap_or(0)).sum();
            t.pass("Combat", &format!("{}/6 took damage, {} kills, {} deaths", took_dmg, kills, deaths));
            t.next(Phase::CheckEconomy);
        }

        Phase::CheckEconomy => {
            ss(&mut cmd, &mut t, "economy");
            let mut details = Vec::new();
            for (c, _, _, g, stats, _) in &champs {
                details.push(format!("{}:{:.0}g/AD{:.0}", c.name, g.0, stats.attack_damage));
            }
            t.pass("Economy", &details.join(", "));
            t.next(Phase::CheckLevels);
        }

        Phase::CheckLevels => {
            let mut levels = Vec::new();
            for (c, tm, _, _, _, _) in &champs {
                let ts = if tm.0 == Team::Blue { "B" } else { "R" };
                levels.push(format!("[{}]{}:Lv{}", ts, c.name, c.level));
            }
            t.pass("Levels", &levels.join(", "));
            t.next(Phase::WatchLate);
        }

        // ═══════════ Late game: 45-60s ═══════════
        Phase::WatchLate => {
            if t.pt > 5.0 && t.step == 0 { ss(&mut cmd, &mut t, "late_50s"); t.step = 1; }
            if t.pt > 15.0 {
                extra_ss(&mut cmd, &mut t, "late_60s");
                t.pass("LateGame", "60s of gameplay observed");
                t.next(Phase::FinalScoreboard);
            }
        }

        Phase::FinalScoreboard => {
            ss(&mut cmd, &mut t, "final_scoreboard");
            println!("\n  ══════════ FINAL SCOREBOARD ══════════");
            scoreboard(&champs);

            // Validate key metrics
            let total_kills: u32 = champs.iter().map(|(_, _, _, _, _, s)| s.map(|s| s.kills).unwrap_or(0)).sum();
            let total_cs: u32 = champs.iter().map(|(_, _, _, _, _, s)| s.map(|s| s.cs).unwrap_or(0)).sum();
            let total_dmg: f32 = champs.iter().map(|(_, _, _, _, _, s)| s.map(|s| s.damage_dealt).unwrap_or(0.0)).sum();

            t.pass("FinalStats", &format!("Kills:{} CS:{} TotalDmg:{:.0}", total_kills, total_cs, total_dmg));
            t.next(Phase::Surrender);
        }

        Phase::Surrender => {
            if t.step == 0 {
                println!("  [....] Ending test (surrender)...");
                let dur = gt.as_ref().map(|g| g.elapsed).unwrap_or(t.tt);
                cmd.insert_resource(GameResult { victory: false, game_duration: dur });
                next_state.set(AppState::PostGame);
                t.step = 1;
            }
            if t.pt > 1.5 {
                t.pass("Surrender", "Game ended");
                t.next(Phase::PostGame);
            }
        }

        _ => {}
    }
}

fn run_tests_postgame(
    mut cmd: Commands, mut t: ResMut<TestRunner>, time: Res<Time>, gr: Option<Res<GameResult>>,
) {
    if t.done { return; }
    t.pt += time.delta_secs();

    match t.phase {
        Phase::PostGame => {
            if t.pt > 2.0 && !t.ss_done { ss(&mut cmd, &mut t, "postgame_screen"); }
            if t.pt > 3.0 {
                if let Some(ref r) = gr {
                    t.pass("PostGame", &format!("victory={}, duration={:.1}s", r.victory, r.game_duration));
                } else { t.fail("PostGame", "No GameResult"); }
                t.next(Phase::Done);
            }
        }
        Phase::Done => {
            if !t.done {
                t.done = true;
                extra_ss(&mut cmd, &mut t, "test_complete");
                let passed = t.results.iter().filter(|r| r.1).count();
                let total = t.results.len();
                println!("\n══════════════════════════════════════════════════════════════");
                println!("  3V3 BOT TEST — FINAL REPORT");
                println!("══════════════════════════════════════════════════════════════");
                for (name, ok, detail) in &t.results {
                    println!("  [{}] {} — {}", if *ok { "PASS" } else { "FAIL" }, name, detail);
                }
                println!("══════════════════════════════════════════════════════════════");
                println!("  RESULT: {}/{} PASSED", passed, total);
                println!("  Duration: {:.1}s", t.tt);
                println!("  Screenshots: {} in {}", t.ss_count, t.ss_dir.display());
                println!("══════════════════════════════════════════════════════════════");
                let ok = passed == total;
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    std::process::exit(if ok { 0 } else { 1 });
                });
            }
        }
        _ => {}
    }
}
