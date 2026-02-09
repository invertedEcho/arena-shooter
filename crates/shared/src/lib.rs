use avian3d::{PhysicsPlugins, prelude::*};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    character_controller::messages::MovementAction, components::DespawnTimer,
    protocol::ProtocolPlugin,
};

pub mod character_controller;
pub mod components;
pub mod enemy;
pub mod game_score;
pub mod messages;
pub mod player;
pub mod protocol;
pub mod shooting;
pub mod utils;

#[derive(Resource, PartialEq)]
pub enum ServerRunMode {
    Headless,
    Headful,
}

/// A resource existing to know whether we are using a remote server to connect to,
/// or a local server is started, e.g. for singleplayer
#[derive(States, PartialEq, Debug, Hash, Clone, Eq)]
pub enum ServerMode {
    RemoteServer,
    LocalServerSinglePlayer,
}

/// The game mode that is running on the server. Must be a component as we can only replicate
/// components with lightyear.
#[derive(Component, Serialize, Deserialize, PartialEq, Debug)]
pub enum GameModeServer {
    Waves,
    FreeForAll,
    FreeRoam,
}

#[derive(Component)]
pub struct Medkit {
    pub active: bool,
    pub health_to_give: f32,
    pub float_direction: MedkitFloatDirection,
    pub respawn_timer: Timer,
}

pub enum MedkitFloatDirection {
    Up,
    Down,
}

pub const GRAVITY: f32 = 9.81;

pub const TINY_TOWN_MAP_PATH: &str = "maps/tiny_town/main.gltf";
pub const MEDIUM_PLASTIC_MAP_PATH: &str = "maps/medium_plastic/scene.gltf";
pub const SPAWN_POINT_MEDIUM_PLASTIC_MAP: Vec3 = vec3(0.0, 10.0, 0.0);

/// Logic for both client and server binary
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_message::<MovementAction>();

        app.add_plugins(PhysicsPlugins::default().build())
            // .add_plugins(PhysicsDebugPlugin)
            .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY));
        app.add_systems(Update, handle_despawn_timer);
    }
}

// This is not a substate, as it needs to exist globally
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub enum SelectedMapState {
    #[default]
    MediumPlastic,
    TinyTown,
}

pub fn handle_despawn_timer(
    despawn_timer_query: Query<(Entity, &mut DespawnTimer)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer) in despawn_timer_query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
