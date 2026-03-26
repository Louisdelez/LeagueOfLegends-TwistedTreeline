use bevy::prelude::*;
use super::{AppState, MenuUI};
use super::styles::*;
use super::data::PlayerProfile;

#[derive(Component)] pub struct PlayButton;
#[derive(Component)] pub struct ProfileButton;
#[derive(Component)] pub struct CollectionButton;
#[derive(Component)] pub struct SettingsButton;
#[derive(Component)] struct HomeTab;
#[derive(Component)] struct SubTab;

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>, ui_assets: Res<UiAssets>, profile: Res<PlayerProfile>, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.01, 0.04, 0.08)),
        MenuUI,
    )).with_children(|root| {
        // ═══════════════════════════════════════════════════════
        // TOP NAV BAR
        // [Logo] [PLAY] | Home  Profile  Collection  TFT |    [BE] [RP] [PURCHASE RP] [profile]
        // ═══════════════════════════════════════════════════════
        root.spawn((
            Node {
                width: Val::Percent(100.0), height: Val::Px(48.0),
                flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(10.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.01, 0.02, 0.05)),
            BorderColor::all(Color::srgba(0.2, 0.18, 0.12, 0.5)),
        )).with_children(|nav| {
            // LoL logo
            nav.spawn((
                Node { width: Val::Px(22.0), height: Val::Px(22.0), margin: UiRect::right(Val::Px(10.0)), ..default() },
                ImageNode::new(ui_assets.league_logo.clone()),
            ));

            // PLAY button — the real one has a distinctive shape with gold/green border
            nav.spawn((
                Node {
                    width: Val::Px(80.0), height: Val::Px(32.0),
                    justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    margin: UiRect::right(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.15, 0.10, 0.9)),
                BorderColor::all(Color::srgb(0.40, 0.70, 0.45)),
                Interaction::default(), PlayButton,
            )).with_children(|btn| {
                btn.spawn((Text::new("PLAY"), heading_font(&fonts, 14.0), TextColor(Color::srgb(0.75, 0.88, 0.78))));
            });

            // Separator
            nav.spawn((Node { width: Val::Px(1.0), height: Val::Px(24.0), margin: UiRect::right(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.4))));

            // Main nav tabs
            spawn_top_tab(nav, "Home", HomeTab, &fonts, true);
            spawn_top_tab(nav, "Profile", ProfileButton, &fonts, false);
            spawn_top_tab(nav, "Collection", CollectionButton, &fonts, false);
            spawn_top_tab(nav, "Teamfight Tactics", SubTab, &fonts, false);

            // Spacer
            nav.spawn(Node { flex_grow: 1.0, ..default() });

            // Currencies
            nav.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(6.0), margin: UiRect::right(Val::Px(8.0)), ..default() }).with_children(|cur| {
                // BE icon + amount
                cur.spawn((
                    Node { width: Val::Px(14.0), height: Val::Px(14.0), ..default() },
                    ImageNode::new(asset_server.load("ui/icons/coin-currency.png")),
                ));
                cur.spawn((Text::new("59"), body_font(&fonts, 12.0), TextColor(Color::srgb(0.16, 0.69, 0.84))));

                cur.spawn(Node { width: Val::Px(6.0), ..default() });

                // RP icon + amount
                cur.spawn((
                    Node { width: Val::Px(14.0), height: Val::Px(14.0), ..default() },
                    ImageNode::new(asset_server.load("ui/icons/store-rp.png")),
                ));
                cur.spawn((Text::new("3547"), body_font(&fonts, 12.0), TextColor(Color::srgb(0.85, 0.55, 0.20))));
            });

            // PURCHASE RP button
            nav.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    margin: UiRect::right(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.9)),
                BorderColor::all(GOLD_BRIGHT),
            )).with_children(|btn| {
                btn.spawn((Text::new("PURCHASE RP"), body_font(&fonts, 10.0), TextColor(GOLD_LIGHT)));
            });

            // Profile summoner name + icon
            nav.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(6.0), ..default() }).with_children(|prof| {
                prof.spawn((Text::new(&profile.summoner_name), body_font(&fonts, 12.0), TextColor(TEXT_BRIGHT)));
                // Mini profile icon
                prof.spawn((
                    Node {
                        width: Val::Px(28.0), height: Val::Px(28.0),
                        justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.06, 0.08, 0.14)),
                    BorderColor::all(GOLD_BRIGHT),
                )).with_children(|ic| {
                    ic.spawn((Text::new("?"), body_font(&fonts, 12.0), TextColor(GOLD)));
                });
            });

            // Settings gear
            nav.spawn((
                Node { padding: UiRect::all(Val::Px(6.0)), margin: UiRect::left(Val::Px(4.0)), ..default() },
                Interaction::default(), SettingsButton,
            )).with_children(|s| {
                s.spawn((Text::new("\u{2699}"), body_font(&fonts, 16.0), TextColor(TEXT_WHITE)));
            });
        });

        // ═══════════════════════════════════════════════════════
        // SUB-NAV BAR (Featured, Champions, Skins, Loot, etc.)
        // ═══════════════════════════════════════════════════════
        root.spawn((
            Node {
                width: Val::Percent(100.0), height: Val::Px(32.0),
                flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(10.0)),
                column_gap: Val::Px(4.0),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.04, 0.07)),
            BorderColor::all(Color::srgba(0.15, 0.14, 0.10, 0.4)),
        )).with_children(|sub| {
            for (i, label) in ["Featured", "Champions", "Skins", "Loot", "Emotes", "Accessories", "Esports"].iter().enumerate() {
                let active = i == 0;
                sub.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        border: UiRect::bottom(Val::Px(if active { 2.0 } else { 0.0 })),
                        ..default()
                    },
                    BorderColor::all(if active { GOLD } else { TRANSPARENT }),
                )).with_children(|tab| {
                    tab.spawn((Text::new(label.to_string()), body_font(&fonts, 12.0), TextColor(if active { TEXT_BRIGHT } else { TEXT_WHITE })));
                });
            }
        });

        // ═══════════════════════════════════════════════════════
        // MAIN CONTENT: [Featured area + cards] | [Friends sidebar]
        // ═══════════════════════════════════════════════════════
        root.spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            ..default()
        }).with_children(|main| {
            // ── LEFT: Main content ──
            main.spawn(Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(10.0),
                overflow: Overflow::clip(),
                ..default()
            }).with_children(|content| {
                // Top row: Featured banner (left) + Promo cards grid (right)
                content.spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                }).with_children(|top_row| {
                    // ── FEATURED BANNER (large, left side) ──
                    top_row.spawn((
                        Node {
                            width: Val::Percent(55.0),
                            height: Val::Px(340.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::FlexEnd,
                            padding: UiRect::all(Val::Px(20.0)),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        ImageNode::new(asset_server.load("ui/backgrounds/split-start-background.png")),
                    )).with_children(|banner| {
                        banner.spawn((
                            Node { padding: UiRect::all(Val::Px(8.0)), ..default() },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                        )).with_children(|overlay| {
                            overlay.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() }).with_children(|txt| {
                                txt.spawn((Text::new("NEW MAP"), body_font(&fonts, 10.0), TextColor(Color::srgb(0.6, 0.75, 0.9))));
                                txt.spawn((Text::new("TWISTED TREELINE"), heading_font(&fonts, FONT_HEADING), TextColor(Color::WHITE)));
                                txt.spawn((Text::new("The 3v3 arena is back. Battle on the Shadow Isles."), body_font(&fonts, 12.0), TextColor(TEXT_BRIGHT)));
                            });
                        });
                    });

                    // ── PROMO CARDS GRID (right side, 2x2) ──
                    top_row.spawn(Node {
                        width: Val::Percent(45.0),
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(8.0),
                        row_gap: Val::Px(8.0),
                        ..default()
                    }).with_children(|grid| {
                        spawn_promo_card(grid, &fonts, "3v3 Ranked", "Climb the ladder\nin Twisted Treeline", Color::srgb(0.08, 0.12, 0.20));
                        spawn_promo_card(grid, &fonts, "Champion Sale", "50% off select\nchampions", Color::srgb(0.12, 0.06, 0.15));
                        spawn_promo_card(grid, &fonts, "Free Rotation", "Annie, Garen, Lux\nThresh, Jinx +4", Color::srgb(0.06, 0.12, 0.10));
                        spawn_promo_card(grid, &fonts, "Vilemaw Event", "Defeat Vilemaw\n10 times for rewards", Color::srgb(0.14, 0.08, 0.06));
                    });
                });

                // ── BOTTOM: Top Sellers / Recent section ──
                content.spawn((Text::new("TOP SELLERS"), heading_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));

                content.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                }).with_children(|sellers| {
                    for (name, price) in [("Garen Skin", "975 RP"), ("Annie Skin", "750 RP"), ("Thresh Skin", "1350 RP"), ("Jinx Skin", "1820 RP"), ("Darius Skin", "975 RP")] {
                        sellers.spawn((
                            Node {
                                width: Val::Px(120.0), height: Val::Px(70.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexEnd,
                                padding: UiRect::all(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.06, 0.08, 0.14)),
                            BorderColor::all(Color::srgba(0.2, 0.2, 0.2, 0.5)),
                        )).with_children(|card| {
                            card.spawn((Text::new(name), body_font(&fonts, 10.0), TextColor(TEXT_BRIGHT)));
                            card.spawn((Text::new(price), body_font(&fonts, 10.0), TextColor(Color::srgb(0.85, 0.55, 0.20))));
                        });
                    }
                });
            });

            // ── RIGHT: Social / Friends sidebar ──
            main.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::left(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.02, 0.03, 0.06)),
                BorderColor::all(Color::srgba(0.2, 0.18, 0.12, 0.4)),
            )).with_children(|sidebar| {
                // Header
                sidebar.spawn((
                    Node {
                        width: Val::Percent(100.0), height: Val::Px(32.0),
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.03, 0.04, 0.07)),
                    BorderColor::all(Color::srgba(0.2, 0.18, 0.12, 0.3)),
                )).with_children(|hdr| {
                    hdr.spawn((Text::new("Social"), body_semibold_font(&fonts, 12.0), TextColor(TEXT_BRIGHT)));
                    hdr.spawn((Text::new("Friends Requests"), body_font(&fonts, 10.0), TextColor(TEXT_WHITE)));
                });

                // Chat tabs (GENERAL / PARTY)
                sidebar.spawn((
                    Node {
                        width: Val::Percent(100.0), height: Val::Px(28.0),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                )).with_children(|tabs| {
                    for (label, active) in [("GENERAL (1/19)", true), ("PARTY", false)] {
                        tabs.spawn((
                            Node {
                                flex_grow: 1.0,
                                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                                border: UiRect::bottom(Val::Px(if active { 2.0 } else { 0.0 })),
                                ..default()
                            },
                            BackgroundColor(if active { Color::srgba(0.04, 0.06, 0.10, 0.8) } else { Color::srgba(0.02, 0.03, 0.06, 0.8) }),
                            BorderColor::all(if active { GOLD } else { TRANSPARENT }),
                        )).with_children(|t| {
                            t.spawn((Text::new(label), body_font(&fonts, 10.0), TextColor(if active { TEXT_BRIGHT } else { TEXT_WHITE })));
                        });
                    }
                });

                // Friends list
                sidebar.spawn(Node {
                    flex_grow: 1.0, flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(4.0),
                    ..default()
                }).with_children(|list| {
                    // Online friends
                    for name in ["Skyra", "DarkMoonfissure", "MichelMorgan"] {
                        spawn_friend_entry(list, &fonts, name, true);
                    }

                    list.spawn(Node { height: Val::Px(8.0), ..default() });

                    // Offline
                    list.spawn((Text::new("OFFLINE"), body_font(&fonts, 9.0), TextColor(Color::srgb(0.35, 0.35, 0.38))));
                    for name in ["ZAMRIsGripka", "Player42"] {
                        spawn_friend_entry(list, &fonts, name, false);
                    }
                });
            });
        });
    });
}

fn spawn_top_tab(parent: &mut ChildSpawnerCommands<'_>, text: &str, marker: impl Component, fonts: &UiFonts, active: bool) {
    parent.spawn((
        Node {
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            ..default()
        },
        Interaction::default(), marker,
    )).with_children(|btn| {
        btn.spawn((Text::new(text), body_font(fonts, 13.0), TextColor(if active { TEXT_BRIGHT } else { TEXT_WHITE })));
    });
}

fn spawn_promo_card(parent: &mut ChildSpawnerCommands<'_>, fonts: &UiFonts, title: &str, desc: &str, bg: Color) {
    parent.spawn((
        Node {
            width: Val::Percent(48.0), height: Val::Px(166.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(bg),
        BorderColor::all(Color::srgba(0.25, 0.22, 0.15, 0.4)),
    )).with_children(|card| {
        card.spawn((Text::new(title), heading_font(fonts, 13.0), TextColor(GOLD_LIGHT)));
        card.spawn((Text::new(desc), body_font(fonts, 10.0), TextColor(TEXT_WHITE)));
    });
}

fn spawn_friend_entry(parent: &mut ChildSpawnerCommands<'_>, fonts: &UiFonts, name: &str, online: bool) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
        column_gap: Val::Px(6.0), padding: UiRect::vertical(Val::Px(2.0)),
        ..default()
    }).with_children(|row| {
        // Status dot
        let dot_color = if online { Color::srgb(0.2, 0.75, 0.4) } else { Color::srgb(0.35, 0.35, 0.38) };
        row.spawn((
            Node { width: Val::Px(8.0), height: Val::Px(8.0), ..default() },
            BackgroundColor(dot_color),
        ));
        // Icon placeholder
        row.spawn((
            Node { width: Val::Px(22.0), height: Val::Px(22.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgb(0.06, 0.08, 0.14)),
        )).with_children(|ic| {
            ic.spawn((Text::new("?"), body_font(fonts, 9.0), TextColor(Color::srgb(0.3, 0.3, 0.35))));
        });
        // Name
        let name_color = if online { TEXT_BRIGHT } else { Color::srgb(0.4, 0.4, 0.42) };
        row.spawn((Text::new(name), body_font(fonts, 11.0), TextColor(name_color)));
    });
}

pub fn interactions(
    play_q: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    profile_q: Query<&Interaction, (Changed<Interaction>, With<ProfileButton>)>,
    collection_q: Query<&Interaction, (Changed<Interaction>, With<CollectionButton>)>,
    settings_q: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &play_q {
        if *interaction == Interaction::Pressed { next_state.set(AppState::PlayLobby); }
    }
    for interaction in &profile_q {
        if *interaction == Interaction::Pressed { next_state.set(AppState::Profile); }
    }
    for interaction in &collection_q {
        if *interaction == Interaction::Pressed { next_state.set(AppState::Collection); }
    }
    for interaction in &settings_q {
        if *interaction == Interaction::Pressed { next_state.set(AppState::Settings); }
    }
}
