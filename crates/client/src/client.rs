use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use shared::SERVER_ADDRESS;
use shared::player::Player;
use shared::protocol::{
    ClientUpdatePositionMessage, OrderedReliableMessageChannel,
    PlayerPositionServer,
};

use crate::ClientId;
use crate::character_controller::components::CharacterControllerBundle;
use crate::game_flow::states::AppState;
use crate::player::camera::messages::SpawnPlayerCamerasMessage;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ConnectToServerMessage>();
        app.add_systems(
            Update,
            (
                handle_connect_to_server_message,
                send_client_update_position.run_if(in_state(AppState::InGame)),
                apply_server_position_other_clients,
            ),
        );
        app.add_observer(handle_new_player);
    }
}

#[derive(Message)]
pub struct ConnectToServerMessage;

pub fn handle_connect_to_server_message(
    mut commands: Commands,
    mut message_reader: MessageReader<ConnectToServerMessage>,
    client_id: Res<ClientId>,
    connected_query: Query<Has<Connected>>,
) {
    for _ in message_reader.read() {
        for connected in connected_query {
            if connected {
                info!("Already connected, skipping ConnectToServerMessage");
                continue;
            }
        }

        info!("Connecting to server...");

        let auth = Authentication::Manual {
            server_addr: SERVER_ADDRESS,
            client_id: client_id.0,
            private_key: Key::default(),
            protocol_id: 0,
        };

        let client = commands
            .spawn((
                Name::new("Client"),
                Client::default(),
                LocalAddr("127.0.0.1:0".parse().unwrap()),
                PeerAddr(SERVER_ADDRESS),
                Link::new(None),
                ReplicationReceiver::default(),
                NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
                UdpIo::default(),
            ))
            .id();

        // Send connect request
        commands.trigger(Connect { entity: client });
    }
}

fn handle_new_player(
    trigger: On<Add, Player>,
    mut commands: Commands,
    player_query: Query<Has<Controlled>, With<Player>>,
    mut spawn_player_camera_message_writer: MessageWriter<
        SpawnPlayerCamerasMessage,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Ok(is_controlled) = player_query.get(trigger.entity)
        && is_controlled
    {
        info!("We found our player!");

        // we insert the character controller locally on our client, as it should only run on the
        // client. as it is not registered in our protocol, it wont be replicated.
        commands.entity(trigger.entity).insert((
            CharacterControllerBundle::default(),
            DespawnOnExit(AppState::InGame),
            Visibility::Visible,
            Transform::from_translation(vec3(0.0, 20.0, 0.0)),
        ));
        spawn_player_camera_message_writer
            .write(SpawnPlayerCamerasMessage(trigger.entity));
    } else {
        info!(
            "A player was added, but it doesn't have Controlled Component, \
             e.g. its not our player! Inserting visuals so we can see that \
             other player"
        );
        commands.entity(trigger.entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(0.2, 1.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: WHITE.into(),
                ..Default::default()
            })),
        ));
    }
}

pub fn send_client_update_position(
    // With<Client> also ensures its the messagesender from our local client, as client component
    // only gets inserted into our own client
    mut message_sender: Single<
        &mut MessageSender<ClientUpdatePositionMessage>,
        With<Client>,
    >,
    player_transform: Query<&Transform, (With<Player>, With<Controlled>)>,
) {
    match player_transform.single() {
        Ok(player_transform) => {
            message_sender.send::<OrderedReliableMessageChannel>(
                ClientUpdatePositionMessage {
                    new_translation: player_transform.translation,
                },
            );
        }
        Err(error) => {
            warn!("Failed to get our own transform! {:?}", error)
        }
    }
}

pub fn apply_server_position_other_clients(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &PlayerPositionServer,
            Has<Controlled>, // present only on own player
        ),
        With<Player>,
    >,
) {
    for (mut transform, server_pos, controlled) in &mut query {
        // Skip our own transform
        if controlled {
            continue;
        }

        let target = server_pos.translation;
        let current = transform.translation;

        let lerp_factor = 10.0 * time.delta_secs();
        transform.translation = current.lerp(target, lerp_factor);
    }
}
