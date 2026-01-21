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
