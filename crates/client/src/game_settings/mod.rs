use std::{
    fs::{self, File},
    io::{Read, Write},
};

use bevy::{ecs::resource::Resource, log::info, prelude::*};
use serde::{Deserialize, Serialize};

use crate::game_settings::utils::{
    ensure_game_settings_directory_exists, get_game_settings_file,
};

mod utils;

#[derive(Serialize, Deserialize, Resource, Clone)]
pub struct GameSettings {
    pub sounds_volume: f32,
    pub music_volume: f32,
    pub fullscreen: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sounds_volume: 50.0,
            music_volume: 50.0,
            fullscreen: true,
        }
    }
}

pub fn get_or_create_game_settings() -> GameSettings {
    let game_settings_directory = ensure_game_settings_directory_exists();
    let game_settings_file = get_game_settings_file(game_settings_directory);

    let game_settings_file_exists = fs::exists(&game_settings_file)
        .expect("Can check if game_settings_file exists");

    if game_settings_file_exists {
        // NOTE: From fs::exists():
        // Note that while this avoids some pitfalls of the `exists()` method, it still can not
        // prevent time-of-check to time-of-use ([TOCTOU]) bugs. You should only use it in scenarios
        // where those bugs are not an issue.
        let mut game_settings_file = File::open(game_settings_file).expect(
            "Can open game_settings_file if existence was previously confirmed",
        );

        let mut file_buffer = String::from("");

        let result = game_settings_file.read_to_string(&mut file_buffer);
        match result {
            Ok(_) => info!("Sucessfully read from game save file into buffer"),
            Err(err) => panic!(
                "Failed to read from game save file into buffer: {}",
                err
            ),
        }

        let game_settings: Result<GameSettings, serde_json::error::Error> =
            serde_json::from_str(&file_buffer);
        match game_settings {
            Ok(game_settings) => {
                info!(
                    "Sucessfully serialized existing game settings to \
                     GameSettings struct."
                );
                game_settings
            }
            Err(error) => {
                panic!(
                    "Failed to parse game save json str into rust struct.: {}",
                    error
                );
            }
        }
    } else {
        let game_settings = GameSettings::default();

        fs::write(
            game_settings_file,
            serde_json::to_vec(&game_settings)
                .expect("Can serialize GameSetting to vec"),
        )
        .expect("Can create initial GAME_SETTINGS_FILE with initial content");

        info!("Created initial game settings file.");

        game_settings
    }
}

pub fn update_game_settings_file(new_game_settings: &GameSettings) {
    let game_settings_directory = ensure_game_settings_directory_exists();
    let game_settings_file = get_game_settings_file(game_settings_directory);

    let write_result = File::create(game_settings_file)
        .expect("Can create game settings file")
        .write_all(
            &serde_json::to_vec(&new_game_settings)
                .expect("Can serialize to json string"),
        );

    match write_result {
        Ok(()) => {
            info!("Sucessfully updated game save!");
        }
        Err(err) => {
            panic!("Failed to update game save: {}", err);
        }
    }
}
