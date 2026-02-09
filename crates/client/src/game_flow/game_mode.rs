use bevy::prelude::*;

use crate::game_flow::states::{AppState, InGameState};

pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StartGameModeMessage>()
            .add_systems(
                Update,
                (
                    handle_start_game_mode_message,
                    // handle_enemy_killed_event,
                ),
            )
            .init_state::<GameModeClient>();
    }
}

#[derive(Message)]
pub struct StartGameModeMessage {
    pub restart: bool,
}

// The current game mode on the client
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default, Copy)]
pub enum GameModeClient {
    #[default]
    FreeRoam,
    Waves,
    Multiplayer,
}

fn handle_start_game_mode_message(
    mut message_reader: MessageReader<StartGameModeMessage>,
    current_game_mode: Res<State<GameModeClient>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut current_in_game_state: ResMut<NextState<InGameState>>,
) {
    for message in message_reader.read() {
        info!(
            "Got game mode start message, game mode: {:?}",
            current_game_mode.get()
        );

        current_in_game_state.set(InGameState::Playing);

        if !message.restart {
            app_state.set(AppState::LoadingGame);
        }
    }
}
