use avian3d::prelude::*;
use bevy::{color::palettes::css::WHITE, prelude::*};
use lightyear::prelude::{server::ClientOf, *};
use shared::{
    AppRole, GameModeServer,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
    },
    components::{EntityPositionServer, Health},
    enemy::components::Enemy,
    game_score::{GameScore, LivingEntityStats},
    multiplayer_messages::{PlayerHitMessage, ShootRequest},
    player::{Player, PlayerBundle},
    protocol::OrderedReliableChannel,
    shooting::MAX_SHOOTING_DISTANCE,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_player_on_new_client);
        app.add_systems(Update, (handle_shoot_requests));
    }
}

fn handle_shoot_requests(
    mut commands: Commands,
    receivers: Query<(&mut MessageReceiver<ShootRequest>, Entity, &RemoteId)>,
    mut health_query: Query<&mut Health>,
    spatial_query: SpatialQuery,
    player_query: Query<(Entity, &ControlledBy), With<Player>>,
    mut game_score: Single<&mut GameScore>,
    game_mode_server: Res<State<GameModeServer>>,
    client_query: Query<&RemoteId, With<ClientOf>>,
    mut server_multi_message_sender: ServerMultiMessageSender,
    server: Single<&Server>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for (mut message_receiver, client_entity_server_side, remote_id) in
        receivers
    {
        for message in message_receiver.receive() {
            let Some(shooter_entity) = player_query
                .iter()
                .find(|(_, controlled_by)| {
                    controlled_by.owner == client_entity_server_side
                })
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

            if let Ok(mut health) = health_query.get_mut(first_hit.entity) {
                health.0 -= 8.0;
                let is_enemy = enemy_query.get(first_hit.entity).is_ok();
                if !is_enemy {
                    if let Ok(client_entity_that_was_hit) =
                        player_query.get(first_hit.entity).map(|i| i.1)
                        && let Ok(client) =
                            client_query.get(client_entity_that_was_hit.owner)
                    {
                        server_multi_message_sender
                            .send::<PlayerHitMessage, OrderedReliableChannel>(
                                &PlayerHitMessage {
                                    origin: message.origin,
                                },
                                &server,
                                &NetworkTarget::Single(client.0),
                            )
                            .ok();
                    } else {
                        error!(
                            "Could not find client that was hit by the bullet"
                        );
                    }
                }

                if health.0 <= 0.0 {
                    let entity_killed = first_hit.entity;
                    commands.entity(entity_killed).insert(ColliderDisabled);

                    match game_score.players.get_mut(&remote_id.to_bits()) {
                        Some(player) => {
                            debug!(
                                "increased kill count of player with \
                                 remote_id: {}",
                                remote_id.to_bits()
                            );
                            player.kills += 1;
                        }
                        None => {
                            warn!(
                                "Failed to find player in game score by \
                                 remote_id {}\nGame score: {:?}",
                                remote_id.to_bits(),
                                *game_score
                            )
                        }
                    }

                    // if we have game mode wave, the entity killed will always be an enemy. so we
                    // skip this case
                    if **game_mode_server == GameModeServer::Waves {
                        return;
                    };
                    match player_query.get(entity_killed) {
                        Ok((_, controlled_by)) => {
                            if let Ok(remote_id) =
                                client_query.get(controlled_by.owner)
                                && let Some(player_score) = game_score
                                    .players
                                    .get_mut(&remote_id.to_bits())
                            {
                                player_score.deaths += 1;
                            } else {
                                warn!(
                                    "Failed to find client of player that was \
                                     killed"
                                );
                            };
                        }
                        Err(error) => {
                            warn!(
                                "Failed to find player that was killed: {}",
                                error
                            );
                        }
                    }
                }
            }
        }
    }
}

fn spawn_player_on_new_client(
    trigger: On<Add, Connected>,
    clients_query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_score: Query<&mut GameScore>,
    app_role: Res<State<AppRole>>,
) {
    if let Ok(remote_id) = clients_query.get(trigger.entity) {
        let peer_id = remote_id.0;

        match game_score.single_mut() {
            Ok(mut game_score) => {
                game_score.players.insert(
                    peer_id.to_bits(),
                    LivingEntityStats {
                        username: format!("Player {}", peer_id.to_bits()),
                        ..default()
                    },
                );
            }
            Err(error) => {
                error!("Failed to add player to game score: {}", error);
            }
        }

        info!(
            "Spawning a player for fully connected Client entity: {} | \
             peer_id: {}",
            trigger.entity, peer_id
        );

        // NOTE: The replicate component gets inserted into the player entity, but only registered
        // components will be replicated to all other clients
        let player_entity = commands
            .spawn((
                PlayerBundle::default(),
                Name::new("Player"),
                Replicate::to_clients(NetworkTarget::All),
                // TODO: think we could override replication behaviour for this component and only
                // replicate to all other clients than the current client
                EntityPositionServer {
                    translation: vec3(0.0, 20.0, 0.0),
                },
                Visibility::Visible,
                // we add the ControlledBy on the server, with the client entity as the owner of this
                // player, so on the client we can then filter by players that have the `Controlled`
                // component and those are the players that are actually owned by that client
                ControlledBy {
                    owner: trigger.entity,
                    lifetime: Lifetime::SessionBased,
                },
                Collider::capsule(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ),
                RigidBody::Kinematic,
            ))
            .insert_if(Controlled, || {
                *app_role.get() == AppRole::ClientAndServer
            })
            .id();

        if *app_role.get() == AppRole::DedicatedServer {
            // on headless setup, materials doesnt exist
            if let Some(mut materials) = materials {
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
