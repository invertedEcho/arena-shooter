use std::path::Path;

use bevy::log::warn;
use shared::ServerRunMode;

pub fn get_run_mode(run_mode_str: Option<&String>) -> ServerRunMode {
    if let Some(run_mode) = run_mode_str {
        if run_mode == "headful" {
            return ServerRunMode::Headful;
        } else if run_mode == "headless" {
            return ServerRunMode::Headless;
        } else {
            warn!(
                "Your given run_mode: {} could not be interpreted. Must \
                 either be 'headless' or 'headful'. Defaulting to headless.",
                run_mode
            );
        }
    }

    ServerRunMode::Headless
}

// The server can be started from workspace root via `cargo run -p server` or from server workspace,
// so we need to ensure we use the correct path
pub fn get_path_to_collider_json() -> String {
    const BASE_PATH: &str = "assets/maps/medium_plastic/colliders.json";
    let path = Path::new(BASE_PATH);
    if path.exists() {
        BASE_PATH.to_string()
    } else {
        "../../".to_owned() + BASE_PATH
    }
}
