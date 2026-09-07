use bevy::prelude::*;
use netvy::prelude::*;
use shared::{
    player::{OurPlayerReady, Player},
    shooting::{PlayerKilled, PlayerWeapons, WeaponKind},
};

use crate::player::{
    camera::{PlayerCameraPlugin, components::PlayerWeaponModel},
    shooting::{
        PlayerShootingPlugin, asset_paths::get_path_to_model_for_weapon_kind,
    },
};

pub mod camera;
pub mod shooting;

#[derive(Message)]
pub struct PlayerDeathMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                mark_players_as_ready,
                add_player_weapon_model_on_new_player,
                hide_player_on_killed,
            ),
        )
        .add_plugins(PlayerCameraPlugin)
        .add_plugins(PlayerShootingPlugin);
    }
}

type PlayersWithoutReadyMarker = (
    With<Player>,
    With<PlayerWeapons>,
    // we only insert PlayerReady component into our own player.
    With<Owned>,
    Without<OurPlayerReady>,
);

fn mark_players_as_ready(
    mut commands: Commands,
    query: Query<Entity, PlayersWithoutReadyMarker>,
) {
    for entity in query {
        debug!("Marking player {entity} as ready");
        commands.entity(entity).insert(OurPlayerReady);
    }
}

fn add_player_weapon_model_on_new_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(Entity, &Owner, &NetEntityId), Added<Player>>,
    our_peer_id: If<Res<OurPeerId>>,
) {
    let weapon_model_path =
        get_path_to_model_for_weapon_kind(&WeaponKind::AK47);
    let weapon_model = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset(weapon_model_path));

    for (player_entity, owner, net_entity_id) in player_query {
        // we dont add player weapon model to our own player as we already do that elsewhere, with
        // different handling
        if owner.0.0 == our_peer_id.0.0.0 {
            continue;
        }
        commands.entity(player_entity).with_children(|parent| {
            parent.spawn((
                Name::new("PlayerWeaponModel"),
                WorldAssetRoot(weapon_model.clone()),
                Transform {
                    translation: vec3(0.2, 0.1, -0.1),
                    ..default()
                },
                PlayerWeaponModel,
                Visibility::Visible,
                AlternateTargetRotation(*net_entity_id),
            ));
        });
    }
}

fn hide_player_on_killed(
    mut message_reader: MessageReader<FromServer<PlayerKilled>>,
    mut player_query: Query<(&mut Visibility, &NetEntityId), With<Player>>,
) {
    for message in message_reader.read() {
        let killed_player_net_entity = message.0.player_killed;

        let Some(mut player_visibility) =
            player_query
                .iter_mut()
                .find_map(|(visibility, net_entity_id)| {
                    if net_entity_id.0 == killed_player_net_entity.0 {
                        Some(visibility)
                    } else {
                        None
                    }
                })
        else {
            error!(
                "Received PlayerKilled message from server but couldnt find player locally"
            );
            continue;
        };
        info!(?killed_player_net_entity, "Hiding killed player");
        *player_visibility = Visibility::Hidden;
    }
}
