use bevy::prelude::*;
use crate::menu::AppState;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SfxHandles::default())
            .add_systems(OnEnter(AppState::ChampionSelect), play_champ_select_music)
            .add_systems(OnEnter(AppState::InGame), (stop_menu_music, play_game_music, load_sfx));
    }
}

#[derive(Component)] struct MenuMusic;
#[derive(Component)] pub struct GameMusic;

/// Preloaded SFX handles for instant playback
#[derive(Resource, Default)]
pub struct SfxHandles {
    pub hit: Option<Handle<AudioSource>>,
    pub death: Option<Handle<AudioSource>>,
    pub levelup: Option<Handle<AudioSource>>,
    pub gold: Option<Handle<AudioSource>>,
}

fn load_sfx(mut sfx: ResMut<SfxHandles>, asset_server: Res<AssetServer>) {
    sfx.hit = Some(asset_server.load("audio/hit.ogg"));
    sfx.death = Some(asset_server.load("audio/death.ogg"));
    sfx.levelup = Some(asset_server.load("audio/levelup.ogg"));
    sfx.gold = Some(asset_server.load("audio/gold.ogg"));
}

/// Call this from other systems to play a one-shot SFX
pub fn play_sfx(commands: &mut Commands, handle: &Option<Handle<AudioSource>>) {
    if let Some(h) = handle {
        commands.spawn(AudioPlayer::new(h.clone()));
    }
}

fn play_champ_select_music(mut commands: Commands, asset_server: Res<AssetServer>, existing: Query<Entity, With<MenuMusic>>) {
    if !existing.is_empty() { return; }
    commands.spawn((AudioPlayer::new(asset_server.load("audio/champ_select_music.ogg")), MenuMusic));
}

fn stop_menu_music(mut commands: Commands, music: Query<Entity, With<MenuMusic>>) {
    for entity in &music { commands.entity(entity).despawn(); }
}

fn play_game_music(mut commands: Commands, asset_server: Res<AssetServer>, existing: Query<Entity, With<GameMusic>>) {
    if !existing.is_empty() { return; }
    commands.spawn((AudioPlayer::new(asset_server.load("audio/game_music.ogg")), GameMusic));
}
