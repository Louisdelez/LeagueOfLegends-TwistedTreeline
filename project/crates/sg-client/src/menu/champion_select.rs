use bevy::prelude::*;
use super::{AppState, MenuUI};
use super::styles::*;
use super::data::*;
use sg_gameplay::champions::{ChampionClass, get_champion};
use sg_core::spells::SummonerSpell;

#[derive(Resource)]
pub struct ChampSelectState {
    pub phase: SelectPhase,
    pub timer: f32,
    pub selected_champion: Option<ChampionClass>,
    pub locked_in: bool,
    pub bans: Vec<ChampionClass>,
    pub ally_picks: Vec<Option<ChampionClass>>,
    pub enemy_picks: Vec<Option<ChampionClass>>,
    pub spell_d: SummonerSpell,
    pub spell_f: SummonerSpell,
}

impl Default for ChampSelectState {
    fn default() -> Self {
        Self {
            phase: SelectPhase::BanPhase,
            timer: 40.0,
            selected_champion: None,
            locked_in: false,
            bans: vec![],
            ally_picks: vec![None; 3],
            enemy_picks: vec![None; 3],
            spell_d: SummonerSpell::Flash,
            spell_f: SummonerSpell::Ignite,
        }
    }
}

#[derive(Component)] pub struct ChampCard(pub ChampionClass);
#[derive(Component)] pub struct LockInButton;
#[derive(Component)] pub struct TimerText;
#[derive(Component)] pub struct PhaseText;
#[derive(Component)] pub struct SpellSlot(pub usize);
#[derive(Component)] pub struct SelectedChampName;
#[derive(Component)] pub struct SpellPicker;
#[derive(Component)] pub struct SpellOption(pub SummonerSpell, pub usize);

pub fn setup(mut commands: Commands, profile: Res<PlayerProfile>, asset_server: Res<AssetServer>, fonts: Res<UiFonts>) {
    let state = ChampSelectState {
        spell_d: profile.spell_d,
        spell_f: profile.spell_f,
        ..default()
    };
    commands.insert_resource(state);

    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() },
        BackgroundColor(DARK_BG), MenuUI,
    )).with_children(|root| {
        // ─── Top bar: Phase + Timer ───
        root.spawn((
            Node { width: Val::Percent(100.0), height: Val::Px(60.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, column_gap: Val::Px(30.0), border: UiRect::bottom(Val::Px(2.0)), ..default() },
            BackgroundColor(DARK_NAVY), BorderColor::all(GOLD),
        )).with_children(|top| {
            top.spawn((Text::new("BAN PHASE"), heading_font(&fonts, FONT_HEADING), TextColor(RED), PhaseText));
            top.spawn((Text::new("40"), heading_font(&fonts, FONT_TITLE), TextColor(GOLD_LIGHT), TimerText));
        });

        // ─── Main area: Allies | Grid | Enemies ───
        root.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Row, ..default() }).with_children(|main| {
            // Ally team panel
            main.spawn((
                Node { width: Val::Px(200.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(15.0)), row_gap: Val::Px(10.0), border: UiRect::right(Val::Px(1.0)), ..default() },
                BackgroundColor(DARK_NAVY), BorderColor::all(BORDER_GRAY),
            )).with_children(|allies| {
                allies.spawn((Text::new("BLUE TEAM"), heading_font(&fonts, FONT_BODY), TextColor(BLUE_TEAM)));
                for i in 0..3 {
                    allies.spawn((
                        Node { width: Val::Percent(100.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(if i == 0 { GOLD } else { BORDER_GRAY }),
                    )).with_children(|slot| {
                        let text = if i == 0 { "YOU".to_string() } else { format!("Ally {}", i + 1) };
                        slot.spawn((Text::new(text), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));
                    });
                }
            });

            // Champion grid (center)
            main.spawn(Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(15.0), ..default() }).with_children(|center| {
                center.spawn((Text::new("SELECT A CHAMPION"), heading_font(&fonts, FONT_SUBHEADING), TextColor(GOLD)));

                // Filter buttons
                center.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() }).with_children(|filters| {
                    for class_name in ["ALL", "MAGE", "FIGHTER", "TANK"] {
                        filters.spawn((
                            Node { padding: UiRect::axes(Val::Px(15.0), Val::Px(6.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                            BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY),
                        )).with_children(|f| {
                            f.spawn((Text::new(class_name.to_string()), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));
                        });
                    }
                });

                // Champion cards
                center.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(15.0), row_gap: Val::Px(15.0), ..default() }).with_children(|grid| {
                    for class in [ChampionClass::Mage, ChampionClass::Fighter, ChampionClass::Tank] {
                        let def = get_champion(class);
                        let portrait_path = match class {
                            ChampionClass::Mage => "ui/portraits/annie.png",
                            ChampionClass::Fighter => "ui/portraits/garen.png",
                            ChampionClass::Tank => "ui/portraits/annie.png",
                        };

                        grid.spawn((
                            Node { width: Val::Px(180.0), height: Val::Px(220.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), row_gap: Val::Px(5.0), border: UiRect::all(Val::Px(2.0)), ..default() },
                            BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY), Interaction::default(), ChampCard(class),
                        )).with_children(|card| {
                            card.spawn((
                                Node { width: Val::Px(120.0), height: Val::Px(120.0), ..default() },
                                ImageNode::new(asset_server.load(portrait_path)),
                            ));
                            card.spawn((Text::new(def.name), heading_font(&fonts, FONT_BODY), TextColor(GOLD_LIGHT)));
                            card.spawn((Text::new(def.title), body_font(&fonts, FONT_TINY), TextColor(TEXT_WHITE)));
                            let class_label = match class { ChampionClass::Mage => "Mage", ChampionClass::Fighter => "Fighter", ChampionClass::Tank => "Tank" };
                            card.spawn((Text::new(class_label), body_font(&fonts, FONT_TINY), TextColor(BLUE_ACCENT)));
                        });
                    }
                });

                center.spawn(Node { height: Val::Px(10.0), ..default() });
                center.spawn((Text::new("No champion selected"), body_font(&fonts, FONT_BODY), TextColor(TEXT_WHITE), SelectedChampName));
            });

            // Enemy team panel
            main.spawn((
                Node { width: Val::Px(200.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(15.0)), row_gap: Val::Px(10.0), border: UiRect::left(Val::Px(1.0)), ..default() },
                BackgroundColor(DARK_NAVY), BorderColor::all(BORDER_GRAY),
            )).with_children(|enemies| {
                enemies.spawn((Text::new("RED TEAM"), heading_font(&fonts, FONT_BODY), TextColor(RED_TEAM)));
                for i in 0..3 {
                    enemies.spawn((
                        Node { width: Val::Percent(100.0), height: Val::Px(60.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY),
                    )).with_children(|slot| {
                        slot.spawn((Text::new(format!("Enemy {}", i + 1)), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));
                    });
                }
            });
        });

        // ─── Bottom panel: Spells + Lock In ───
        root.spawn((
            Node { width: Val::Percent(100.0), height: Val::Px(100.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, align_items: AlignItems::Center, column_gap: Val::Px(30.0), border: UiRect::top(Val::Px(2.0)), padding: UiRect::horizontal(Val::Px(30.0)), ..default() },
            BackgroundColor(DARK_NAVY), BorderColor::all(GOLD),
        )).with_children(|bottom| {
            // Summoner Spell D
            bottom.spawn((
                Node { width: Val::Px(50.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
                BackgroundColor(PANEL_BG), BorderColor::all(GOLD_BRIGHT),
                ImageNode::new(asset_server.load(profile.spell_d.icon_path())),
                Interaction::default(), SpellSlot(0),
            ));

            // Summoner Spell F
            bottom.spawn((
                Node { width: Val::Px(50.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
                BackgroundColor(PANEL_BG), BorderColor::all(GOLD_BRIGHT),
                ImageNode::new(asset_server.load(profile.spell_f.icon_path())),
                Interaction::default(), SpellSlot(1),
            ));

            bottom.spawn(Node { width: Val::Px(30.0), ..default() });

            // Lock In button
            spawn_primary_button(bottom, "LOCK IN", LockInButton, &fonts);

            bottom.spawn(Node { width: Val::Px(30.0), ..default() });

            // Rune page indicator
            bottom.spawn((
                Node { padding: UiRect::axes(Val::Px(15.0), Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                BackgroundColor(PANEL_BG_ALPHA), BorderColor::all(BORDER_GRAY),
            )).with_children(|rune_btn| {
                rune_btn.spawn((Text::new("RUNES: Default Page"), body_font(&fonts, FONT_SMALL), TextColor(TEXT_WHITE)));
            });
        });
    });
}

pub fn interactions(
    time: Res<Time>,
    mut state: ResMut<ChampSelectState>,
    champ_q: Query<(&Interaction, &ChampCard), Changed<Interaction>>,
    lock_q: Query<&Interaction, (Changed<Interaction>, With<LockInButton>)>,
    spell_q: Query<(&Interaction, &SpellSlot), Changed<Interaction>>,
    mut timer_text: Query<&mut Text, (With<TimerText>, Without<PhaseText>, Without<SelectedChampName>)>,
    mut phase_text: Query<&mut Text, (With<PhaseText>, Without<TimerText>, Without<SelectedChampName>)>,
    mut champ_text: Query<&mut Text, (With<SelectedChampName>, Without<TimerText>, Without<PhaseText>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut profile: ResMut<PlayerProfile>,
) {
    state.timer -= time.delta_secs();
    if let Ok(mut text) = timer_text.single_mut() {
        **text = format!("{:.0}", state.timer.max(0.0));
    }

    if state.timer <= 0.0 {
        match state.phase {
            SelectPhase::BanPhase => {
                state.phase = SelectPhase::PickPhase;
                state.timer = 87.0;
                if let Ok(mut text) = phase_text.single_mut() { **text = "PICK PHASE".to_string(); }
            }
            SelectPhase::PickPhase => {
                state.phase = SelectPhase::Finalization;
                state.timer = 10.0;
                if let Ok(mut text) = phase_text.single_mut() { **text = "FINALIZATION".to_string(); }
            }
            SelectPhase::Finalization => {
                if state.selected_champion.is_none() {
                    state.selected_champion = Some(ChampionClass::Mage);
                }
                profile.preferred_champion = state.selected_champion;
                profile.spell_d = state.spell_d;
                profile.spell_f = state.spell_f;
                next_state.set(AppState::Loading);
            }
        }
    }

    for (interaction, card) in &champ_q {
        if *interaction == Interaction::Pressed {
            state.selected_champion = Some(card.0);
            let def = get_champion(card.0);
            if let Ok(mut text) = champ_text.single_mut() {
                **text = format!("{} \u{2014} {}", def.name, def.title);
            }
        }
    }

    for interaction in &lock_q {
        if *interaction == Interaction::Pressed && state.selected_champion.is_some() && !state.locked_in {
            state.locked_in = true;
            state.phase = SelectPhase::Finalization;
            state.timer = 10.0;
            if let Ok(mut text) = phase_text.single_mut() { **text = "LOCKED IN \u{2014} FINALIZATION".to_string(); }
        }
    }

    for (interaction, slot) in &spell_q {
        if *interaction == Interaction::Pressed {
            let all = SummonerSpell::all();
            let current = if slot.0 == 0 { state.spell_d } else { state.spell_f };
            let idx = all.iter().position(|s| *s == current).unwrap_or(0);
            let next = all[(idx + 1) % all.len()];
            if slot.0 == 0 { state.spell_d = next; } else { state.spell_f = next; }
        }
    }
}
