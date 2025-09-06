use crate::{
    common::systems::apply_render_layers_to_children,
    player::{
        camera::{PLAYER_CAMERA_Y_OFFSET, components::PlayerCameraMode},
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
                PlayerWeapon,
            ))
            .observe(apply_render_layers_to_children);
    });
}

pub fn camera_orbit_player(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player_camera_query: Single<
        (&PlayerCamera, &mut Transform),
        Without<Player>,
    >,
    // mut player_query: Single<&mut Transform, With<Player>>,
) {
    let player_camera = player_camera_query.0;
    if !player_camera.mouse_motion_enabled {
        return;
    }

    let delta = mouse_motion.delta;

    if delta != Vec2::ZERO {
        // pitch like nodding yes with your head
        let delta_pitch = -delta.y * 0.001;

        // yaw like nodding no with your head
        let delta_yaw = -delta.x * 0.002;

        // existing rotation
        let (current_yaw, current_pitch, current_roll) =
            player_camera_query.1.rotation.to_euler(EulerRot::YXZ);

        let new_yaw = delta_yaw + current_yaw;

        let new_pitch =
            (delta_pitch + current_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        player_camera_query.1.rotation =
            Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, current_roll);
        // now we just need to adjust yaw of player, so other players can also see in which direction
        // the person is looking.
        // note that we cant just simply update yaw of player, because then player camera yaw would
        // be duplicated? or something like that

        // later, we can adjust pitch of player weapon so other players can also see if aiming to
        // sky or to bottom.
    }
}

pub fn update_player_camera_distance(
    player_camera_query: Single<
        (&mut Transform, &PlayerCamera),
        Changed<PlayerCamera>,
    >,
) {
    let player_camera_mode = &player_camera_query.1.mode;
    let mut player_camera_translation = player_camera_query.0.translation;

    info!("new player camera mode: {:?}", player_camera_mode);
    if *player_camera_mode == PlayerCameraMode::ThirdPerson {
        player_camera_translation.z = 3.0;
    } else if *player_camera_mode == PlayerCameraMode::FirstPerson {
        player_camera_translation.z = 0.0;
    }

    info!(
        "new player camera translation z: {}",
        player_camera_translation.z
    );
}

pub fn switch_between_first_and_third_person(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Single<&mut PlayerCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyV) {
        match query.mode {
            PlayerCameraMode::FirstPerson => {
                query.mode = PlayerCameraMode::ThirdPerson;
            }
            PlayerCameraMode::ThirdPerson => {
                query.mode = PlayerCameraMode::FirstPerson;
            }
        }
    }
}

pub fn change_mouse_motion_enabled(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Single<&mut PlayerCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        query.mouse_motion_enabled = !query.mouse_motion_enabled;
    }
}
