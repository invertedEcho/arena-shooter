use bevy::prelude::*;

use crate::game_flow::systems::{
    free_mouse, grab_mouse, handle_escape, handle_player_death_event,
};

pub mod systems;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDeathEvent>()
            .add_systems(OnEnter(GameState::InGame), grab_mouse)
            .add_systems(OnEnter(GameState::Paused), free_mouse)
            .add_systems(Update, (handle_escape, handle_player_death_event))
            .init_state::<GameState>();
    }
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    InGame,
    Paused,
    Death,
    Settings,
}

#[derive(Event)]
pub struct PlayerDeathEvent;
