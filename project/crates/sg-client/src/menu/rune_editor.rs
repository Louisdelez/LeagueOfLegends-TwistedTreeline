use bevy::prelude::*;
use sg_core::runes::*;

/// Rune editor state (used as overlay in champion select or collection)
#[derive(Resource, Default)]
pub struct RuneEditorState {
    pub open: bool,
    pub editing_page: usize,
}

// Placeholder — full implementation in Phase 5
// The rune editor will be an overlay that can be opened from champion select or collection
