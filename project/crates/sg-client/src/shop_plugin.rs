use bevy::prelude::*;
use sg_core::components::*;
use sg_core::types::*;
use crate::menu::AppState;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShopState::default())
            .insert_resource(ItemDatabase::default())
            .add_systems(Update, (toggle_shop, buy_item_keyboard).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Resource, Default)]
pub struct ShopState {
    pub open: bool,
}

/// Tracks items the player has purchased
#[derive(Component, Default)]
pub struct PlayerInventory {
    pub items: Vec<u32>, // item IDs, max 6
}

#[derive(Component)]
struct ShopUI;

#[derive(Clone, Debug)]
pub struct ShopItem {
    pub id: u32,
    pub name: &'static str,
    pub cost: u32,
    pub icon: &'static str,  // asset path to icon PNG
    pub ad: f32,
    pub ap: f32,
    pub hp: f32,
    pub armor: f32,
    pub mr: f32,
    pub attack_speed: f32,
}

#[derive(Resource)]
pub struct ItemDatabase {
    pub items: Vec<ShopItem>,
}

impl Default for ItemDatabase {
    fn default() -> Self {
        Self { items: vec![
            // === Consumables ===
            ShopItem { id: 2003, name: "Health Potion", cost: 35, icon: "ui/items/2003.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            // === Starters ===
            ShopItem { id: 1054, name: "Doran's Shield", cost: 440, icon: "ui/items/1054.png", ad: 0.0, ap: 0.0, hp: 80.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1055, name: "Doran's Blade", cost: 440, icon: "ui/items/1055.png", ad: 7.0, ap: 0.0, hp: 70.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1056, name: "Doran's Ring", cost: 400, icon: "ui/items/1056.png", ad: 0.0, ap: 15.0, hp: 60.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            // === Components - AD ===
            ShopItem { id: 1036, name: "Long Sword", cost: 360, icon: "ui/items/1036.png", ad: 10.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1037, name: "Pickaxe", cost: 875, icon: "ui/items/1037.png", ad: 25.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1038, name: "B. F. Sword", cost: 1550, icon: "ui/items/1038.png", ad: 50.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1053, name: "Vampiric Scepter", cost: 440, icon: "ui/items/1053.png", ad: 10.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            // === Components - AP ===
            ShopItem { id: 1052, name: "Amplifying Tome", cost: 435, icon: "ui/items/1052.png", ad: 0.0, ap: 20.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1026, name: "Blasting Wand", cost: 860, icon: "ui/items/1026.png", ad: 0.0, ap: 40.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1058, name: "Needlessly Large Rod", cost: 1600, icon: "ui/items/1058.png", ad: 0.0, ap: 80.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            // === Components - Defense ===
            ShopItem { id: 1028, name: "Ruby Crystal", cost: 400, icon: "ui/items/1028.png", ad: 0.0, ap: 0.0, hp: 150.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1011, name: "Giant's Belt", cost: 1000, icon: "ui/items/1011.png", ad: 0.0, ap: 0.0, hp: 380.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1029, name: "Cloth Armor", cost: 300, icon: "ui/items/1029.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 15.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1031, name: "Chain Vest", cost: 800, icon: "ui/items/1031.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 40.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 1033, name: "Null-Magic Mantle", cost: 450, icon: "ui/items/1033.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 25.0, attack_speed: 0.0 },
            // === Boots ===
            ShopItem { id: 1001, name: "Boots of Speed", cost: 325, icon: "ui/items/1001.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3006, name: "Berserker's Greaves", cost: 225, icon: "ui/items/3006.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.25 },
            ShopItem { id: 3009, name: "Boots of Swiftness", cost: 675, icon: "ui/items/3009.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3020, name: "Sorcerer's Shoes", cost: 775, icon: "ui/items/3020.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3047, name: "Ninja Tabi", cost: 375, icon: "ui/items/3047.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 25.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3111, name: "Mercury's Treads", cost: 375, icon: "ui/items/3111.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 25.0, attack_speed: 0.0 },
            // === Finished - AD ===
            ShopItem { id: 3031, name: "Infinity Edge", cost: 645, icon: "ui/items/3031.png", ad: 80.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3072, name: "Bloodthirster", cost: 1150, icon: "ui/items/3072.png", ad: 80.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3071, name: "Black Cleaver", cost: 1263, icon: "ui/items/3071.png", ad: 50.0, ap: 0.0, hp: 200.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3153, name: "Blade of the Ruined King", cost: 900, icon: "ui/items/3153.png", ad: 25.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.4 },
            ShopItem { id: 3142, name: "Youmuu's Ghostblade", cost: 563, icon: "ui/items/3142.png", ad: 30.0, ap: 0.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3044, name: "Phage", cost: 565, icon: "ui/items/3044.png", ad: 20.0, ap: 0.0, hp: 200.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3022, name: "Frozen Mallet", cost: 1025, icon: "ui/items/3022.png", ad: 30.0, ap: 0.0, hp: 700.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            // === Finished - AP ===
            ShopItem { id: 3089, name: "Rabadon's Deathcap", cost: 840, icon: "ui/items/3089.png", ad: 0.0, ap: 120.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3100, name: "Lich Bane", cost: 850, icon: "ui/items/3100.png", ad: 0.0, ap: 80.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3135, name: "Void Staff", cost: 1000, icon: "ui/items/3135.png", ad: 0.0, ap: 70.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3157, name: "Zhonya's Hourglass", cost: 500, icon: "ui/items/3157.png", ad: 0.0, ap: 120.0, hp: 0.0, armor: 50.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3165, name: "Morellonomicon", cost: 680, icon: "ui/items/3165.png", ad: 0.0, ap: 80.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3027, name: "Rod of Ages", cost: 740, icon: "ui/items/3027.png", ad: 0.0, ap: 60.0, hp: 450.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3023, name: "Twin Shadows", cost: 630, icon: "ui/items/3023.png", ad: 0.0, ap: 80.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3025, name: "Iceborn Gauntlet", cost: 750, icon: "ui/items/3025.png", ad: 0.0, ap: 30.0, hp: 0.0, armor: 60.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3057, name: "Sheen", cost: 365, icon: "ui/items/3057.png", ad: 0.0, ap: 25.0, hp: 0.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            // === Finished - Tank ===
            ShopItem { id: 3068, name: "Sunfire Cape", cost: 850, icon: "ui/items/3068.png", ad: 0.0, ap: 0.0, hp: 450.0, armor: 45.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3143, name: "Randuin's Omen", cost: 800, icon: "ui/items/3143.png", ad: 0.0, ap: 0.0, hp: 500.0, armor: 70.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3075, name: "Thornmail", cost: 1050, icon: "ui/items/3075.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 100.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3083, name: "Warmog's Armor", cost: 300, icon: "ui/items/3083.png", ad: 0.0, ap: 0.0, hp: 800.0, armor: 0.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3110, name: "Frozen Heart", cost: 450, icon: "ui/items/3110.png", ad: 0.0, ap: 0.0, hp: 0.0, armor: 100.0, mr: 0.0, attack_speed: 0.0 },
            ShopItem { id: 3065, name: "Spirit Visage", cost: 700, icon: "ui/items/3065.png", ad: 0.0, ap: 0.0, hp: 400.0, armor: 0.0, mr: 55.0, attack_speed: 0.0 },
            ShopItem { id: 3102, name: "Banshee's Veil", cost: 1150, icon: "ui/items/3102.png", ad: 0.0, ap: 0.0, hp: 450.0, armor: 0.0, mr: 55.0, attack_speed: 0.0 },
            // === Hybrid ===
            ShopItem { id: 3078, name: "Trinity Force", cost: 78, icon: "ui/items/3078.png", ad: 30.0, ap: 30.0, hp: 250.0, armor: 0.0, mr: 0.0, attack_speed: 0.3 },
        ]}
    }
}

/// Aggregate stat bonuses from all items in inventory
pub fn total_item_bonuses(inventory: &PlayerInventory, db: &ItemDatabase) -> (f32, f32, f32, f32, f32, f32, f32) {
    let (mut ad, mut ap, mut hp, mut armor, mut mr, mut atk_spd, mut ms) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for &item_id in &inventory.items {
        if let Some(item) = db.items.iter().find(|i| i.id == item_id) {
            ad += item.ad;
            ap += item.ap;
            hp += item.hp;
            armor += item.armor;
            mr += item.mr;
            atk_spd += item.attack_speed;
            match item.id {
                1001 => ms += 25.0,
                3006 | 3020 | 3047 | 3111 => ms += 45.0,
                3009 => ms += 60.0,
                _ => {}
            }
        }
    }
    (ad, ap, hp, armor, mr, atk_spd, ms)
}

fn toggle_shop(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut shop: ResMut<ShopState>,
    existing: Query<Entity, With<ShopUI>>,
    asset_server: Res<AssetServer>,
    db: Res<ItemDatabase>,
) {
    if !keys.just_pressed(KeyCode::KeyP) { return; }

    shop.open = !shop.open;

    if shop.open {
        // Create shop UI overlay
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(10.0),
                left: Val::Percent(25.0),
                width: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.92)),
            ShopUI,
        )).with_children(|panel| {
            // Title
            panel.spawn((
                Text::new("SHOP — Press 1-0 to buy, P to close"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.0)),
            ));

            // Item grid
            panel.spawn(Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(8.0),
                ..default()
            }).with_children(|grid| {
                for (i, item) in db.items.iter().enumerate() {
                    let key = if i < 9 { format!("{}", i + 1) } else { "0".to_string() };

                    grid.spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.0),
                            padding: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
                    )).with_children(|row| {
                        // Key number
                        row.spawn((
                            Text::new(&key),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::srgb(0.7, 0.7, 0.3)),
                        ));
                        // Item icon
                        row.spawn((
                            Node { width: Val::Px(36.0), height: Val::Px(36.0), ..default() },
                            ImageNode::new(asset_server.load(item.icon)),
                        ));
                        // Name + cost
                        row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        }).with_children(|info| {
                            info.spawn((
                                Text::new(item.name),
                                TextFont { font_size: 13.0, ..default() },
                                TextColor(Color::WHITE),
                            ));
                            info.spawn((
                                Text::new(format!("{}g", item.cost)),
                                TextFont { font_size: 11.0, ..default() },
                                TextColor(Color::srgb(1.0, 0.85, 0.0)),
                            ));
                        });
                    });
                }
            });
        });
    } else {
        // Close shop
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

/// Marker component to trigger stat recalculation
#[derive(Component)]
pub struct InventoryChanged;

fn buy_item_keyboard(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    shop: Res<ShopState>,
    db: Res<ItemDatabase>,
    net: Res<crate::net_plugin::NetClient>,
    mut player: Query<(Entity, &mut Gold, &mut PlayerInventory), With<PlayerControlled>>,
) {
    if !shop.open { return; }
    let Ok((entity, mut gold, mut inventory)) = player.single_mut() else { return };

    let item_keys = [
        (KeyCode::Digit1, 0), (KeyCode::Digit2, 1), (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3), (KeyCode::Digit5, 4), (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6), (KeyCode::Digit8, 7), (KeyCode::Digit9, 8),
        (KeyCode::Digit0, 9),
    ];

    for (key, idx) in item_keys {
        if keys.just_pressed(key) && idx < db.items.len() {
            let item = &db.items[idx];
            if gold.0 >= item.cost as f32 && inventory.items.len() < 6 {
                if net.connected {
                    crate::net_plugin::send_buy_item(&net, item.id);
                }
                gold.0 -= item.cost as f32;
                inventory.items.push(item.id);
                commands.entity(entity).insert(InventoryChanged);
                println!("Bought {} for {}g", item.name, item.cost);
            }
        }
    }
}
