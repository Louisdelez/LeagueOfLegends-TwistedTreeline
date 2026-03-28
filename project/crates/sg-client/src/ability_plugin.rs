use bevy::prelude::*;
use bevy::scene::SceneRoot;
use sg_core::components::*;
use sg_core::types::*;
use sg_core::GameSet;
use sg_gameplay::champions::{ChampionClass, ChampionId, get_champion_by_id};
use crate::menu::AppState;

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ability_input,
            process_skillshots,
            process_aoe_effects,
            tick_shields,
            tick_tibbers,
        ).chain().in_set(GameSet::Combat).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
pub struct Skillshot {
    pub direction: Vec3,
    pub speed: f32,
    pub damage: f32,
    pub range: f32,
    pub traveled: f32,
    pub team: Team,
    pub cc: Option<sg_core::BuffType>,
    pub cc_duration: f32,
}

#[derive(Component)]
pub struct AoeZone {
    pub center: Vec3,
    pub radius: f32,
    pub damage: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub team: Team,
    pub has_hit: bool,
    pub cc: Option<sg_core::BuffType>,
    pub cc_duration: f32,
}

#[derive(Component)]
pub struct Shield {
    pub amount: f32,
    pub remaining: f32,
}

#[derive(Component)]
pub struct AbilityCooldowns {
    pub q: f32,
    pub w: f32,
    pub e: f32,
    pub r: f32,
}

impl Default for AbilityCooldowns {
    fn default() -> Self {
        Self { q: 0.0, w: 0.0, e: 0.0, r: 0.0 }
    }
}

/// Stored on the champion to know which kit to use
#[derive(Component)]
pub struct ChampionKit(pub ChampionClass);

/// Tibbers pet with limited lifetime
#[derive(Component)]
pub struct TibbersPet { pub lifetime: f32 }

/// Champion identity for ability stats lookup
#[derive(Component)]
pub struct ChampionIdentity(pub ChampionId);

fn ability_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut player_q: Query<
        (Entity, &Transform, &TeamMember, &CombatStats, &mut AbilityCooldowns, &mut Mana, Option<&ChampionKit>, Option<&ChampionIdentity>, &Champion),
        (With<PlayerControlled>, Without<Dead>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((player_entity, player_tf, team, stats, mut cds, mut mana, kit_opt, champ_id_opt, champion)) = player_q.single_mut() else { return };
    let dt = time.delta_secs();
    let kit = kit_opt.map(|k| k.0).unwrap_or(ChampionClass::Mage);

    // Get champion-specific stats if available
    let champ_def = champ_id_opt.map(|id| get_champion_by_id(id.0));
    let level = champion.level as usize;
    let rank_q = (level / 2).min(4); // ability rank scales with level
    let rank_r = if level >= 16 { 2 } else if level >= 11 { 1 } else { 0 };

    // Use champion-specific mana costs or defaults
    let mana_costs = match kit {
        ChampionClass::Mage => [60.0, 80.0, 50.0, 100.0],
        ChampionClass::Fighter => [40.0, 50.0, 45.0, 80.0],
        ChampionClass::Tank => [50.0, 60.0, 55.0, 90.0],
    };

    cds.q = (cds.q - dt).max(0.0);
    cds.w = (cds.w - dt).max(0.0);
    cds.e = (cds.e - dt).max(0.0);
    cds.r = (cds.r - dt).max(0.0);

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_tf)) = camera_q.single() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else { return };
    let Some(dist) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else { return };
    let target_pos = ray.get_point(dist);

    let player_pos = player_tf.translation;
    let direction = (target_pos - player_pos).normalize_or_zero();

    // === Q ===
    if keys.just_pressed(KeyCode::KeyA) && cds.q <= 0.0 && mana.current >= mana_costs[0] {
        let (q_cd, q_dmg, q_ratio) = if let Some(ref def) = champ_def {
            (def.q_cd[rank_q], def.q_dmg[rank_q], def.q_ap_ratio)
        } else {
            match kit { ChampionClass::Mage => (5.0, 80.0, 0.65), ChampionClass::Fighter => (6.0, 60.0, 0.8), ChampionClass::Tank => (8.0, 40.0, 0.3) }
        };
        let q_total = q_dmg + if kit == ChampionClass::Fighter { stats.attack_damage * q_ratio } else { stats.ability_power * q_ratio };
        cds.q = q_cd;
        mana.current -= mana_costs[0];
        match kit {
            ChampionClass::Mage => {
                spawn_skillshot(&mut commands, &mut meshes, &mut materials,
                    player_pos, direction, 2000.0, q_total, 1200.0, team.0,
                    Color::srgb(0.3, 0.5, 1.0), [0.5, 1.0, 3.0]);
            }
            ChampionClass::Fighter => {
                let dash_target = player_pos + direction * 300.0;
                commands.entity(player_entity).remove::<AttackTarget>().insert(
                    MoveTarget { position: Vec2::new(dash_target.x, dash_target.z) }
                );
                spawn_aoe(&mut commands, &mut meshes, &mut materials,
                    dash_target, 120.0, q_total, 0.3, team.0,
                    Color::srgba(0.9, 0.5, 0.1, 0.4), [2.0, 0.5, 0.0]);
            }
            ChampionClass::Tank => {
                spawn_aoe_cc(&mut commands, &mut meshes, &mut materials,
                    player_pos, 200.0, q_total, 0.5, team.0,
                    Color::srgba(0.4, 0.7, 0.3, 0.3), [0.5, 1.0, 0.2],
                    Some(sg_core::BuffType::Stun), 1.5);
            }
        }
    }

    // === W ===
    if keys.just_pressed(KeyCode::KeyW) && cds.w <= 0.0 && mana.current >= mana_costs[1] {
        let (w_cd, w_dmg, w_ratio) = if let Some(ref def) = champ_def {
            (def.w_cd[rank_q], def.w_dmg[rank_q], def.w_ap_ratio)
        } else {
            match kit { ChampionClass::Mage => (8.0, 120.0, 0.7), ChampionClass::Fighter => (12.0, 80.0, 0.5), ChampionClass::Tank => (14.0, 100.0, 0.3) }
        };
        cds.w = w_cd;
        mana.current -= mana_costs[1];
        match kit {
            ChampionClass::Mage => {
                let w_total = w_dmg + stats.ability_power * w_ratio;
                spawn_aoe(&mut commands, &mut meshes, &mut materials,
                    target_pos, 150.0, w_total, 0.5, team.0,
                    Color::srgba(0.9, 0.3, 0.1, 0.4), [2.0, 0.5, 0.0]);
            }
            ChampionClass::Fighter => {
                let shield_amount = w_dmg + stats.attack_damage * w_ratio;
                commands.entity(player_entity).insert(Shield { amount: shield_amount, remaining: 4.0 });
            }
            ChampionClass::Tank => {
                let shield_amount = w_dmg + stats.armor * 0.3 + stats.magic_resist * 0.3;
                commands.entity(player_entity).insert(Shield { amount: shield_amount, remaining: 5.0 });
            }
        }
    }

    // === E ===
    if keys.just_pressed(KeyCode::KeyE) && cds.e <= 0.0 && mana.current >= mana_costs[2] {
        let (e_cd, e_dmg, e_ratio) = if let Some(ref def) = champ_def {
            (def.e_cd[rank_q], def.e_dmg[rank_q], def.e_ad_ratio)
        } else {
            match kit { ChampionClass::Mage => (14.0, 0.0, 0.0), ChampionClass::Fighter => (10.0, 70.0, 0.6), ChampionClass::Tank => (12.0, 60.0, 0.5) }
        };
        cds.e = e_cd;
        mana.current -= mana_costs[2];
        match kit {
            ChampionClass::Mage => {
                let dash_target = player_pos + direction * 400.0;
                commands.entity(player_entity).remove::<AttackTarget>().insert(
                    MoveTarget { position: Vec2::new(dash_target.x, dash_target.z) }
                );
            }
            ChampionClass::Fighter => {
                let e_total = e_dmg + stats.attack_damage * e_ratio;
                spawn_skillshot_cc(&mut commands, &mut meshes, &mut materials,
                    player_pos, direction, 1500.0, e_total, 800.0, team.0,
                    Color::srgb(0.8, 0.4, 0.1), [1.5, 0.5, 0.0],
                    Some(sg_core::BuffType::Stun), 1.0);
            }
            ChampionClass::Tank => {
                let e_total = e_dmg + stats.attack_damage * e_ratio;
                spawn_aoe_cc(&mut commands, &mut meshes, &mut materials,
                    target_pos, 200.0, e_total, 1.0, team.0,
                    Color::srgba(0.5, 0.4, 0.2, 0.3), [0.8, 0.5, 0.1],
                    Some(sg_core::BuffType::Slow { percent: 0.4 }), 2.0);
            }
        }
    }

    // === R ===
    if keys.just_pressed(KeyCode::KeyR) && cds.r <= 0.0 && mana.current >= mana_costs[3] {
        let (r_cd, r_dmg, r_ratio) = if let Some(ref def) = champ_def {
            (def.r_cd[rank_r], def.r_dmg[rank_r], def.r_ap_ratio)
        } else {
            match kit { ChampionClass::Mage => (90.0, 250.0, 0.9), ChampionClass::Fighter => (100.0, 150.0, 0.6), ChampionClass::Tank => (120.0, 200.0, 0.6) }
        };
        let r_total = r_dmg + if kit == ChampionClass::Fighter { stats.attack_damage * r_ratio } else { stats.ability_power * r_ratio };
        cds.r = r_cd;
        mana.current -= mana_costs[3];
        match kit {
            ChampionClass::Mage => {
                // AOE damage at target
                spawn_aoe(&mut commands, &mut meshes, &mut materials,
                    target_pos, 300.0, r_total, 0.3, team.0,
                    Color::srgba(0.8, 0.1, 0.1, 0.3), [3.0, 0.0, 0.0]);

                // Annie special: spawn Tibbers pet
                if champ_id_opt.map(|id| id.0) == Some(ChampionId::Annie) {
                    let tibbers_mesh = meshes.add(Sphere::new(40.0));
                    let tibbers_mat = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.4, 0.2, 0.1),
                        emissive: bevy::color::LinearRgba::rgb(1.0, 0.3, 0.0),
                        ..default()
                    });
                    let tibbers = commands.spawn((
                        Mesh3d(tibbers_mesh), MeshMaterial3d(tibbers_mat),
                        Transform::from_translation(target_pos + Vec3::Y * 40.0),
                        sg_core::components::TeamMember(team.0),
                        sg_core::components::Health { current: 1200.0, max: 1200.0, regen: 0.0 },
                        sg_core::components::CombatStats {
                            attack_damage: 80.0 + stats.ability_power * 0.15,
                            ability_power: 0.0, armor: 30.0, magic_resist: 30.0,
                            attack_speed: 0.8, move_speed: 350.0,
                            crit_chance: 0.0, cdr: 0.0, armor_pen_flat: 0.0, armor_pen_pct: 0.0,
                            magic_pen_flat: 0.0, magic_pen_pct: 0.0, life_steal: 0.0, spell_vamp: 0.0,
                        },
                        sg_core::components::AutoAttackRange(200.0),
                        sg_core::components::AttackCooldown(0.0),
                        TibbersPet { lifetime: 45.0 },
                    )).id();
                    // Load Tibbers model
                    commands.entity(tibbers).with_children(|parent| {
                        parent.spawn((
                            SceneRoot(asset_server.load("models/champions/tibbers.glb#Scene0")),
                            Transform::from_translation(Vec3::new(0.0, -40.0, 0.0)),
                        ));
                    });
                }
            }
            ChampionClass::Fighter => {
                spawn_aoe(&mut commands, &mut meshes, &mut materials,
                    player_pos, 250.0, r_total, 2.0, team.0,
                    Color::srgba(0.9, 0.3, 0.0, 0.3), [2.0, 0.3, 0.0]);
            }
            ChampionClass::Tank => {
                spawn_aoe_cc(&mut commands, &mut meshes, &mut materials,
                    player_pos, 350.0, r_total, 0.5, team.0,
                    Color::srgba(0.3, 0.5, 0.2, 0.4), [0.5, 1.5, 0.2],
                    Some(sg_core::BuffType::Stun), 1.5);
            }
        }
    }
}

fn spawn_skillshot(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3, direction: Vec3, speed: f32, damage: f32, range: f32, team: Team,
    color: Color, emissive: [f32; 3],
) {
    spawn_skillshot_cc(commands, meshes, materials, origin, direction, speed, damage, range, team, color, emissive, None, 0.0);
}

fn spawn_skillshot_cc(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3, direction: Vec3, speed: f32, damage: f32, range: f32, team: Team,
    color: Color, emissive: [f32; 3],
    cc: Option<sg_core::BuffType>, cc_duration: f32,
) {
    let mesh = meshes.add(Sphere::new(15.0));
    let mat = materials.add(StandardMaterial {
        base_color: color,
        emissive: bevy::color::LinearRgba::rgb(emissive[0], emissive[1], emissive[2]),
        ..default()
    });
    commands.spawn((
        Mesh3d(mesh), MeshMaterial3d(mat),
        Transform::from_translation(origin + Vec3::Y * 50.0),
        Skillshot { direction, speed, damage, range, traveled: 0.0, team, cc, cc_duration },
    ));
}

fn spawn_aoe(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center: Vec3, radius: f32, damage: f32, duration: f32, team: Team,
    color: Color, emissive: [f32; 3],
) {
    spawn_aoe_cc(commands, meshes, materials, center, radius, damage, duration, team, color, emissive, None, 0.0);
}

fn spawn_aoe_cc(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center: Vec3, radius: f32, damage: f32, duration: f32, team: Team,
    color: Color, emissive: [f32; 3],
    cc: Option<sg_core::BuffType>, cc_duration: f32,
) {
    let mesh = meshes.add(Cylinder::new(radius, 5.0));
    let mat = materials.add(StandardMaterial {
        base_color: color,
        emissive: bevy::color::LinearRgba::rgb(emissive[0], emissive[1], emissive[2]),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(mesh), MeshMaterial3d(mat),
        Transform::from_translation(center + Vec3::Y * 2.0),
        AoeZone { center, radius, damage, duration, elapsed: 0.0, team, has_hit: false, cc, cc_duration },
    ));
}

fn process_skillshots(
    mut commands: Commands,
    time: Res<Time>,
    mut skillshots: Query<(Entity, &mut Transform, &mut Skillshot)>,
    mut targets: Query<(Entity, &Transform, &TeamMember, &mut Health, Option<&mut ActiveBuffs>), Without<Skillshot>>,
) {
    let dt = time.delta_secs();
    for (shot_entity, mut shot_tf, mut shot) in &mut skillshots {
        let step = shot.speed * dt;
        shot_tf.translation += shot.direction * step;
        shot.traveled += step;
        if shot.traveled >= shot.range {
            commands.entity(shot_entity).despawn();
            continue;
        }
        let mut hit = false;
        for (_target_entity, target_tf, target_team, mut health, buffs) in &mut targets {
            if target_team.0 == shot.team || health.current <= 0.0 { continue; }
            if shot_tf.translation.distance(target_tf.translation) < 60.0 {
                health.current -= shot.damage;
                // Apply CC if any
                if let (Some(cc_type), Some(mut active_buffs)) = (shot.cc.clone(), buffs) {
                    active_buffs.0.push(sg_core::BuffData {
                        buff_type: cc_type,
                        duration: shot.cc_duration,
                        remaining: shot.cc_duration,
                        source: None,
                    });
                }
                hit = true;
                break;
            }
        }
        if hit { commands.entity(shot_entity).despawn(); }
    }
}

fn process_aoe_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut aoes: Query<(Entity, &mut AoeZone)>,
    mut targets: Query<(Entity, &Transform, &TeamMember, &mut Health, Option<&mut ActiveBuffs>), Without<AoeZone>>,
) {
    let dt = time.delta_secs();
    for (aoe_entity, mut aoe) in &mut aoes {
        aoe.elapsed += dt;
        if !aoe.has_hit {
            aoe.has_hit = true;
            for (_te, target_tf, target_team, mut health, buffs) in &mut targets {
                if target_team.0 == aoe.team || health.current <= 0.0 { continue; }
                if aoe.center.distance(target_tf.translation) < aoe.radius {
                    health.current -= aoe.damage;
                    // Apply CC if any
                    if let (Some(cc_type), Some(mut active_buffs)) = (aoe.cc.clone(), buffs) {
                        active_buffs.0.push(sg_core::BuffData {
                            buff_type: cc_type,
                            duration: aoe.cc_duration,
                            remaining: aoe.cc_duration,
                            source: None,
                        });
                    }
                }
            }
        }
        if aoe.elapsed >= aoe.duration { commands.entity(aoe_entity).despawn(); }
    }
}

fn tick_shields(
    mut commands: Commands,
    time: Res<Time>,
    mut shields: Query<(Entity, &mut Shield)>,
) {
    let dt = time.delta_secs();
    for (entity, mut shield) in &mut shields {
        shield.remaining -= dt;
        if shield.remaining <= 0.0 {
            commands.entity(entity).remove::<Shield>();
        }
    }
}

/// Tick Tibbers lifetime and despawn when expired
fn tick_tibbers(
    mut commands: Commands,
    time: Res<Time>,
    mut tibbers: Query<(Entity, &mut TibbersPet, &Health)>,
) {
    for (entity, mut pet, health) in &mut tibbers {
        pet.lifetime -= time.delta_secs();
        if pet.lifetime <= 0.0 || health.current <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
