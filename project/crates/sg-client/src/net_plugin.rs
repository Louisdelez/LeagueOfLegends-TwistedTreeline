use bevy::prelude::*;
use std::net::UdpSocket;
use sg_protocol::*;
use sg_core::GameSet;

#[derive(Resource)]
pub struct NetClient {
    pub socket: Option<UdpSocket>,
    pub server_addr: String,
    pub my_id: Option<u8>,
    pub my_team: Option<u8>,
    pub connected: bool,
    pub latest_snapshot: Option<GameSnapshot>,
    pub events: Vec<GameEvent>,
    pub champion_class: u8,
}

impl Default for NetClient {
    fn default() -> Self {
        Self {
            socket: None,
            server_addr: format!("127.0.0.1:{}", SERVER_PORT),
            my_id: None,
            my_team: None,
            connected: false,
            latest_snapshot: None,
            events: vec![],
            champion_class: 0,
        }
    }
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetClient::default())
            .add_systems(Update, (
                try_connect,
                receive_server_packets,
            ).in_set(GameSet::Input));
    }
}

fn try_connect(
    keys: Res<ButtonInput<KeyCode>>,
    mut net: ResMut<NetClient>,
) {
    // Press F5 to connect to server
    if keys.just_pressed(KeyCode::F5) && !net.connected {
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                socket.set_nonblocking(true).unwrap();
                let join = encode_packet(&ClientPacket::Join {
                    name: "Player".into(),
                    champion_class: net.champion_class,
                });
                if let Err(e) = socket.send_to(&join, &net.server_addr) {
                    eprintln!("Failed to send join: {}", e);
                    return;
                }
                net.socket = Some(socket);
                println!("Connecting to {}...", net.server_addr);
            }
            Err(e) => eprintln!("Failed to bind socket: {}", e),
        }
    }
}

fn receive_server_packets(
    mut net: ResMut<NetClient>,
) {
    if net.socket.is_none() { return; }

    let mut buf = [0u8; 65535];
    let mut packets = vec![];

    {
        let socket = net.socket.as_ref().unwrap();
        while let Ok((len, _addr)) = socket.recv_from(&mut buf) {
            let data = buf[..len].to_vec();
            if let Some(packet) = decode_packet::<ServerPacket>(&data) {
                packets.push(packet);
            }
        }
    }

    for packet in packets {
        match packet {
            ServerPacket::Welcome { player_id, team, spawn } => {
                net.my_id = Some(player_id);
                net.my_team = Some(team);
                net.connected = true;
                println!("Connected! ID={}, Team={}, Spawn={:?}", player_id, team, spawn);
            }
            ServerPacket::Snapshot(snapshot) => {
                net.latest_snapshot = Some(snapshot);
            }
            ServerPacket::Event(event) => {
                match &event {
                    GameEvent::PlayerJoined { id, name, team } => {
                        println!("Player {} ({}) joined team {}", name, id, team);
                    }
                    GameEvent::GameStart => {
                        println!("Game started!");
                    }
                    GameEvent::Kill { killer, victim, gold } => {
                        println!("Player {} killed player {} (+{}g)", killer, victim, gold);
                    }
                    GameEvent::TurretDestroyed { turret_id, team } => {
                        println!("Turret {} (team {}) destroyed!", turret_id, team);
                    }
                    GameEvent::VilemawKilled { killer_team } => {
                        println!("Vilemaw slain by team {}!", killer_team);
                    }
                    GameEvent::AltarCaptured { side, team } => {
                        let side_name = if *side == 0 { "Left" } else { "Right" };
                        println!("{} altar captured by team {}", side_name, team);
                    }
                    GameEvent::LevelUp { player_id, new_level } => {
                        println!("Player {} reached level {}!", player_id, new_level);
                    }
                    GameEvent::ItemPurchased { player_id, item_id } => {
                        println!("Player {} purchased item {}", player_id, item_id);
                    }
                    _ => {}
                }
                net.events.push(event);
            }
            ServerPacket::Chat { player_id, text } => {
                println!("[Player {}]: {}", player_id, text);
            }
        }
    }
}

/// Send input to server (called from input_plugin when connected)
pub fn send_input_to_server(net: &NetClient, input: &PlayerInput) {
    if !net.connected { return; }
    if let Some(socket) = &net.socket {
        let data = encode_packet(&ClientPacket::Input(input.clone()));
        let _ = socket.send_to(&data, &net.server_addr);
    }
}

/// Send buy item request to server
pub fn send_buy_item(net: &NetClient, item_id: u32) {
    if !net.connected { return; }
    if let Some(socket) = &net.socket {
        let data = encode_packet(&ClientPacket::BuyItem { item_id });
        let _ = socket.send_to(&data, &net.server_addr);
    }
}

/// Send sell item request to server
pub fn send_sell_item(net: &NetClient, slot: u8) {
    if !net.connected { return; }
    if let Some(socket) = &net.socket {
        let data = encode_packet(&ClientPacket::SellItem { slot });
        let _ = socket.send_to(&data, &net.server_addr);
    }
}
