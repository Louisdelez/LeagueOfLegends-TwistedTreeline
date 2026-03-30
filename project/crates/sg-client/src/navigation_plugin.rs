//! Navigation plugin — loads LoL NGRID and provides A* pathfinding
//! Uses the real Twisted Treeline navigation grid (306x129 cells, 50 units each)

use bevy::prelude::*;
use sg_core::components::*;
use sg_core::GameSet;
use sg_navigation::{NavGrid, find_path};
use crate::menu::AppState;

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NavGrid::default())
            .add_systems(OnEnter(AppState::InGame), load_ngrid)
            .add_systems(Update, (
                compute_paths.in_set(GameSet::AI),
                follow_nav_path.in_set(GameSet::AI),
            ).run_if(in_state(AppState::InGame)));
    }
}

/// Desired navigation destination
#[derive(Component)]
pub struct NavGoal {
    pub position: Vec2,
}

/// Computed A* path with waypoints
#[derive(Component)]
pub struct NavPath {
    pub waypoints: Vec<Vec2>,
    pub current_index: usize,
}

/// Load the real LoL NGRID file
fn load_ngrid(mut grid: ResMut<NavGrid>) {
    let path = std::path::Path::new("assets/maps/AIPath.aimesh_ngrid");
    match std::fs::read(path) {
        Ok(data) => {
            if let Some(parsed) = sg_navigation::parse_ngrid(&data) {
                *grid = parsed;
            } else {
                println!("[NAV] ERROR: Failed to parse NGRID file");
            }
        }
        Err(e) => println!("[NAV] ERROR: Cannot read NGRID file: {}", e),
    }
}

/// Compute A* path when NavGoal is set (no NavPath yet)
fn compute_paths(
    mut commands: Commands,
    grid: Res<NavGrid>,
    query: Query<(Entity, &Transform, &NavGoal), (Without<NavPath>, Without<Dead>)>,
) {
    if !grid.loaded { return; }

    let count = query.iter().count();

    for (entity, tf, goal) in &query {
        let start = Vec2::new(tf.translation.x, tf.translation.z);
        let waypoints = find_path(&grid, start, goal.position);

        if let Ok(mut ecmd) = commands.get_entity(entity) {
            if waypoints.len() >= 2 {
                ecmd.insert(NavPath { waypoints: waypoints.clone(), current_index: 1 });
                ecmd.insert(MoveTarget { position: waypoints[1] });
            } else {
                ecmd.remove::<NavGoal>();
            }
        }
    }
}

/// Follow NavPath by advancing MoveTarget through waypoints
fn follow_nav_path(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut NavPath)>,
) {
    for (entity, tf, mut path) in &mut query {
        let pos = Vec2::new(tf.translation.x, tf.translation.z);

        // Advance past reached waypoints
        while path.current_index < path.waypoints.len() {
            if pos.distance(path.waypoints[path.current_index]) < 120.0 {
                path.current_index += 1;
            } else {
                break;
            }
        }

        if let Ok(mut ecmd) = commands.get_entity(entity) {
            if path.current_index < path.waypoints.len() {
                ecmd.insert(MoveTarget { position: path.waypoints[path.current_index] });
            } else {
                ecmd.remove::<NavPath>().remove::<NavGoal>();
            }
        }
    }
}
