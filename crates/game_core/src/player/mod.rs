use avian3d::prelude::*;
use bevy::{color::palettes::css::WHITE, prelude::*};
use netvy::prelude::*;
use shared::{
    AppRole,
    character_controller::{
        CHARACTER_CAPSULE_LENGTH, CHARACTER_CAPSULE_RADIUS,
    },
    components::Health,
    enemy::components::Enemy,
    game_score::{GameScore, LivingEntityStats},
    multiplayer_messages::{PlayerHitMessage, ShootRequest},
    player::{Player, PlayerBundle},
    shooting::{MAX_SHOOTING_DISTANCE, PlayerKilled, PlayerWeapons},
};

use crate::{
    enemy::ai::messages::PlayerHitEnemy, game_score::AddKillAndDeathGameScore,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (handle_shoot_requests, spawn_player_on_new_client),
        );
    }
}

fn spawn_player_on_new_client(
    added_clients_query: Query<&PeerId, (Added<PeerId>, With<Client>)>,
    mut commands: Commands,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_score: Query<&mut GameScore>,
    app_role: Res<State<AppRole>>,
) {
    for peer_id in added_clients_query {
        if *app_role.get() == AppRole::ClientOnly {
            info!(
                "Not spawning a player, game_core is running in ClientOnly \
                 mode."
            );
            return;
        }

        let player_entity = commands
            .spawn((
                PlayerBundle::default(),
                Name::new("Player"),
                ReplicateEntity,
                SyncPosition::default(),
                SyncRotation::default(),
                Visibility::Visible,
                Owner(*peer_id),
                // we give the client authority too, as we dont have client-prediction yet in netvy.
                // Otherwise, the client wouldnt be able to change the transform of its own player,
                // as it would get overriden by `apply_internal_sync_position`
                Authority(*peer_id),
                Collider::capsule(
                    CHARACTER_CAPSULE_RADIUS,
                    CHARACTER_CAPSULE_LENGTH,
                ),
                RigidBody::Kinematic,
            ))
            .id();

        info!(
            "Spawned a player for fully connected Client. \
             (player_entity={player_entity}, peer_id={})",
            peer_id.0
        );

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

        // TODO: could be moved into seperate system
        match game_score.single_mut() {
            Ok(mut game_score) => {
                game_score.players.insert(
                    *peer_id,
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
    }
}

fn handle_shoot_requests(
    mut commands: Commands,
    mut message_reader: MessageReader<FromClient<ShootRequest>>,
    mut message_writer: MessageWriter<ToClients<PlayerHitMessage>>,
    mut health_query: Query<&mut Health>,
    spatial_query: SpatialQuery,
    player_query: Query<(Entity, &Owner, &PlayerWeapons), With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut player_hit_enemy_message_writer: MessageWriter<PlayerHitEnemy>,
    mut add_kill_and_death_game_score_message_writer: MessageWriter<
        AddKillAndDeathGameScore,
    >,
    mut player_killed_msg_writer: MessageWriter<ToClients<PlayerKilled>>,
    net_entities: Query<(&NetEntityId, &Owner), With<Player>>,
) {
    for message in message_reader.read() {
        let source_client = message.source_client;
        let message = &message.message;

        // the player entity that sent this ShootRequest
        let Some((shooter_entity, _, player_weapons)) = player_query
            .iter()
            .find(|(_, controlled_by, _)| controlled_by.0 == source_client)
        else {
            warn!(
                ?source_client,
                ?player_query,
                "Received a ShootRequest but couldn't determine from which \
                 player this came from"
            );
            continue;
        };

        debug!(?shooter_entity, ?source_client, "Received a shoot request");

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
            debug!(
                "Entity hit was nothing that has health component: {}",
                entity_hit
            );
            continue;
        };

        let health_to_substract = player_weapons.weapons
            [player_weapons.active_weapon_slot]
            .game_weapon
            .damage;
        health.0 -= health_to_substract;

        let is_enemy = enemy_query.get(entity_hit).is_ok();

        if is_enemy {
            player_hit_enemy_message_writer.write(PlayerHitEnemy {
                player_entity: shooter_entity,
                enemy_entity: entity_hit,
            });
        } else {
            let Ok((_, owner, _)) = player_query.get(entity_hit) else {
                error!("Could not determine which player was hit");
                continue;
            };
            debug!(
                "Substracted {health_to_substract} from health component of \
                 player {:?}",
                owner.0
            );
            message_writer.write(ToClients {
                message: PlayerHitMessage {
                    origin: message.origin,
                },
                target: NetworkMessageTarget::Clients(vec![owner.0]),
            });
        }

        if health.0 <= 0.0 {
            let entity_killed = first_hit.entity;
            commands.entity(entity_killed).insert(ColliderDisabled);

            add_kill_and_death_game_score_message_writer.write(
                AddKillAndDeathGameScore {
                    entity_that_shot: shooter_entity,
                    entity_killed,
                },
            );

            if !is_enemy {
                match net_entities.get(entity_killed) {
                    Ok((net_entity_killed, owner)) => {
                        let message = PlayerKilled {
                            player_killed: *net_entity_killed,
                        };

                        info!(
                            "Notifying all clients except killed one about player killed"
                        );
                        player_killed_msg_writer.write(ToClients {
                            message,
                            target: NetworkMessageTarget::Except(vec![owner.0]),
                        });
                    }
                    Err(error) => {
                        error!(
                            "Failed to find net information about player killed: {error:?}"
                        );
                    }
                };
            }
        }
    }
}
