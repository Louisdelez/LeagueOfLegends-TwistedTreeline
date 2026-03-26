use bevy::prelude::*;
use super::{AppState, MenuUI};
use super::styles::*;

#[derive(Component)] pub struct BackButton;

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>, ui_assets: Res<UiAssets>) {
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
            nav.spawn((Text::new("SETTINGS"), heading_font(&fonts, FONT_HEADING), TextColor(GOLD)));
        });

        // Content
        root.spawn(Node { flex_grow: 1.0, padding: UiRect::all(Val::Px(40.0)), flex_direction: FlexDirection::Row, column_gap: Val::Px(40.0), ..default() }).with_children(|main| {
            // Left: categories
            main.spawn(Node { width: Val::Px(200.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(10.0), ..default() }).with_children(|cats| {
                for label in ["VIDEO", "AUDIO", "CONTROLS", "INTERFACE"] {
                    cats.spawn((
                        Node { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(10.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY),
                    )).with_children(|c| {
                        c.spawn((Text::new(label.to_string()), body_semibold_font(&fonts, FONT_BODY), TextColor(GOLD_LIGHT)));
                    });
                }
            });

            // Right: settings
            main.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, row_gap: Val::Px(15.0), ..default() }).with_children(|settings| {
                settings.spawn((Text::new("VIDEO SETTINGS"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));

                for (label, value) in [("Resolution", "1920x1080"), ("Fullscreen", "Enabled"), ("Quality", "High"), ("VSync", "On"), ("Shadow Quality", "High"), ("Anti-Aliasing", "4x MSAA")] {
                    settings.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, width: Val::Px(500.0), ..default() }).with_children(|row| {
                        row.spawn((Text::new(label), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE)));
                        row.spawn((
                            Node { padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                            BackgroundColor(Color::srgba(0.004, 0.04, 0.075, 0.85)), BorderColor::all(BORDER_GRAY),
                        )).with_children(|val| {
                            val.spawn((Text::new(value), body_font(&fonts, FONT_BODY), TextColor(GOLD_LIGHT)));
                        });
                    });
                }

                settings.spawn(Node { height: Val::Px(20.0), ..default() });
                settings.spawn((Text::new("AUDIO SETTINGS"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));

                for (label, value) in [("Master Volume", "100%"), ("Music Volume", "80%"), ("Effects Volume", "100%"), ("Voice Volume", "100%")] {
                    settings.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, width: Val::Px(500.0), ..default() }).with_children(|row| {
                        row.spawn((Text::new(label), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE)));
                        row.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|vol| {
                            vol.spawn((
                                Node { width: Val::Px(150.0), height: Val::Px(8.0), border: UiRect::all(Val::Px(1.0)), ..default() },
                                BackgroundColor(DARK_BG), BorderColor::all(BORDER_GRAY),
                            )).with_children(|bar| {
                                bar.spawn((Node { width: Val::Percent(80.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(GOLD)));
                            });
                            vol.spawn((Text::new(value), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));
                        });
                    });
                }

                settings.spawn(Node { height: Val::Px(20.0), ..default() });
                settings.spawn((Text::new("CONTROLS"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));

                for (action, key) in [("Move", "Right Click"), ("Ability Q", "A"), ("Ability W", "W"), ("Ability E", "E"), ("Ability R", "R"), ("Shop", "P"), ("Recall", "B"), ("Debug", "F3")] {
                    settings.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, width: Val::Px(500.0), ..default() }).with_children(|row| {
                        row.spawn((Text::new(action), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE)));
                        row.spawn((
                            Node { padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                            BackgroundColor(Color::srgba(0.004, 0.04, 0.075, 0.85)), BorderColor::all(GOLD_BRIGHT),
                        )).with_children(|val| {
                            val.spawn((Text::new(key), body_font(&fonts, FONT_BODY), TextColor(GOLD_LIGHT)));
                        });
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
