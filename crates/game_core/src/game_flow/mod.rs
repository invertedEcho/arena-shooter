use bevy::prelude::*;
use lightyear::{connection::host::HostClient, prelude::RemoteId};
use shared::{
    EnemyKilledMessage, enemy::components::Enemy, game_score::GameScore,
    player::DEFAULT_PLAYER_WEAPONS, shooting::PlayerWeapons,
};

use crate::{
    DespawnEnemyMessage, GameStateWave, RetryWaveGameMode,
    enemy::spawn::{EnemySpawnStrategy, SpawnEnemiesMessage},
};

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RetryWaveGameMode>();
        app.add_message::<DespawnEnemyMessage>();
        app.add_systems(
            Update,
            (
                handle_enemy_killed_message,
                handle_retry_wave_game_mode_message,
                handle_despawn_enemy_message,
                update_game_score_on_retry_wave_game_mode,
            ),
        );
    }
}

// TODO: spawn enemies that have smarter ai
pub fn get_enemy_count_per_wave(wave: usize) -> usize {
    if wave == 1 { 1 } else { wave + 2 }
}

fn handle_enemy_killed_message(
    mut game_state_wave: If<ResMut<GameStateWave>>,
    mut enemy_killed_message_reader: MessageReader<EnemyKilledMessage>,
    mut spawn_enemies_message_writer: MessageWriter<SpawnEnemiesMessage>,
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
            debug!("No enemies left, spawning new wave!");
            let new_wave = game_state_wave.current_wave + 1;
            let enemy_count = get_enemy_count_per_wave(new_wave);

            **game_state_wave = GameStateWave {
                current_wave: new_wave,
                enemies_left_from_current_wave: enemy_count,
                enemies_killed: game_state_wave.enemies_killed,
            };

            spawn_enemies_message_writer.write(SpawnEnemiesMessage {
                enemy_count,
                spawn_strategy: EnemySpawnStrategy::RandomSelection,
            });
        }
    }
}

fn handle_retry_wave_game_mode_message(
    mut message_reader: MessageReader<RetryWaveGameMode>,
    mut game_state_wave: If<ResMut<GameStateWave>>,
    mut spawn_enemies_message_writer: MessageWriter<SpawnEnemiesMessage>,
    mut player_weapons: Single<&mut PlayerWeapons>,
    enemies: Query<Entity, With<Enemy>>,
    mut despawn_enemy_message_writer: MessageWriter<DespawnEnemyMessage>,
) {
    for _ in message_reader.read() {
        let wave = 1;
        let enemy_count = get_enemy_count_per_wave(wave);
        **game_state_wave = GameStateWave {
            current_wave: wave,
            enemies_killed: 0,
            enemies_left_from_current_wave: enemy_count,
        };
        spawn_enemies_message_writer.write(SpawnEnemiesMessage {
            enemy_count,
            spawn_strategy: EnemySpawnStrategy::RandomSelection,
        });
        **player_weapons = DEFAULT_PLAYER_WEAPONS;

        despawn_enemy_message_writer.write(DespawnEnemyMessage {
            enemies_to_despawn: enemies.iter().collect(),
        });
    }
}

fn update_game_score_on_retry_wave_game_mode(
    mut message_reader: MessageReader<RetryWaveGameMode>,
    mut game_score: Single<&mut GameScore>,
    host_client: Single<&RemoteId, With<HostClient>>,
) {
    for _ in message_reader.read() {
        if let Some(player_score) =
            game_score.players.get_mut(&host_client.to_bits())
        {
            player_score.kills = 0;
            player_score.deaths = 0;
        }
    }
}

fn handle_despawn_enemy_message(
    mut commands: Commands,
    mut message_reader: MessageReader<DespawnEnemyMessage>,
    mut game_score: Single<&mut GameScore>,
) {
    for message in message_reader.read() {
        for enemy in &message.enemies_to_despawn {
            commands.entity(*enemy).despawn();
            game_score.enemies.remove(enemy);
        }
    }
}
