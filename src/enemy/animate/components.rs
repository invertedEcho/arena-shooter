use bevy::prelude::*;

/// To be inserted into an Enemy entity, pointing to the Entity of the AnimationPlayer and
/// AnimationTransitions.
#[derive(Component)]
pub struct AnimationPlayerEntityPointer(pub Entity);

#[derive(Component)]
pub struct PlayHitAnimationTimer(pub Timer);
