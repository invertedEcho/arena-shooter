use bevy::prelude::*;

use crate::game_flow::systems::{free_mouse, grab_mouse};

pub mod systems;

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    InGame,
    Paused,
    Death,
}

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), grab_mouse)
            .add_systems(OnEnter(GameState::Paused), free_mouse)
            .add_systems(Update, handle_escape)
            .init_state::<GameState>();
    }
}

fn handle_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if *current_game_state == GameState::InGame {
            next_game_state.set(GameState::Paused);
        } else if *current_game_state == GameState::Paused {
            next_game_state.set(GameState::InGame);
        }
    }
}
