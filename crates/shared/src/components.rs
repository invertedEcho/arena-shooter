use bevy::prelude::*;

/// A marker component indcating which client this player belongs to
#[derive(Component)]
pub struct ControlledByClient(pub Entity);
