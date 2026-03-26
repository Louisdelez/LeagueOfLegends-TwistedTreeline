use bevy::prelude::*;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::net::SocketAddr;
use sg_protocol::*;
use sg_map::layout::MapLayout;
use sg_core::constants::*;
use sg_core::types::*;
use sg_gameplay::combat::calculate_damage;
use sg_gameplay::leveling::{death_timer, level_from_xp, kill_xp, shared_xp};
use sg_gameplay::economy::{kill_gold, minion_gold};
use sg_gameplay::champions::{ChampionClass, get_champion};

// ─── Resources ───

#[derive(Resource)]
struct NetServer {
    socket: UdpSocket,
    clients: HashMap<SocketAddr, u8>,
    next_id: u8,
}

#[derive(Resource)]
struct ServerGameState {
    tick: u32,
    game_time: f32,
    players: HashMap<u8, ServerPlayer>,
    minions: Vec<ServerMinion>,
    turrets: Vec<ServerTurret>,
    jungle_camps: Vec<ServerJungleCamp>,
    vilemaw: Option<ServerVilemaw>,
    altars: [ServerAltar; 2],
    next_minion_id: u16,
    next_wave_time: f32,
    wave_count: u32,
    started: bool,
    first_blood: bool,
    event_queue: Vec<GameEvent>,
    layout: MapLayout,
    vilemaw_next_spawn: f32,
}

// ─── Data structs ───

struct ServerPlayer {
    team: u8,
    champion_class: u8,
    position: [f32; 2],
    health: f32,
    max_health: f32,
    mana: f32,
    max_mana: f32,
    gold: f32,
    level: u8,
    xp: f32,
    alive: bool,
    move_target: Option<[f32; 2]>,
    respawn_at: f32,
    base_ad: f32,
    base_ap: f32,
    base_armor: f32,
    base_mr: f32,
    base_attack_speed: f32,
    base_move_speed: f32,
    base_attack_range: f32,
    hp_per_level: f32,
    mana_per_level: f32,
    ad_per_level: f32,
    armor_per_level: f32,
    mr_per_level: f32,
    bonus_ad: f32,
    bonus_ap: f32,
    bonus_armor: f32,
    bonus_mr: f32,
    bonus_hp: f32,
    bonus_attack_speed: f32,
    bonus_move_speed: f32,
    attack_cooldown: f32,
    ability_cooldowns: [f32; 4],
    items: Vec<u32>,
    buffs: Vec<ServerBuff>,
    kills: u32,
    deaths: u32,
    hp_regen: f32,
    mana_regen: f32,
}

impl ServerPlayer {
    fn total_ad(&self) -> f32 { self.base_ad + self.ad_per_level * self.level.saturating_sub(1) as f32 + self.bonus_ad }
    fn total_ap(&self) -> f32 { self.base_ap + self.bonus_ap }
    fn total_armor(&self) -> f32 { self.base_armor + self.armor_per_level * self.level.saturating_sub(1) as f32 + self.bonus_armor }
    fn total_mr(&self) -> f32 { self.base_mr + self.mr_per_level * self.level.saturating_sub(1) as f32 + self.bonus_mr }
    fn total_attack_speed(&self) -> f32 { self.base_attack_speed + self.bonus_attack_speed }
    fn total_move_speed(&self) -> f32 {
        let base = self.base_move_speed + self.bonus_move_speed;
        let mut speed = base;
        for buff in &self.buffs {
            if buff.buff_type == 1 { speed += base * ALTAR_1_MOVE_SPEED_BONUS; }
            if buff.buff_type == 3 { speed += 80.0; }
        }
        speed
    }
    fn combat_stats(&self) -> sg_core::components::CombatStats {
        sg_core::components::CombatStats {
            attack_damage: self.total_ad(),
            ability_power: self.total_ap(),
            armor: self.total_armor(),
            magic_resist: self.total_mr(),
            attack_speed: self.total_attack_speed(),
            move_speed: self.total_move_speed(),
            crit_chance: 0.0, cdr: 0.0,
            armor_pen_flat: 0.0, armor_pen_pct: 0.0,
            magic_pen_flat: 0.0, magic_pen_pct: 0.0,
            life_steal: 0.0, spell_vamp: 0.0,
        }
    }
}

struct ServerMinion {
    id: u16,
    team: u8,
    minion_type: MinionType,
    position: [f32; 2],
    health: f32,
    max_health: f32,
    ad: f32,
    move_speed: f32,
    waypoints: Vec<[f32; 2]>,
    waypoint_idx: usize,
    attack_cooldown: f32,
}

struct ServerTurret {
    id: u8,
    team: u8,
    position: [f32; 2],
    health: f32,
    max_health: f32,
    ad: f32,
    armor: f32,
    attack_speed: f32,
    range: f32,
    attack_cooldown: f32,
    alive: bool,
}

struct ServerJungleCamp {
    id: u8,
    position: [f32; 2],
    health: f32,
    max_health: f32,
    ad: f32,
    alive: bool,
    respawn_at: f32,
    attack_cooldown: f32,
    aggro_target: Option<u8>,
    spawn_position: [f32; 2],
}

struct ServerVilemaw {
    position: [f32; 2],
    health: f32,
    max_health: f32,
    ad: f32,
    attack_cooldown: f32,
    aggro_target: Option<u8>,
}

struct ServerAltar {
    side: u8,
    owner: u8, // 0=none, 1=blue, 2=red
    capture_progress: f32,
    lockout: f32,
}

struct ServerBuff {
    buff_type: u8,
    remaining: f32,
}

// ─── Item DB ───

struct ItemDef { id: u32, cost: u32, sell_value: u32, ad: f32, ap: f32, hp: f32, armor: f32, mr: f32, attack_speed: f32, move_speed: f32 }

fn item_database() -> Vec<ItemDef> {
    vec![
        ItemDef { id: 1036, cost: 350, sell_value: 245, ad: 10.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0, move_speed: 0.0 },
        ItemDef { id: 1037, cost: 875, sell_value: 612, ad: 25.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0, move_speed: 0.0 },
        ItemDef { id: 1026, cost: 850, sell_value: 595, ad: 0.0, ap: 40.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0, move_speed: 0.0 },
        ItemDef { id: 1028, cost: 400, sell_value: 280, ad: 0.0, ap: 0.0, hp: 150.0, armor: 0.0, mr: 0.0, attack_speed: 0.0, move_speed: 0.0 },
        ItemDef { id: 1029, cost: 300, sell_value: 210, ad: 0.0, ap: 0.0, hp: 0.0, armor: 15.0, mr: 0.0, attack_speed: 0.0, move_speed: 0.0 },
        ItemDef { id: 1033, cost: 450, sell_value: 315, ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 25.0, attack_speed: 0.0, move_speed: 0.0 },
        ItemDef { id: 1001, cost: 300, sell_value: 210, ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0, move_speed: 25.0 },
        ItemDef { id: 1011, cost: 1000, sell_value: 700, ad: 0.0, ap: 0.0, hp: 380.0, armor: 0.0, mr: 0.0, attack_speed: 0.0, move_speed: 0.0 },
        ItemDef { id: 1031, cost: 800, sell_value: 560, ad: 0.0, ap: 0.0, hp: 0.0, armor: 40.0, mr: 0.0, attack_speed: 0.0, move_speed: 0.0 },
        ItemDef { id: 1027, cost: 350, sell_value: 245, ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0, move_speed: 0.0 },
    ]
}

fn find_item(id: u32) -> Option<ItemDef> { item_database().into_iter().find(|i| i.id == id) }

// ─── Main ───

fn main() {
    println!("Shadow Grove Server v0.3.0");

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", SERVER_PORT)).expect("Failed to bind");
    socket.set_nonblocking(true).unwrap();
    println!("Listening on port {}", SERVER_PORT);

    let layout = MapLayout::twisted_treeline();

    let turrets: Vec<ServerTurret> = layout.turrets.iter().enumerate().map(|(i, t)| {
        let hp = match t.structure_type {
            StructureType::OuterTurret => OUTER_TURRET_HP,
            StructureType::InnerTurret => INNER_TURRET_HP,
            StructureType::InhibitorTurret => INHIB_TURRET_HP,
            StructureType::NexusTurret => NEXUS_TURRET_HP,
            _ => OUTER_TURRET_HP,
        };
        let team = match t.team { Team::Blue => 0u8, Team::Red => 1, _ => 2 };
        ServerTurret { id: i as u8, team, position: [t.position.x, t.position.y], health: hp, max_health: hp, ad: 152.0, armor: 100.0, attack_speed: 0.83, range: 800.0, attack_cooldown: 0.0, alive: true }
    }).collect();

    let jungle_camps: Vec<ServerJungleCamp> = layout.jungle_camps.iter().enumerate().map(|(i, c)| {
        let (hp, ad) = match c.camp_type { CampType::Wolf => (1200.0, 35.0), CampType::Wraith => (900.0, 25.0), CampType::Golem => (1500.0, 40.0), CampType::Vilemaw => (VILEMAW_HP, VILEMAW_AD) };
        ServerJungleCamp { id: i as u8, position: [c.position.x, c.position.y], health: hp, max_health: hp, ad, alive: false, respawn_at: JUNGLE_FIRST_SPAWN, attack_cooldown: 0.0, aggro_target: None, spawn_position: [c.position.x, c.position.y] }
    }).collect();

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(NetServer { socket, clients: HashMap::new(), next_id: 0 })
        .insert_resource(ServerGameState {
            tick: 0, game_time: 0.0, players: HashMap::new(), minions: vec![], turrets, jungle_camps,
            vilemaw: None, altars: [ServerAltar { side: 0, owner: 0, capture_progress: 0.0, lockout: 0.0 }, ServerAltar { side: 1, owner: 0, capture_progress: 0.0, lockout: 0.0 }],
            next_minion_id: 1, next_wave_time: MINION_FIRST_SPAWN, wave_count: 0, started: false, first_blood: false,
            event_queue: vec![], layout, vilemaw_next_spawn: VILEMAW_FIRST_SPAWN,
        })
        .add_systems(Update, (receive_packets, update_game_state, send_snapshots).chain())
        .run();
}

// ─── Network ───

fn receive_packets(mut server: ResMut<NetServer>, mut state: ResMut<ServerGameState>) {
    let mut buf = [0u8; 4096];
    let s = &mut *state;  // destructure to allow split borrows
    while let Ok((len, addr)) = server.socket.recv_from(&mut buf) {
        if let Some(packet) = decode_packet::<ClientPacket>(&buf[..len]) {
            match packet {
                ClientPacket::Join { name, champion_class } => {
                    if server.clients.contains_key(&addr) { continue; }
                    let id = server.next_id;
                    server.next_id += 1;
                    server.clients.insert(addr, id);
                    let team = if id % 2 == 0 { 0u8 } else { 1u8 };
                    let spawn = if team == 0 { [s.layout.blue_spawn.x, s.layout.blue_spawn.y] } else { [s.layout.red_spawn.x, s.layout.red_spawn.y] };
                    let class = match champion_class { 0 => ChampionClass::Mage, 1 => ChampionClass::Fighter, _ => ChampionClass::Tank };
                    let def = get_champion(class);
                    s.players.insert(id, ServerPlayer {
                        team, champion_class, position: spawn, health: def.hp, max_health: def.hp,
                        mana: def.mana, max_mana: def.mana, gold: STARTING_GOLD, level: 1, xp: 0.0,
                        alive: true, move_target: None, respawn_at: 0.0,
                        base_ad: def.ad, base_ap: def.ap, base_armor: def.armor, base_mr: def.mr,
                        base_attack_speed: def.attack_speed, base_move_speed: def.move_speed, base_attack_range: def.attack_range,
                        hp_per_level: def.hp_per_level, mana_per_level: def.mana_per_level, ad_per_level: def.ad_per_level,
                        armor_per_level: def.armor_per_level, mr_per_level: def.mr_per_level,
                        bonus_ad: 0.0, bonus_ap: 0.0, bonus_armor: 0.0, bonus_mr: 0.0, bonus_hp: 0.0, bonus_attack_speed: 0.0, bonus_move_speed: 0.0,
                        attack_cooldown: 0.0, ability_cooldowns: [0.0; 4], items: vec![], buffs: vec![],
                        kills: 0, deaths: 0, hp_regen: 5.0, mana_regen: 3.0,
                    });
                    let _ = server.socket.send_to(&encode_packet(&ServerPacket::Welcome { player_id: id, team, spawn }), addr);
                    s.event_queue.push(GameEvent::PlayerJoined { id, name, team });
                    println!("Player {} joined (team {}, class {})", id, team, champion_class);
                    if server.clients.len() >= 2 && !s.started { s.started = true; s.event_queue.push(GameEvent::GameStart); println!("Game started!"); }
                }
                ClientPacket::Input(input) => {
                    if let Some(&id) = server.clients.get(&addr) {
                        if let Some(p) = s.players.get_mut(&id) {
                            if p.alive {
                                if input.move_target.is_some() { p.move_target = input.move_target; }
                            }
                        }
                        // Process ability separately to avoid borrow conflict
                        if let Some(ability_idx) = input.ability_cast {
                            if s.players.get(&id).is_some_and(|p| p.alive) {
                                process_ability(&mut s.players, &mut s.minions, &mut s.jungle_camps, &mut s.vilemaw, &mut s.event_queue, id, ability_idx, input.cursor_pos);
                            }
                        }
                    }
                }
                ClientPacket::Chat { text } => {
                    if let Some(&id) = server.clients.get(&addr) {
                        let data = encode_packet(&ServerPacket::Chat { player_id: id, text });
                        for &a in server.clients.keys() { let _ = server.socket.send_to(&data, a); }
                    }
                }
                ClientPacket::Surrender { .. } => {}
                ClientPacket::BuyItem { item_id } => {
                    if let Some(&id) = server.clients.get(&addr) {
                        buy_item(&mut s.players, &mut s.event_queue, &s.layout, id, item_id);
                    }
                }
                ClientPacket::SellItem { slot } => {
                    if let Some(&id) = server.clients.get(&addr) {
                        if let Some(player) = s.players.get_mut(&id) {
                            let slot = slot as usize;
                            if slot < player.items.len() {
                                let item_id = player.items.remove(slot);
                                if let Some(item) = find_item(item_id) {
                                    player.gold += item.sell_value as f32;
                                    player.bonus_ad -= item.ad; player.bonus_ap -= item.ap;
                                    player.bonus_armor -= item.armor; player.bonus_mr -= item.mr;
                                    player.bonus_attack_speed -= item.attack_speed; player.bonus_move_speed -= item.move_speed;
                                    if item.hp > 0.0 { player.bonus_hp -= item.hp; player.max_health -= item.hp; player.health = player.health.min(player.max_health); }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn process_ability(
    players: &mut HashMap<u8, ServerPlayer>,
    minions: &mut Vec<ServerMinion>,
    jungle_camps: &mut Vec<ServerJungleCamp>,
    vilemaw: &mut Option<ServerVilemaw>,
    events: &mut Vec<GameEvent>,
    caster_id: u8,
    ability_idx: u8,
    cursor_pos: [f32; 2],
) {
    let idx = ability_idx as usize;
    if idx > 3 { return; }

    // Read caster data
    let (class_idx, mana, cd, pos, team, ad, ap) = {
        let p = match players.get(&caster_id) { Some(p) => p, None => return };
        if !p.alive || p.ability_cooldowns[idx] > 0.0 { return; }
        (p.champion_class as usize, p.mana, p.ability_cooldowns[idx], p.position, p.team, p.total_ad(), p.total_ap())
    };

    let class = match class_idx { 0 => ChampionClass::Mage, 1 => ChampionClass::Fighter, _ => ChampionClass::Tank };
    let def = get_champion(class);

    let mana_costs = [[60.0, 80.0, 50.0, 100.0], [40.0, 50.0, 45.0, 80.0], [50.0, 60.0, 55.0, 90.0]];
    let mana_cost = mana_costs[class_idx.min(2)][idx];
    if mana < mana_cost { return; }

    let cooldown = match idx { 0 => def.q_cd[0], 1 => def.w_cd[0], 2 => def.e_cd[0], 3 => def.r_cd[0], _ => 10.0 };
    let base_dmg = match idx { 0 => def.q_dmg[0], 1 => def.w_dmg[0], 2 => def.e_dmg[0], 3 => def.r_dmg[0], _ => 0.0 };
    let ratio = match idx { 0 => def.q_ap_ratio, 1 => def.w_ap_ratio, 2 => def.e_ad_ratio, 3 => def.r_ap_ratio, _ => 0.0 };

    let damage = base_dmg + if ratio > 0.0 { if idx == 2 { ad * ratio } else { ap * ratio } } else { 0.0 };

    // Apply cost and cooldown
    let p = players.get_mut(&caster_id).unwrap();
    p.mana -= mana_cost;
    p.ability_cooldowns[idx] = cooldown;

    let aoe_radius = match idx { 0 => 150.0, 1 => 200.0, 2 => 150.0, 3 => 300.0, _ => 100.0 };

    // Dash abilities
    let is_dash = (class_idx == 0 && idx == 2) || (class_idx == 1 && idx == 0);
    if is_dash {
        let dx = cursor_pos[0] - pos[0];
        let dz = cursor_pos[1] - pos[1];
        let dist = (dx * dx + dz * dz).sqrt().max(1.0);
        let range = if class_idx == 0 { 400.0 } else { 300.0 };
        p.move_target = Some([pos[0] + dx / dist * range, pos[1] + dz / dist * range]);
    }

    // Shield abilities
    let is_shield = idx == 1 && (class_idx == 1 || class_idx == 2);
    if is_shield {
        let shield = if class_idx == 1 { 80.0 } else { 100.0 };
        p.health = (p.health + shield).min(p.max_health);
    }

    // Damage
    if damage > 0.0 && !is_dash && !is_shield {
        // Collect enemy player IDs in range
        let targets: Vec<u8> = players.iter()
            .filter(|(pid, p)| **pid != caster_id && p.team != team && p.alive)
            .filter(|(_, p)| {
                let dx = p.position[0] - cursor_pos[0];
                let dz = p.position[1] - cursor_pos[1];
                (dx * dx + dz * dz).sqrt() < aoe_radius
            })
            .map(|(pid, _)| *pid)
            .collect();

        for tid in targets {
            if let Some(target) = players.get_mut(&tid) {
                let res = if idx == 2 { target.total_armor() } else { target.total_mr() };
                target.health -= damage * 100.0 / (100.0 + res);
            }
        }

        // Damage minions
        for minion in minions.iter_mut() {
            if minion.health <= 0.0 || minion.team == team { continue; }
            let dx = minion.position[0] - cursor_pos[0];
            let dz = minion.position[1] - cursor_pos[1];
            if (dx * dx + dz * dz).sqrt() < aoe_radius { minion.health -= damage; }
        }

        // Damage jungle camps
        for camp in jungle_camps.iter_mut() {
            if !camp.alive || camp.health <= 0.0 { continue; }
            let dx = camp.position[0] - cursor_pos[0];
            let dz = camp.position[1] - cursor_pos[1];
            if (dx * dx + dz * dz).sqrt() < aoe_radius { camp.health -= damage; camp.aggro_target = Some(caster_id); }
        }

        // Damage vilemaw
        if let Some(vim) = vilemaw.as_mut() {
            let dx = vim.position[0] - cursor_pos[0];
            let dz = vim.position[1] - cursor_pos[1];
            if (dx * dx + dz * dz).sqrt() < aoe_radius { vim.health -= damage; vim.aggro_target = Some(caster_id); }
        }
    }

    events.push(GameEvent::AbilityCast { caster_id, ability: ability_idx, target_pos: cursor_pos });
}

fn buy_item(players: &mut HashMap<u8, ServerPlayer>, events: &mut Vec<GameEvent>, layout: &MapLayout, player_id: u8, item_id: u32) {
    let Some(item) = find_item(item_id) else { return };
    let Some(player) = players.get_mut(&player_id) else { return };
    if !player.alive || player.gold < item.cost as f32 || player.items.len() >= 6 { return; }
    let fountain = if player.team == 0 { [layout.blue_fountain.x, layout.blue_fountain.y] } else { [layout.red_fountain.x, layout.red_fountain.y] };
    let dx = player.position[0] - fountain[0]; let dz = player.position[1] - fountain[1];
    if (dx * dx + dz * dz).sqrt() > 800.0 { return; }
    player.gold -= item.cost as f32; player.items.push(item.id);
    player.bonus_ad += item.ad; player.bonus_ap += item.ap; player.bonus_armor += item.armor; player.bonus_mr += item.mr;
    player.bonus_attack_speed += item.attack_speed; player.bonus_move_speed += item.move_speed;
    if item.hp > 0.0 { player.bonus_hp += item.hp; player.max_health += item.hp; player.health += item.hp; }
    events.push(GameEvent::ItemPurchased { player_id, item_id });
}

// ─── Game tick ───

fn update_game_state(time: Res<Time>, mut state: ResMut<ServerGameState>) {
    if !state.started { return; }
    let dt = time.delta_secs();
    state.game_time += dt;
    state.tick += 1;
    let game_time = state.game_time;

    // ── Tick cooldowns & buffs ──
    for p in state.players.values_mut() {
        for cd in p.ability_cooldowns.iter_mut() { *cd = (*cd - dt).max(0.0); }
        p.attack_cooldown = (p.attack_cooldown - dt).max(0.0);
        p.buffs.retain_mut(|b| { b.remaining -= dt; b.remaining > 0.0 });
    }

    // ── Move players & regen ──
    let layout_blue_spawn = [state.layout.blue_spawn.x, state.layout.blue_spawn.y];
    let layout_red_spawn = [state.layout.red_spawn.x, state.layout.red_spawn.y];
    let layout_blue_fountain = [state.layout.blue_fountain.x, state.layout.blue_fountain.y];
    let layout_red_fountain = [state.layout.red_fountain.x, state.layout.red_fountain.y];

    for p in state.players.values_mut() {
        if !p.alive {
            if game_time >= p.respawn_at && p.respawn_at > 0.0 {
                p.alive = true; p.health = p.max_health; p.mana = p.max_mana;
                p.position = if p.team == 0 { layout_blue_spawn } else { layout_red_spawn };
            }
            continue;
        }
        if game_time > 90.0 { p.gold += AMBIENT_GOLD_PER_TICK * dt / AMBIENT_GOLD_INTERVAL; }
        p.health = (p.health + p.hp_regen * dt).min(p.max_health);
        p.mana = (p.mana + p.mana_regen * dt).min(p.max_mana);
        let fountain = if p.team == 0 { layout_blue_fountain } else { layout_red_fountain };
        let fdx = p.position[0] - fountain[0]; let fdz = p.position[1] - fountain[1];
        if (fdx * fdx + fdz * fdz).sqrt() < 500.0 {
            p.health = (p.health + p.max_health * 0.10 * dt).min(p.max_health);
            p.mana = (p.mana + p.max_mana * 0.10 * dt).min(p.max_mana);
        }
        let ms = p.total_move_speed();
        if let Some(target) = p.move_target {
            let dx = target[0] - p.position[0]; let dz = target[1] - p.position[1];
            let dist = (dx * dx + dz * dz).sqrt();
            if dist < 5.0 { p.move_target = None; }
            else { let step = ms * dt; if step >= dist { p.position = target; p.move_target = None; } else { p.position[0] += dx / dist * step; p.position[1] += dz / dist * step; } }
        }
    }

    // ── Champion auto-attacks (collect then apply) ──
    let mut player_auto_damage: Vec<(u8, u8, f32)> = vec![]; // (attacker, victim, dmg)
    {
        let pids: Vec<u8> = state.players.keys().copied().collect();
        for &aid in &pids {
            let attacker = &state.players[&aid];
            if !attacker.alive || attacker.attack_cooldown > 0.0 || attacker.move_target.is_some() { continue; }
            let my_pos = attacker.position; let my_team = attacker.team;
            let my_range = attacker.base_attack_range; let my_ad = attacker.total_ad();

            let mut target: Option<(u8, f32)> = None;
            for &eid in &pids {
                if eid == aid { continue; }
                let e = &state.players[&eid];
                if !e.alive || e.team == my_team { continue; }
                let dx = e.position[0] - my_pos[0]; let dz = e.position[1] - my_pos[1];
                let dist = (dx * dx + dz * dz).sqrt();
                if dist <= my_range && target.map_or(true, |(_, d)| dist < d) { target = Some((eid, dist)); }
            }
            if let Some((tid, _)) = target {
                let as_stats = attacker.combat_stats();
                let td_stats = state.players[&tid].combat_stats();
                let dmg = calculate_damage(my_ad, DamageType::Physical, &as_stats, &td_stats);
                player_auto_damage.push((aid, tid, dmg));
            }
        }
    }
    for (aid, tid, dmg) in &player_auto_damage {
        state.players.get_mut(aid).unwrap().attack_cooldown = 1.0 / state.players[aid].total_attack_speed();
        state.players.get_mut(tid).unwrap().health -= dmg;
    }

    // ── Handle player deaths ──
    let pids: Vec<u8> = state.players.keys().copied().collect();
    let mut kill_events: Vec<GameEvent> = vec![];
    for &id in &pids {
        let p = &state.players[&id];
        if p.alive && p.health <= 0.0 {
            let victim_level = p.level;
            let victim_team = p.team;
            let p = state.players.get_mut(&id).unwrap();
            p.alive = false; p.health = 0.0; p.deaths += 1;
            p.respawn_at = game_time + death_timer(victim_level, game_time);

            // Find attacker from auto damage this tick
            let killer = player_auto_damage.iter().find(|(_, tid, _)| *tid == id).map(|(aid, _, _)| *aid);
            if let Some(kid) = killer {
                let gold_reward = if !state.first_blood { state.first_blood = true; BASE_KILL_GOLD + FIRST_BLOOD_BONUS } else { let (g, _) = kill_gold(0, 0); g };
                let killer = state.players.get_mut(&kid).unwrap();
                killer.kills += 1; killer.gold += gold_reward;
                killer.xp += kill_xp(victim_level, killer.level);
                kill_events.push(GameEvent::Kill { killer: kid, victim: id, gold: gold_reward });
            }
        }
    }
    state.event_queue.extend(kill_events);

    // ── Level up ──
    let pids: Vec<u8> = state.players.keys().copied().collect();
    for &id in &pids {
        let new_level = level_from_xp(state.players[&id].xp);
        let old_level = state.players[&id].level;
        if new_level > old_level && new_level <= 18 {
            let p = state.players.get_mut(&id).unwrap();
            let hp_gain = p.hp_per_level * (new_level - old_level) as f32;
            let mana_gain = p.mana_per_level * (new_level - old_level) as f32;
            p.level = new_level; p.max_health += hp_gain; p.health += hp_gain; p.max_mana += mana_gain; p.mana += mana_gain;
            state.event_queue.push(GameEvent::LevelUp { player_id: id, new_level });
        }
    }

    // ── Spawn minion waves ──
    if state.game_time >= state.next_wave_time {
        state.next_wave_time += MINION_WAVE_INTERVAL;
        let is_cannon = state.wave_count % 3 == 2;
        state.wave_count += 1;
        let lanes: Vec<(u8, Vec<[f32; 2]>)> = vec![
            (0, state.layout.lane_paths.top_blue.iter().map(|v| [v.x, v.y]).collect()),
            (1, state.layout.lane_paths.top_red.iter().map(|v| [v.x, v.y]).collect()),
            (0, state.layout.lane_paths.bottom_blue.iter().map(|v| [v.x, v.y]).collect()),
            (1, state.layout.lane_paths.bottom_red.iter().map(|v| [v.x, v.y]).collect()),
        ];
        let count = if is_cannon { 7 } else { 6 };
        for (team, waypoints) in &lanes {
            let spawn = waypoints[0];
            for i in 0..count {
                let id = state.next_minion_id; state.next_minion_id = state.next_minion_id.wrapping_add(1);
                let (mtype, hp, ad) = if i < 3 { (MinionType::Melee, 475.0, 12.0) } else if i < 6 { (MinionType::Caster, 290.0, 23.0) } else { (MinionType::Siege, 700.0, 40.0) };
                state.minions.push(ServerMinion { id, team: *team, minion_type: mtype, position: [spawn[0] + i as f32 * 40.0, spawn[1]], health: hp, max_health: hp, ad, move_speed: 325.0, waypoints: waypoints.clone(), waypoint_idx: 0, attack_cooldown: 0.0 });
            }
        }
    }

    // ── Move minions ──
    for minion in state.minions.iter_mut() {
        if minion.health <= 0.0 { continue; }
        if minion.waypoint_idx < minion.waypoints.len() {
            let wp = minion.waypoints[minion.waypoint_idx];
            let dx = wp[0] - minion.position[0]; let dz = wp[1] - minion.position[1];
            let dist = (dx * dx + dz * dz).sqrt();
            if dist < 50.0 { minion.waypoint_idx += 1; }
            else { let step = minion.move_speed * dt; minion.position[0] += dx / dist * step; minion.position[1] += dz / dist * step; }
        }
    }

    // ── Minion vs minion combat (collect then apply) ──
    let mut minion_dmg: Vec<(u16, f32)> = vec![];
    let mc = state.minions.len();
    for i in 0..mc {
        if state.minions[i].health <= 0.0 { continue; }
        state.minions[i].attack_cooldown -= dt;
        if state.minions[i].attack_cooldown > 0.0 { continue; }
        let my_team = state.minions[i].team; let my_pos = state.minions[i].position; let my_ad = state.minions[i].ad;
        let mut closest: Option<(u16, f32)> = None;
        for j in 0..mc {
            if i == j || state.minions[j].team == my_team || state.minions[j].health <= 0.0 { continue; }
            let dx = state.minions[j].position[0] - my_pos[0]; let dz = state.minions[j].position[1] - my_pos[1];
            let dist = (dx * dx + dz * dz).sqrt();
            if dist < 475.0 && closest.map_or(true, |(_, d)| dist < d) { closest = Some((state.minions[j].id, dist)); }
        }
        if let Some((tid, _)) = closest { minion_dmg.push((tid, my_ad)); state.minions[i].attack_cooldown = 1.0 / 0.625; }
    }
    for (tid, dmg) in &minion_dmg {
        if let Some(m) = state.minions.iter_mut().find(|m| m.id == *tid) { m.health -= dmg; }
    }

    // ── Gold/XP from dead minions ──
    let dead_minions: Vec<([f32; 2], u8, MinionType)> = state.minions.iter().filter(|m| m.health <= 0.0).map(|m| (m.position, m.team, m.minion_type)).collect();
    for (pos, mteam, mtype) in &dead_minions {
        for p in state.players.values_mut() {
            if !p.alive || p.team == *mteam { continue; }
            let dx = p.position[0] - pos[0]; let dz = p.position[1] - pos[1];
            let dist = (dx * dx + dz * dz).sqrt();
            if dist < 550.0 { p.gold += minion_gold(*mtype, game_time); }
            if dist < XP_RANGE { p.xp += shared_xp(kill_xp(1, p.level), 1); }
            if dist < 550.0 && p.buffs.iter().any(|b| b.buff_type == 2) {
                p.health = (p.health + p.max_health * ALTAR_2_HP_RESTORE_PCT).min(p.max_health);
            }
        }
    }
    state.minions.retain(|m| m.health > 0.0);

    // ── Turret AI (collect damage, then apply) ──
    let mut turret_dmg_minion: Vec<(u16, f32)> = vec![];
    let mut turret_dmg_player: Vec<(u8, f32)> = vec![];
    {
        // Collect minion positions/teams and player positions/teams for targeting
        let minion_info: Vec<(u16, [f32; 2], u8, f32)> = state.minions.iter().filter(|m| m.health > 0.0).map(|m| (m.id, m.position, m.team, m.health)).collect();
        let player_info: Vec<(u8, [f32; 2], u8, bool)> = state.players.iter().map(|(id, p)| (*id, p.position, p.team, p.alive)).collect();

        for turret in state.turrets.iter_mut() {
            if !turret.alive { continue; }
            turret.attack_cooldown -= dt;
            if turret.attack_cooldown > 0.0 { continue; }

            // Find target: minions first, then players
            let mut best_minion: Option<(u16, f32)> = None;
            for &(mid, mpos, mteam, _) in &minion_info {
                if mteam == turret.team { continue; }
                let dx = mpos[0] - turret.position[0]; let dz = mpos[1] - turret.position[1];
                let dist = (dx * dx + dz * dz).sqrt();
                if dist < turret.range && best_minion.map_or(true, |(_, d)| dist < d) { best_minion = Some((mid, dist)); }
            }
            let mut best_player: Option<(u8, f32)> = None;
            for &(pid, ppos, pteam, palive) in &player_info {
                if !palive || pteam == turret.team { continue; }
                let dx = ppos[0] - turret.position[0]; let dz = ppos[1] - turret.position[1];
                let dist = (dx * dx + dz * dz).sqrt();
                if dist < turret.range && best_player.map_or(true, |(_, d)| dist < d) { best_player = Some((pid, dist)); }
            }

            if let Some((mid, _)) = best_minion {
                turret_dmg_minion.push((mid, turret.ad));
                turret.attack_cooldown = 1.0 / turret.attack_speed;
            } else if let Some((pid, _)) = best_player {
                turret_dmg_player.push((pid, turret.ad));
                turret.attack_cooldown = 1.0 / turret.attack_speed;
            }
        }
    }
    for (mid, dmg) in turret_dmg_minion { if let Some(m) = state.minions.iter_mut().find(|m| m.id == mid) { m.health -= dmg; } }
    for (pid, dmg) in turret_dmg_player { if let Some(p) = state.players.get_mut(&pid) { let r = p.total_armor(); p.health -= dmg * 100.0 / (100.0 + r); } }

    // ── Minions attack turrets ──
    {
        // Copy turret info to avoid borrow conflict
        let turret_info: Vec<(u8, [f32; 2], u8, f32, bool)> = state.turrets.iter().map(|t| (t.id, t.position, t.team, t.armor, t.alive)).collect();
        let mut attacks: Vec<(u8, f32)> = vec![];
        for minion in state.minions.iter_mut() {
            if minion.health <= 0.0 || minion.attack_cooldown > 0.0 { continue; }
            for &(tid, tpos, tteam, tarmor, talive) in &turret_info {
                if !talive || tteam == minion.team { continue; }
                let dx = tpos[0] - minion.position[0]; let dz = tpos[1] - minion.position[1];
                if (dx * dx + dz * dz).sqrt() < 200.0 {
                    attacks.push((tid, minion.ad * 100.0 / (100.0 + tarmor)));
                    minion.attack_cooldown = 1.0 / 0.625;
                    break;
                }
            }
        }
        for (tid, dmg) in attacks {
            if let Some(t) = state.turrets.iter_mut().find(|t| t.id == tid) { t.health -= dmg; }
        }
    }

    // ── Turret death events ──
    let mut turret_death_events: Vec<GameEvent> = vec![];
    for turret in state.turrets.iter_mut() {
        if turret.alive && turret.health <= 0.0 {
            turret.alive = false;
            turret_death_events.push(GameEvent::TurretDestroyed { turret_id: turret.id, team: turret.team });
        }
    }
    state.event_queue.extend(turret_death_events);

    // ── Jungle camps ──
    for camp in state.jungle_camps.iter_mut() {
        if !camp.alive && game_time >= camp.respawn_at {
            camp.alive = true; camp.health = camp.max_health; camp.position = camp.spawn_position; camp.aggro_target = None;
        }
    }
    let mut camp_dmg_player: Vec<(u8, f32)> = vec![];
    let mut dead_camps: Vec<(u8, Option<u8>)> = vec![];
    {
        let player_info: Vec<(u8, [f32; 2], bool)> = state.players.iter().map(|(id, p)| (*id, p.position, p.alive)).collect();
        for camp in state.jungle_camps.iter_mut() {
            if !camp.alive || camp.health <= 0.0 { continue; }
            camp.attack_cooldown -= dt;
            let mut closest: Option<(u8, f32)> = None;
            for &(pid, ppos, palive) in &player_info {
                if !palive { continue; }
                let dx = ppos[0] - camp.position[0]; let dz = ppos[1] - camp.position[1];
                let dist = (dx * dx + dz * dz).sqrt();
                if dist < 500.0 && closest.map_or(true, |(_, d)| dist < d) { closest = Some((pid, dist)); }
            }
            if let Some((pid, _)) = closest {
                camp.aggro_target = Some(pid);
                if camp.attack_cooldown <= 0.0 { camp_dmg_player.push((pid, camp.ad)); camp.attack_cooldown = 1.5; }
            } else {
                camp.aggro_target = None;
                let dx = camp.spawn_position[0] - camp.position[0]; let dz = camp.spawn_position[1] - camp.position[1];
                if (dx * dx + dz * dz).sqrt() > 800.0 { camp.health = camp.max_health; camp.position = camp.spawn_position; }
            }
            if camp.health <= 0.0 { camp.alive = false; camp.respawn_at = game_time + JUNGLE_RESPAWN; dead_camps.push((camp.id, camp.aggro_target)); }
        }
    }
    for (pid, dmg) in camp_dmg_player { if let Some(p) = state.players.get_mut(&pid) { let r = p.total_armor(); p.health -= dmg * 100.0 / (100.0 + r); } }
    for (camp_id, killer) in &dead_camps {
        if let Some(kid) = killer { if let Some(p) = state.players.get_mut(kid) { p.gold += 60.0; p.xp += 80.0; } }
        state.event_queue.push(GameEvent::JungleCampKilled { camp_id: *camp_id, killer_id: killer.unwrap_or(255) });
    }

    // ── Vilemaw ──
    if state.vilemaw.is_none() && game_time >= state.vilemaw_next_spawn {
        let pos = state.layout.vilemaw_spawn;
        state.vilemaw = Some(ServerVilemaw { position: [pos.x, pos.y], health: VILEMAW_HP, max_health: VILEMAW_HP, ad: VILEMAW_AD, attack_cooldown: 0.0, aggro_target: None });
    }
    let mut vim_dmg_player: Vec<(u8, f32)> = vec![];
    let mut vim_dead_team: Option<u8> = None;
    {
        // Collect player info first to avoid borrow conflict with vilemaw
        let player_info: Vec<(u8, [f32; 2], bool, u8)> = state.players.iter().map(|(id, p)| (*id, p.position, p.alive, p.team)).collect();
        if let Some(ref mut vim) = state.vilemaw {
            vim.attack_cooldown -= dt;
            let mut closest: Option<(u8, f32)> = None;
            for &(pid, ppos, palive, _) in &player_info {
                if !palive { continue; }
                let dx = ppos[0] - vim.position[0]; let dz = ppos[1] - vim.position[1];
                let dist = (dx * dx + dz * dz).sqrt();
                if dist < 800.0 && closest.map_or(true, |(_, d)| dist < d) { closest = Some((pid, dist)); }
            }
            if let Some((pid, _)) = closest {
                vim.aggro_target = Some(pid);
                if vim.attack_cooldown <= 0.0 { vim_dmg_player.push((pid, vim.ad)); vim.attack_cooldown = 2.0; }
            } else { vim.aggro_target = None; }
            if vim.health <= 0.0 {
                let killer_team = vim.aggro_target.and_then(|pid| player_info.iter().find(|i| i.0 == pid).map(|i| i.3));
                vim_dead_team = killer_team;
            }
        }
    }
    for (pid, dmg) in vim_dmg_player { if let Some(p) = state.players.get_mut(&pid) { let r = p.total_armor(); p.health -= dmg * 100.0 / (100.0 + r); } }
    if let Some(killer_team) = vim_dead_team {
        state.vilemaw = None;
        state.vilemaw_next_spawn = game_time + VILEMAW_RESPAWN;
        for p in state.players.values_mut() {
            if p.team == killer_team && p.alive { p.buffs.push(ServerBuff { buff_type: 0, remaining: VILEMAW_BUFF_DURATION }); p.gold += 150.0; p.xp += 200.0; }
        }
        state.event_queue.push(GameEvent::VilemawKilled { killer_team });
    }

    // ── Altars ──
    if game_time >= ALTAR_UNLOCK_TIME {
        let altar_positions = [
            [state.layout.altars[0].position.x, state.layout.altars[0].position.y],
            [state.layout.altars[1].position.x, state.layout.altars[1].position.y],
        ];
        let player_pos_team: Vec<([f32; 2], u8, bool)> = state.players.values().map(|p| (p.position, p.team, p.alive)).collect();

        let mut altar_events: Vec<GameEvent> = vec![];
        for (i, altar) in state.altars.iter_mut().enumerate() {
            altar.lockout = (altar.lockout - dt).max(0.0);
            if altar.lockout > 0.0 { continue; }

            let apos = altar_positions[i];
            let mut capturing: Option<u8> = None;
            for &(ppos, pteam, palive) in &player_pos_team {
                if !palive { continue; }
                let dx = ppos[0] - apos[0]; let dz = ppos[1] - apos[1];
                if (dx * dx + dz * dz).sqrt() < 200.0 { capturing = Some(pteam); break; }
            }

            if let Some(team) = capturing {
                let owner_team = if team == 0 { 1u8 } else { 2u8 };
                if altar.owner != owner_team {
                    altar.capture_progress += dt;
                    if altar.capture_progress >= ALTAR_CAPTURE_TIME {
                        altar.owner = owner_team; altar.capture_progress = 0.0; altar.lockout = ALTAR_LOCKOUT_TIME;
                        altar_events.push(GameEvent::AltarCaptured { side: altar.side, team: owner_team });
                    }
                }
            } else {
                altar.capture_progress = (altar.capture_progress - dt * 2.0).max(0.0);
            }
        }
        state.event_queue.extend(altar_events);

        // Apply altar buffs
        let altars_blue = state.altars.iter().filter(|a| a.owner == 1).count();
        let altars_red = state.altars.iter().filter(|a| a.owner == 2).count();
        for p in state.players.values_mut() {
            let my_altars = if p.team == 0 { altars_blue } else { altars_red };
            p.buffs.retain(|b| b.buff_type != 1 && b.buff_type != 2);
            if my_altars >= 1 { p.buffs.push(ServerBuff { buff_type: 1, remaining: 999.0 }); }
            if my_altars >= 2 { p.buffs.push(ServerBuff { buff_type: 2, remaining: 999.0 }); }
        }
    }

    // ── Speed shrine ──
    let shrine = [state.layout.speed_shrine.x, state.layout.speed_shrine.y];
    if game_time >= SPEED_SHRINE_UNLOCK {
        for p in state.players.values_mut() {
            if !p.alive || p.buffs.iter().any(|b| b.buff_type == 3) { continue; }
            let dx = p.position[0] - shrine[0]; let dz = p.position[1] - shrine[1];
            if (dx * dx + dz * dz).sqrt() < 100.0 { p.buffs.push(ServerBuff { buff_type: 3, remaining: 4.0 }); }
        }
    }

    // ── Health relics ──
    let relics: Vec<[f32; 2]> = state.layout.health_relics.iter().map(|r| [r.x, r.y]).collect();
    if game_time >= HEALTH_RELIC_UNLOCK {
        for rpos in &relics {
            for p in state.players.values_mut() {
                if !p.alive { continue; }
                let dx = p.position[0] - rpos[0]; let dz = p.position[1] - rpos[1];
                if (dx * dx + dz * dz).sqrt() < 100.0 { p.health = (p.health + p.max_health * 0.1).min(p.max_health); }
            }
        }
    }

    // ── Final death check (from turrets/jungle/vilemaw) ──
    for p in state.players.values_mut() {
        if p.alive && p.health <= 0.0 {
            p.alive = false; p.health = 0.0; p.deaths += 1;
            p.respawn_at = game_time + death_timer(p.level, game_time);
        }
    }
}

// ─── Send snapshots ───

fn send_snapshots(server: Res<NetServer>, mut state: ResMut<ServerGameState>) {
    if !state.started { return; }

    let events: Vec<GameEvent> = state.event_queue.drain(..).collect();
    for event in &events {
        let data = encode_packet(&ServerPacket::Event(event.clone()));
        for &addr in server.clients.keys() { let _ = server.socket.send_to(&data, addr); }
    }

    if state.tick % 3 != 0 { return; }

    let snapshot = GameSnapshot {
        tick: state.tick, game_time: state.game_time,
        players: state.players.iter().map(|(&id, p)| PlayerState {
            id, team: p.team, champion_class: p.champion_class, position: p.position,
            health: p.health, max_health: p.max_health, mana: p.mana, max_mana: p.max_mana,
            gold: p.gold, level: p.level, xp: p.xp, alive: p.alive,
            ad: p.total_ad(), ap: p.total_ap(), armor: p.total_armor(), mr: p.total_mr(),
            attack_speed: p.total_attack_speed(), move_speed: p.total_move_speed(),
            cooldowns: p.ability_cooldowns, items: p.items.clone(),
            buffs: p.buffs.iter().map(|b| BuffSnap { buff_type: b.buff_type, remaining: b.remaining }).collect(),
        }).collect(),
        minions: state.minions.iter().map(|m| MinionState { id: m.id, team: m.team, position: m.position, health: m.health, max_health: m.max_health }).collect(),
        turrets: state.turrets.iter().filter(|t| t.alive).map(|t| TurretState { id: t.id, team: t.team, position: t.position, health: t.health, max_health: t.max_health, target_id: None }).collect(),
        jungle_camps: state.jungle_camps.iter().map(|c| JungleCampState { id: c.id, position: c.position, health: c.health, max_health: c.max_health, alive: c.alive }).collect(),
        vilemaw: state.vilemaw.as_ref().map(|v| BossState { position: v.position, health: v.health, max_health: v.max_health }),
        altars: [
            AltarSnap { owner: state.altars[0].owner, progress: state.altars[0].capture_progress, lockout: state.altars[0].lockout },
            AltarSnap { owner: state.altars[1].owner, progress: state.altars[1].capture_progress, lockout: state.altars[1].lockout },
        ],
    };

    let data = encode_packet(&ServerPacket::Snapshot(snapshot));
    for &addr in server.clients.keys() { let _ = server.socket.send_to(&data, addr); }
}
