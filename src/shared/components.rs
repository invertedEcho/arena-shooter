use bevy::prelude::*;

/// Insert this component into entities that you want to have despawned when the given Timer has
/// reached its end. The timer will be automatically ticked in `src/common/systems.rs`
#[derive(Component)]
pub struct DespawnTimer(pub Timer);

/// To be inserted into any entity that has a AnimationPlayer somewhere in its hierarchy tree,
/// pointing to the Entity of the AnimationPlayer and AnimationTransitions.
#[derive(Component)]
pub struct AnimationPlayerEntityPointer(pub Entity);

#[derive(Component)]
pub struct Health(pub f32);
