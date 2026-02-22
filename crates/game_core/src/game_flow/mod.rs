use bevy::prelude::*;
use shared::EnemyKilledMessage;

use crate::{
    GameStateWave,
    enemy::spawn::{EnemySpawnStrategy, SpawnEnemiesMessage},
};

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_enemy_killed_message);
    }
}

// // TODO: spawn enemies that make the player take more damage
// // or have smarter ai
pub fn get_enemy_count_per_wave(wave: usize) -> usize {
    if wave == 1 { 1 } else { wave + 2 }
}

fn handle_enemy_killed_message(
    mut game_state_wave: If<ResMut<GameStateWave>>,
    mut enemy_killed_message_reader: MessageReader<EnemyKilledMessage>,
    mut spawn_enemies_event_writer: MessageWriter<SpawnEnemiesMessage>,
) {
    for _ in enemy_killed_message_reader.read() {
        game_state_wave.enemies_killed += 1;

        let new_enemies_left_count =
            game_state_wave.enemies_left_from_current_wave - 1;
        **game_state_wave = GameStateWave {
            current_wave: game_state_wave.current_wave,
            enemies_left_from_current_wave: new_enemies_left_count,
            enemies_killed: game_state_wave.enemies_killed + 1,
        };

        if new_enemies_left_count == 0 {
            info!("no enemies left, spawning new wave!");
            let new_wave = game_state_wave.current_wave + 1;
            let enemy_count = get_enemy_count_per_wave(new_wave);

            **game_state_wave = GameStateWave {
                current_wave: new_wave,
                enemies_left_from_current_wave: enemy_count,
                enemies_killed: game_state_wave.enemies_killed,
            };

            spawn_enemies_event_writer.write(SpawnEnemiesMessage {
                enemy_count,
                spawn_strategy: EnemySpawnStrategy::RandomSelection,
            });
        }
    }
}
