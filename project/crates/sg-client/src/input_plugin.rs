use bevy::prelude::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::scene::SceneRoot;
use sg_core::components::*;
use sg_core::constants::STARTING_GOLD;
use sg_core::types::*;
use sg_core::GameSet;
use sg_gameplay::champions::{ChampionClass, ChampionId, get_champion, get_champion_by_id};
use crate::ability_plugin::{AbilityCooldowns, ChampionKit, ChampionIdentity};
use crate::movement_plugin::ChampionAnimState;
use crate::shop_plugin::PlayerInventory;
use crate::menu::AppState;
use crate::menu::data::PlayerProfile;
use sg_ai::champion_ai::BotController;

/// In-game chat log
#[derive(Resource, Default)]
pub struct ChatLog {
    pub messages: Vec<(String, String)>, // (sender, text)
    pub input_active: bool,
    pub input_text: String,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChatLog::default())
            .add_systems(OnEnter(AppState::InGame), spawn_champion_on_enter)
            .add_systems(Update, (
                click_to_move.in_set(GameSet::Input),
                minimap_click_to_move.in_set(GameSet::Input),
                ping_system.in_set(GameSet::Input),
                recall_system,
                mana_regen,
                tick_pings,
                draw_pings,
            ).run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                chat_input,
                draw_chat,
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

    // Map each champion to their own 3D model
    let model_path = Some(match champ_id {
        ChampionId::Annie => "models/champions/annie_animated.glb#Scene0",
        ChampionId::Garen => "models/champions/garen_animated.glb#Scene0",
        ChampionId::Ashe => "models/champions/ashe_animated.glb#Scene0",
        ChampionId::Darius => "models/champions/darius_animated.glb#Scene0",
        ChampionId::Lux => "models/champions/lux_animated.glb#Scene0",
        ChampionId::Thresh => "models/champions/thresh_animated.glb#Scene0",
        ChampionId::Jinx => "models/champions/jinx_animated.glb#Scene0",
        ChampionId::Yasuo => "models/champions/yasuo_animated.glb#Scene0",
        ChampionId::MasterYi => "models/champions/masteryi_animated.glb#Scene0",
        ChampionId::Jax => "models/champions/jax_animated.glb#Scene0",
        ChampionId::Teemo => "models/champions/teemo_animated.glb#Scene0",
        ChampionId::Singed => "models/champions/singed_animated.glb#Scene0",
        ChampionId::Tryndamere => "models/champions/tryndamere_animated.glb#Scene0",
        ChampionId::Mordekaiser => "models/champions/mordekaiser_animated.glb#Scene0",
        ChampionId::Poppy => "models/champions/poppy_animated.glb#Scene0",
    });

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
        ChampionAnimState::default(),
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
        // 3 Red enemies — spawn in walkable area of Red base
        (Team::Red, ChampionId::Lux, Lane::Top, Vec3::new(14321.0, 55.0, 7135.0)),
        (Team::Red, ChampionId::Darius, Lane::Bottom, Vec3::new(14321.0, 55.0, 7135.0)),
        (Team::Red, ChampionId::Mordekaiser, Lane::Top, Vec3::new(14321.0, 55.0, 7235.0)),
    ];

    for (bot_team, bot_champ_id, bot_lane, bot_pos) in bot_configs {
        let bot_def = get_champion_by_id(bot_champ_id);
        let bot_mesh = meshes.add(Capsule3d::new(25.0, 60.0));
        let bot_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(bot_def.color[0], bot_def.color[1], bot_def.color[2]),
            emissive: bevy::color::LinearRgba::rgb(bot_def.emissive[0], bot_def.emissive[1], bot_def.emissive[2]),
            ..default()
        });

        // Each bot gets their own champion model
        let bot_model = Some(match bot_champ_id {
            ChampionId::Annie => "models/champions/annie_animated.glb#Scene0",
            ChampionId::Garen => "models/champions/garen_animated.glb#Scene0",
            ChampionId::Ashe => "models/champions/ashe_animated.glb#Scene0",
            ChampionId::Darius => "models/champions/darius_animated.glb#Scene0",
            ChampionId::Lux => "models/champions/lux_animated.glb#Scene0",
            ChampionId::Thresh => "models/champions/thresh_animated.glb#Scene0",
            ChampionId::Jinx => "models/champions/jinx_animated.glb#Scene0",
            ChampionId::Yasuo => "models/champions/yasuo_animated.glb#Scene0",
            ChampionId::MasterYi => "models/champions/masteryi_animated.glb#Scene0",
            ChampionId::Jax => "models/champions/jax_animated.glb#Scene0",
            ChampionId::Teemo => "models/champions/teemo_animated.glb#Scene0",
            ChampionId::Singed => "models/champions/singed_animated.glb#Scene0",
            ChampionId::Tryndamere => "models/champions/tryndamere_animated.glb#Scene0",
            ChampionId::Mordekaiser => "models/champions/mordekaiser_animated.glb#Scene0",
            ChampionId::Poppy => "models/champions/poppy_animated.glb#Scene0",
        });

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
            ChampionAnimState::default(),
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
            }
        } else if keys.just_pressed(KeyCode::KeyB) {
            commands.entity(entity).insert(Recalling { timer: 8.0 });
            commands.entity(entity).remove::<MoveTarget>().remove::<AttackTarget>();
        }
    }
}

/// Check if cursor is in the minimap area (bottom-right corner)
fn is_cursor_on_minimap(cursor: Vec2, w: f32, h: f32) -> bool {
    let margin_x = w * 0.005 + 3.0;
    let margin_y = h * 0.005 + 3.0;
    let mm_size = 270.0;
    let mm_left = w - margin_x - mm_size;
    let mm_top = h - margin_y - mm_size;
    cursor.x >= mm_left && cursor.y >= mm_top
}

/// Convert minimap pixel coordinates to world position
fn minimap_to_world(cursor: Vec2, w: f32, h: f32) -> Vec2 {
    let margin_x = w * 0.005 + 3.0;
    let margin_y = h * 0.005 + 3.0;
    let mm_size = 270.0;
    let mm_left = w - margin_x - mm_size;
    let mm_top = h - margin_y - mm_size;
    let map_size = 15398.0;
    let rel_x = (cursor.x - mm_left) / mm_size;
    let rel_y = (cursor.y - mm_top) / mm_size;
    Vec2::new(rel_x * map_size, rel_y * map_size)
}

fn click_to_move(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut player_q: Query<(Entity, Option<&Recalling>), (With<PlayerControlled>, Without<Dead>)>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Right) { return; }
    // Alt+click = ping, don't move
    if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) { return; }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    // Skip if clicking on minimap (handled by minimap_click_to_move)
    if is_cursor_on_minimap(cursor_pos, window.width(), window.height()) { return; }

    let Ok((camera, cam_tf)) = camera_q.single() else { return };
    let Ok((player_entity, recalling)) = player_q.single() else { return };

    if recalling.is_some() {
        commands.entity(player_entity).remove::<Recalling>();
    }

    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else { return };
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else { return };
    let world_pos = ray.get_point(distance);

    commands.entity(player_entity).remove::<AttackTarget>().insert(MoveTarget { position: Vec2::new(world_pos.x, world_pos.z) });
}

/// Click on minimap to move champion
fn minimap_click_to_move(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut player_q: Query<Entity, (With<PlayerControlled>, Without<Dead>)>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Right) { return; }
    if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) { return; }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    if !is_cursor_on_minimap(cursor_pos, window.width(), window.height()) { return; }

    let world_pos = minimap_to_world(cursor_pos, window.width(), window.height());
    let Ok(entity) = player_q.single() else { return };
    commands.entity(entity).remove::<AttackTarget>().insert(MoveTarget { position: world_pos });
}

/// Ping system: Alt+right-click spawns a visible ping marker
#[derive(Component)]
pub struct PingMarker {
    pub lifetime: f32,
}

fn ping_system(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Right) { return; }
    if !keys.pressed(KeyCode::AltLeft) && !keys.pressed(KeyCode::AltRight) { return; }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_tf)) = camera_q.single() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_tf, cursor_pos) else { return };
    let Some(dist) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else { return };
    let world_pos = ray.get_point(dist);

    commands.spawn((
        Transform::from_translation(world_pos + Vec3::Y * 5.0),
        PingMarker { lifetime: 3.0 },
    ));
}

fn tick_pings(
    mut commands: Commands,
    time: Res<Time>,
    mut pings: Query<(Entity, &mut PingMarker)>,
) {
    for (entity, mut ping) in &mut pings {
        ping.lifetime -= time.delta_secs();
        if ping.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_pings(
    mut gizmos: Gizmos,
    pings: Query<(&Transform, &PingMarker)>,
) {
    for (tf, ping) in &pings {
        let alpha = (ping.lifetime / 3.0).clamp(0.0, 1.0);
        let pulse = 1.0 + (ping.lifetime * 4.0).sin() * 0.2;
        let size = 80.0 * pulse;
        gizmos.circle(
            Isometry3d::from_translation(tf.translation),
            size,
            Color::srgba(1.0, 0.9, 0.0, alpha),
        );
        gizmos.sphere(
            Isometry3d::from_translation(tf.translation + Vec3::Y * 20.0),
            15.0,
            Color::srgba(1.0, 0.9, 0.0, alpha * 0.8),
        );
    }
}

fn mana_regen(
    time: Res<Time>,
    mut query: Query<&mut Mana, (With<PlayerControlled>, Without<Dead>)>,
) {
    for mut mana in &mut query {
        mana.current = (mana.current + mana.regen * time.delta_secs()).min(mana.max);
    }
}

/// Chat input: Enter to toggle, type message, Enter to send
fn chat_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut chat: ResMut<ChatLog>,
    net: Option<Res<crate::net_plugin::NetClient>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        if chat.input_active {
            // Send message
            if !chat.input_text.is_empty() {
                let text = chat.input_text.clone();
                chat.messages.push(("You".to_string(), text.clone()));

                // Send to server if connected
                if let Some(ref net) = net {
                    if net.connected {
                        crate::net_plugin::send_chat(&net, &text);
                    }
                }

                // Keep max 20 messages
                if chat.messages.len() > 20 {
                    chat.messages.remove(0);
                }
                chat.input_text.clear();
            }
            chat.input_active = false;
        } else {
            chat.input_active = true;
            chat.input_text.clear();
        }
        return;
    }

    if !chat.input_active { return; }

    // Escape to cancel
    if keys.just_pressed(KeyCode::Escape) {
        chat.input_active = false;
        chat.input_text.clear();
        return;
    }

    // Backspace
    if keys.just_pressed(KeyCode::Backspace) {
        chat.input_text.pop();
        return;
    }

    // Space
    if keys.just_pressed(KeyCode::Space) {
        chat.input_text.push(' ');
        return;
    }

    // Capture letter keys via just_pressed
    let key_map: &[(KeyCode, char)] = &[
        (KeyCode::KeyA, 'a'), (KeyCode::KeyB, 'b'), (KeyCode::KeyC, 'c'),
        (KeyCode::KeyD, 'd'), (KeyCode::KeyE, 'e'), (KeyCode::KeyF, 'f'),
        (KeyCode::KeyG, 'g'), (KeyCode::KeyH, 'h'), (KeyCode::KeyI, 'i'),
        (KeyCode::KeyJ, 'j'), (KeyCode::KeyK, 'k'), (KeyCode::KeyL, 'l'),
        (KeyCode::KeyM, 'm'), (KeyCode::KeyN, 'n'), (KeyCode::KeyO, 'o'),
        (KeyCode::KeyP, 'p'), (KeyCode::KeyQ, 'q'), (KeyCode::KeyR, 'r'),
        (KeyCode::KeyS, 's'), (KeyCode::KeyT, 't'), (KeyCode::KeyU, 'u'),
        (KeyCode::KeyV, 'v'), (KeyCode::KeyW, 'w'), (KeyCode::KeyX, 'x'),
        (KeyCode::KeyY, 'y'), (KeyCode::KeyZ, 'z'),
        (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'), (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'), (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ];
    for &(code, c) in key_map {
        if keys.just_pressed(code) && chat.input_text.len() < 100 {
            chat.input_text.push(c);
        }
    }
}

/// Draw chat messages in bottom-left using gizmos
fn draw_chat(
    mut gizmos: Gizmos,
    chat: Res<ChatLog>,
    player_q: Query<&Transform, With<PlayerControlled>>,
) {
    let Ok(player_tf) = player_q.single() else { return };
    let base = player_tf.translation + Vec3::new(-800.0, 300.0, -400.0);

    // Show last 5 messages as colored spheres
    let start = chat.messages.len().saturating_sub(5);
    for (i, (_sender, _text)) in chat.messages[start..].iter().enumerate() {
        let pos = base + Vec3::new(0.0, i as f32 * 30.0, 0.0);
        gizmos.sphere(
            Isometry3d::from_translation(pos),
            8.0,
            Color::srgba(0.3, 0.7, 1.0, 0.7),
        );
        // Message indicator dot (length proportional to text length)
        let bar_len = _text.len().min(50) as f32 * 3.0;
        gizmos.line(
            pos + Vec3::new(15.0, 0.0, 0.0),
            pos + Vec3::new(15.0 + bar_len, 0.0, 0.0),
            Color::srgba(0.8, 0.8, 0.8, 0.5),
        );
    }

    // Show input indicator when typing
    if chat.input_active {
        let input_pos = base + Vec3::new(0.0, -30.0, 0.0);
        let pulse = 1.0;
        gizmos.sphere(
            Isometry3d::from_translation(input_pos),
            10.0 * pulse,
            Color::srgba(1.0, 1.0, 0.3, 0.8),
        );
        // Input text length bar
        let bar_len = chat.input_text.len().min(50) as f32 * 3.0;
        gizmos.line(
            input_pos + Vec3::new(15.0, 0.0, 0.0),
            input_pos + Vec3::new(15.0 + bar_len, 0.0, 0.0),
            Color::srgba(1.0, 1.0, 0.5, 0.7),
        );
    }
}
