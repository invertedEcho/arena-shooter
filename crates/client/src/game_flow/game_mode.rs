use bevy::prelude::*;

use crate::game_flow::states::AppState;

pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StartGameModeMessage>()
            .add_systems(
                Update,
                (
                    handle_start_game_mode_message,
                    handle_game_state_wave_changed,
                    // handle_enemy_killed_event,
                ),
            )
            .init_state::<GameModeState>()
            .add_sub_state::<GameStateWave>();
    }
}

#[derive(Message)]
pub struct StartGameModeMessage {
    pub restart: bool,
}

#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub enum GameModeState {
    #[default]
    FreeRoam,
    Waves,
    Multiplayer,
}

#[derive(SubStates, Eq, Debug, PartialEq, Hash, Clone, Default)]
#[source(GameModeState = GameModeState::Waves)]
pub struct GameStateWave {
    pub current_wave: usize,
    pub enemies_killed: usize,
    pub enemies_left_from_current_wave: usize,
}

// FIXME: enemies need to be spawned on server and replicated to client
fn handle_start_game_mode_message(
    mut message_reader: MessageReader<StartGameModeMessage>,
    // mut spawn_enemies_message_writer: MessageWriter<SpawnEnemiesMessage>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    // existing_enemies: Query<Entity, With<Enemy>>,
    current_game_mode: Res<State<GameModeState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for message in message_reader.read() {
        info!(
            "Got game mode start message, game mode: {:?}",
            current_game_mode.get()
        );

        if !message.restart {
            app_state.set(AppState::LoadingGame);
        }

        match current_game_mode.get() {
            GameModeState::Waves => {
                // for existing_enemy in existing_enemies {
                //     commands.entity(existing_enemy).despawn();
                // }
                let enemy_count = get_enemy_count_per_wave(1);

                // this state should exist on the server and be synced to the client
                next_game_state_wave.set(GameStateWave {
                    current_wave: 1,
                    enemies_left_from_current_wave: enemy_count,
                    enemies_killed: 0,
                });
                // spawn_enemies_message_writer.write(SpawnEnemiesMessage {
                //     enemy_count,
                //     spawn_strategy: EnemySpawnStrategy::RandomSelection,
                // });
            }
            GameModeState::FreeRoam => {}
            GameModeState::Multiplayer => {}
        }
    }
}

// TODO: spawn enemies that make the player take more damage
// or have smarter ai
pub fn get_enemy_count_per_wave(wave: usize) -> usize {
    if wave == 1 { 1 } else { wave + 2 }
}

fn handle_game_state_wave_changed(
    game_state_wave: If<Res<State<GameStateWave>>>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    // mut spawn_enemies_message_writer: MessageWriter<SpawnEnemiesMessage>,
) {
    let game_state_wave_changed = game_state_wave.is_changed();
    let no_enemies_left = game_state_wave.enemies_left_from_current_wave == 0;
    if game_state_wave_changed && no_enemies_left {
        info!(
            "Game State wave changed and no enemies left from current, \
             spawning new enemies and increasing current_wave"
        );
        let new_wave = game_state_wave.current_wave + 1;
        let new_enemy_count = get_enemy_count_per_wave(new_wave);

        next_game_state_wave.set(GameStateWave {
            current_wave: new_wave,
            enemies_left_from_current_wave: new_enemy_count,
            enemies_killed: game_state_wave.enemies_killed,
        });
        // spawn_enemies_message_writer.write(SpawnEnemiesMessage {
        //     enemy_count: new_enemy_count,
        //     spawn_strategy: EnemySpawnStrategy::RandomSelection,
        // });
    }
}

// fn handle_enemy_killed_event(
//     current_game_mode: Res<State<GameModeState>>,
//     maybe_game_state_wave: Option<Res<State<GameStateWave>>>,
//     mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
//     mut enemy_killed_message_reader: MessageReader<EnemyKilledMessage>,
//     mut game_score: ResMut<GameScore>,
//     mut spawn_enemies_event_writer: MessageWriter<SpawnEnemiesMessage>,
// ) {
//     for _ in enemy_killed_message_reader.read() {
//         game_score.player += 1;
//         match *current_game_mode.get() {
//             GameModeState::Multiplayer => {
//                 warn!(
//                     "Not yet implemented: Enemy was killed message in \
//                      multiplayer mode"
//                 );
//             }
//             GameModeState::Waves => {
//                 let Some(ref game_state_wave) = maybe_game_state_wave else {
//                     warn!(
//                         "Enemy killed, current game mode is Waves, but \
//                          GameStateWave doesn't exist"
//                     );
//                     continue;
//                 };
//
//                 let new_enemies_left_count =
//                     game_state_wave.enemies_left_from_current_wave - 1;
//                 next_game_state_wave.set(GameStateWave {
//                     current_wave: game_state_wave.current_wave,
//                     enemies_left_from_current_wave: new_enemies_left_count,
//                     enemies_killed: game_state_wave.enemies_killed + 1,
//                 });
//                 if new_enemies_left_count == 0 {
//                     info!("no enemies left, spawning new wave!");
//                     let new_wave = game_state_wave.current_wave + 1;
//                     let enemy_count = get_enemy_count_per_wave(new_wave);
//                     next_game_state_wave.set(GameStateWave {
//                         current_wave: new_wave,
//                         enemies_left_from_current_wave: enemy_count,
//                         enemies_killed: game_state_wave.enemies_killed + 1,
//                     });
//                     spawn_enemies_event_writer.write(SpawnEnemiesMessage {
//                         enemy_count,
//                         spawn_strategy: EnemySpawnStrategy::RandomSelection,
//                     });
//                 }
//             }
//             GameModeState::FreeRoam => {}
//         }
//     }
// }
