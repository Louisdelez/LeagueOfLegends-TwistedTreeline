use bevy::prelude::*;
use super::{AppState, MenuUI};
use super::styles::*;
use super::data::PlayerProfile;
use sg_gameplay::champions::{ChampionClass, get_champion};

#[derive(Resource)]
pub struct LoadingTimer(pub f32);

#[derive(Component)] pub struct LoadingBarFill;
#[derive(Component)] pub struct LoadingText;

pub fn setup(mut commands: Commands, profile: Res<PlayerProfile>, asset_server: Res<AssetServer>, fonts: Res<UiFonts>) {
    commands.insert_resource(LoadingTimer(0.0));

    let class = profile.preferred_champion.unwrap_or(ChampionClass::Mage);
    let def = get_champion(class);
    let portrait = match class { ChampionClass::Mage => "ui/portraits/annie.png", ChampionClass::Fighter => "ui/portraits/garen.png", _ => "ui/portraits/annie.png" };

    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: Val::Px(30.0), ..default() },
        BackgroundColor(DARK_BG), MenuUI,
    )).with_children(|root| {
        root.spawn((Text::new("LOADING"), heading_font(&fonts, FONT_TITLE), TextColor(GOLD)));

        // Player cards row
        root.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(20.0), ..default() }).with_children(|cards| {
            // Blue team
            for i in 0..3 {
                let is_player = i == 0;
                cards.spawn((
                    Node { width: Val::Px(150.0), height: Val::Px(200.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(5.0), border: UiRect::all(Val::Px(2.0)), ..default() },
                    BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BLUE_TEAM),
                )).with_children(|card| {
                    if is_player {
                        card.spawn((Node { width: Val::Px(80.0), height: Val::Px(80.0), ..default() }, ImageNode::new(asset_server.load(portrait))));
                        card.spawn((Text::new(&profile.summoner_name), body_semibold_font(&fonts, FONT_SMALL), TextColor(GOLD_LIGHT)));
                        card.spawn((Text::new(def.name), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                        card.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(4.0), ..default() }).with_children(|spells| {
                            spells.spawn((Node { width: Val::Px(24.0), height: Val::Px(24.0), ..default() }, ImageNode::new(asset_server.load(profile.spell_d.icon_path()))));
                            spells.spawn((Node { width: Val::Px(24.0), height: Val::Px(24.0), ..default() }, ImageNode::new(asset_server.load(profile.spell_f.icon_path()))));
                        });
                    } else {
                        card.spawn((Node { width: Val::Px(80.0), height: Val::Px(80.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(DARK_BG))).with_children(|icon| {
                            icon.spawn((Text::new("?"), heading_font(&fonts, FONT_HEADING), TextColor(BORDER_GRAY)));
                        });
                        card.spawn((Text::new(format!("Ally {}", i + 1)), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));
                    }
                });
            }

            // VS
            cards.spawn(Node { width: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }).with_children(|vs| {
                vs.spawn((Text::new("VS"), heading_font(&fonts, FONT_HEADING), TextColor(GOLD)));
            });

            // Red team
            for i in 0..3 {
                cards.spawn((
                    Node { width: Val::Px(150.0), height: Val::Px(200.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(5.0), border: UiRect::all(Val::Px(2.0)), ..default() },
                    BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(RED_TEAM),
                )).with_children(|card| {
                    card.spawn((Node { width: Val::Px(80.0), height: Val::Px(80.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(DARK_BG))).with_children(|icon| {
                        icon.spawn((Text::new("?"), heading_font(&fonts, FONT_HEADING), TextColor(BORDER_GRAY)));
                    });
                    card.spawn((Text::new(format!("Enemy {}", i + 1)), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));
                });
            }
        });

        root.spawn((Text::new("Tip: Use the Speed Shrine in the center of the map for a quick burst of movement speed!"), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));

        // Loading bar
        root.spawn((
            Node { width: Val::Px(600.0), height: Val::Px(16.0), border: UiRect::all(Val::Px(1.0)), ..default() },
            BackgroundColor(DARK_NAVY), BorderColor::all(GOLD_BRIGHT),
        )).with_children(|bar| {
            bar.spawn((Node { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(GOLD), LoadingBarFill));
        });

        root.spawn((Text::new("Loading... 0%"), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE), LoadingText));
    });
}

pub fn tick(
    time: Res<Time>,
    mut timer: ResMut<LoadingTimer>,
    mut bar_q: Query<&mut Node, With<LoadingBarFill>>,
    mut text_q: Query<&mut Text, With<LoadingText>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    timer.0 += time.delta_secs() * 25.0;
    let pct = timer.0.min(100.0);

    if let Ok(mut node) = bar_q.single_mut() { node.width = Val::Percent(pct); }
    if let Ok(mut text) = text_q.single_mut() { **text = format!("Loading... {:.0}%", pct); }

    if pct >= 100.0 {
        next_state.set(AppState::InGame);
    }
}
