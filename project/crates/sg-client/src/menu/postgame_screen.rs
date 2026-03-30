use bevy::prelude::*;
use super::{AppState, MenuUI};
use super::styles::*;
use sg_core::components::{GameResult, GameStats};

#[derive(Component)] pub struct ContinueButton;
#[derive(Component)] pub struct HonorButton(pub u8);

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>, ui_assets: Res<UiAssets>, game_result: Option<Res<GameResult>>, player_stats: Query<&GameStats, With<sg_core::components::PlayerControlled>>) {
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: Val::Px(30.0), ..default() },
        MenuUI,
    )).with_children(|root| {
        // Background
        root.spawn((
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, ..default() },
            ImageNode::new(ui_assets.postgame_background.clone()),
        ));

        // Victory / Defeat
        let (result_text, result_color) = match &game_result {
            Some(r) if r.victory => ("VICTORY", GOLD),
            Some(_) => ("DEFEAT", RED),
            None => ("VICTORY", GOLD),
        };
        root.spawn((Text::new(result_text), heading_font(&fonts, 72.0), TextColor(result_color)));

        // Duration
        if let Some(r) = &game_result {
            let mins = (r.game_duration / 60.0) as u32;
            let secs = (r.game_duration % 60.0) as u32;
            root.spawn((Text::new(format!("Game Duration: {}:{:02}", mins, secs)), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE)));
        }

        // Honor section
        root.spawn((Text::new("HONOR A TEAMMATE"), heading_font(&fonts, FONT_HEADING), TextColor(GOLD_LIGHT)));

        root.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(30.0), ..default() }).with_children(|honors| {
            for i in 0..2 {
                honors.spawn((
                    Node { width: Val::Px(200.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(10.0), padding: UiRect::all(Val::Px(15.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                    BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY),
                )).with_children(|card| {
                    card.spawn((
                        Node { width: Val::Px(60.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(DARK_BG),
                    )).with_children(|icon| {
                        icon.spawn((Text::new("?"), heading_font(&fonts, FONT_HEADING), TextColor(BORDER_GRAY)));
                    });
                    card.spawn((Text::new(format!("Ally {}", i + 1)), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE)));

                    card.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(5.0), ..default() }).with_children(|btns| {
                        for (idx, label) in [(0, "Cool"), (1, "Lead"), (2, "GG")] {
                            btns.spawn((
                                Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                                BackgroundColor(Color::srgba(0.004, 0.04, 0.075, 0.85)), BorderColor::all(GOLD_BRIGHT), Interaction::default(), HonorButton(idx),
                            )).with_children(|b| {
                                b.spawn((Text::new(label), body_font(&fonts, FONT_TINY), TextColor(GOLD_LIGHT)));
                            });
                        }
                    });
                });
            }
        });

        // Stats
        root.spawn(Node { height: Val::Px(20.0), ..default() });
        root.spawn((Text::new("POST-GAME STATS"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));

        let p_stats = player_stats.iter().next();
        let kda_str = p_stats.map(|s| format!("{}/{}/{}", s.kills, s.deaths, s.assists)).unwrap_or_else(|| "0/0/0".to_string());
        let cs_str = p_stats.map(|s| format!("{}", s.cs)).unwrap_or_else(|| "0".to_string());
        let gold_str = p_stats.map(|s| format!("{:.0}", s.gold_earned)).unwrap_or_else(|| "0".to_string());
        let dur_str = game_result.as_ref().map(|r| { let m = (r.game_duration / 60.0) as u32; let sc = (r.game_duration % 60.0) as u32; format!("{}:{:02}", m, sc) }).unwrap_or_else(|| "0:00".to_string());
        let dmg_str = p_stats.map(|s| format!("{:.0}", s.damage_dealt)).unwrap_or_else(|| "0".to_string());
        let dmg_champ_str = p_stats.map(|s| format!("{:.0}", s.damage_to_champions)).unwrap_or_else(|| "0".to_string());
        let wards_str = p_stats.map(|s| format!("{}", s.wards_placed)).unwrap_or_else(|| "0".to_string());
        let multi_str = p_stats.map(|s| {
            match s.largest_multi_kill { 0 | 1 => "-".to_string(), 2 => "Double Kill".to_string(), 3 => "Triple Kill".to_string(), _ => "Quadra+".to_string() }
        }).unwrap_or_else(|| "-".to_string());
        let stat_values: Vec<(String, String)> = vec![
            ("KDA".into(), kda_str), ("CS".into(), cs_str), ("Gold".into(), gold_str),
            ("Duration".into(), dur_str), ("Damage".into(), dmg_str),
            ("To Champs".into(), dmg_champ_str), ("Wards".into(), wards_str),
            ("Multi Kill".into(), multi_str),
        ];

        root.spawn((
            Node { width: Val::Px(600.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceEvenly, padding: UiRect::all(Val::Px(15.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
            BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY),
        )).with_children(|stats_panel| {
            for (label, value) in &stat_values {
                stats_panel.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }).with_children(|col| {
                    col.spawn((Text::new(value.clone()), heading_font(&fonts, FONT_HEADING), TextColor(GOLD_LIGHT)));
                    col.spawn((Text::new(label.clone()), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                });
            }
        });

        root.spawn(Node { height: Val::Px(20.0), ..default() });

        // Continue button
        spawn_button(root, "CONTINUE", ContinueButton, 200.0, 50.0, &fonts);
    });
}

pub fn interactions(
    continue_q: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &continue_q {
        if *interaction == Interaction::Pressed { next_state.set(AppState::MainMenu); }
    }
}
