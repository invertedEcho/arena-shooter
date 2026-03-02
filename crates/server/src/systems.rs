use std::{fs::File, io::Read};

use avian3d::prelude::Collider;
use bevy::prelude::*;
use game_core::GameCoreLoadingState;
use shared::{ServerRunMode, StartGame};

use crate::utils::get_path_to_collider_json;

pub fn spawn_map_colliders(
    mut commands: Commands,
    mut next_game_core_loading_state: ResMut<NextState<GameCoreLoadingState>>,
) {
    let file_path = get_path_to_collider_json();

    let mut file_buffer = String::from("");
    let mut collider_file =
        File::open(file_path).expect("Can open medium_plastic_colliders.json");

    collider_file.read_to_string(&mut file_buffer).unwrap();

    let colliders: Result<
        Vec<(Collider, GlobalTransform)>,
        serde_json::error::Error,
    > = serde_json::from_str(&file_buffer);

    match colliders {
        Ok(colliders_ok) => {
            info!(
                "Loaded colliders and their transform from json, spawning \
                 them."
            );
            commands.spawn_batch(colliders_ok);
        }
        Err(error) => {
            panic!(
                "Failed to load colliders and their transform from json: {}",
                error
            );
        }
    }
    next_game_core_loading_state.set(GameCoreLoadingState::CollidersSpawned);
}

pub fn write_start_game_message(mut message_writer: MessageWriter<StartGame>) {
    message_writer.write(StartGame);
}

pub fn spawn_camera_if_headful(
    mut commands: Commands,
    server_run_mode: Res<ServerRunMode>,
) {
    if *server_run_mode == ServerRunMode::Headful {
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(10.0, 30.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
        ));
        commands.spawn((Node { ..default() }, Text::new("Server")));
    }
}
