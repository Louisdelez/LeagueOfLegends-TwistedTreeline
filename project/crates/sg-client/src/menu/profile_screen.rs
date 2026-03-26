use bevy::prelude::*;
use super::{AppState, MenuUI};
use super::styles::*;
use super::data::*;

#[derive(Component)] pub struct BackButton;

pub fn setup(mut commands: Commands, profile: Res<PlayerProfile>, fonts: Res<UiFonts>, ui_assets: Res<UiAssets>, asset_server: Res<AssetServer>) {
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
            nav.spawn((Text::new("PROFILE"), heading_font(&fonts, FONT_HEADING), TextColor(GOLD)));
        });

        // Content
        root.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Row, padding: UiRect::all(Val::Px(40.0)), column_gap: Val::Px(40.0), ..default() }).with_children(|main| {
            // Left: Player identity
            main.spawn((
                Node { width: Val::Px(300.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(15.0), padding: UiRect::all(Val::Px(20.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(GOLD_BRIGHT),
            )).with_children(|left| {
                // Icon placeholder
                left.spawn((
                    Node { width: Val::Px(100.0), height: Val::Px(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(3.0)), ..default() },
                    BackgroundColor(DARK_BG), BorderColor::all(GOLD),
                )).with_children(|icon| {
                    icon.spawn((Text::new("?"), heading_font(&fonts, FONT_TITLE), TextColor(GOLD)));
                });

                left.spawn((Text::new(&profile.summoner_name), heading_font(&fonts, FONT_HEADING), TextColor(GOLD_LIGHT)));
                left.spawn((Text::new(format!("Level {}", profile.level)), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE)));
                left.spawn((Text::new(format!("Honor Level {}", profile.honor_level)), body_font(&fonts, FONT_SMALL), TextColor(GREEN)));
            });

            // Center: Rank with emblem image
            main.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(15.0), ..default() }).with_children(|center| {
                center.spawn((Text::new("RANKED 3v3"), heading_font(&fonts, FONT_HEADING), TextColor(GOLD)));

                let rank = &profile.rank;
                let tier_color = rank.tier.color();
                let tier_bevy = Color::srgb(tier_color[0], tier_color[1], tier_color[2]);

                // Rank emblem — use extracted rank icon if available
                let rank_icon_path = match rank.tier {
                    RankedTier::Iron => "ui/rank/iron.png",
                    RankedTier::Bronze => "ui/rank/bronze.png",
                    RankedTier::Silver => "ui/rank/silver.png",
                    RankedTier::Gold => "ui/rank/gold.png",
                    RankedTier::Platinum => "ui/rank/platinum.png",
                    RankedTier::Diamond => "ui/rank/diamond.png",
                    RankedTier::Master => "ui/rank/master.png",
                    RankedTier::Grandmaster => "ui/rank/grandmaster.png",
                    RankedTier::Challenger => "ui/rank/challenger.png",
                    RankedTier::Unranked => "ui/rank/unranked.png",
                };

                center.spawn((
                    Node { width: Val::Px(200.0), height: Val::Px(200.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                    ImageNode::new(asset_server.load(rank_icon_path)),
                ));

                if rank.tier != RankedTier::Unranked {
                    center.spawn((Text::new(format!("{} {}", rank.tier.name(), rank.division.name())), heading_font(&fonts, FONT_HEADING), TextColor(tier_bevy)));
                    center.spawn((Text::new(format!("{} LP", rank.lp)), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD_LIGHT)));

                    // LP progress bar
                    center.spawn((
                        Node { width: Val::Px(300.0), height: Val::Px(12.0), border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(DARK_BG), BorderColor::all(BORDER_GRAY),
                    )).with_children(|bar| {
                        bar.spawn((
                            Node { width: Val::Percent(rank.lp as f32), height: Val::Percent(100.0), ..default() },
                            BackgroundColor(GOLD),
                        ));
                    });

                    center.spawn((Text::new(format!("{}W {}L", rank.wins, rank.losses)), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE)));
                } else {
                    center.spawn((Text::new("UNRANKED"), heading_font(&fonts, FONT_HEADING), TextColor(tier_bevy)));
                }
            });

            // Right: Match history
            main.spawn((
                Node { width: Val::Px(350.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), ..default() },
            )).with_children(|right| {
                right.spawn((Text::new("MATCH HISTORY"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));

                if profile.match_history.is_empty() {
                    right.spawn((Text::new("No matches played yet"), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE)));
                } else {
                    for m in profile.match_history.iter().rev().take(10) {
                        let result_color = if m.won { GREEN } else { RED };
                        let result_text = if m.won { "WIN" } else { "LOSS" };
                        let champ = match m.champion_class { 0 => "Pyralis", 1 => "Kael", _ => "Thornwall" };
                        right.spawn((
                            Node { width: Val::Percent(100.0), height: Val::Px(40.0), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(10.0), padding: UiRect::horizontal(Val::Px(10.0)), border: UiRect::left(Val::Px(3.0)), ..default() },
                            BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(result_color),
                        )).with_children(|row| {
                            row.spawn((Text::new(result_text), body_semibold_font(&fonts, FONT_SMALL), TextColor(result_color)));
                            row.spawn((Text::new(champ), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));
                            row.spawn((Text::new(format!("{}/{}/{}", m.kills, m.deaths, m.assists)), body_font(&fonts, FONT_SMALL), TextColor(GOLD_LIGHT)));
                            row.spawn((Text::new(format!("{}g", m.gold)), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                        });
                    }
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
