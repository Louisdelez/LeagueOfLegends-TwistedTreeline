use bevy::prelude::*;
use super::{AppState, MenuUI};
use super::styles::*;
use super::data::QueueType;

#[derive(Component)] pub struct BackButton;
#[derive(Component)] pub struct FindMatchButton;
#[derive(Component)] pub struct QueueOption(pub QueueType);
#[derive(Resource)] pub struct LobbyState { pub selected_queue: QueueType, pub searching: bool, pub search_timer: f32 }
impl Default for LobbyState { fn default() -> Self { Self { selected_queue: QueueType::BlindPick, searching: false, search_timer: 0.0 } } }
#[derive(Component)] pub struct SelectedQueueText;

pub fn setup(mut commands: Commands, fonts: Res<UiFonts>, ui_assets: Res<UiAssets>) {
    commands.insert_resource(LobbyState::default());

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
            nav.spawn((Text::new("SELECT GAME MODE"), heading_font(&fonts, FONT_HEADING), TextColor(GOLD)));
        });

        // Content
        root.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Row, padding: UiRect::all(Val::Px(40.0)), column_gap: Val::Px(40.0), ..default() }).with_children(|main| {
            // Left: Queue types
            main.spawn(Node { width: Val::Px(400.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(12.0), ..default() }).with_children(|list| {
                list.spawn((Text::new("QUEUE TYPE"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));

                for qt in [QueueType::BlindPick, QueueType::DraftPick, QueueType::Ranked, QueueType::Custom, QueueType::Practice] {
                    list.spawn((
                        Node { width: Val::Percent(100.0), height: Val::Px(60.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, padding: UiRect::all(Val::Px(12.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY), Interaction::default(), QueueOption(qt),
                    )).with_children(|card| {
                        card.spawn((Text::new(qt.name()), body_semibold_font(&fonts, FONT_BODY), TextColor(GOLD_LIGHT)));
                        card.spawn((Text::new(qt.description()), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                    });
                }
            });

            // Right: Lobby
            main.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: Val::Px(20.0), ..default() }).with_children(|lobby| {
                lobby.spawn((Text::new("TWISTED TREELINE"), heading_font(&fonts, FONT_TITLE), TextColor(GOLD)));
                lobby.spawn((Text::new("3v3 Arena"), body_font(&fonts, FONT_SUBHEADING), TextColor(TEXT_BRIGHT)));

                lobby.spawn(Node { height: Val::Px(20.0), ..default() });

                lobby.spawn((Text::new("Normal (Blind Pick)"), body_semibold_font(&fonts, FONT_BODY), TextColor(GOLD_BRIGHT), SelectedQueueText));

                // Party slots
                lobby.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(20.0), ..default() }).with_children(|slots| {
                    for i in 0..3 {
                        slots.spawn((
                            Node { width: Val::Px(80.0), height: Val::Px(80.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
                            BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(if i == 0 { GOLD } else { BORDER_GRAY }),
                        )).with_children(|slot| {
                            let text = if i == 0 { "YOU" } else { "+" };
                            slot.spawn((Text::new(text), heading_font(&fonts, FONT_BODY), TextColor(if i == 0 { GOLD_LIGHT } else { TEXT_WHITE })));
                        });
                    }
                });

                lobby.spawn(Node { height: Val::Px(30.0), ..default() });

                spawn_primary_button(lobby, "FIND MATCH", FindMatchButton, &fonts);
            });
        });
    });
}

pub fn interactions(
    time: Res<Time>,
    back_q: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    find_q: Query<&Interaction, (Changed<Interaction>, With<FindMatchButton>)>,
    queue_q: Query<(&Interaction, &QueueOption), Changed<Interaction>>,
    mut lobby: ResMut<LobbyState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut text_q: Query<&mut Text, With<SelectedQueueText>>,
) {
    for interaction in &back_q {
        if *interaction == Interaction::Pressed { next_state.set(AppState::MainMenu); }
    }
    for (interaction, opt) in &queue_q {
        if *interaction == Interaction::Pressed {
            lobby.selected_queue = opt.0;
            if let Ok(mut text) = text_q.single_mut() {
                **text = opt.0.name().to_string();
            }
        }
    }
    for interaction in &find_q {
        if *interaction == Interaction::Pressed {
            if !lobby.searching {
                lobby.searching = true;
                lobby.search_timer = 0.0;
            }
        }
    }
    if lobby.searching {
        lobby.search_timer += time.delta_secs();
        if lobby.search_timer >= 2.0 {
            lobby.searching = false;
            next_state.set(AppState::ChampionSelect);
        }
    }
}
