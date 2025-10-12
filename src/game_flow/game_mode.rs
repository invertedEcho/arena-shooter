use bevy::prelude::*;

use crate::{
    enemy::spawn::{EnemySpawnStrategy, SpawnEnemiesEvent},
    game_flow::{
        AppState,
        states::{InGameState, MainMenuState},
    },
    player::{
        camera::events::SpawnPlayerCamerasEvent,
        spawn::{PlayerSpawnEvent, components::PlayerSpawnLocation},
    },
    user_interface::main_menu::MainMenuCamera,
};

pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartGameModeEvent>()
            .add_systems(
                Update,
                (handle_start_game_mode_event, handle_game_state_wave_changed),
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

#[derive(Event)]
pub struct StartGameModeEvent(pub GameMode);

// TODO:
// how do we store data about current game state, but mode depending?
// have a state for each possible game mode? feels bad
// generalize it? how?
#[derive(States, Eq, Debug, PartialEq, Hash, Clone, Default)]
pub struct GameStateWave {
    pub current_wave: usize,
    pub enemies_left_from_current_wave: usize,
}

// TODO: this function takes wayyyy to many arguments
fn handle_start_game_mode_event(
    mut commands: Commands,
    mut event_reader: EventReader<StartGameModeEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut spawn_enemies_event_writer: EventWriter<SpawnEnemiesEvent>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    player_spawn_location: Single<&Transform, With<PlayerSpawnLocation>>,
    main_menu_camera: Single<Entity, With<MainMenuCamera>>,
    mut spawn_player_event_writer: EventWriter<PlayerSpawnEvent>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    mut spawn_player_cameras_event_writer: EventWriter<SpawnPlayerCamerasEvent>,
) {
    for event in event_reader.read() {
        info!("got start game mode event, despawning main menu camera");
        commands.entity(*main_menu_camera).despawn();
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
                spawn_enemies_event_writer.write(SpawnEnemiesEvent {
                    enemy_count,
                    spawn_strategy: EnemySpawnStrategy::RandomSelection,
                });
            }
            GameMode::FreePlay => {
                info!("does this happen?");
                next_app_state.set(AppState::InGame);
                info!("huh");
            }
        }

        info!("writing playerspawnevent");
        spawn_player_event_writer.write(PlayerSpawnEvent {
            spawn_location: player_spawn_location.translation,
        });
        info!("writing SpawnPlayerCamerasEvent");
        spawn_player_cameras_event_writer.write(SpawnPlayerCamerasEvent);
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
    mut spawn_enemies_event_writer: EventWriter<SpawnEnemiesEvent>,
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
            spawn_enemies_event_writer.write(SpawnEnemiesEvent {
                enemy_count: new_enemy_count,
                spawn_strategy: EnemySpawnStrategy::RandomSelection,
            });
        }
    }
}
