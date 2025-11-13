use std::fs::{self, create_dir};

pub fn get_game_settings_directory() -> String {
    let Some(config_dir) = dirs::config_dir() else {
        panic!("Could not get config directory");
    };
    config_dir.to_str().unwrap().to_owned() + "/fun-shooter/"
}

pub fn get_game_settings_file(game_settings_directory: String) -> String {
    game_settings_directory + "config.json"
}

/// Returns the game settings directory path
pub fn ensure_game_settings_directory_exists() -> String {
    let game_settings_directory = get_game_settings_directory();
    let game_settings_dir_exists = fs::exists(&game_settings_directory)
        .expect("Can check if GAME_SETTINGS_DIR_PATH exists");
    if !game_settings_dir_exists {
        create_dir(&game_settings_directory);
    }
    game_settings_directory
}
