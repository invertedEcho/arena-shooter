use bevy::prelude::*;

use crate::{
    enemy::{
        shooting::messages::EnemyKilledMessage,
        spawn::{EnemySpawnStrategy, SpawnEnemiesMessage},
    },
    game_flow::{
        AppState,
        score::GameScore,
        states::{InGameState, MainMenuState},
    },
    player::{
        camera::messages::SpawnPlayerCamerasMessage,
        spawn::{PlayerSpawnMessage, components::PlayerSpawnLocation},
    },
};

pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StartGameModeMessage>()
            .add_systems(
                Update,
                (
                    handle_start_game_mode_event,
                    handle_game_state_wave_changed,
                    handle_enemy_killed_event,
                ),
            )
            .init_state::<GameMode>()
            .init_state::<GameStateWave>();
    }
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub enum GameMode {
    #[default]
    FreePlay,
    Waves,
}

#[derive(Message)]
pub struct StartGameModeMessage(pub GameMode);

// TODO:
// how do we store data about current game state, but mode depending?
// have a state for each possible game mode? feels bad
// generalize it? how?
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub struct GameStateWave {
    pub current_wave: usize,
    pub enemies_left_from_current_wave: usize,
}

fn handle_start_game_mode_event(
    mut message_reader: MessageReader<StartGameModeMessage>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut spawn_enemies_message_writer: MessageWriter<SpawnEnemiesMessage>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    player_spawn_location: Single<&Transform, With<PlayerSpawnLocation>>,
    mut spawn_player_message_writer: MessageWriter<PlayerSpawnMessage>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut spawn_player_cameras_message_writer: MessageWriter<
        SpawnPlayerCamerasMessage,
    >,
) {
    for event in message_reader.read() {
        next_main_menu_state.set(MainMenuState::None);
        next_in_game_state.set(InGameState::Playing);

        match event.0 {
            GameMode::Waves => {
                let enemy_count = get_enemy_count_per_wave(1);
                next_game_state_wave.set(GameStateWave {
                    current_wave: 1,
                    enemies_left_from_current_wave: enemy_count,
                });
                next_app_state.set(AppState::InGame);
                spawn_enemies_message_writer.write(SpawnEnemiesMessage {
                    enemy_count,
                    spawn_strategy: EnemySpawnStrategy::RandomSelection,
                });
            }
            GameMode::FreePlay => {
                next_app_state.set(AppState::InGame);
            }
        }

        spawn_player_message_writer.write(PlayerSpawnMessage {
            spawn_location: player_spawn_location.translation,
        });
        spawn_player_cameras_message_writer.write(SpawnPlayerCamerasMessage);
    }
}

pub fn get_enemy_count_per_wave(wave: usize) -> usize {
    return match wave {
        1 => 3,
        2 => 4,
        3 => 6,
        4 => 8,
        5 => 10,
        6 => 12,
        _ => 14,
    };
    // TODO: spawn enemies that make the player take more damage
    // or have smarter ai
}

fn handle_game_state_wave_changed(
    game_state_wave: Res<State<GameStateWave>>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    mut spawn_enemies_message_writer: MessageWriter<SpawnEnemiesMessage>,
) {
    if game_state_wave.is_changed() && !game_state_wave.is_changed() {
        if game_state_wave.enemies_left_from_current_wave == 0 {
            info!(
                "no enemies left from current, spawning new enemies and \
                 increasing current_wave"
            );
            let new_wave = game_state_wave.current_wave + 1;
            let new_enemy_count = get_enemy_count_per_wave(new_wave);

            next_game_state_wave.set(GameStateWave {
                current_wave: new_wave,
                enemies_left_from_current_wave: new_enemy_count,
            });
            spawn_enemies_message_writer.write(SpawnEnemiesMessage {
                enemy_count: new_enemy_count,
                spawn_strategy: EnemySpawnStrategy::RandomSelection,
            });
        }
    }
}

fn handle_enemy_killed_event(
    current_game_mode: Res<State<GameMode>>,
    game_state_wave: Res<State<GameStateWave>>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    mut enemy_killed_message_reader: MessageReader<EnemyKilledMessage>,
    mut game_score: ResMut<GameScore>,
    mut spawn_enemies_event_writer: MessageWriter<SpawnEnemiesMessage>,
) {
    for _ in enemy_killed_message_reader.read() {
        game_score.player += 1;
        match *current_game_mode.get() {
            GameMode::Waves => {
                let new_enemies_left_count =
                    game_state_wave.enemies_left_from_current_wave - 1;
                next_game_state_wave.set(GameStateWave {
                    current_wave: game_state_wave.current_wave,
                    enemies_left_from_current_wave: new_enemies_left_count,
                });
                if new_enemies_left_count == 0 {
                    info!("no enemies left, spawning new wave!");
                    let new_wave = game_state_wave.current_wave + 1;
                    let enemy_count = get_enemy_count_per_wave(new_wave);
                    next_game_state_wave.set(GameStateWave {
                        current_wave: new_wave,
                        enemies_left_from_current_wave: enemy_count,
                    });
                    spawn_enemies_event_writer.write(SpawnEnemiesMessage {
                        enemy_count,
                        spawn_strategy: EnemySpawnStrategy::RandomSelection,
                    });
                }
            }
            GameMode::FreePlay => {}
        }
    }
}
