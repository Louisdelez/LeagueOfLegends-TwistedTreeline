pub mod styles;
pub mod data;
pub mod home_screen;
pub mod lobby_screen;
pub mod champion_select;
pub mod rune_editor;
pub mod profile_screen;
pub mod collection_screen;
pub mod settings_screen;
pub mod loading_screen;
pub mod postgame_screen;

use bevy::prelude::*;

/// Application state machine — controls which screen is active
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    MainMenu,
    PlayLobby,
    Profile,
    Collection,
    Settings,
    ChampionSelect,
    Loading,
    #[default]
    InGame,
    PostGame,
}

/// Marker for all menu UI entities (despawned on screen exit)
#[derive(Component)]
pub struct MenuUI;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        styles::load_ui_assets(app);
        app.init_state::<AppState>()
            .insert_resource(data::PlayerProfile::default())
            .add_systems(Startup, setup_menu_camera)
            .add_systems(OnEnter(AppState::InGame), despawn_menu_camera)
            .add_systems(Update, styles::button_hover_system)
            // Home
            .add_systems(OnEnter(AppState::MainMenu), home_screen::setup)
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu)
            .add_systems(Update, home_screen::interactions.run_if(in_state(AppState::MainMenu)))
            // Lobby
            .add_systems(OnEnter(AppState::PlayLobby), lobby_screen::setup)
            .add_systems(OnExit(AppState::PlayLobby), cleanup_menu)
            .add_systems(Update, lobby_screen::interactions.run_if(in_state(AppState::PlayLobby)))
            // Champion Select
            .add_systems(OnEnter(AppState::ChampionSelect), champion_select::setup)
            .add_systems(OnExit(AppState::ChampionSelect), cleanup_menu)
            .add_systems(Update, champion_select::interactions.run_if(in_state(AppState::ChampionSelect)))
            // Profile
            .add_systems(OnEnter(AppState::Profile), profile_screen::setup)
            .add_systems(OnExit(AppState::Profile), cleanup_menu)
            .add_systems(Update, profile_screen::interactions.run_if(in_state(AppState::Profile)))
            // Collection
            .add_systems(OnEnter(AppState::Collection), collection_screen::setup)
            .add_systems(OnExit(AppState::Collection), cleanup_menu)
            .add_systems(Update, collection_screen::interactions.run_if(in_state(AppState::Collection)))
            // Settings
            .add_systems(OnEnter(AppState::Settings), settings_screen::setup)
            .add_systems(OnExit(AppState::Settings), cleanup_menu)
            .add_systems(Update, settings_screen::interactions.run_if(in_state(AppState::Settings)))
            // Loading
            .add_systems(OnEnter(AppState::Loading), loading_screen::setup)
            .add_systems(OnExit(AppState::Loading), cleanup_menu)
            .add_systems(Update, loading_screen::tick.run_if(in_state(AppState::Loading)))
            // PostGame
            .add_systems(OnEnter(AppState::PostGame), postgame_screen::setup)
            .add_systems(OnExit(AppState::PostGame), cleanup_menu)
            .add_systems(Update, postgame_screen::interactions.run_if(in_state(AppState::PostGame)));
    }
}

/// Marker for the menu camera (despawned when entering InGame)
#[derive(Component)]
pub struct MenuCamera;

fn setup_menu_camera(mut commands: Commands, state: Res<State<AppState>>) {
    // Don't spawn menu camera if starting directly in InGame
    if *state.get() == AppState::InGame { return; }
    commands.spawn((
        Camera2d,
        MenuCamera,
    ));
}

fn despawn_menu_camera(mut commands: Commands, cameras: Query<Entity, With<MenuCamera>>) {
    for entity in &cameras {
        commands.entity(entity).despawn();
    }
}

fn cleanup_menu(mut commands: Commands, entities: Query<Entity, With<MenuUI>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
