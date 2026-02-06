use bevy::prelude::*;

#[derive(Message)]
pub struct MovementAction {
    pub desired_velocity: MovementDirection,
    pub character_controller_entity: Entity,
    pub sprint: bool,
}

pub enum MovementDirection {
    // TODO: should be possible to just have Vec2
    Move(Vec3),
    Jump,
}
