use bevy::prelude::*;
use crate::menu::AppState;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ChampionSelect), play_champ_select_music)
            .add_systems(OnEnter(AppState::InGame), (stop_menu_music, play_game_music));
    }
}

#[derive(Component)]
struct MenuMusic;

#[derive(Component)]
pub struct GameMusic;

fn play_champ_select_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<MenuMusic>>,
) {
    if !existing.is_empty() { return; }
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/champ_select_music.ogg")),
        MenuMusic,
    ));
}

fn stop_menu_music(
    mut commands: Commands,
    music: Query<Entity, With<MenuMusic>>,
) {
    for entity in &music {
        commands.entity(entity).despawn();
    }
}

fn play_game_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<GameMusic>>,
) {
    if !existing.is_empty() { return; }
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/game_music.ogg")),
        GameMusic,
    ));
}
