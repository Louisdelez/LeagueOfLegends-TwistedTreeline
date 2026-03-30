use bevy::prelude::*;
use bevy_hui::prelude::*;
use sg_core::components::*;
use sg_core::constants::*;
use sg_core::types::*;
use crate::spawn_plugin::GameTimer;
use crate::ability_plugin::AbilityCooldowns;
use crate::menu::AppState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_hui::HuiPlugin)
            .add_systems(OnEnter(AppState::InGame), setup_hud)
            .add_systems(Update, (update_hud_properties, sys_world_hp, draw_minimap, draw_scoreboard, draw_death_timer).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
struct HudRoot;

fn setup_hud(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn((
        HtmlNode(server.load("hud/bottom_bar.html")),
        TemplateProperties::default()
            .with("hp_pct", "100")
            .with("mana_pct", "100")
            .with("hp_text", "600/600")
            .with("mana_text", "400/400")
            .with("gold", "850")
            .with("level", "1")
            .with("portrait", "ui/portraits/annie_circle.png")
            .with("q_icon", "ui/abilities/annie_q.png")
            .with("w_icon", "ui/abilities/annie_w.png")
            .with("e_icon", "ui/abilities/annie_e.png")
            .with("r_icon", "ui/abilities/annie_r1.png")
            .with("spell_d", "ui/spells/summoner_flash.png")
            .with("spell_f", "ui/spells/summonerignite.png"),
        HudRoot,
    ));
}

fn update_hud_properties(
    mut cmd: Commands,
    player_q: Query<(&Health, &Mana, &Gold, &Champion, Option<&crate::ability_plugin::ChampionKit>, Option<&GameStats>, Option<&AbilityCooldowns>), With<PlayerControlled>>,
    mut hud_q: Query<(Entity, &mut TemplateProperties), With<HudRoot>>,
    game_timer: Res<GameTimer>,
) {
    let Ok((health, mana, gold, champion, kit_opt, stats_opt, cd_opt)) = player_q.single() else { return };
    let Ok((entity, mut props)) = hud_q.single_mut() else { return };

    let hp_pct = ((health.current / health.max).clamp(0.0, 1.0) * 100.0) as u32;
    let mana_pct = ((mana.current / mana.max).clamp(0.0, 1.0) * 100.0) as u32;

    let new_hp_pct = format!("{}", hp_pct);
    let new_gold = format!("{:.0}", gold.0);
    let new_level = format!("{}", champion.level);

    let has_cd = cd_opt.map_or(false, |c| c.q > 0.0 || c.w > 0.0 || c.e > 0.0 || c.r > 0.0);
    if props.get("hp_pct").map(|s| s.as_str()) != Some(&new_hp_pct)
        || props.get("gold").map(|s| s.as_str()) != Some(&new_gold)
        || has_cd
    {
        props.insert("hp_pct".to_string(), new_hp_pct);
        props.insert("hp_text".to_string(), format!("{:.0}/{:.0}", health.current, health.max));
        props.insert("mana_pct".to_string(), format!("{}", mana_pct));
        props.insert("mana_text".to_string(), format!("{:.0}/{:.0}", mana.current, mana.max));
        props.insert("gold".to_string(), new_gold);
        props.insert("level".to_string(), new_level);

        if let Some(kit) = kit_opt {
            use sg_gameplay::champions::ChampionClass;
            let (portrait, q, w, e, r) = match kit.0 {
                ChampionClass::Mage => (
                    "ui/portraits/annie_circle.png",
                    "ui/abilities/annie_q.png", "ui/abilities/annie_w.png",
                    "ui/abilities/annie_e.png", "ui/abilities/annie_r1.png",
                ),
                _ => (
                    "ui/portraits/garen_circle.png",
                    "ui/abilities/garen_q.png", "ui/abilities/garen_w.png",
                    "ui/abilities/garen_e1.png", "ui/abilities/garen_r.png",
                ),
            };
            props.insert("portrait".to_string(), portrait.to_string());
            props.insert("q_icon".to_string(), q.to_string());
            props.insert("w_icon".to_string(), w.to_string());
            props.insert("e_icon".to_string(), e.to_string());
            props.insert("r_icon".to_string(), r.to_string());
        }

        // KDA stats
        if let Some(stats) = stats_opt {
            props.insert("kda".to_string(), format!("{}/{}/{}", stats.kills, stats.deaths, stats.assists));
            props.insert("cs".to_string(), format!("{}", stats.cs));
        }

        // Game timer
        let mins = (game_timer.elapsed / 60.0) as u32;
        let secs = (game_timer.elapsed % 60.0) as u32;
        props.insert("game_time".to_string(), format!("{}:{:02}", mins, secs));

        // Ability cooldowns
        if let Some(cds) = cd_opt {
            let fmt = |v: f32| if v > 0.0 { format!("{:.0}", v.ceil()) } else { String::new() };
            props.insert("q_cd".to_string(), fmt(cds.q));
            props.insert("w_cd".to_string(), fmt(cds.w));
            props.insert("e_cd".to_string(), fmt(cds.e));
            props.insert("r_cd".to_string(), fmt(cds.r));
        }

        cmd.trigger(CompileContextEvent { entity });
    }
}

/// Minimap: draw unit positions as colored dots in bottom-right
/// Draw minimap as gizmo dots in a fixed world position
fn draw_minimap(
    mut g: Gizmos,
    units: Query<(&Transform, &TeamMember, &Health, Option<&Visible>)>,
    player_q: Query<(&TeamMember, &Transform), With<PlayerControlled>>,
) {
    let my_team = player_q.iter().next().map(|(t, _)| t.0).unwrap_or(Team::Blue);
    let player_pos = player_q.iter().next().map(|(_, t)| t.translation).unwrap_or(Vec3::ZERO);

    let map_size = 15398.0f32;
    let mm_size = 200.0;
    // Place minimap in world space above the player's approximate area
    let mm_y = 2500.0;
    let mm_center = Vec3::new(map_size / 2.0, mm_y, map_size / 2.0);

    // Draw units as colored spheres on the minimap plane
    for (tf, team, health, visible) in &units {
        if health.current <= 0.0 { continue; }

        // Fog: hide enemies not visible
        if team.0 != my_team && team.0 != Team::Neutral {
            if let Some(vis) = visible {
                let can_see = if my_team == Team::Blue { vis.to_blue } else { vis.to_red };
                if !can_see { continue; }
            }
        }

        let mx = (tf.translation.x / map_size) * mm_size + mm_center.x - mm_size / 2.0;
        let mz = (tf.translation.z / map_size) * mm_size + mm_center.z - mm_size / 2.0;

        let color = if team.0 == my_team {
            Color::srgb(0.2, 0.7, 1.0)
        } else if team.0 == Team::Neutral {
            Color::srgb(0.9, 0.9, 0.2)
        } else {
            Color::srgb(1.0, 0.2, 0.2)
        };

        g.sphere(Isometry3d::from_translation(Vec3::new(mx, mm_y, mz)), 30.0, color);
    }

    // Player indicator (larger, brighter)
    let px = (player_pos.x / map_size) * mm_size + mm_center.x - mm_size / 2.0;
    let pz = (player_pos.z / map_size) * mm_size + mm_center.z - mm_size / 2.0;
    g.sphere(Isometry3d::from_translation(Vec3::new(px, mm_y + 10.0, pz)), 50.0, Color::srgb(0.3, 1.0, 0.3));
}

/// Tab scoreboard: show all champions stats when Tab is held
fn draw_scoreboard(
    mut g: Gizmos,
    keys: Res<ButtonInput<KeyCode>>,
    champions: Query<(&Champion, &TeamMember, &Health, &Gold, Option<&GameStats>)>,
    player: Query<(&TeamMember, &Transform), With<PlayerControlled>>,
) {
    if !keys.pressed(KeyCode::Tab) { return; }

    let my_team = player.iter().next().map(|(t, _)| t.0).unwrap_or(Team::Blue);
    let cam_pos = player.iter().next().map(|(_, t)| t.translation).unwrap_or(Vec3::ZERO);

    // Draw scoreboard as gizmo lines in world space above player
    let base = cam_pos + Vec3::new(0.0, 800.0, -500.0);
    let mut y_offset = 0.0;

    // Header
    let header_col = Color::srgba(0.9, 0.85, 0.4, 0.9);
    g.sphere(Isometry3d::from_translation(base + Vec3::new(0.0, y_offset, 0.0)), 15.0, header_col);
    y_offset -= 60.0;

    for (champ, team, health, gold, stats_opt) in &champions {
        let col = if team.0 == my_team {
            Color::srgba(0.2, 0.6, 1.0, 0.8)
        } else {
            Color::srgba(1.0, 0.3, 0.2, 0.8)
        };

        let pos = base + Vec3::new(0.0, y_offset, 0.0);

        // Name dot (color = team)
        g.sphere(Isometry3d::from_translation(pos), 10.0, col);

        // Level indicator (size = level)
        g.sphere(Isometry3d::from_translation(pos + Vec3::new(150.0, 0.0, 0.0)), 5.0 + champ.level as f32, Color::srgba(1.0, 0.9, 0.3, 0.7));

        // HP bar
        let hp_pct = (health.current / health.max).clamp(0.0, 1.0);
        let bar_w = 200.0;
        g.line(pos + Vec3::new(250.0, 0.0, 0.0), pos + Vec3::new(250.0 + bar_w, 0.0, 0.0), Color::srgba(0.2, 0.2, 0.2, 0.6));
        g.line(pos + Vec3::new(250.0, 0.0, 0.0), pos + Vec3::new(250.0 + bar_w * hp_pct, 0.0, 0.0), Color::srgba(0.1, 0.8, 0.1, 0.8));

        // Gold dot
        g.sphere(Isometry3d::from_translation(pos + Vec3::new(500.0, 0.0, 0.0)), 5.0 + (gold.0 / 500.0).min(10.0), Color::srgba(1.0, 0.85, 0.0, 0.7));

        // KDA dots (kills = green, deaths = red)
        if let Some(stats) = stats_opt {
            for k in 0..stats.kills.min(10) {
                g.sphere(Isometry3d::from_translation(pos + Vec3::new(600.0 + k as f32 * 15.0, 0.0, 0.0)), 5.0, Color::srgba(0.2, 1.0, 0.2, 0.8));
            }
            for d in 0..stats.deaths.min(10) {
                g.sphere(Isometry3d::from_translation(pos + Vec3::new(600.0 + d as f32 * 15.0, -15.0, 0.0)), 5.0, Color::srgba(1.0, 0.2, 0.2, 0.8));
            }
        }

        y_offset -= 50.0;
    }
}

/// Death timer: show countdown when player is dead
fn draw_death_timer(
    mut g: Gizmos,
    player_dead: Query<(&Dead, &Transform), With<PlayerControlled>>,
) {
    for (dead, tf) in &player_dead {
        let timer = dead.respawn_timer;
        // Big pulsing sphere above corpse position
        let pulse = 1.0 + (timer * 3.0).sin() * 0.3;
        let size = 80.0 * pulse;
        let pos = tf.translation + Vec3::Y * 300.0;
        let alpha = 0.6 + (timer * 2.0).sin().abs() * 0.3;

        // Gray death sphere
        g.sphere(Isometry3d::from_translation(pos), size, Color::srgba(0.5, 0.5, 0.6, alpha));

        // Timer indicator rings (one per second remaining)
        let secs = timer.ceil() as u32;
        for i in 0..secs.min(10) {
            let ring_r = 40.0 + i as f32 * 25.0;
            g.circle(
                Isometry3d::from_translation(pos),
                ring_r,
                Color::srgba(0.4, 0.4, 0.5, alpha * 0.3),
            );
        }
    }
}

fn sys_world_hp(mut g: Gizmos, e: Query<(&Transform, &Health, &TeamMember)>, p: Query<&TeamMember, With<PlayerControlled>>) {
    let my = p.iter().next().map(|t| t.0).unwrap_or(Team::Blue);
    for (tf, hp, tm) in &e {
        if hp.current <= 0.0 { continue; }
        let c = tf.translation + Vec3::Y * 120.0;
        let w = 100.0;
        let r = (hp.current / hp.max).clamp(0.0, 1.0);
        let col = if tm.0 == my { Color::srgb(0.1, 0.9, 0.1) }
            else if tm.0 == Team::Neutral { Color::srgb(0.9, 0.9, 0.2) }
            else { Color::srgb(0.9, 0.1, 0.1) };
        for o in 0..4 {
            let y = o as f32 * 2.0;
            g.line(c + Vec3::new(-w * 0.5, y, 0.0), c + Vec3::new(w * 0.5, y, 0.0), Color::srgba(0.06, 0.06, 0.06, 0.9));
            g.line(c + Vec3::new(-w * 0.5, y, 0.0), c + Vec3::new(-w * 0.5 + w * r, y, 0.0), col);
        }
    }
}
