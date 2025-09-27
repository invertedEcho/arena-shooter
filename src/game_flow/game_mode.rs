use bevy::prelude::*;

use crate::{
    WorldUiCamera,
    enemy::spawn::{EnemySpawnMethod, SpawnEnemiesEvent},
    game_flow::AppState,
    player::spawn::{PlayerSpawnEvent, components::PlayerSpawnLocation},
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
    pub current_wave: usize,
    pub enemies_left_from_current_wave: usize,
}

fn handle_start_game_mode_event(
    mut commands: Commands,
    mut event_reader: EventReader<StartGameModeEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut spawn_enemies_event_writer: EventWriter<SpawnEnemiesEvent>,
    mut next_game_state_wave: ResMut<NextState<GameStateWave>>,
    player_spawn_location: Single<&Transform, With<PlayerSpawnLocation>>,
    world_ui_camera: Single<Entity, With<WorldUiCamera>>,
    mut spawn_player_event_writer: EventWriter<PlayerSpawnEvent>,
) {
    for event in event_reader.read() {
        commands.entity(*world_ui_camera).despawn();

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
                    spawn_method: EnemySpawnMethod::RandomSelection,
                });
            }
            // idk maybe this shouldnt even exist at the first place
            GameMode::None => {}
        }

        spawn_player_event_writer.write(PlayerSpawnEvent {
            spawn_location: player_spawn_location.translation,
        });
    }
}

pub fn get_enemy_count_per_wave(wave: usize) -> usize {
    return match wave {
        1 => 1,
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
                "no enemies left from current, spawning new enemies and increasing current_wave"
            );
            let new_wave = game_state_wave.current_wave + 1;
            let new_enemy_count = get_enemy_count_per_wave(new_wave);

            next_game_state_wave.set(GameStateWave {
                current_wave: new_wave,
                enemies_left_from_current_wave: new_enemy_count,
            });
            spawn_enemies_event_writer.write(SpawnEnemiesEvent {
                enemy_count: new_enemy_count,
                spawn_method: EnemySpawnMethod::RandomSelection,
            });
        }
    }
}
