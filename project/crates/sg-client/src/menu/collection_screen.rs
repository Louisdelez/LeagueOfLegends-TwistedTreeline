use bevy::prelude::*;
use super::{AppState, MenuUI};
use super::styles::*;
use sg_gameplay::champions::{ChampionClass, get_champion};
use sg_core::spells::SummonerSpell;
use sg_core::runes::*;

#[derive(Component)] pub struct BackButton;
#[derive(Component)] pub struct TabButton(pub u8);

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, fonts: Res<UiFonts>, ui_assets: Res<UiAssets>) {
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() },
        MenuUI,
    )).with_children(|root| {
        // Background
        root.spawn((
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, ..default() },
            ImageNode::new(ui_assets.main_background.clone()),
        ));

        // Top bar
        root.spawn((
            Node { width: Val::Percent(100.0), height: Val::Px(60.0), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, padding: UiRect::horizontal(Val::Px(20.0)), column_gap: Val::Px(15.0), border: UiRect::bottom(Val::Px(1.0)), ..default() },
            BackgroundColor(Color::srgba(0.004, 0.04, 0.075, 0.85)), BorderColor::all(GOLD_BRIGHT),
        )).with_children(|nav| {
            spawn_button(nav, "< BACK", BackButton, 100.0, 36.0, &fonts);
            nav.spawn((Text::new("COLLECTION"), heading_font(&fonts, FONT_HEADING), TextColor(GOLD)));
            nav.spawn(Node { width: Val::Px(30.0), ..default() });
            spawn_nav_tab(nav, "CHAMPIONS", TabButton(0), &fonts);
            spawn_nav_tab(nav, "RUNES", TabButton(1), &fonts);
            spawn_nav_tab(nav, "SPELLS", TabButton(2), &fonts);
        });

        // Content
        root.spawn(Node { flex_grow: 1.0, padding: UiRect::all(Val::Px(30.0)), flex_direction: FlexDirection::Column, row_gap: Val::Px(15.0), overflow: Overflow::scroll_y(), ..default() }).with_children(|content| {
            content.spawn((Text::new("CHAMPIONS"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));

            // Champion grid
            content.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(20.0), row_gap: Val::Px(20.0), ..default() }).with_children(|grid| {
                for class in [ChampionClass::Mage, ChampionClass::Fighter, ChampionClass::Tank] {
                    let def = get_champion(class);
                    let portrait = match class { ChampionClass::Mage => "ui/portraits/annie.png", ChampionClass::Fighter => "ui/portraits/garen.png", _ => "ui/portraits/annie.png" };

                    grid.spawn((
                        Node { width: Val::Px(200.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(12.0)), row_gap: Val::Px(8.0), border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(GOLD_BRIGHT),
                    )).with_children(|card| {
                        card.spawn((Node { width: Val::Px(120.0), height: Val::Px(120.0), ..default() }, ImageNode::new(asset_server.load(portrait))));
                        card.spawn((Text::new(def.name), heading_font(&fonts, FONT_BODY), TextColor(GOLD_LIGHT)));
                        card.spawn((Text::new(def.title), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                        card.spawn((Text::new(format!("HP: {} | AD: {} | Armor: {}", def.hp, def.ad, def.armor)), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                        card.spawn((Text::new(format!("Range: {} | MS: {}", def.attack_range, def.move_speed)), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                    });
                }
            });

            content.spawn(Node { height: Val::Px(20.0), ..default() });

            // Spells section
            content.spawn((Text::new("SUMMONER SPELLS"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));
            content.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(10.0), row_gap: Val::Px(10.0), ..default() }).with_children(|spells| {
                for spell in SummonerSpell::all() {
                    spells.spawn((
                        Node { width: Val::Px(250.0), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(8.0), padding: UiRect::all(Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY),
                    )).with_children(|row| {
                        row.spawn((Node { width: Val::Px(32.0), height: Val::Px(32.0), ..default() }, ImageNode::new(asset_server.load(spell.icon_path()))));
                        row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|info| {
                            info.spawn((Text::new(spell.name()), body_semibold_font(&fonts, FONT_SMALL), TextColor(GOLD_LIGHT)));
                            info.spawn((Text::new(format!("CD: {}s", spell.cooldown())), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                        });
                    });
                }
            });

            content.spawn(Node { height: Val::Px(20.0), ..default() });

            // Rune paths
            content.spawn((Text::new("RUNE PATHS"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));
            content.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(15.0), ..default() }).with_children(|paths| {
                for path in RunePath::all() {
                    let rgb = path.color_rgb();
                    paths.spawn((
                        Node { width: Val::Px(160.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(10.0)), row_gap: Val::Px(5.0), border: UiRect::all(Val::Px(2.0)), ..default() },
                        BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(Color::srgb(rgb[0], rgb[1], rgb[2])),
                    )).with_children(|card| {
                        card.spawn((Text::new(path.name()), heading_font(&fonts, FONT_BODY), TextColor(Color::srgb(rgb[0], rgb[1], rgb[2]))));
                        for ks in Keystone::keystones_for(*path) {
                            card.spawn((Text::new(format!("  {}", ks.name())), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                        }
                    });
                }
            });
        });
    });
}

pub fn interactions(
    back_q: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &back_q {
        if *interaction == Interaction::Pressed { next_state.set(AppState::MainMenu); }
    }
}
