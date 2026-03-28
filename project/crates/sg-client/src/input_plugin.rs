use bevy::prelude::*;
use bevy::scene::SceneRoot;
use sg_core::components::*;
use sg_core::constants::STARTING_GOLD;
use sg_core::types::*;
use sg_core::GameSet;
use sg_gameplay::champions::{ChampionClass, ChampionId, get_champion, get_champion_by_id};
use crate::ability_plugin::{AbilityCooldowns, ChampionKit, ChampionIdentity};
use crate::shop_plugin::PlayerInventory;
use crate::menu::AppState;
use crate::menu::data::PlayerProfile;
use sg_ai::champion_ai::BotController;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_champion_on_enter)
            .add_systems(Update, (
                click_to_move.in_set(GameSet::Input),
                recall_system,
                mana_regen,
            ).run_if(in_state(AppState::InGame)));
    }
}

/// Spawn the champion when entering InGame state (from Loading screen)
fn spawn_champion_on_enter(
    mut commands: Commands,
    profile: Res<PlayerProfile>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_player: Query<Entity, With<PlayerControlled>>,
) {
    // Don't spawn if already exists
    if !existing_player.is_empty() { return; }

    let class = profile.preferred_champion.unwrap_or(ChampionClass::Mage);
    let champ_id = profile.selected_champion_id.unwrap_or(match class {
        ChampionClass::Mage => ChampionId::Annie,
        ChampionClass::Fighter => ChampionId::Garen,
        ChampionClass::Tank => ChampionId::Poppy,
    });
    let def = get_champion_by_id(champ_id);
    println!("Spawning champion: {} — {}", def.name, def.title);

    // Map champion to available 3D model (Annie for mages, Garen for fighters/tanks)
    let model_path = match def.class {
        ChampionClass::Mage => Some("models/champions/annie.glb#Scene0"),
        ChampionClass::Fighter | ChampionClass::Tank => Some("models/champions/garen.glb#Scene0"),
    };

    let champion_mesh = meshes.add(Capsule3d::new(25.0, 60.0));
    let champion_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(def.color[0], def.color[1], def.color[2]),
        emissive: bevy::color::LinearRgba::rgb(def.emissive[0], def.emissive[1], def.emissive[2]),
        ..default()
    });

    let entity = commands.spawn((
        Mesh3d(champion_mesh),
        MeshMaterial3d(champion_mat),
        Transform::from_xyz(1059.62, 55.0, 7297.66),
        Champion { name: def.name.to_string(), level: 1, xp: 0.0, team: Team::Blue },
        TeamMember(Team::Blue),
        Health { current: def.hp, max: def.hp, regen: 1.5 },
        CombatStats {
            attack_damage: def.ad, ability_power: def.ap,
            armor: def.armor, magic_resist: def.mr,
            attack_speed: def.attack_speed, move_speed: def.move_speed,
            crit_chance: 0.0, cdr: 0.0, armor_pen_flat: 0.0, armor_pen_pct: 0.0,
            magic_pen_flat: 0.0, magic_pen_pct: 0.0, life_steal: 0.0, spell_vamp: 0.0,
        },
        AutoAttackRange(def.attack_range),
        VisionRange(1200.0),
        Mana { current: def.mana, max: def.mana, regen: 2.0 },
    )).id();
    commands.entity(entity).insert((
        PlayerControlled,
        Gold(STARTING_GOLD),
        AbilityCooldowns::default(),
        ChampionKit(class),
        ChampionIdentity(champ_id),
        PlayerInventory::default(),
        BaseStats {
            attack_damage: def.ad, ability_power: def.ap,
            armor: def.armor, magic_resist: def.mr,
            attack_speed: def.attack_speed, move_speed: def.move_speed,
            ad_per_level: def.ad_per_level, armor_per_level: def.armor_per_level,
            mr_per_level: def.mr_per_level, hp_per_level: def.hp_per_level,
            mana_per_level: def.mana_per_level, base_hp: def.hp, base_mana: def.mana,
        },
        KillStreak::default(),
        GameStats::default(),
    ));

    if let Some(path) = model_path {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                SceneRoot(asset_server.load(path)),
                Transform::from_translation(Vec3::new(0.0, -55.0, 0.0))
                    .with_scale(Vec3::splat(1.0)),
            ));
        });
    }

    // Spawn 2 allied bots + 3 enemy bots with real champions
    let bot_configs: [(Team, ChampionId, Lane, Vec3); 5] = [
        // 2 Blue allies
        (Team::Blue, ChampionId::Garen, Lane::Top, Vec3::new(1059.0, 55.0, 7200.0)),
        (Team::Blue, ChampionId::Poppy, Lane::Bottom, Vec3::new(1059.0, 55.0, 7400.0)),
        // 3 Red enemies
        (Team::Red, ChampionId::Lux, Lane::Top, Vec3::new(14321.0, 55.0, 7135.0)),
        (Team::Red, ChampionId::Darius, Lane::Bottom, Vec3::new(14321.0, 55.0, 7335.0)),
        (Team::Red, ChampionId::Mordekaiser, Lane::Top, Vec3::new(14321.0, 55.0, 7435.0)),
    ];

    for (bot_team, bot_champ_id, bot_lane, bot_pos) in bot_configs {
        let bot_def = get_champion_by_id(bot_champ_id);
        let bot_mesh = meshes.add(Capsule3d::new(25.0, 60.0));
        let bot_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(bot_def.color[0], bot_def.color[1], bot_def.color[2]),
            emissive: bevy::color::LinearRgba::rgb(bot_def.emissive[0], bot_def.emissive[1], bot_def.emissive[2]),
            ..default()
        });

        // Pick 3D model based on class
        let bot_model = match bot_def.class {
            ChampionClass::Mage => Some("models/champions/annie.glb#Scene0"),
            ChampionClass::Fighter | ChampionClass::Tank => Some("models/champions/garen.glb#Scene0"),
        };

        let bot_entity = commands.spawn((
            Mesh3d(bot_mesh),
            MeshMaterial3d(bot_mat),
            Transform::from_translation(bot_pos),
            Champion { name: bot_def.name.to_string(), level: 1, xp: 0.0, team: bot_team },
            TeamMember(bot_team),
            Health { current: bot_def.hp, max: bot_def.hp, regen: 1.5 },
            CombatStats {
                attack_damage: bot_def.ad, ability_power: bot_def.ap,
                armor: bot_def.armor, magic_resist: bot_def.mr,
                attack_speed: bot_def.attack_speed, move_speed: bot_def.move_speed,
                crit_chance: 0.0, cdr: 0.0, armor_pen_flat: 0.0, armor_pen_pct: 0.0,
                magic_pen_flat: 0.0, magic_pen_pct: 0.0, life_steal: 0.0, spell_vamp: 0.0,
            },
            AutoAttackRange(bot_def.attack_range),
            VisionRange(1200.0),
            Mana { current: bot_def.mana, max: bot_def.mana, regen: 2.0 },
        )).id();
        commands.entity(bot_entity).insert((
            Gold(STARTING_GOLD),
            BotController::new(bot_lane),
            ChampionKit(bot_def.class),
            ChampionIdentity(bot_champ_id),
            AbilityCooldowns::default(),
            PlayerInventory::default(),
            AttackCooldown(0.0),
            BaseStats {
                attack_damage: bot_def.ad, ability_power: bot_def.ap,
                armor: bot_def.armor, magic_resist: bot_def.mr,
                attack_speed: bot_def.attack_speed, move_speed: bot_def.move_speed,
                ad_per_level: bot_def.ad_per_level, armor_per_level: bot_def.armor_per_level,
                mr_per_level: bot_def.mr_per_level, hp_per_level: bot_def.hp_per_level,
                mana_per_level: bot_def.mana_per_level, base_hp: bot_def.hp, base_mana: bot_def.mana,
            },
            KillStreak::default(),
            GameStats::default(),
        ));

        // Attach 3D model
        if let Some(path) = bot_model {
            commands.entity(bot_entity).with_children(|parent| {
                parent.spawn((
                    SceneRoot(asset_server.load(path)),
                    Transform::from_translation(Vec3::new(0.0, -55.0, 0.0)),
                ));
            });
        }
    }
}

/// Recall system: press B to channel recall (8s), teleport to fountain
#[derive(Component)]
pub struct Recalling {
    pub timer: f32,
}

fn recall_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map: Res<crate::map_plugin::MapData>,
    mut query: Query<(Entity, &mut Transform, &TeamMember, &mut Health, Option<&mut Recalling>), (With<PlayerControlled>, Without<Dead>)>,
) {
    for (entity, mut tf, team, mut health, recalling) in &mut query {
        if let Some(mut recall) = recalling {
            recall.timer -= time.delta_secs();
            if recall.timer <= 0.0 {
                let spawn = if team.0 == Team::Blue { map.0.blue_fountain } else { map.0.red_fountain };
                tf.translation.x = spawn.x;
                tf.translation.z = spawn.y;
                health.current = health.max;
                commands.entity(entity).remove::<Recalling>();
                println!("Recall complete!");
            }
        } else if keys.just_pressed(KeyCode::KeyB) {
            commands.entity(entity).insert(Recalling { timer: 8.0 });
            commands.entity(entity).remove::<MoveTarget>().remove::<AttackTarget>();
            println!("Recalling... (8s)");
        }
    }
}

fn click_to_move(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut player_q: Query<(Entity, Option<&Recalling>), (With<PlayerControlled>, Without<Dead>)>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Right) { return; }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_tf)) = camera_q.single() else { return };
    let Ok((player_entity, recalling)) = player_q.single() else { return };

    if recalling.is_some() {
        commands.entity(player_entity).remove::<Recalling>();
        println!("Recall cancelled");
    }

    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else { return };
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else { return };
    let world_pos = ray.get_point(distance);

    commands.entity(player_entity).remove::<AttackTarget>().insert(MoveTarget { position: Vec2::new(world_pos.x, world_pos.z) });
}

fn mana_regen(
    time: Res<Time>,
    mut query: Query<&mut Mana, (With<PlayerControlled>, Without<Dead>)>,
) {
    for mut mana in &mut query {
        mana.current = (mana.current + mana.regen * time.delta_secs()).min(mana.max);
    }
}
