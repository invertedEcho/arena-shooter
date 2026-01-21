use bevy::prelude::*;

/// A marker component added to entities that should be hidden when the player pauses the game, and
/// make it visible on resume.
#[derive(Component)]
pub struct HideOnPause;
