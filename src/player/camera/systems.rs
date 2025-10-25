use crate::{
    player::camera::{
        PLAYER_CAMERA_Y_OFFSET,
        components::{
            FreeCam, PlayerCameraState, PlayerWeaponModel, WorldModelCamera,
        },
        messages::SpawnPlayerCamerasMessage,
    },
    shared::systems::apply_render_layers_to_children,
};
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    camera::visibility::RenderLayers, input::mouse::AccumulatedMouseMotion,
    light::NotShadowCaster, prelude::*,
};
use bevy_inspector_egui::bevy_egui;

use crate::player::animate::PLAYER_ARM_WEAPON_PATH;
use crate::player::{Player, camera::ViewModelCamera};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

pub fn setup_player_cameras(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    player_entity: Single<Entity, With<Player>>,
    mut spawn_player_cameras_message_reader: MessageReader<
        SpawnPlayerCamerasMessage,
    >,
) {
    for _ in spawn_player_cameras_message_reader.read() {
        info!(
            "Received SpawnPlayerCamerasMessage, spawning weapon model and \
             player cameras"
        );

        let weapon_model = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(PLAYER_ARM_WEAPON_PATH));

        commands.entity(*player_entity).with_children(|parent| {
            parent.spawn((
                WorldModelCamera,
                Camera {
                    order: 0,
                    ..default()
                },
                Camera3d::default(),
                Transform::from_xyz(0.0, PLAYER_CAMERA_Y_OFFSET, 0.0),
                Projection::from(PerspectiveProjection {
                    fov: 90.0_f32.to_radians(),
                    ..default()
                }),
            ));

            parent.spawn((
                ViewModelCamera,
                Camera3d::default(),
                Camera {
                    order: 1,
                    ..default()
                },
                // needed so our inspector is shown again when we enter game, as we despawn
                // `WorldUiCamera` and spawn player camera
                bevy_egui::PrimaryEguiContext,
                RenderLayers::layer(1),
                Transform::from_xyz(0.0, PLAYER_CAMERA_Y_OFFSET, 0.0),
            ));
            parent
                .spawn((
                    SceneRoot(weapon_model),
                    Transform {
                        translation: Vec3 {
                            x: 0.05,
                            y: -0.14,
                            z: -0.2,
                        },
                        scale: Vec3::splat(1.0),
                        // rotate 180 degrees as weapon is spawned wrong way
                        // radians are a different way of representing rotations
                        // PI = 180 degrees
                        // FRAC_PI_2 (e.g. PI / 2) = 90 degrees
                        rotation: Quat::from_rotation_y(PI),
                    },
                    RenderLayers::layer(1),
                    NotShadowCaster,
                    PlayerWeaponModel,
                    Visibility::Visible,
                ))
                .observe(apply_render_layers_to_children);
        });
    }
}

/// We seperate between player transform and camera transform.
/// This is because if the user would look straight down, the collider would literally lay on the
/// ground. So, we only change yaw of player, and do pitch via the camera transform
pub fn update_yaw_pitch_on_mouse_motion(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut world_model_camera_transform: Single<
        &mut Transform,
        (With<WorldModelCamera>, Without<Player>),
    >,
    mut player_transform: Single<&mut Transform, With<Player>>,
) {
    let delta = mouse_motion.delta;

    if delta != Vec2::ZERO {
        // pitch like nodding yes with your head
        let delta_pitch = -delta.y * 0.001;

        // yaw like nodding no with your head
        let delta_yaw = -delta.x * 0.002;

        // existing rotation
        let (current_yaw_camera, current_pitch_camera, current_roll_camera) =
            world_model_camera_transform
                .rotation
                .to_euler(EulerRot::YXZ);

        let (current_yaw_player, current_pitch_player, current_roll_player) =
            player_transform.rotation.to_euler(EulerRot::YXZ);

        let new_yaw_player = delta_yaw + current_yaw_player;

        let new_pitch_camera = (delta_pitch + current_pitch_camera)
            .clamp(-PITCH_LIMIT, PITCH_LIMIT);

        world_model_camera_transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            current_yaw_camera,
            new_pitch_camera,
            current_roll_camera,
        );
        player_transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            new_yaw_player,
            current_pitch_player,
            current_roll_player,
        );
    }
}

pub fn toggle_freecam(
    mut player_query: Single<(Entity, &Transform, &mut Player)>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_camera_entities_query: Query<
        Entity,
        Or<(
            With<ViewModelCamera>,
            With<WorldModelCamera>,
            With<PlayerWeaponModel>,
        )>,
    >,
    free_cam_entity_query: Query<Entity, With<FreeCam>>,
    mut spawn_player_cameras_message_writer: MessageWriter<
        SpawnPlayerCamerasMessage,
    >,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        match player_query.2.camera_state {
            PlayerCameraState::Normal => {
                player_query.2.camera_state = PlayerCameraState::FreeCam;
                for player_camera_entity in player_camera_entities_query {
                    commands.entity(player_camera_entity).despawn();
                }
                let player_transform = player_query.1;
                commands.spawn((
                    Camera3d::default(),
                    Projection::from(PerspectiveProjection {
                        fov: 80.0_f32.to_radians(),
                        ..default()
                    }),
                    Transform::from_xyz(
                        player_transform.translation.x,
                        player_transform.translation.y + 2.0,
                        player_transform.translation.z,
                    ),
                    FreeCam,
                ));
            }
            PlayerCameraState::FreeCam => {
                info!("requested freecam -> normal");
                player_query.2.camera_state = PlayerCameraState::Normal;
                info!("player camera state now set to normal");
                for free_cam_entity in free_cam_entity_query {
                    debug!("despawning free cam entity {}", free_cam_entity);
                    commands.entity(free_cam_entity).despawn();
                }
                spawn_player_cameras_message_writer
                    .write(SpawnPlayerCamerasMessage);
            }
        }
    }
}

pub fn handle_free_cam_movement(
    mut free_cam_transform: Single<&mut Transform, With<FreeCam>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec3::ZERO;
    let speed = 0.02;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += *free_cam_transform.forward();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction -= *free_cam_transform.forward();
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        direction -= *free_cam_transform.right();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += *free_cam_transform.right();
    }

    if keyboard_input.pressed(KeyCode::KeyQ) {
        direction += *free_cam_transform.up();
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        direction -= *free_cam_transform.up();
    }

    if direction != Vec3::ZERO {
        direction = direction.normalize();
    }

    free_cam_transform.translation += direction * speed;
}

pub fn free_cam_orbit(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut free_cam_transform: Single<&mut Transform, With<FreeCam>>,
) {
    let delta = mouse_motion.delta;

    if delta != Vec2::ZERO {
        // pitch like nodding yes with your head
        let delta_pitch = -delta.y * 0.001;

        // yaw like nodding no with your head
        let delta_yaw = -delta.x * 0.002;

        // existing rotation
        let (current_yaw, current_pitch, current_roll) =
            free_cam_transform.rotation.to_euler(EulerRot::YXZ);

        let new_yaw = delta_yaw + current_yaw;

        let new_pitch =
            (delta_pitch + current_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        free_cam_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, current_roll);
    }
}

pub fn make_player_weapon_visible(
    mut player_weapon: Single<&mut Visibility, With<PlayerWeaponModel>>,
) {
    **player_weapon = Visibility::Visible;
}

pub fn make_player_weapon_hidden(
    mut player_weapon: Single<&mut Visibility, With<PlayerWeaponModel>>,
) {
    **player_weapon = Visibility::Hidden;
}
