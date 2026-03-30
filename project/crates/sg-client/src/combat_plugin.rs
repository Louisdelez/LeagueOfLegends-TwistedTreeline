use bevy::prelude::*;
use sg_core::components::*;
use sg_core::constants::*;
use sg_core::types::*;
use sg_core::GameSet;
use sg_gameplay::combat::calculate_damage;
use sg_gameplay::economy::{minion_gold, kill_gold};
use sg_gameplay::leveling::{death_timer, level_from_xp, shared_xp, kill_xp};
use sg_ai::champion_ai::{BotController, BotState};
use crate::spawn_plugin::GameTimer;
use crate::map_plugin::MapData;
use crate::menu::AppState;
use crate::shop_plugin::{PlayerInventory, ItemDatabase, InventoryChanged, total_item_bonuses};
use crate::audio_plugin::{SfxHandles, play_sfx};

#[derive(Resource, Default)]
pub struct FirstBloodState {
    pub awarded: bool,
}

/// Track inhibitor state for super minion spawning
#[derive(Resource, Default)]
pub struct InhibitorState {
    pub blue_top_alive: bool,
    pub blue_bot_alive: bool,
    pub red_top_alive: bool,
    pub red_bot_alive: bool,
    pub blue_top_respawn: f32,
    pub blue_bot_respawn: f32,
    pub red_top_respawn: f32,
    pub red_bot_respawn: f32,
}

const INHIBITOR_RESPAWN_TIME: f32 = 300.0; // 5 minutes

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FirstBloodState::default())
            .insert_resource(InhibitorState {
                blue_top_alive: true, blue_bot_alive: true,
                red_top_alive: true, red_bot_alive: true,
                blue_top_respawn: 0.0, blue_bot_respawn: 0.0,
                red_top_respawn: 0.0, red_bot_respawn: 0.0,
            })
            // Target acquisition runs in AI phase (BEFORE movement)
            .add_systems(
            Update,
            (
                turret_acquire_target,
                jungle_camp_acquire_target,
                champion_acquire_target,
            )
                .chain()
                .in_set(GameSet::AI)
                .run_if(in_state(AppState::InGame)),
        )
            // Combat execution runs in Combat phase (AFTER movement)
            .add_systems(
            Update,
            (
                auto_follow_target,
                execute_auto_attacks,
                gold_and_xp_on_kill,
                passive_gold,
                check_level_up,
                handle_champion_death,
                tick_respawn,
            )
                .chain()
                .in_set(GameSet::Combat)
                .run_if(in_state(AppState::InGame)),
        )
            .add_systems(
            Update,
            (
                fountain_heal,
                hp_mana_regen,
                tick_damage_popups,
                draw_damage_popups,
                draw_level_up_effects,
                draw_bounty_popups,
                bot_decision,
                check_inhibitor_destroyed,
                tick_inhibitor_respawn,
                tick_buffs,
                surrender_vote,
                check_nexus_destroyed,
                recalculate_stats,
                draw_cc_indicators,
            )
                .chain()
                .in_set(GameSet::Combat)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// Turret target priority component for aggro tracking
#[derive(Component)]
pub struct TurretAggro {
    pub consecutive_hits: u32,
    pub last_target: Option<Entity>,
}

fn turret_acquire_target(
    mut commands: Commands,
    turrets: Query<
        (Entity, &Transform, &TeamMember, &AutoAttackRange, Option<&TurretAggro>),
        (With<Structure>, Without<AttackTarget>),
    >,
    enemies: Query<(Entity, &Transform, &TeamMember, &Health, Has<Minion>, Has<Champion>)>,
    ally_champs_under_attack: Query<(&AttackTarget, &TeamMember), With<Champion>>,
) {
    for (turret_entity, turret_tf, turret_team, range, aggro) in &turrets {
        let mut best: Option<(Entity, u32, f32)> = None;

        // Check if any enemy champion is attacking an allied champion in turret range
        let mut aggro_swap_target: Option<Entity> = None;
        for (atk, atk_team) in &ally_champs_under_attack {
            // Find the attacker
            if let Ok((_, atk_tf, _, atk_health, _, is_champ)) = enemies.get(atk.entity) {
                if is_champ && atk_health.current > 0.0 {
                    // Is the attacker's target an allied champion?
                    if atk_team.0 == turret_team.0 {
                        let dist = turret_tf.translation.distance(atk_tf.translation);
                        if dist <= range.0 {
                            aggro_swap_target = Some(atk.entity);
                        }
                    }
                }
            }
        }

        if let Some(swap) = aggro_swap_target {
            best = Some((swap, 0, 0.0)); // highest priority
        } else {
            for (enemy_entity, enemy_tf, enemy_team, health, is_minion, _is_champ) in &enemies {
                if enemy_team.0 == turret_team.0 || health.current <= 0.0 { continue; }
                let dist = turret_tf.translation.distance(enemy_tf.translation);
                if dist > range.0 { continue; }
                let priority = if is_minion { 1 } else { 2 };
                if best.map_or(true, |(_, bp, bd)| priority < bp || (priority == bp && dist < bd)) {
                    best = Some((enemy_entity, priority, dist));
                }
            }
        }

        if let Some((target, _, _)) = best {
            // Track consecutive hits for damage ramp
            let hits = if let Some(a) = aggro {
                if a.last_target == Some(target) { a.consecutive_hits } else { 0 }
            } else { 0 };

            commands.entity(turret_entity)
                .insert(AttackTarget { entity: target })
                .insert(AttackCooldown(0.0))
                .insert(TurretAggro { consecutive_hits: hits, last_target: Some(target) });
        }
    }
}

/// Minion target acquisition with LoL priority system
/// Priority: 6=closest enemy minion, 7=closest enemy champion, 8=closest structure
fn minion_acquire_target(
    mut commands: Commands,
    minions: Query<
        (Entity, &Transform, &TeamMember),
        (With<Minion>, Without<AttackTarget>, Without<Dead>),
    >,
    enemy_minions: Query<(Entity, &Transform, &TeamMember, &Health), (With<Minion>, Without<Dead>)>,
    enemy_champs: Query<(Entity, &Transform, &TeamMember, &Health), (With<Champion>, Without<Dead>)>,
    structures: Query<(Entity, &Transform, &TeamMember, &Health), (With<Structure>, Without<Dead>)>,
) {
    for (entity, tf, team) in &minions {
        let pos = tf.translation;
        let mut best: Option<(Entity, u8, f32)> = None; // (entity, priority, dist)

        // Priority 6: closest enemy minion (most common target)
        for (e, e_tf, e_team, e_hp) in &enemy_minions {
            if e_team.0 == team.0 || e_hp.current <= 0.0 { continue; }
            let dist = pos.distance(e_tf.translation);
            if dist > 500.0 { continue; }
            if best.map_or(true, |(_, bp, bd)| 6 < bp || (6 == bp && dist < bd)) {
                best = Some((e, 6, dist));
            }
        }

        // Priority 7: closest enemy champion
        for (e, e_tf, e_team, e_hp) in &enemy_champs {
            if e_team.0 == team.0 || e_hp.current <= 0.0 { continue; }
            let dist = pos.distance(e_tf.translation);
            if dist > 500.0 { continue; }
            if best.map_or(true, |(_, bp, bd)| 7 < bp || (7 == bp && dist < bd)) {
                best = Some((e, 7, dist));
            }
        }

        // Priority 8: closest enemy structure
        for (e, e_tf, e_team, e_hp) in &structures {
            if e_team.0 == team.0 || e_hp.current <= 0.0 { continue; }
            let dist = pos.distance(e_tf.translation);
            if dist > 500.0 { continue; }
            if best.map_or(true, |(_, bp, bd)| 8 < bp || (8 == bp && dist < bd)) {
                best = Some((e, 8, dist));
            }
        }

        if let Some((target, _, _)) = best {
            commands.entity(entity)
                .insert(AttackTarget { entity: target })
                .insert(AttackCooldown(0.0))
                .remove::<MoveTarget>()
                .remove::<crate::navigation_plugin::NavGoal>()
                .remove::<crate::navigation_plugin::NavPath>();
        }
    }
}

/// Minions drop their target when it dies or leaves range, and resume patrol
fn minion_retarget(
    mut commands: Commands,
    minions: Query<(Entity, &Transform, &AttackTarget), (With<Minion>, Without<Dead>)>,
    targets: Query<(&Transform, &Health)>,
) {
    for (entity, tf, target) in &minions {
        let should_drop = if let Ok((tgt_tf, tgt_hp)) = targets.get(target.entity) {
            tgt_hp.current <= 0.0 || tf.translation.distance(tgt_tf.translation) > 700.0
        } else {
            true // target despawned
        };

        if should_drop {
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.remove::<AttackTarget>();
            }
        }
    }
}

fn jungle_camp_acquire_target(
    mut commands: Commands,
    camps: Query<(Entity, &Transform, &AutoAttackRange), (With<JungleCamp>, Without<AttackTarget>, Without<Dead>)>,
    champions: Query<(Entity, &Transform, &Health), With<Champion>>,
) {
    for (camp_entity, camp_tf, range) in &camps {
        let mut closest: Option<(Entity, f32)> = None;
        for (champ_entity, champ_tf, health) in &champions {
            if health.current <= 0.0 { continue; }
            let dist = camp_tf.translation.distance(champ_tf.translation);
            if dist > 475.0 { continue; } // detect range
            if closest.map_or(true, |(_, d)| dist < d) {
                closest = Some((champ_entity, dist));
            }
        }
        if let Some((target, _)) = closest {
            commands.entity(camp_entity)
                .insert(AttackTarget { entity: target })
                .insert(AttackCooldown(0.0));
        }
    }
}

fn champion_acquire_target(
    mut commands: Commands,
    player_q: Query<
        (Entity, &Transform, &TeamMember, &AutoAttackRange),
        (With<PlayerControlled>, Without<MoveTarget>, Without<AttackTarget>, Without<Dead>),
    >,
    enemies: Query<(Entity, &Transform, &TeamMember, &Health), Or<(With<Minion>, With<Structure>, With<Champion>)>>,
) {
    let Ok((player_entity, player_tf, player_team, range)) = player_q.single() else { return };
    let mut closest: Option<(Entity, f32)> = None;
    for (enemy_entity, enemy_tf, enemy_team, health) in &enemies {
        if enemy_team.0 == player_team.0 || health.current <= 0.0 { continue; }
        let dist = player_tf.translation.distance(enemy_tf.translation);
        if dist > range.0 { continue; }
        if closest.map_or(true, |(_, d)| dist < d) {
            closest = Some((enemy_entity, dist));
        }
    }
    if let Some((target, _)) = closest {
        commands.entity(player_entity)
            .insert(AttackTarget { entity: target })
            .insert(AttackCooldown(0.0));
    }
}

/// Floating damage number
#[derive(Component)]
struct DamagePopup {
    lifetime: f32,
    velocity_y: f32,
}

/// Auto-follow: when a champion has AttackTarget but is out of range, move toward target
/// This is the core LoL mechanic: right-click enemy = walk to range + attack
fn auto_follow_target(
    mut commands: Commands,
    attackers: Query<(Entity, &Transform, &AttackTarget, &AutoAttackRange, Option<&Minion>), Without<Dead>>,
    alive_entities: Query<&Transform, Without<Dead>>,
) {
    // Collect actions first to avoid borrow issues with despawned entities
    struct FollowAction { entity: Entity, action: u8, pos: Vec2 } // 0=stop, 1=move_direct, 2=nav_goal, 3=remove_target
    let mut actions = Vec::new();

    for (entity, atk_tf, target, range, is_minion) in &attackers {
        if entity == target.entity { continue; }
        let Ok(tgt_tf) = alive_entities.get(target.entity) else {
            actions.push(FollowAction { entity, action: 3, pos: Vec2::ZERO });
            continue;
        };
        let tgt_pos = tgt_tf.translation;
        let dist = atk_tf.translation.distance(tgt_pos);
        if dist > range.0 + 30.0 {
            let pos = Vec2::new(tgt_pos.x, tgt_pos.z);
            if is_minion.is_some() {
                actions.push(FollowAction { entity, action: 1, pos });
            } else {
                actions.push(FollowAction { entity, action: 2, pos });
            }
        } else {
            actions.push(FollowAction { entity, action: 0, pos: Vec2::ZERO });
        }
    }

    // Apply actions (safe — entity might have been despawned but commands will silently fail)
    for a in actions {
        if let Ok(mut ecmd) = commands.get_entity(a.entity) {
            match a.action {
                0 => { ecmd.remove::<MoveTarget>().remove::<crate::navigation_plugin::NavGoal>().remove::<crate::navigation_plugin::NavPath>(); }
                1 => { ecmd.insert(MoveTarget { position: a.pos }); }
                2 => { ecmd.insert(crate::navigation_plugin::NavGoal { position: a.pos }); }
                3 => { ecmd.remove::<AttackTarget>(); }
                _ => {}
            }
        }
    }
}

fn execute_auto_attacks(
    mut commands: Commands,
    time: Res<Time>,
    sfx: Res<SfxHandles>,
    mut attackers: Query<(
        Entity, &Transform, &CombatStats, &mut AttackCooldown, &AttackTarget, &AutoAttackRange,
    ), (Without<Stunned>, Without<Structure>, Without<Minion>)>,
    mut turret_attackers: Query<(
        Entity, &Transform, &CombatStats, &mut AttackCooldown, &AttackTarget, &AutoAttackRange,
        &mut TurretAggro,
    ), With<Structure>>,
    mut targets: Query<(&Transform, &mut Health, Option<&CombatStats>, Option<&mut crate::ability_plugin::Shield>)>,
) {
    // Phase 1: Collect all attacks (read-only pass on attackers)
    struct Attack { atk: Entity, tgt: Entity, ad: f32, atk_speed: f32, range: f32, atk_pos: Vec3 }
    let mut attacks: Vec<Attack> = Vec::new();

    // Champion/minion attackers
    for (entity, atk_tf, stats, mut cooldown, target, range) in &mut attackers {
        cooldown.0 -= time.delta_secs();
        if cooldown.0 > 0.0 { continue; }
        attacks.push(Attack {
            atk: entity, tgt: target.entity, ad: stats.attack_damage,
            atk_speed: stats.attack_speed, range: range.0, atk_pos: atk_tf.translation,
        });
    }

    // Turret attackers
    for (entity, atk_tf, stats, mut cooldown, target, range, _aggro) in &mut turret_attackers {
        cooldown.0 -= time.delta_secs();
        if cooldown.0 > 0.0 { continue; }
        attacks.push(Attack {
            atk: entity, tgt: target.entity, ad: stats.attack_damage * 1.25, // turret bonus
            atk_speed: stats.attack_speed, range: range.0, atk_pos: atk_tf.translation,
        });
    }

    // Phase 2: Apply damage
    for atk in attacks {
        let Ok((tgt_tf, mut tgt_health, tgt_stats, tgt_shield)) = targets.get_mut(atk.tgt) else {
            commands.entity(atk.atk).remove::<AttackTarget>();
            continue;
        };

        if tgt_health.current <= 0.0 {
            commands.entity(atk.atk).remove::<AttackTarget>();
            continue;
        }

        // Range check
        let dist = atk.atk_pos.distance(tgt_tf.translation);
        if dist > atk.range + 50.0 {
            continue; // auto_follow_target will move us closer
        }

        // Reset cooldown
        if let Ok((_, _, _, mut cd, _, _)) = attackers.get_mut(atk.atk) {
            cd.0 = 1.0 / atk.atk_speed;
        } else if let Ok((_, _, _, mut cd, _, _, _)) = turret_attackers.get_mut(atk.atk) {
            cd.0 = 1.0 / atk.atk_speed;
        }

        // Calculate damage with armor reduction
        let mut damage = if let Some(def_stats) = tgt_stats {
            calculate_damage(atk.ad, DamageType::Physical,
                &CombatStats { attack_damage: atk.ad, ..CombatStats::ZERO }, def_stats)
        } else {
            atk.ad
        };

        // Shield absorption
        if let Some(mut shield) = tgt_shield {
            if shield.amount > 0.0 {
                if shield.amount >= damage { shield.amount -= damage; damage = 0.0; }
                else { damage -= shield.amount; shield.amount = 0.0; }
            }
        }

        tgt_health.current -= damage;
        play_sfx(&mut commands, &sfx.hit);

        // Damage popup
        let popup_pos = tgt_tf.translation + Vec3::new(
            (rand::random::<f32>() - 0.5) * 40.0, 150.0, (rand::random::<f32>() - 0.5) * 40.0,
        );
        commands.spawn((
            Transform::from_translation(popup_pos),
            DamagePopup { lifetime: 1.0, velocity_y: 80.0 },
            DamageAmount(damage),
        ));
    }
}

#[derive(Component)]
struct DamageAmount(f32);

#[derive(Component)]
struct GoldPopup(f32);

#[derive(Component)]
struct BountyPopup {
    gold: f32,
    is_shutdown: bool,
}

/// Award gold and XP when a minion or jungle camp dies near a champion
fn gold_and_xp_on_kill(
    mut commands: Commands,
    sfx: Res<SfxHandles>,
    game_timer: Res<GameTimer>,
    mut champions: Query<(&Transform, &mut Gold, &mut Champion, &TeamMember, &mut GameStats)>,
    minions: Query<(&Transform, &Health, &Minion, &TeamMember)>,
    jungle_camps: Query<(&Transform, &Health, &JungleCamp)>,
) {
    for (champ_tf, mut gold, mut champion, champ_team, mut stats) in &mut champions {
        // Minion gold/xp is handled by MinionPlugin (minion_gold_xp system)
        // Only jungle camps remain here

        // Jungle camp kills
        for (camp_tf, health, camp) in &jungle_camps {
            if health.current > 0.0 { continue; }
            let dist = champ_tf.translation.distance(camp_tf.translation);
            if dist < 550.0 {
                let camp_gold = match camp.camp_type {
                    sg_core::types::CampType::Golem => GOLEM_GOLD,
                    sg_core::types::CampType::Wolf => WOLF_GOLD,
                    sg_core::types::CampType::Wraith => WRAITH_GOLD,
                    sg_core::types::CampType::Vilemaw => 150.0,
                };
                let camp_xp = match camp.camp_type {
                    sg_core::types::CampType::Golem => GOLEM_XP,
                    sg_core::types::CampType::Wolf => WOLF_XP,
                    sg_core::types::CampType::Wraith => WRAITH_XP,
                    sg_core::types::CampType::Vilemaw => 400.0,
                };
                gold.0 += camp_gold;
                stats.cs += 1;
                stats.gold_earned += camp_gold;
                champion.xp += camp_xp;
            }
        }
    }
}

/// Passive gold income
fn passive_gold(
    time: Res<Time>,
    game_timer: Res<GameTimer>,
    mut query: Query<&mut Gold, With<Champion>>,
) {
    if game_timer.elapsed < 90.0 { return; } // gold starts after 1:30
    for mut gold in &mut query {
        gold.0 += AMBIENT_GOLD_PER_TICK * time.delta_secs() / AMBIENT_GOLD_INTERVAL;
    }
}

/// Level up when XP threshold reached
fn check_level_up(
    mut commands: Commands,
    sfx: Res<SfxHandles>,
    mut query: Query<(Entity, &Transform, &mut Champion)>,
) {
    for (_entity, tf, mut champion) in &mut query {
        let new_level = level_from_xp(champion.xp);
        if new_level > champion.level && new_level <= 18 {
            let _old = champion.level;
            champion.level = new_level;
            play_sfx(&mut commands, &sfx.levelup);
            // Level up visual effect
            commands.spawn((
                Transform::from_translation(tf.translation + Vec3::Y * 80.0),
                DamagePopup { lifetime: 1.5, velocity_y: 100.0 },
                LevelUpEffect(new_level),
            ));
        }
    }
}

#[derive(Component)]
struct LevelUpEffect(u8);

/// Handle champion death — award kill gold/XP with bounty system
fn handle_champion_death(
    mut commands: Commands,
    sfx: Res<SfxHandles>,
    game_timer: Res<GameTimer>,
    mut first_blood: ResMut<FirstBloodState>,
    mut champs: Query<(Entity, &Health, &mut Champion, &TeamMember, &Transform, &mut KillStreak, &mut GameStats, &mut Gold), Without<Dead>>,
) {
    // Collect info about dead champions first (to avoid borrow issues)
    let dead_info: Vec<_> = champs.iter()
        .filter(|(_, h, _, _, _, _, _, _)| h.current <= 0.0)
        .map(|(e, _, c, t, tf, ks, _, _)| (e, c.level, t.0, tf.translation, ks.kills, ks.deaths))
        .collect();

    for (dead_entity, dead_level, dead_team, dead_pos, victim_kills, victim_deaths) in &dead_info {
        let timer = death_timer(*dead_level, game_timer.elapsed);
        commands.entity(*dead_entity).insert(Dead { respawn_timer: timer });
        commands.entity(*dead_entity).insert(Visibility::Hidden);
        play_sfx(&mut commands, &sfx.death);

        // Find closest enemy champion within XP_RANGE to award kill gold
        let mut best: Option<(Entity, f32)> = None;
        for (e, _, _, team, tf, _, _, _) in champs.iter() {
            if e == *dead_entity { continue; }
            if team.0 == *dead_team { continue; }
            let dist = tf.translation.distance(*dead_pos);
            if dist < XP_RANGE && best.map_or(true, |(_, d)| dist < d) {
                best = Some((e, dist));
            }
        }

        // Award gold/XP to killer
        if let Some((killer_entity, _)) = best {
            if let Ok((_, _, mut killer_champ, _, _, mut killer_ks, mut killer_stats, mut killer_gold)) = champs.get_mut(killer_entity) {
                let (kill_g, _assist_pool) = kill_gold(*victim_kills, *victim_deaths);
                let mut total = kill_g;

                if !first_blood.awarded {
                    total += FIRST_BLOOD_BONUS;
                    first_blood.awarded = true;
                }

                killer_gold.0 += total;
                killer_ks.kills += 1;
                killer_stats.kills += 1;
                killer_stats.gold_earned += total;

                let xp = kill_xp(*dead_level, killer_champ.level);
                killer_champ.xp += xp;


                // Bounty popup
                let is_shutdown = *victim_kills >= 3;
                commands.spawn((
                    Transform::from_translation(*dead_pos + Vec3::Y * 200.0),
                    DamagePopup { lifetime: 2.0, velocity_y: 50.0 },
                    BountyPopup { gold: total, is_shutdown },
                ));
            }
        }

        // Update victim death count
        if let Ok((_, _, _, _, _, mut victim_ks, mut victim_stats, _)) = champs.get_mut(*dead_entity) {
            victim_ks.deaths += 1;
            victim_stats.deaths += 1;
        }
    }
}

/// Tick respawn timer and resurrect
fn tick_respawn(
    mut commands: Commands,
    time: Res<Time>,
    map: Res<MapData>,
    mut query: Query<(Entity, &mut Dead, &mut Health, &mut Transform, &TeamMember), With<Champion>>,
) {
    for (entity, mut dead, mut health, mut tf, team) in &mut query {
        dead.respawn_timer -= time.delta_secs();
        if dead.respawn_timer <= 0.0 {
            // Respawn at fountain
            let spawn = if team.0 == Team::Blue {
                map.0.blue_fountain
            } else {
                map.0.red_fountain
            };
            tf.translation.x = spawn.x;
            tf.translation.z = spawn.y;
            tf.translation.y = 55.0;
            health.current = health.max;
            commands.entity(entity)
                .remove::<Dead>()
                .remove::<AttackTarget>()
                .insert(Visibility::Inherited);
        }
    }
}

/// Despawn minions with 0 HP
fn cleanup_dead_minions(
    mut commands: Commands,
    query: Query<(Entity, &Health), (With<Minion>, Without<Dead>)>,
) {
    for (entity, health) in &query {
        if health.current <= 0.0 {
            // Despawn dead minions (instant for now)
            if let Ok(mut ecmd) = commands.get_entity(entity) {
                ecmd.despawn();
            }
        }
    }
}

/// Push ALL units apart based on their collision radii (LoL creep block)
/// Melee/Caster radius: 48, Siege/Super radius: 65, Champions: 35
fn unit_separation(
    mut units: Query<(Entity, &mut Transform, Option<&Minion>, Option<&Champion>), Without<Dead>>,
) {
    // Collect all positions + radii
    let data: Vec<(Entity, Vec3, f32)> = units.iter()
        .map(|(e, tf, minion, champ)| {
            let radius = if champ.is_some() {
                35.0
            } else if let Some(m) = minion {
                match m.minion_type {
                    sg_core::types::MinionType::Siege | sg_core::types::MinionType::Super => 65.0,
                    _ => 48.0,
                }
            } else {
                35.0
            };
            (e, tf.translation, radius)
        })
        .collect();

    for (entity, mut tf, _, _) in &mut units {
        let my_pos = tf.translation;
        let my_radius = data.iter().find(|(e, _, _)| *e == entity).map(|(_, _, r)| *r).unwrap_or(35.0);
        let mut push = Vec3::ZERO;

        for (other_e, other_pos, other_radius) in &data {
            if *other_e == entity { continue; }
            let diff = my_pos - *other_pos;
            let dist_xz = Vec2::new(diff.x, diff.z).length();
            let min_dist = my_radius + other_radius;
            if dist_xz < min_dist && dist_xz > 0.1 {
                let overlap = min_dist - dist_xz;
                let dir = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();
                push += dir * overlap * 0.5;
            }
        }

        if push.length() > 0.1 {
            tf.translation.x += push.x;
            tf.translation.z += push.z;
        }
    }
}

fn tick_damage_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut popups: Query<(Entity, &mut Transform, &mut DamagePopup)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut popup) in &mut popups {
        popup.lifetime -= dt;
        tf.translation.y += popup.velocity_y * dt;
        popup.velocity_y *= 0.95; // slow down
        if popup.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_damage_popups(
    mut gizmos: Gizmos,
    damage_popups: Query<(&Transform, &DamagePopup, &DamageAmount)>,
    gold_popups: Query<(&Transform, &DamagePopup, &GoldPopup)>,
) {
    // Damage numbers
    for (tf, popup, dmg) in &damage_popups {
        let alpha = popup.lifetime.clamp(0.0, 1.0);
        let color = if dmg.0 > 200.0 {
            Color::srgba(1.0, 0.2, 0.1, alpha)
        } else if dmg.0 > 50.0 {
            Color::srgba(1.0, 0.8, 0.1, alpha)
        } else {
            Color::srgba(1.0, 1.0, 1.0, alpha)
        };
        let size = 8.0 + dmg.0 * 0.08;
        gizmos.sphere(
            Isometry3d::from_translation(tf.translation),
            size.min(30.0),
            color,
        );
    }

    // Gold popups (golden spheres)
    for (tf, popup, _gold) in &gold_popups {
        let alpha = popup.lifetime.clamp(0.0, 1.0);
        gizmos.sphere(
            Isometry3d::from_translation(tf.translation),
            12.0,
            Color::srgba(1.0, 0.85, 0.0, alpha),
        );
    }
}

fn draw_level_up_effects(
    mut gizmos: Gizmos,
    effects: Query<(&Transform, &DamagePopup, &LevelUpEffect)>,
) {
    for (tf, popup, _lvl) in &effects {
        let alpha = popup.lifetime.clamp(0.0, 1.0);
        // Expanding ring effect
        let ring_size = 50.0 + (1.5 - popup.lifetime) * 100.0;
        gizmos.circle(
            Isometry3d::from_translation(tf.translation),
            ring_size,
            Color::srgba(0.3, 0.8, 1.0, alpha * 0.8),
        );
        gizmos.sphere(
            Isometry3d::from_translation(tf.translation),
            20.0,
            Color::srgba(1.0, 1.0, 0.3, alpha),
        );
    }
}

fn draw_bounty_popups(
    mut gizmos: Gizmos,
    popups: Query<(&Transform, &DamagePopup, &BountyPopup)>,
) {
    for (tf, popup, bounty) in &popups {
        let alpha = popup.lifetime.clamp(0.0, 1.0);
        let color = if bounty.is_shutdown {
            Color::srgba(1.0, 0.3, 0.0, alpha) // orange for shutdown
        } else {
            Color::srgba(1.0, 0.85, 0.0, alpha) // gold
        };
        let size = 15.0 + (bounty.gold / 50.0).min(25.0);
        gizmos.sphere(Isometry3d::from_translation(tf.translation), size, color);
        if bounty.is_shutdown {
            gizmos.circle(
                Isometry3d::from_translation(tf.translation),
                size + 10.0,
                Color::srgba(1.0, 0.1, 0.0, alpha * 0.6),
            );
        }
    }
}

/// Draw visual CC indicators above affected units
fn draw_cc_indicators(
    mut gizmos: Gizmos,
    time: Res<Time>,
    query: Query<(&Transform, Option<&Stunned>, Option<&Rooted>, Option<&Silenced>)>,
) {
    let t = time.elapsed_secs();
    for (tf, stunned, rooted, silenced) in &query {
        let head = tf.translation + Vec3::Y * 160.0;

        if stunned.is_some() {
            // Yellow spinning stars above head
            let spin = t * 5.0;
            for i in 0..3 {
                let angle = spin + i as f32 * std::f32::consts::TAU / 3.0;
                let offset = Vec3::new(angle.cos() * 30.0, 0.0, angle.sin() * 30.0);
                gizmos.sphere(
                    Isometry3d::from_translation(head + offset),
                    8.0,
                    Color::srgba(1.0, 0.9, 0.0, 0.9),
                );
            }
        }

        if rooted.is_some() {
            // Green circle at feet
            let feet = tf.translation + Vec3::Y * 2.0;
            let pulse = 1.0 + (t * 3.0).sin() * 0.15;
            gizmos.circle(
                Isometry3d::from_translation(feet),
                50.0 * pulse,
                Color::srgba(0.2, 0.8, 0.1, 0.7),
            );
            gizmos.circle(
                Isometry3d::from_translation(feet),
                30.0 * pulse,
                Color::srgba(0.1, 0.6, 0.1, 0.5),
            );
        }

        if silenced.is_some() {
            // Red X above head
            let size = 20.0;
            gizmos.line(
                head + Vec3::new(-size, size, 0.0),
                head + Vec3::new(size, -size, 0.0),
                Color::srgba(1.0, 0.1, 0.1, 0.8),
            );
            gizmos.line(
                head + Vec3::new(-size, -size, 0.0),
                head + Vec3::new(size, size, 0.0),
                Color::srgba(1.0, 0.1, 0.1, 0.8),
            );
        }
    }
}

/// Heal champions standing in their fountain
fn fountain_heal(
    time: Res<Time>,
    map: Res<MapData>,
    mut query: Query<(&Transform, &TeamMember, &mut Health, &mut Mana), (With<Champion>, Without<Dead>)>,
) {
    let dt = time.delta_secs();
    for (tf, team, mut health, mut mana) in &mut query {
        let fountain = if team.0 == Team::Blue { map.0.blue_fountain } else { map.0.red_fountain };
        let dist = Vec2::new(tf.translation.x, tf.translation.z).distance(fountain);
        if dist < 500.0 {
            // Heal 10% max HP and 10% max mana per second in fountain
            health.current = (health.current + health.max * 0.10 * dt).min(health.max);
            mana.current = (mana.current + mana.max * 0.10 * dt).min(mana.max);
        }
    }
}

/// Natural HP and Mana regeneration
fn hp_mana_regen(
    time: Res<Time>,
    mut query: Query<(&mut Health, &mut Mana), (With<Champion>, Without<Dead>)>,
) {
    let dt = time.delta_secs();
    for (mut health, mut mana) in &mut query {
        health.current = (health.current + health.regen * dt).min(health.max);
        mana.current = (mana.current + mana.regen * dt).min(mana.max);
    }
}

/// Check if an inhibitor was destroyed
fn check_inhibitor_destroyed(
    mut inhib_state: ResMut<InhibitorState>,
    game_timer: Res<GameTimer>,
    structures: Query<(&Structure, &Health)>,
) {
    for (structure, health) in &structures {
        if structure.structure_type != StructureType::Inhibitor { continue; }
        if health.current > 0.0 { continue; }

        match (structure.team, structure.lane) {
            (Team::Blue, Some(Lane::Top)) if inhib_state.blue_top_alive => {
                inhib_state.blue_top_alive = false;
                inhib_state.blue_top_respawn = game_timer.elapsed + INHIBITOR_RESPAWN_TIME;
            }
            (Team::Blue, Some(Lane::Bottom)) if inhib_state.blue_bot_alive => {
                inhib_state.blue_bot_alive = false;
                inhib_state.blue_bot_respawn = game_timer.elapsed + INHIBITOR_RESPAWN_TIME;
            }
            (Team::Red, Some(Lane::Top)) if inhib_state.red_top_alive => {
                inhib_state.red_top_alive = false;
                inhib_state.red_top_respawn = game_timer.elapsed + INHIBITOR_RESPAWN_TIME;
            }
            (Team::Red, Some(Lane::Bottom)) if inhib_state.red_bot_alive => {
                inhib_state.red_bot_alive = false;
                inhib_state.red_bot_respawn = game_timer.elapsed + INHIBITOR_RESPAWN_TIME;
            }
            _ => {}
        }
    }
}

/// Tick inhibitor respawn timers
fn tick_inhibitor_respawn(
    game_timer: Res<GameTimer>,
    mut inhib_state: ResMut<InhibitorState>,
    mut structures: Query<(&Structure, &mut Health)>,
) {
    // Check each inhib for respawn
    let checks = [
        (Team::Blue, Lane::Top, inhib_state.blue_top_alive, inhib_state.blue_top_respawn),
        (Team::Blue, Lane::Bottom, inhib_state.blue_bot_alive, inhib_state.blue_bot_respawn),
        (Team::Red, Lane::Top, inhib_state.red_top_alive, inhib_state.red_top_respawn),
        (Team::Red, Lane::Bottom, inhib_state.red_bot_alive, inhib_state.red_bot_respawn),
    ];

    for (team, lane, alive, respawn_time) in checks {
        if !alive && game_timer.elapsed >= respawn_time {
            // Respawn the inhibitor
            for (structure, mut health) in &mut structures {
                if structure.structure_type == StructureType::Inhibitor
                    && structure.team == team
                    && structure.lane == Some(lane)
                {
                    health.current = health.max;
                }
            }
            match (team, lane) {
                (Team::Blue, Lane::Top) => inhib_state.blue_top_alive = true,
                (Team::Blue, Lane::Bottom) => inhib_state.blue_bot_alive = true,
                (Team::Red, Lane::Top) => inhib_state.red_top_alive = true,
                (Team::Red, Lane::Bottom) => inhib_state.red_bot_alive = true,
                _ => {}
            }
        }
    }
}

/// Bot AI decision system
fn bot_decision(
    mut commands: Commands,
    time: Res<Time>,
    map: Res<MapData>,
    item_db: Res<ItemDatabase>,
    mut bots: Query<(
        Entity, &Transform, &mut BotController, &TeamMember, &Health, &AutoAttackRange,
        &mut Gold, &CombatStats,
    ), (With<Champion>, Without<Dead>)>,
    potential_targets: Query<(Entity, &Transform, &TeamMember, &Health), (Without<Dead>, Without<BotController>)>,
    enemy_champs: Query<(Entity, &Transform, &TeamMember, &Health), (With<Champion>, Without<Dead>)>,
    minions: Query<(Entity, &Transform, &TeamMember, &Health), (With<Minion>, Without<Dead>)>,
    mut inventories: Query<&mut PlayerInventory>,
) {
    let dt = time.delta_secs();

    // Bot AI decision loop
    for (entity, tf, mut bot, team, health, range, mut gold, _stats) in &mut bots {
        bot.decision_timer -= dt;
        if bot.decision_timer > 0.0 { continue; }
        bot.decision_timer = 0.2;

        let pos = Vec2::new(tf.translation.x, tf.translation.z);
        let hp_pct = health.current / health.max;
        let fountain = if team.0 == Team::Blue { map.0.blue_fountain } else { map.0.red_fountain };
        let at_fountain = pos.distance(fountain) < 600.0;

        // Shopping: buy best affordable item when at fountain or dead
        if at_fountain && gold.0 >= 400.0 {
            if let Ok(mut inv) = inventories.get_mut(entity) {
                if inv.items.len() < 6 {
                    // Find most expensive affordable item
                    let mut best: Option<(u32, u32)> = None; // (id, cost)
                    for item in &item_db.items {
                        if item.cost as f32 <= gold.0 && best.map_or(true, |(_, c)| item.cost > c) {
                            best = Some((item.id, item.cost));
                        }
                    }
                    if let Some((id, cost)) = best {
                        inv.items.push(id);
                        gold.0 -= cost as f32;
                        commands.entity(entity).insert(InventoryChanged);
                    }
                }
            }
        }

        // State transitions
        if hp_pct < 0.30 && bot.state != BotState::Retreating {
            bot.state = BotState::Retreating;
        } else if hp_pct > 0.80 && bot.state == BotState::Retreating {
            bot.state = BotState::Laning;
            bot.patrol_set = false; // Reset patrol to re-find closest waypoint
        }

        match bot.state {
            BotState::Retreating => {
                commands.entity(entity).remove::<AttackTarget>()
                    .insert(crate::navigation_plugin::NavGoal { position: fountain })
                    .remove::<crate::navigation_plugin::NavPath>();
            }
            BotState::Laning | BotState::Fighting => {
                // Priority 1: Low HP enemy champion nearby (finish them off)
                let mut low_champ: Option<(Entity, f32, f32)> = None; // (entity, dist, hp_pct)
                let mut closest_champ: Option<(Entity, f32)> = None;
                for (e, e_tf, e_team, e_health) in &enemy_champs {
                    if e_team.0 == team.0 || e_health.current <= 0.0 { continue; }
                    let dist = pos.distance(Vec2::new(e_tf.translation.x, e_tf.translation.z));
                    if dist < 1500.0 {
                        let ehp = e_health.current / e_health.max;
                        if ehp < 0.4 && low_champ.map_or(true, |(_, _, h)| ehp < h) {
                            low_champ = Some((e, dist, ehp));
                        }
                        if closest_champ.map_or(true, |(_, d)| dist < d) {
                            closest_champ = Some((e, dist));
                        }
                    }
                }

                // Priority 2: Last-hit minions (attack weakest enemy minion in range)
                let mut weakest_minion: Option<(Entity, f32)> = None; // (entity, hp)
                for (e, e_tf, e_team, e_health) in &minions {
                    if e_team.0 == team.0 || e_health.current <= 0.0 { continue; }
                    let dist = pos.distance(Vec2::new(e_tf.translation.x, e_tf.translation.z));
                    if dist < range.0 + 100.0 && weakest_minion.map_or(true, |(_, h)| e_health.current < h) {
                        weakest_minion = Some((e, e_health.current));
                    }
                }

                // Decision: low champ > any champ in range > weakest minion > push lane
                if let Some((target, _, _)) = low_champ {
                    bot.state = BotState::Fighting;
                    commands.entity(entity).remove::<MoveTarget>()
                        .insert(AttackTarget { entity: target })
                        .insert(AttackCooldown(0.0));
                } else if let Some((target, dist)) = closest_champ {
                    if dist < range.0 + 50.0 {
                        bot.state = BotState::Fighting;
                        commands.entity(entity).remove::<MoveTarget>()
                            .insert(AttackTarget { entity: target })
                            .insert(AttackCooldown(0.0));
                    } else {
                        // Move toward enemy champion to engage
                        bot.state = BotState::Fighting;
                        if let Ok((_, e_tf, _, _)) = enemy_champs.get(target) {
                            commands.entity(entity).insert(crate::navigation_plugin::NavGoal {
                                position: Vec2::new(e_tf.translation.x, e_tf.translation.z),
                            }).remove::<crate::navigation_plugin::NavPath>();
                        }
                    }
                } else if let Some((target, _)) = weakest_minion {
                    bot.state = BotState::Laning;
                    commands.entity(entity).remove::<MoveTarget>()
                        .insert(AttackTarget { entity: target })
                        .insert(AttackCooldown(0.0));
                } else {
                    // No targets: follow lane waypoints sequentially (like minions)
                    bot.state = BotState::Laning;
                    let waypoints = match (team.0, bot.assigned_lane) {
                        (Team::Blue, sg_core::types::Lane::Top) => &map.0.lane_paths.top_blue,
                        (Team::Blue, sg_core::types::Lane::Bottom) => &map.0.lane_paths.bottom_blue,
                        (Team::Red, sg_core::types::Lane::Top) => &map.0.lane_paths.top_red,
                        (Team::Red, sg_core::types::Lane::Bottom) => &map.0.lane_paths.bottom_red,
                        _ => &map.0.lane_paths.top_blue,
                    };

                    if waypoints.is_empty() { continue; }

                    // Initialize: find closest waypoint as starting index
                    if !bot.patrol_set {
                        let mut closest_idx = 0;
                        let mut closest_dist = f32::MAX;
                        for (i, wp) in waypoints.iter().enumerate() {
                            let d = pos.distance(*wp);
                            if d < closest_dist {
                                closest_dist = d;
                                closest_idx = i;
                            }
                        }
                        bot.lane_waypoint_index = closest_idx;
                        bot.patrol_set = true;
                    }

                    // Advance to next waypoint if close enough
                    let idx = bot.lane_waypoint_index.min(waypoints.len() - 1);
                    let current_wp = waypoints[idx];
                    if pos.distance(current_wp) < 200.0 {
                        if bot.lane_waypoint_index < waypoints.len() - 1 {
                            bot.lane_waypoint_index += 1;
                        } else {
                            // Reached end of lane — loop back or stay
                            bot.lane_waypoint_index = 0;
                        }
                    }

                    let target_idx = bot.lane_waypoint_index.min(waypoints.len() - 1);
                    commands.entity(entity)
                        .insert(crate::navigation_plugin::NavGoal { position: waypoints[target_idx] })
                        .remove::<crate::navigation_plugin::NavPath>();
                }
            }
            BotState::Dead => {}
        }
    }
}

/// Tick active buffs: decrement duration, apply/remove CC markers
fn tick_buffs(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ActiveBuffs, &mut CombatStats, Option<&BaseStats>)>,
) {
    use sg_core::BuffType;
    let dt = time.delta_secs();

    for (entity, mut buffs, mut stats, base_stats) in &mut query {
        let mut has_stun = false;
        let mut has_root = false;
        let mut has_silence = false;
        let mut slow_pct = 0.0_f32;
        let mut speed_bonus = 0.0_f32;

        // Tick down and collect active effects
        buffs.0.retain_mut(|buff| {
            buff.remaining -= dt;
            if buff.remaining <= 0.0 {
                return false; // remove expired
            }
            match &buff.buff_type {
                BuffType::Stun => has_stun = true,
                BuffType::Root => has_root = true,
                BuffType::Silence => has_silence = true,
                BuffType::Slow { percent } => slow_pct = slow_pct.max(*percent),
                BuffType::SpeedShrine { bonus } => speed_bonus += bonus,
                _ => {}
            }
            true
        });

        // Insert/remove CC marker components
        if has_stun {
            commands.entity(entity).insert(Stunned);
        } else {
            commands.entity(entity).remove::<Stunned>();
        }
        if has_root {
            commands.entity(entity).insert(Rooted);
        } else {
            commands.entity(entity).remove::<Rooted>();
        }
        if has_silence {
            commands.entity(entity).insert(Silenced);
        } else {
            commands.entity(entity).remove::<Silenced>();
        }

        // Apply slow to move speed (restore base first, then apply slow)
        if let Some(base) = base_stats {
            let base_ms = base.move_speed + speed_bonus;
            stats.move_speed = base_ms * (1.0 - slow_pct);
        }
    }
}

/// Recalculate combat stats from base + level scaling + item bonuses
fn recalculate_stats(
    mut commands: Commands,
    db: Res<ItemDatabase>,
    mut query: Query<(Entity, &Champion, &BaseStats, &PlayerInventory, &mut CombatStats, &mut Health, &mut Mana), With<InventoryChanged>>,
) {
    for (entity, champion, base, inventory, mut stats, mut health, mut mana) in &mut query {
        let level = champion.level as f32 - 1.0;
        let (item_ad, item_ap, item_hp, item_armor, item_mr, item_as, item_ms) = total_item_bonuses(inventory, &db);

        let old_max_hp = health.max;
        let old_max_mana = mana.max;

        stats.attack_damage = base.attack_damage + base.ad_per_level * level + item_ad;
        stats.ability_power = base.ability_power + item_ap;
        stats.armor = base.armor + base.armor_per_level * level + item_armor;
        stats.magic_resist = base.magic_resist + base.mr_per_level * level + item_mr;
        stats.attack_speed = base.attack_speed + item_as;
        stats.move_speed = base.move_speed + item_ms;

        health.max = base.base_hp + base.hp_per_level * level + item_hp;
        mana.max = base.base_mana + base.mana_per_level * level;

        // Adjust current HP/mana proportionally
        if old_max_hp > 0.0 {
            let hp_pct = health.current / old_max_hp;
            health.current = health.max * hp_pct;
        }
        if old_max_mana > 0.0 {
            let mana_pct = mana.current / old_max_mana;
            mana.current = mana.max * mana_pct;
        }

        commands.entity(entity).remove::<InventoryChanged>();
    }
}

/// Check if a nexus is destroyed — game over
/// Surrender vote: F6 = instant surrender in solo mode
fn surrender_vote(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    game_timer: Res<GameTimer>,
    mut next_state: ResMut<NextState<AppState>>,
    existing_result: Option<Res<GameResult>>,
) {
    if existing_result.is_some() { return; }
    if keys.just_pressed(KeyCode::F6) {
        commands.insert_resource(GameResult {
            victory: false,
            game_duration: game_timer.elapsed,
        });
        next_state.set(AppState::PostGame);
    }
}

fn check_nexus_destroyed(
    mut commands: Commands,
    structures: Query<(&Structure, &Health)>,
    player_q: Query<&TeamMember, With<PlayerControlled>>,
    game_timer: Res<GameTimer>,
    mut next_state: ResMut<NextState<AppState>>,
    existing_result: Option<Res<GameResult>>,
) {
    // Don't trigger twice
    if existing_result.is_some() { return; }

    let my_team = match player_q.iter().next() {
        Some(t) => t.0,
        None => return,
    };

    for (structure, health) in &structures {
        if structure.structure_type != StructureType::Nexus { continue; }
        if health.current <= 0.0 {
            let victory = structure.team != my_team;
            commands.insert_resource(GameResult {
                victory,
                game_duration: game_timer.elapsed,
            });
            next_state.set(AppState::PostGame);
            return;
        }
    }
}
