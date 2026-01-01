use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::{AppComponentExt, PredictionRegistrationExt};

use crate::{
    character_controller::components::{CharacterController, Grounded},
    player::Player,
};

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // lightyear protocol
        app.register_component::<Player>();
        app.register_component::<Position>()
            .add_prediction()
            .add_linear_correction_fn()
            .add_linear_correction_fn();
        app.register_component::<Rotation>()
            .add_prediction()
            .add_linear_correction_fn()
            .add_linear_correction_fn();
        app.register_component::<CharacterController>();

        app.register_component::<LinearVelocity>()
            .add_prediction()
            .add_should_rollback(linear_velocity_should_rollback);
        app.register_component::<Grounded>();
    }
}

// Custom rollback condition
fn linear_velocity_should_rollback(
    this: &LinearVelocity,
    that: &LinearVelocity,
) -> bool {
    (this.0 - that.0).length() >= 0.0001
}
