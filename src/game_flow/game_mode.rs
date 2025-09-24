use bevy::prelude::*;

use crate::{
    enemy::spawn::{EnemySpawnMethod, SpawnEnemiesEvent},
    game_flow::AppState,
};

pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartGameModeEvent>()
            .add_systems(
                Update,
                (handle_start_game_mode_event, handle_game_state_wave_changed)
                    .run_if(in_state(GameMode::Waves)),
            )
            .init_state::<GameMode>()
            .init_state::<GameStateWave>();
    }
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub enum GameMode {
    #[default]
    None,
    Waves,
}

#[derive(Event)]
pub struct StartGameModeEvent(pub GameMode);

// TODO:
// how do we store data about current game state, but mode depending?
// have a state for each possible game mode? feels bad
// generalize it? how?
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub struct GameStateWave {
    pub current_wave_index: usize,
    pub enemies_left_from_current_wave: usize,
}

fn handle_start_game_mode_event(
    mut event_reader: EventReader<StartGameModeEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut spawn_enemies_event_writer: EventWriter<SpawnEnemiesEvent>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
) {
    for event in event_reader.read() {
        match event.0 {
            GameMode::Waves => {
                next_game_state_wave.set(GameStateWave {
                    current_wave_index: 0,
                    enemies_left_from_current_wave: 3,
                });
                next_app_state.set(AppState::InGame);
                spawn_enemies_event_writer.write(SpawnEnemiesEvent {
                    enemy_count: 3,
                    spawn_method: EnemySpawnMethod::RandomSelection,
                });
            }
            // idk maybe this shouldnt even exist at the first place
            GameMode::None => {}
        }
    }
}

pub fn get_enemy_count_per_wave(wave_index: usize) -> usize {
    return match wave_index {
        0 => 3,
        1 => 4,
        2 => 6,
        3 => 8,
        4 => 10,
        5 => 12,
        _ => 14,
    };
    // TODO: spawn enemies that make the player take more damage
    // or have smarter ai
}

fn handle_game_state_wave_changed(
    current_game_mode: Res<State<GameMode>>,
    game_state_wave: Res<State<GameStateWave>>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    mut spawn_enemies_event_writer: EventWriter<SpawnEnemiesEvent>,
) {
    if game_state_wave.is_changed() {
        if game_state_wave.enemies_left_from_current_wave == 0 {
            let new_wave_index = game_state_wave.current_wave_index + 1;
            let new_enemy_count = get_enemy_count_per_wave(new_wave_index);

            next_game_state_wave.set(GameStateWave {
                current_wave_index: new_wave_index,
                enemies_left_from_current_wave: new_enemy_count,
            });
            spawn_enemies_event_writer.write(SpawnEnemiesEvent {
                enemy_count: new_enemy_count,
                spawn_method: EnemySpawnMethod::RandomSelection,
            });
        }
    }
}
