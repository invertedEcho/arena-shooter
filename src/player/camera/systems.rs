use crate::{
    common::systems::apply_render_layers_to_children,
    player::{
        PlayerState, camera::PLAYER_CAMERA_Y_OFFSET,
        shooting::components::PlayerWeapon,
    },
};
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    input::mouse::AccumulatedMouseMotion, pbr::NotShadowCaster, prelude::*,
    render::view::RenderLayers,
};

use crate::player::{Player, camera::PlayerCamera};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

pub fn setup_player_camera(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    player_query: Single<Entity, Added<Player>>,
) {
    let weapon_model = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("weapons/rifle/WA_2000.glb#Scene0"),
    );

    commands.entity(*player_query).with_children(|parent| {
        // camera for weapon render only, so weapon model is always on top of everything
        parent.spawn((
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            RenderLayers::layer(1),
        ));

        parent.spawn((
            Camera3d::default(),
            PlayerCamera::default(),
            Projection::from(PerspectiveProjection {
                fov: 80.0_f32.to_radians(),
                ..default()
            }),
            Transform::from_xyz(0.0, PLAYER_CAMERA_Y_OFFSET, 0.0),
        ));

        parent
            .spawn((
                SceneRoot(weapon_model),
                Transform {
                    translation: Vec3 {
                        x: 1.0,
                        y: -0.25,
                        z: -2.0,
                    },
                    scale: Vec3::splat(0.25),
                    // rotate 180 degrees as weapon is spawned wrong way
                    // radians are a different way of representing rotations
                    // PI = 180 degrees
                    // FRAC_PI_2 (e.g. PI / 2) = 90 degrees
                    rotation: Quat::from_rotation_y(PI),
                    ..default()
                },
                RenderLayers::layer(1),
                NotShadowCaster,
                // TODO: Its kinda weid that we spawn "PlayerWeapon" in `player/camera` module
                PlayerWeapon {
                    loaded_ammo: 30,
                    carried_ammo: 99999999,
                    max_loaded_ammo: 30,
                    moving_to_right: false,
                },
            ))
            .observe(apply_render_layers_to_children);
    });
}

pub fn camera_orbit_player(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player_camera_transform: Single<&mut Transform, With<PlayerCamera>>,
) {
    let delta = mouse_motion.delta;

    if delta != Vec2::ZERO {
        // pitch like nodding yes with your head
        let delta_pitch = -delta.y * 0.001;

        // yaw like nodding no with your head
        let delta_yaw = -delta.x * 0.002;

        // existing rotation
        let (current_yaw, current_pitch, current_roll) =
            player_camera_transform.rotation.to_euler(EulerRot::YXZ);

        let new_yaw = delta_yaw + current_yaw;

        let new_pitch =
            (delta_pitch + current_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        player_camera_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, current_roll);
        // now we just need to adjust yaw of player, so other players can also see in which direction
        // the person is looking.
        // note that we cant just simply update yaw of player, because then player camera yaw would
        // be duplicated? or something like that

        // later, we can adjust pitch of player weapon so other players can also see if aiming to
        // sky or to bottom.
    }
}

pub fn player_walk_animation(
    player: Single<&Player>,
    player_weapon_query: Query<(&mut Transform, &mut PlayerWeapon)>,
    time: Res<Time>,
) {
    for (mut player_weapon_transform, mut player_weapon) in player_weapon_query
    {
        // TODO: i feel like this can be simplified completely because we decrease/increase by
        // exact same step always, and boundary is always set to 0.5
        if player.state != PlayerState::Idle {
            let factor = if player.state == PlayerState::Walking {
                0.1
            } else {
                0.2
            };

            if player_weapon.moving_to_right {
                player_weapon_transform.translation.x +=
                    factor * time.delta_secs();
                if player_weapon_transform.translation.x >= 1.02 {
                    player_weapon.moving_to_right = false;
                }
            } else {
                player_weapon_transform.translation.x -=
                    factor * time.delta_secs();
                if player_weapon_transform.translation.x <= 0.98 {
                    player_weapon.moving_to_right = true;
                }
            }

            // TODO: move weapon up and down, like `f(x) = x^2`, e.g. at middle stop moving up/down
            // if player_weapon.moving_to_bottom {
            //     player_weapon_transform.translation.y -=
            //         0.2 * time.delta_secs();
            // } else {
            //     player_weapon_transform.translation.y +=
            //         0.2 * time.delta_secs();
            // }
        }
    }
}
