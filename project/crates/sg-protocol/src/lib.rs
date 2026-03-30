use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const SERVER_PORT: u16 = 5000;
pub const PROTOCOL_ID: u32 = 0x5347_0001; // "SG" v1
pub const TICK_RATE_HZ: u32 = 60;
pub const MAX_PLAYERS: usize = 6;

// === Client → Server ===

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientPacket {
    /// Join request with desired champion name
    Join { name: String, champion_class: u8 },
    /// Player input for this tick
    Input(PlayerInput),
    /// Chat message
    Chat { text: String },
    /// Surrender vote
    Surrender { vote: bool },
    /// Ready up in lobby
    Ready,
    /// Buy item from shop
    BuyItem { item_id: u32 },
    /// Sell item from inventory slot
    SellItem { slot: u8 },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlayerInput {
    /// Right-click move target (world X, Z)
    pub move_target: Option<[f32; 2]>,
    /// Ability cast: 0=Q, 1=W, 2=E, 3=R
    pub ability_cast: Option<u8>,
    /// Cursor world position for aim
    pub cursor_pos: [f32; 2],
    /// Attack-move target (A-click on enemy entity ID)
    pub attack_target_id: Option<u8>,
}

// === Server → Client ===

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerPacket {
    /// Welcome message with assigned player ID and team
    Welcome { player_id: u8, team: u8, spawn: [f32; 2] },
    /// Full game state snapshot (sent periodically)
    Snapshot(GameSnapshot),
    /// Game event
    Event(GameEvent),
    /// Chat from another player
    Chat { player_id: u8, text: String },
    /// Lobby update with connected players
    LobbyUpdate { players: Vec<LobbyPlayerInfo>, countdown: Option<f32> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameSnapshot {
    pub tick: u32,
    pub game_time: f32,
    pub players: Vec<PlayerState>,
    pub minions: Vec<MinionState>,
    pub turrets: Vec<TurretState>,
    pub jungle_camps: Vec<JungleCampState>,
    pub vilemaw: Option<BossState>,
    pub altars: [AltarSnap; 2],
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    pub id: u8,
    pub team: u8,
    pub champion_class: u8, // 0=Mage, 1=Fighter, 2=Tank
    pub position: [f32; 2],
    pub health: f32,
    pub max_health: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub gold: f32,
    pub level: u8,
    pub xp: f32,
    pub alive: bool,
    pub ad: f32,
    pub ap: f32,
    pub armor: f32,
    pub mr: f32,
    pub attack_speed: f32,
    pub move_speed: f32,
    pub cooldowns: [f32; 4], // Q, W, E, R remaining cd
    pub items: Vec<u32>,     // item IDs in inventory
    pub buffs: Vec<BuffSnap>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LobbyPlayerInfo {
    pub id: u8,
    pub name: String,
    pub team: u8,
    pub champion_class: u8,
    pub ready: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuffSnap {
    pub buff_type: u8,  // 0=Vilemaw, 1=AltarSpeed, 2=AltarHp, 3=SpeedShrine, 4=Stun, 5=Slow, 6=Root
    pub remaining: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MinionState {
    pub id: u16,
    pub team: u8,
    pub position: [f32; 2],
    pub health: f32,
    pub max_health: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TurretState {
    pub id: u8,
    pub team: u8,
    pub position: [f32; 2],
    pub health: f32,
    pub max_health: f32,
    pub target_id: Option<u16>, // who it's shooting at
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JungleCampState {
    pub id: u8,
    pub position: [f32; 2],
    pub health: f32,
    pub max_health: f32,
    pub alive: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BossState {
    pub position: [f32; 2],
    pub health: f32,
    pub max_health: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AltarSnap {
    pub owner: u8,   // 0=none, 1=blue, 2=red
    pub progress: f32,
    pub lockout: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GameEvent {
    PlayerJoined { id: u8, name: String, team: u8 },
    PlayerLeft { id: u8 },
    Kill { killer: u8, victim: u8, gold: f32 },
    TurretDestroyed { turret_id: u8, team: u8 },
    InhibitorDestroyed { team: u8 },
    JungleCampKilled { camp_id: u8, killer_id: u8 },
    VilemawKilled { killer_team: u8 },
    AltarCaptured { side: u8, team: u8 },
    AbilityCast { caster_id: u8, ability: u8, target_pos: [f32; 2] },
    ItemPurchased { player_id: u8, item_id: u32 },
    LevelUp { player_id: u8, new_level: u8 },
    GameStart,
    GameEnd { winner: u8 },
}

// === Packet serialization helpers ===

pub fn encode_packet<T: Serialize>(packet: &T) -> Vec<u8> {
    let mut data = PROTOCOL_ID.to_le_bytes().to_vec();
    data.extend(bincode::serialize(packet).unwrap_or_default());
    data
}

pub fn decode_packet<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Option<T> {
    if data.len() < 4 { return None; }
    let proto = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if proto != PROTOCOL_ID { return None; }
    bincode::deserialize(&data[4..]).ok()
}
