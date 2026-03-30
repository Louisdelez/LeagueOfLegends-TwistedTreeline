use bevy::prelude::*;
use sg_core::types::*;

/// Bot AI controller — attached to non-player champions
#[derive(Component, Debug)]
pub struct BotController {
    pub state: BotState,
    pub assigned_lane: Lane,
    pub decision_timer: f32,
    pub patrol_set: bool,
    pub lane_waypoint_index: usize,
    pub last_nav_goal: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BotState {
    Laning,
    Fighting,
    Retreating,
    Dead,
}

impl BotController {
    pub fn new(lane: Lane) -> Self {
        Self {
            state: BotState::Laning,
            assigned_lane: lane,
            decision_timer: 0.0,
            patrol_set: false,
            lane_waypoint_index: 0,
            last_nav_goal: None,
        }
    }
}
