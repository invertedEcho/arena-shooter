use avian3d::prelude::*;
use bevy::{color::palettes::css::WHITE, prelude::*};
use netvy::prelude::*;
use shared::{
    AppRole, GameModeServer,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
    },
    components::Health,
    enemy::components::Enemy,
    game_score::{GameScore, LivingEntityStats},
    multiplayer_messages::{PlayerHitMessage, ShootRequest},
    player::{Player, PlayerBundle},
    shooting::MAX_SHOOTING_DISTANCE,
};

use crate::enemy::ai::messages::PlayerHitEnemy;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_shoot_requests, spawn_player_on_new_client),
        );
    }
}

fn handle_shoot_requests(
    mut commands: Commands,
    message_readers: Query<(&mut NetMessageReader<ShootRequest>, &PeerId)>,
    mut health_query: Query<&mut Health>,
    spatial_query: SpatialQuery,
    player_query: Query<(Entity, &OwnedBy), With<Player>>,
    mut game_score: Single<&mut GameScore>,
    game_mode_server: Res<State<GameModeServer>>,
    client_query: Query<&PeerId, With<Client>>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut player_hit_enemy_message_writer: MessageWriter<PlayerHitEnemy>,
    mut server_net_message_writer: Single<
        &mut NetMessageWriter<PlayerHitMessage>,
        With<Server>,
    >,
) {
    for (mut message_reader, peer_id) in message_readers {
        for message in message_reader.read() {
            // the player entity that sent this ShootRequest
            let Some(shooter_entity) = player_query
                .iter()
                .find(|(_, controlled_by)| controlled_by.0.0 == peer_id.0)
                .map(|i| i.0)
            else {
                warn!(
                    "Received a ShootRequest but couldn't determine from \
                     which player this came from"
                );
                continue;
            };

            let Some(first_hit) = spatial_query.cast_ray(
                message.origin,
                message.direction,
                MAX_SHOOTING_DISTANCE,
                false,
                &SpatialQueryFilter::default()
                    .with_excluded_entities([shooter_entity]),
            ) else {
                continue;
            };

            let entity_hit = first_hit.entity;

            // if we cant find health, this collider is just an obstacle
            let Ok(mut health) = health_query.get_mut(entity_hit) else {
                continue;
            };

            health.0 -= 8.0;

            let is_enemy = enemy_query.get(first_hit.entity).is_ok();
            if is_enemy {
                player_hit_enemy_message_writer.write(PlayerHitEnemy {
                    player_entity: shooter_entity,
                    enemy_entity: entity_hit,
                });
            } else {
                if let Ok(client_entity_that_was_hit) =
                    player_query.get(entity_hit).map(|i| i.0)
                    && let Ok(client) =
                        client_query.get(client_entity_that_was_hit)
                {
                    server_net_message_writer.write(PlayerHitMessage {
                        origin: message.origin,
                    });
                    // server_multi_message_sender
                    //     .send::<PlayerHitMessage, OrderedReliableChannel>(
                    //         &PlayerHitMessage {
                    //             origin: message.origin,
                    //         },
                    //         &server,
                    //         &NetworkTarget::Single(client.0),
                    //     )
                    //     .ok();
                } else {
                    error!("Could not find client that was hit by the bullet");
                }
            }

            if health.0 <= 0.0 {
                let entity_killed = first_hit.entity;
                commands.entity(entity_killed).insert(ColliderDisabled);

                match game_score.players.get_mut(&peer_id.0) {
                    Some(player) => {
                        debug!(
                            "increased kill count of player with peer_id: {}",
                            peer_id.0
                        );
                        player.kills += 1;
                    }
                    None => {
                        warn!(
                            "Failed to find player in game score by peer_id \
                             {}\nGame score: {:?}",
                            peer_id.0, *game_score
                        )
                    }
                }

                // if we have game mode wave, the entity killed will always be an enemy. so we
                // skip this case
                if **game_mode_server == GameModeServer::Waves {
                    return;
                };
                let Some(player_score) = game_score.players.get_mut(&peer_id.0)
                else {
                    warn!("Failed to find client of player that was killed");
                    continue;
                };
                player_score.deaths += 1;
            }
        }
    }
}

fn spawn_player_on_new_client(
    clients_query: Query<(&PeerId, &ConnectionState), Changed<ConnectionState>>,
    mut commands: Commands,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_score: Query<&mut GameScore>,
    app_role: Res<State<AppRole>>,
) {
    for (peer_id, connection_state) in clients_query {
        if *connection_state == ConnectionState::Connected {
            match game_score.single_mut() {
                Ok(mut game_score) => {
                    game_score.players.insert(
                        peer_id.0,
                        LivingEntityStats {
                            username: format!("Player {}", peer_id.0),
                            ..default()
                        },
                    );
                }
                Err(error) => {
                    error!("Failed to add player to game score: {}", error);
                }
            }

            info!(
                "Spawning a player for fully connected Client. (peer_id={})",
                peer_id.0
            );

            let player_entity = commands
                .spawn((
                    PlayerBundle::default(),
                    Name::new("Player"),
                    ReplicateEntity,
                    SyncPosition::default(),
                    Visibility::Visible,
                    OwnedBy(*peer_id),
                    Collider::capsule(
                        CHARACTER_CAPSULE_RADIUS,
                        CHARACTER_CAPSULE_LENGTH,
                    ),
                    RigidBody::Kinematic,
                ))
                .insert_if(Owned, || {
                    *app_role.get() == AppRole::ClientAndServer
                })
                .id();

            if *app_role.get() == AppRole::DedicatedServer {
                // on headless setup, materials doesnt exist
                if let Some(ref mut materials) = materials {
                    commands.entity(player_entity).insert((
                        Mesh3d(meshes.add(Capsule3d::new(
                            CHARACTER_CAPSULE_RADIUS,
                            CHARACTER_CAPSULE_LENGTH,
                        ))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: WHITE.into(),
                            ..Default::default()
                        })),
                    ));
                }
            }
        }
    }
}
