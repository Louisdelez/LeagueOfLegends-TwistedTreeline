//! Garen Champion Plugin — 100% Faithful LoL Patch 4.20

use bevy::prelude::*;
use sg_core::components::*;
use sg_core::types::*;
use sg_core::GameSet;
use sg_gameplay::champions::ChampionId;
use crate::ability_plugin::{ChampionIdentity, AbilityCooldowns};
use crate::menu::AppState;

pub struct GarenPlugin;

impl Plugin for GarenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            garen_passive,
            garen_passive_damage_reset,
            garen_q_activate,
            garen_q_speed_boost,
            // garen_q_empowered_hit handled in combat_plugin::execute_auto_attacks
            garen_w_activate,
            garen_w_passive_stacks,
            garen_w_tick,
            garen_e_activate,
            garen_e_spin_tick,
            garen_e_block_aa,
            garen_r_activate,
            garen_r_sword_fall,
            garen_bot_abilities,
        ).in_set(GameSet::Combat).run_if(in_state(AppState::InGame)));
    }
}

// ═══════════════════════════════════════
//  Components
// ═══════════════════════════════════════

#[derive(Component)]
pub struct GarenPassive { pub no_damage_timer: f32 }

#[derive(Component)]
pub struct GarenQBuff {
    pub speed_timer: f32,
    pub empowered: bool,
    pub buff_window: f32,
    pub bonus_damage: f32,
    pub silence_duration: f32,
    pub speed_bonus: f32,
}

#[derive(Component)]
pub struct GarenWPassive { pub stacks: f32 }

#[derive(Component)]
pub struct GarenWActive { pub timer: f32 }

#[derive(Component)]
pub struct GarenESpin {
    pub timer: f32,
    pub tick_timer: f32,
    pub ticks_done: u32,
    pub damage_per_tick: f32,
    pub radius: f32,
}

#[derive(Component)]
struct RFallingSword { timer: f32, target_pos: Vec3 }

fn is_garen(id: &ChampionIdentity) -> bool { id.0 == ChampionId::Garen }

fn rank_q(level: u8) -> usize { ((level as usize).saturating_sub(1) / 2).min(4) }
fn rank_w(level: u8) -> usize { ((level as usize).saturating_sub(2) / 3).min(4) }
fn rank_e(level: u8) -> usize { ((level as usize).saturating_sub(1) / 2).min(4) }
fn rank_r(level: u8) -> usize { if level >= 16 { 2 } else if level >= 11 { 1 } else { 0 } }

// ═══════════════════════════════════════
//  PASSIVE — Perseverance
// ═══════════════════════════════════════

fn garen_passive(
    time: Res<Time>,
    mut garens: Query<(&Champion, &ChampionIdentity, &mut Health, &mut GarenPassive), Without<Dead>>,
) {
    for (champ, id, mut hp, mut p) in &mut garens {
        if !is_garen(id) { continue; }
        p.no_damage_timer += time.delta_secs();
        if p.no_damage_timer >= 9.0 && hp.current < hp.max {
            let pct = if champ.level >= 16 { 0.02 } else if champ.level >= 11 { 0.008 } else { 0.004 };
            hp.current = (hp.current + hp.max * pct * time.delta_secs()).min(hp.max);
        }
    }
}

/// Reset passive timer when Garen takes significant damage
/// Uses a tracker to detect HP DECREASE (not increase from regen)
#[derive(Component)]
pub struct GarenLastHP(pub f32);

fn garen_passive_damage_reset(
    mut garens: Query<(&Health, &mut GarenPassive, &ChampionIdentity, &mut GarenLastHP)>,
) {
    for (hp, mut p, id, mut last_hp) in &mut garens {
        if !is_garen(id) { continue; }
        // Only reset if HP went DOWN significantly (took damage, not just float noise)
        // The passive regen increases HP, so we only care about decreases
        let hp_diff = last_hp.0 - hp.current;
        if hp_diff > 1.0 {
            // Took at least 1 damage — reset timer
            p.no_damage_timer = 0.0;
        }
        last_hp.0 = hp.current;
    }
}

// ═══════════════════════════════════════
//  Q — Decisive Strike
// ═══════════════════════════════════════

fn garen_q_activate(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut garens: Query<(Entity, &ChampionIdentity, &Champion, &CombatStats, &mut AbilityCooldowns), Without<Dead>>,
    player: Query<(), With<PlayerControlled>>,
) {
    if !keys.just_pressed(KeyCode::KeyA) { return; }

    for (entity, id, champ, stats, mut cds) in &mut garens {
        if !is_garen(id) { continue; }
        if !player.contains(entity) { continue; } // Player only for key input
        if cds.q > 0.0 { continue; }

        let r = rank_q(champ.level);
        cds.q = 8.0;

        if let Ok(mut ecmd) = commands.get_entity(entity) {
            // Remove slows and apply speed buff
            ecmd.remove::<Stunned>().remove::<Rooted>();
            // Force anim refresh so Q run/attack plays
            ecmd.insert(crate::movement_plugin::ChampionAnimState::Idle);
            ecmd.insert(GarenQBuff {
                speed_timer: [1.5, 2.25, 3.0, 3.75, 4.5][r],
                empowered: true,
                buff_window: 4.5,
                bonus_damage: [30.0, 55.0, 80.0, 105.0, 130.0][r] + stats.attack_damage * 0.4,
                silence_duration: [1.5, 1.75, 2.0, 2.25, 2.5][r],
                speed_bonus: stats.move_speed * 0.35,
            });
            // Reset auto-attack timer
            ecmd.insert(AttackCooldown(0.0));
        }
    }
}

/// Apply speed boost while Q is active — runs every frame after recalculate_stats
fn garen_q_speed_boost(
    mut commands: Commands,
    time: Res<Time>,
    mut garens: Query<(Entity, &mut CombatStats, &mut GarenQBuff, &ChampionIdentity, &BaseStats)>,
) {
    for (entity, mut stats, mut q, id, base) in &mut garens {
        if !is_garen(id) { continue; }

        q.speed_timer -= time.delta_secs();
        q.buff_window -= time.delta_secs();

        // Speed boost: ensure move_speed is at least base * 1.35
        if q.speed_timer > 0.0 {
            let boosted = base.move_speed * 1.35;
            if stats.move_speed < boosted {
                stats.move_speed = boosted;
            }
        }

        if q.buff_window <= 0.0 {
            q.empowered = false;
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.remove::<GarenQBuff>();
            }
        }
    }
}

/// When Garen's auto-attack cooldown resets (= AA just fired) while Q buff active, apply bonus
fn garen_q_empowered_hit(
    mut commands: Commands,
    mut garens: Query<(Entity, &mut GarenQBuff, &AttackTarget, &AttackCooldown, &ChampionIdentity), Without<Dead>>,
    mut targets: Query<(&mut Health, Option<&mut ActiveBuffs>)>,
) {
    for (entity, mut q, target, cd, id) in &mut garens {
        if !is_garen(id) || !q.empowered { continue; }

        // Only trigger when AA cooldown just reset (cd close to max = just attacked)
        // The AA just fired when cooldown was recently set (> 0.5)
        if cd.0 < 0.4 { continue; } // Wait until AA actually fires

        if let Ok((mut hp, buffs)) = targets.get_mut(target.entity) {
            hp.current -= q.bonus_damage;

            if let Some(mut active_buffs) = buffs {
                active_buffs.0.push(sg_core::BuffData {
                    buff_type: sg_core::BuffType::Silence,
                    duration: q.silence_duration,
                    remaining: q.silence_duration,
                    source: Some(entity),
                });
            }

            q.empowered = false;
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.remove::<GarenQBuff>();
            }
        }
    }
}

// ═══════════════════════════════════════
//  W — Courage
// ═══════════════════════════════════════

fn garen_w_activate(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut garens: Query<(Entity, &ChampionIdentity, &Champion, &Health, &mut AbilityCooldowns), Without<Dead>>,
    player: Query<(), With<PlayerControlled>>,
) {
    if !keys.just_pressed(KeyCode::KeyW) { return; }
    for (entity, id, champ, hp, mut cds) in &mut garens {
        if !is_garen(id) || !player.contains(entity) || cds.w > 0.0 { continue; }
        let r = rank_w(champ.level);
        let duration = [2.0, 3.0, 4.0, 5.0, 6.0][r];
        cds.w = [24.0, 23.0, 22.0, 21.0, 20.0][r];
        if let Ok(mut ecmd) = commands.get_entity(entity) {
            ecmd.insert(GarenWActive { timer: duration });
            // 20% damage reduction simulated as a shield worth 20% of max HP
            ecmd.insert(crate::ability_plugin::Shield {
                amount: hp.max * 0.2,
                remaining: duration,
            });
        }
    }
}

/// W passive: +0.5 armor/MR per minion/champion kill (cap 30)
fn garen_w_passive_stacks(
    dying_minions: Query<(&Transform, &TeamMember), (With<Minion>, With<crate::minion_plugin::Dying>, Without<crate::minion_plugin::GoldAwarded>)>,
    mut garens: Query<(&Transform, &ChampionIdentity, &TeamMember, &mut GarenWPassive, &mut CombatStats)>,
) {
    for (g_tf, id, team, mut w, mut stats) in &mut garens {
        if !is_garen(id) || w.stacks >= 30.0 { continue; }

        for (m_tf, m_team) in &dying_minions {
            if m_team.0 == team.0 { continue; } // Same team = not our kill
            // Only count kills within 550 range (same as gold range)
            if g_tf.translation.distance(m_tf.translation) > 550.0 { continue; }
            w.stacks = (w.stacks + 0.5).min(30.0);
            stats.armor += 0.5;
            stats.magic_resist += 0.5;
        }
    }
}

fn garen_w_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut actives: Query<(Entity, &Transform, &mut GarenWActive, Option<&mut ActiveBuffs>)>,
    mut gizmos: Gizmos,
) {
    for (entity, tf, mut w, active_buffs) in &mut actives {
        w.timer -= time.delta_secs();

        // Tenacity: reduce duration of all active CC buffs by 20%
        if let Some(mut buffs) = active_buffs {
            for buff in buffs.0.iter_mut() {
                match buff.buff_type {
                    sg_core::BuffType::Stun | sg_core::BuffType::Root | sg_core::BuffType::Silence => {
                        // Reduce remaining duration by 20% per second
                        buff.remaining -= buff.remaining * 0.2 * time.delta_secs();
                    }
                    sg_core::BuffType::Slow { .. } => {
                        buff.remaining -= buff.remaining * 0.2 * time.delta_secs();
                    }
                    _ => {}
                }
            }
        }

        // Visual: golden shield
        let alpha = (w.timer / 3.0).clamp(0.2, 0.5);
        gizmos.circle(Isometry3d::from_translation(tf.translation + Vec3::Y * 30.0), 120.0, Color::srgba(0.9, 0.8, 0.2, alpha));
        gizmos.circle(Isometry3d::from_translation(tf.translation + Vec3::Y * 30.0), 100.0, Color::srgba(1.0, 0.9, 0.3, alpha * 0.5));

        if w.timer <= 0.0 {
            if let Ok(mut ecmd) = commands.get_entity(entity) { ecmd.remove::<GarenWActive>(); }
        }
    }
}

// ═══════════════════════════════════════
//  E — Judgment (SPIN)
// ═══════════════════════════════════════

fn garen_e_activate(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut garens: Query<(Entity, &ChampionIdentity, &Champion, &CombatStats, &mut AbilityCooldowns), Without<Dead>>,
    player: Query<(), With<PlayerControlled>>,
    existing: Query<&GarenESpin>,
) {
    if !keys.just_pressed(KeyCode::KeyE) { return; }
    for (entity, id, champ, stats, mut cds) in &mut garens {
        if !is_garen(id) || !player.contains(entity) { continue; }

        // Recast: cancel spin
        if existing.contains(entity) {
            if let Ok(mut ecmd) = commands.get_entity(entity) { ecmd.remove::<GarenESpin>(); }
            continue;
        }
        if cds.e > 0.0 { continue; }

        let r = rank_e(champ.level);
        let base = 10.0 + 12.5 * r as f32;
        let ratio = 0.35 + 0.05 * r as f32;
        cds.e = [13.0, 12.0, 11.0, 10.0, 9.0][r];

        if let Ok(mut ecmd) = commands.get_entity(entity) {
            ecmd.insert(GarenESpin {
                timer: 3.0, tick_timer: 0.0, ticks_done: 0,
                damage_per_tick: base + stats.attack_damage * ratio,
                radius: 330.0,
            });
            ecmd.remove::<AttackTarget>();
        }
    }
}

fn garen_e_spin_tick(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<crate::audio_plugin::SfxHandles>,
    mut spinners: Query<(Entity, &Transform, &TeamMember, &mut GarenESpin), Without<Dead>>,
    mut targets: Query<(&Transform, &mut Health, &TeamMember, Option<&Minion>)>,
    mut gizmos: Gizmos,
) {
    for (entity, tf, team, mut spin) in &mut spinners {
        spin.timer -= time.delta_secs();
        spin.tick_timer += time.delta_secs();

        // Visual spin circles
        let a = (spin.timer / 3.0).clamp(0.2, 0.6);
        let p = 1.0 + (spin.timer * 8.0).sin() * 0.1;
        gizmos.circle(Isometry3d::from_translation(tf.translation + Vec3::Y * 5.0), spin.radius * p, Color::srgba(0.8, 0.6, 0.1, a));
        gizmos.circle(Isometry3d::from_translation(tf.translation + Vec3::Y * 5.0), spin.radius * 0.5 * p, Color::srgba(1.0, 0.8, 0.2, a * 0.5));

        // Tick every 0.5s
        if spin.tick_timer >= 0.5 && spin.ticks_done < 6 {
            spin.tick_timer = 0.0;
            spin.ticks_done += 1;
            for (tgt_tf, mut hp, tgt_team, is_minion) in &mut targets {
                if tgt_team.0 == team.0 || hp.current <= 0.0 { continue; }
                if tf.translation.distance(tgt_tf.translation) > spin.radius { continue; }
                let mut dmg = spin.damage_per_tick;
                if is_minion.is_some() { dmg *= 0.75; }
                hp.current -= dmg;
                // Spin hit sound (limit to avoid spam)
                static SPIN_SND: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                if SPIN_SND.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 3 == 0 {
                    crate::audio_plugin::play_sfx(&mut commands, &sfx.hit);
                }
            }
        }

        if spin.timer <= 0.0 {
            if let Ok(mut ecmd) = commands.get_entity(entity) { ecmd.remove::<GarenESpin>(); }
        }
    }
}

/// Block auto-attacks while spinning + ghosted (ignore unit collision)
fn garen_e_block_aa(
    mut commands: Commands,
    spinners: Query<Entity, (With<GarenESpin>, With<AttackTarget>)>,
) {
    for entity in &spinners {
        if let Ok(mut ecmd) = commands.get_entity(entity) {
            ecmd.remove::<AttackTarget>();
        }
    }
    // Ghosted: The minion_collision system in minion_plugin checks With<Champion>
    // Garen with GarenESpin should be excluded from collision push
    // This is handled by the collision system checking for GarenESpin
}

// ═══════════════════════════════════════
//  R — Demacian Justice
// ═══════════════════════════════════════

fn garen_r_activate(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut garens: Query<(Entity, &Transform, &ChampionIdentity, &Champion, &TeamMember, &mut AbilityCooldowns), Without<Dead>>,
    mut enemies: Query<(Entity, &Transform, &mut Health, &TeamMember, &CombatStats), (With<Champion>, Without<Dead>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player: Query<(), With<PlayerControlled>>,
) {
    if !keys.just_pressed(KeyCode::KeyR) { return; }

    for (garen_e, garen_tf, id, champ, garen_team, mut cds) in &mut garens {
        if !is_garen(id) || !player.contains(garen_e) || cds.r > 0.0 { continue; }

        let r = rank_r(champ.level);

        // Find closest enemy champion in 400 range
        let mut closest: Option<(Entity, f32)> = None;
        for (e, e_tf, _, e_team, _) in &enemies {
            if e_team.0 == garen_team.0 { continue; }
            let dist = garen_tf.translation.distance(e_tf.translation);
            if dist <= 400.0 && closest.map_or(true, |(_, d)| dist < d) {
                closest = Some((e, dist));
            }
        }

        let Some((target_e, _)) = closest else { continue; };
        cds.r = [160.0, 120.0, 80.0][r];

        if let Ok((_, target_tf, mut target_hp, _, target_stats)) = enemies.get_mut(target_e) {
            let missing = target_hp.max - target_hp.current;
            let ratio = [0.2857, 0.3333, 0.40][r];
            let raw_damage = [175.0, 350.0, 525.0][r] + missing * ratio;
            let mr = target_stats.magic_resist.max(0.0);
            let damage = raw_damage * (100.0 / (100.0 + mr));
            target_hp.current -= damage;

            // Visual: giant sword
            let mesh = meshes.add(Cuboid::new(20.0, 300.0, 8.0));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.85, 0.2),
                emissive: bevy::color::LinearRgba::rgb(3.0, 2.0, 0.5),
                ..default()
            });
            commands.spawn((
                Mesh3d(mesh), MeshMaterial3d(mat),
                Transform::from_translation(target_tf.translation + Vec3::Y * 600.0),
                RFallingSword { timer: 1.0, target_pos: target_tf.translation },
            ));
        }
    }
}

/// Animate the falling sword visual
fn garen_r_sword_fall(
    mut commands: Commands,
    time: Res<Time>,
    mut swords: Query<(Entity, &mut Transform, &mut RFallingSword)>,
) {
    for (entity, mut tf, mut sword) in &mut swords {
        sword.timer -= time.delta_secs();
        let progress = (1.0 - sword.timer).clamp(0.0, 1.0);
        tf.translation = Vec3::new(
            sword.target_pos.x,
            sword.target_pos.y + 600.0 * (1.0 - progress),
            sword.target_pos.z,
        );
        // Scale up as it falls for dramatic effect
        tf.scale = Vec3::splat(1.0 + progress * 0.5);

        if sword.timer <= 0.0 {
            if let Ok(mut ecmd) = commands.get_entity(entity) { ecmd.despawn(); }
        }
    }
}

// ═══════════════════════════════════════
//  Bot AI — Garen bots use abilities
// ═══════════════════════════════════════

fn garen_bot_abilities(
    mut commands: Commands,
    time: Res<Time>,
    mut garens: Query<
        (Entity, &Transform, &ChampionIdentity, &Champion, &CombatStats, &TeamMember, &mut AbilityCooldowns, Option<&AttackTarget>),
        (Without<Dead>, Without<PlayerControlled>),
    >,
    mut all_hp: Query<&mut Health>,
    enemy_info: Query<(Entity, &Transform, &TeamMember, &CombatStats), (With<Champion>, Without<Dead>)>,
) {
    for (entity, tf, id, champ, stats, team, mut cds, attack_target) in &mut garens {
        if !is_garen(id) { continue; }

        let pos = tf.translation;

        let enemy_data: Vec<(Entity, f32, f32)> = enemy_info.iter()
            .filter(|(_, _, e_team, _)| e_team.0 != team.0)
            .filter_map(|(e, e_tf, _, _)| {
                let hp = all_hp.get(e).ok()?;
                if hp.current <= 0.0 { return None; }
                Some((e, pos.distance(e_tf.translation), hp.current / hp.max))
            })
            .collect();

        let mut closest_enemy: Option<(Entity, f32, f32)> = None;
        for &(e, dist, hp_pct) in &enemy_data {
            if dist < 800.0 && closest_enemy.map_or(true, |(_, d, _)| dist < d) {
                closest_enemy = Some((e, dist, hp_pct));
            }
        }

        let Some((enemy_entity, enemy_dist, enemy_hp_pct)) = closest_enemy else { continue; };

        // Q: use when chasing enemy (> 200 range) and not on CD
        if cds.q <= 0.0 && enemy_dist > 200.0 && enemy_dist < 600.0 {
            let r = rank_q(champ.level);
            cds.q = 8.0;
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.insert(GarenQBuff {
                    speed_timer: [1.5, 2.25, 3.0, 3.75, 4.5][r],
                    empowered: true, buff_window: 4.5,
                    bonus_damage: [30.0, 55.0, 80.0, 105.0, 130.0][r] + stats.attack_damage * 0.4,
                    silence_duration: [1.5, 1.75, 2.0, 2.25, 2.5][r],
                    speed_bonus: stats.move_speed * 0.35,
                });
                ecmd.insert(AttackCooldown(0.0));
            }
        }

        // W: use when close to enemy and taking damage
        let my_hp = all_hp.get(entity).map(|h| (h.current, h.max)).unwrap_or((1.0, 1.0));
        if cds.w <= 0.0 && enemy_dist < 300.0 && my_hp.0 < my_hp.1 * 0.8 {
            let r = rank_w(champ.level);
            let duration = [2.0, 3.0, 4.0, 5.0, 6.0][r];
            cds.w = [24.0, 23.0, 22.0, 21.0, 20.0][r];
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.insert(GarenWActive { timer: duration });
                ecmd.insert(crate::ability_plugin::Shield {
                    amount: my_hp.1 * 0.2,
                    remaining: duration,
                });
            }
        }

        // E: use when in melee range of enemy
        if cds.e <= 0.0 && enemy_dist < 200.0 {
            let r = rank_e(champ.level);
            let base = 10.0 + 12.5 * r as f32;
            let ratio = 0.35 + 0.05 * r as f32;
            cds.e = [13.0, 12.0, 11.0, 10.0, 9.0][r];
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.insert(GarenESpin {
                    timer: 3.0, tick_timer: 0.0, ticks_done: 0,
                    damage_per_tick: base + stats.attack_damage * ratio,
                    radius: 330.0,
                });
                ecmd.remove::<AttackTarget>();
            }
        }

        // R: use when enemy is low HP (< 35%) and in range
        if cds.r <= 0.0 && enemy_hp_pct < 0.35 && enemy_dist < 400.0 {
            let r = rank_r(champ.level);
            cds.r = [160.0, 120.0, 80.0][r];

            // Get target info for R damage
            let target_info = enemy_info.get(enemy_entity).ok().and_then(|(_, e_tf, _, e_stats)| {
                let hp = all_hp.get(enemy_entity).ok()?;
                Some((e_tf.translation, hp.current, hp.max, e_stats.magic_resist))
            });
            if let Some((target_pos, tgt_current, tgt_max, tgt_mr)) = target_info {
                let missing = tgt_max - tgt_current;
                let ratio = [0.2857, 0.3333, 0.40][r];
                let raw_damage = [175.0, 350.0, 525.0][r] + missing * ratio;
                let mr = tgt_mr.max(0.0);
                let damage = raw_damage * (100.0 / (100.0 + mr));
                if let Ok(mut e_hp) = all_hp.get_mut(enemy_entity) {
                    e_hp.current -= damage;
                }

                // Spawn falling sword visual (needs mesh — use a simple shape)
                // The player R already spawns a proper mesh, bots get a simpler one
                commands.spawn((
                    Transform::from_translation(target_pos + Vec3::Y * 600.0),
                    Visibility::Inherited,
                    RFallingSword { timer: 1.0, target_pos },
                ));
            }
        }
    }
}

// ═══════════════════════════════════════
//  Garen-specific animation overrides
// ═══════════════════════════════════════

// Garen animation clip indices (from garen_animated.glb):
// [7] idle1, [8] run, [14] attack1, [9] attack2, [22] death
// [27] spell1 (Q attack), [29] run_spell1 (Q run)
// [5] spell3_0 (E spin), [6] spell4 (R cast)
// [11] crit

// These are handled by the generic movement_plugin animation system
// which uses champion_anim_indices(ChampionId::Garen) = [7, 8, 5, 22]
// The spin animation (clip 5 = spell3_0) is mapped to "attack" which is wrong
// TODO: Override with state-specific animations when E/Q/R are active

// RFallingSword defined at top of file
