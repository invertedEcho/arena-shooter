use avian3d::prelude::*;
use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    character_controller::components::{CharacterController, Grounded},
    player::Player,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Reflect, Default)]
pub struct PlayerInputs {
    pub direction: Direction,
    pub run: bool,
    pub jump: bool,
    // We need the cameras rotation on the server too to apply it in movement, but we can't just
    // replicate Camera component as the camera can and must only live on the client
    pub camera_yaw: f32,
    pub camera_pitch: f32,
}

// this ensures that entities are referenced correctly on server and client as entities will have
// different entity ids on client and server
impl MapEntities for PlayerInputs {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

#[derive(
    Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Reflect,
)]
pub struct Direction {
    pub forward: bool,
    pub backwards: bool,
    pub left: bool,
    pub right: bool,
}

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<Player>();
        app.register_component::<CharacterController>();

        app.register_component::<LinearVelocity>()
            .add_prediction()
            .add_should_rollback(linear_velocity_should_rollback);

        app.register_component::<AngularVelocity>()
            .add_prediction()
            .add_should_rollback(angular_velocity_should_rollback);

        app.register_component::<Position>()
            .add_prediction()
            .add_should_rollback(position_should_rollback)
            .add_linear_correction_fn()
            .add_linear_interpolation();

        app.register_component::<Rotation>()
            .add_prediction()
            .add_should_rollback(rotation_should_rollback)
            .add_linear_correction_fn()
            .add_linear_interpolation();

        app.register_component::<Grounded>();

        app.register_component::<RigidBody>();
    }
}

fn position_should_rollback(this: &Position, that: &Position) -> bool {
    (this.0 - that.0).length() >= 0.01
}

fn rotation_should_rollback(this: &Rotation, that: &Rotation) -> bool {
    this.angle_between(*that) >= 0.01
}

fn linear_velocity_should_rollback(
    this: &LinearVelocity,
    that: &LinearVelocity,
) -> bool {
    (this.0 - that.0).length() >= 0.01
}

fn angular_velocity_should_rollback(
    this: &AngularVelocity,
    that: &AngularVelocity,
) -> bool {
    (this.0 - that.0).length() >= 0.01
}
