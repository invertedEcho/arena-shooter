use bevy::prelude::*;

#[derive(Message)]
pub struct JumpAction {
    pub character_controller_entity: Entity,
}
