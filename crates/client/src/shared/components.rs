use bevy::prelude::*;

/// A marker component added to entities that should only be visible when InGameState::Playing
#[derive(Component)]
pub struct OnlyVisibleInGame;
