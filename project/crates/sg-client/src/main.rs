mod camera_plugin;
mod map_plugin;
mod input_plugin;
mod movement_plugin;
mod spawn_plugin;
mod combat_plugin;
mod debug_plugin;
mod hud_plugin;
mod ability_plugin;
mod objectives_plugin;
pub mod net_plugin;
mod fog_plugin;
mod garen_plugin;
mod minion_plugin;
mod navigation_plugin;
mod shop_plugin;
mod audio_plugin;
mod menu;

use bevy::prelude::*;
use sg_core::GameSet;
use menu::AppState;

fn main() {
    App::new()
        .set_error_handler(bevy::ecs::error::warn)
        .insert_resource(ClearColor(Color::srgb(0.01, 0.01, 0.02)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "League of Legends — Twisted Treeline".into(),
                resolution: bevy::window::WindowResolution::new(1920, 1080),
                ..default()
            }),
            ..default()
        }))
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::AI,
                GameSet::Movement,
                GameSet::Combat,
                GameSet::Spawn,
            )
                .chain(),
        )
        // Menu system (active in all states, manages transitions)
        .add_plugins(menu::MenuPlugin)
        // Core (only active in InGame state)
        .add_plugins(map_plugin::MapPlugin)
        .add_plugins(camera_plugin::CameraPlugin)
        .add_plugins(input_plugin::InputPlugin)
        .add_plugins(movement_plugin::MovementPlugin)
        .add_plugins(spawn_plugin::SpawnPlugin)
        // Gameplay (only active in InGame state)
        .add_plugins(combat_plugin::CombatPlugin)
        .add_plugins(ability_plugin::AbilityPlugin)
        .add_plugins(objectives_plugin::ObjectivesPlugin)
        .add_plugins(navigation_plugin::NavigationPlugin)
        .add_plugins(minion_plugin::MinionPlugin)
        .add_plugins(garen_plugin::GarenPlugin)
        .add_plugins(fog_plugin::FogPlugin)
        .add_plugins(shop_plugin::ShopPlugin)
        // UI & Network
        .add_plugins(hud_plugin::HudPlugin)
        .add_plugins(net_plugin::NetPlugin)
        .add_plugins(audio_plugin::AudioPlugin)
        .add_plugins(debug_plugin::DebugPlugin)
        .run();
}
