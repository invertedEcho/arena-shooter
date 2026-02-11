use bevy::prelude::*;

/// To be inserted into any entity that has a AnimationPlayer somewhere in its hierarchy tree,
/// pointing to the Entity of the AnimationPlayer and AnimationTransitions.
#[derive(Component)]
pub struct AnimationPlayerEntityPointer(pub Entity);
