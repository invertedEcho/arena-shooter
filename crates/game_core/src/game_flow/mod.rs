use bevy::prelude::*;
use lightyear::{connection::host::HostClient, prelude::RemoteId};
use shared::{
    EnemyKilledMessage, GameStateServer, NextWaveTimer, WaveFinishedMessage,
    enemy::components::Enemy,
    game_score::GameScore,
    player::{DEFAULT_PLAYER_WEAPONS, PlayerCash},
    shooting::PlayerWeapons,
};

use crate::{
    DespawnEnemyMessage, GameStateWave, RequestNewWave, RetryWaveGameMode,
    enemy::spawn::{EnemySpawnStrategy, SpawnEnemiesMessage},
};

const CASH_PER_ENEMY_KILL: usize = 100;
const CASH_PER_WAVE_FINISHED: usize = 750;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RetryWaveGameMode>()
            .add_message::<DespawnEnemyMessage>()
            .add_message::<WaveFinishedMessage>();

        app.add_systems(
            Update,
            (
                handle_enemy_killed_message,
                handle_retry_wave_game_mode_message,
                handle_despawn_enemy_message,
                update_game_score_on_retry_wave_game_mode,
                add_cash_on_enemy_killed,
                handle_request_new_wave_message,
                add_cash_on_wave_finished,
                insert_next_wave_timer_on_wave_finished,
            ),
        );

        app.add_systems(
            Update,
            (handle_next_wave_timer).run_if(in_state(GameStateServer::Running)),
        );

        app.add_systems(
            Update,
            detect_wave_finished
                .run_if(resource_exists_and_changed::<GameStateWave>),
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

fn add_cash_on_enemy_killed(
    mut message_reader: MessageReader<EnemyKilledMessage>,
    mut player_cash: Single<&mut PlayerCash>,
) {
    for _ in message_reader.read() {
        player_cash.0 += CASH_PER_ENEMY_KILL;
    }
}

fn add_cash_on_wave_finished(
    mut message_reader: MessageReader<WaveFinishedMessage>,
    mut player_cash: Single<&mut PlayerCash>,
) {
    for _ in message_reader.read() {
        player_cash.0 += CASH_PER_WAVE_FINISHED;
    }
}

fn handle_request_new_wave_message(
    mut message_reader: MessageReader<RequestNewWave>,
    mut spawn_enemies_message_writer: MessageWriter<SpawnEnemiesMessage>,
    mut game_state_wave: If<ResMut<GameStateWave>>,
) {
    for _ in message_reader.read() {
        let current_wave_ended =
            game_state_wave.enemies_left_from_current_wave == 0;

        if !current_wave_ended {
            debug!(
                "Ignored RequestNewWave message, not all enemies have been \
                 killed from current wave"
            );
            continue;
        }

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

fn detect_wave_finished(
    game_state_wave: If<Res<GameStateWave>>,
    mut message_writer: MessageWriter<WaveFinishedMessage>,
) {
    let no_enemies_left = game_state_wave.enemies_left_from_current_wave == 0;

    if game_state_wave.is_changed()
        && !game_state_wave.is_added()
        && no_enemies_left
    {
        message_writer.write(WaveFinishedMessage);
    }
}

fn insert_next_wave_timer_on_wave_finished(
    mut commands: Commands,
    mut message_reader: MessageReader<WaveFinishedMessage>,
) {
    for _ in message_reader.read() {
        commands.insert_resource(NextWaveTimer(Timer::from_seconds(
            10.0,
            TimerMode::Once,
        )));
    }
}

fn handle_next_wave_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: If<ResMut<NextWaveTimer>>,
    mut new_wave_message_writer: MessageWriter<RequestNewWave>,
) {
    timer.0.0.tick(time.delta());
    if timer.0.0.is_finished() {
        commands.remove_resource::<NextWaveTimer>();
        new_wave_message_writer.write(RequestNewWave);
    }
}
