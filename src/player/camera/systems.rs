use crate::{
    game_flow::states::AppState,
    player::{
        camera::{
            PLAYER_CAMERA_Y_OFFSET,
            components::{
                FreeCam, PlayerCameraState, PlayerWeaponModel, WorldModelCamera,
            },
            messages::SpawnPlayerCamerasMessage,
            weapon_positions::{AimType, get_position_for_weapon},
        },
        shooting::{
            asset_paths::get_asset_path_for_weapon_type,
            components::PlayerWeapon,
        },
    },
    shared::{WeaponType, systems::apply_render_layers_to_children},
};
use std::f32::consts::FRAC_PI_2;

use bevy::{
    camera::visibility::RenderLayers, input::mouse::AccumulatedMouseMotion,
    light::NotShadowCaster, prelude::*,
};
use bevy_inspector_egui::bevy_egui;

use crate::player::{Player, camera::ViewModelCamera};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

pub fn setup_player_cameras(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut message_reader: MessageReader<SpawnPlayerCamerasMessage>,
) {
    for message in message_reader.read() {
        info!(
            "Received SpawnPlayerCamerasMessage, spawning weapon model and \
             player cameras"
        );

        info!("Inserting player cameras into player entity {}", message.0);
        commands.entity(message.0).with_children(|parent| {
            parent.spawn((
                Name::new("WorldModelCamera"),
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
                Name::new("ViewModelCamera"),
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

            let weapon_model_path =
                get_asset_path_for_weapon_type(&WeaponType::Pistol);
            let weapon_model = asset_server
                .load(GltfAssetLabel::Scene(0).from_asset(weapon_model_path));

            let weapon_position =
                get_position_for_weapon(&WeaponType::Pistol, AimType::Normal);
            parent
                .spawn((
                    Name::new("PlayerWeaponModel"),
                    SceneRoot(weapon_model),
                    Transform {
                        translation: weapon_position,
                        scale: Vec3::splat(3.0),
                        rotation: Quat::from_rotation_y(FRAC_PI_2),
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

pub fn handle_player_scope_aim(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut player_weapon_model_transform: Single<
        &mut Transform,
        With<PlayerWeaponModel>,
    >,
    player_weapon: Single<&PlayerWeapon>,
) {
    if player_weapon.reloading {
        return;
    }

    if mouse_input.just_pressed(MouseButton::Right) {
        let weapon_position = get_position_for_weapon(
            &player_weapon.weapon_type,
            AimType::Scoped,
        );
        player_weapon_model_transform.translation = weapon_position;
    } else if mouse_input.just_released(MouseButton::Right) {
        let weapon_position = get_position_for_weapon(
            &player_weapon.weapon_type,
            AimType::Normal,
        );
        player_weapon_model_transform.translation = weapon_position;
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

    if delta == Vec2::ZERO {
        return;
    }

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

    let new_pitch_camera =
        (delta_pitch + current_pitch_camera).clamp(-PITCH_LIMIT, PITCH_LIMIT);

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

// 'w -> the world borrow lifetime, e.g. how long this query can read/write world data
// 'a -> the system lifetime, e.g. how long this query is valid inside a system function
type AnyCamEntityQuery<'w, 's> = Query<
    'w,
    's,
    Entity,
    Or<(
        With<ViewModelCamera>,
        With<WorldModelCamera>,
        With<PlayerWeaponModel>,
    )>,
>;

pub fn toggle_freecam(
    player_query: Single<(Entity, &Transform, &mut PlayerCameraState)>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_entities: AnyCamEntityQuery,
    free_cam_entity_query: Query<Entity, With<FreeCam>>,
    mut spawn_player_cameras_message_writer: MessageWriter<
        SpawnPlayerCamerasMessage,
    >,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        let (player_entity, player_transform, mut player_camera_state) =
            player_query.into_inner();

        match *player_camera_state {
            PlayerCameraState::Normal => {
                *player_camera_state = PlayerCameraState::FreeCam;
                for player_camera_entity in camera_entities {
                    commands.entity(player_camera_entity).despawn();
                }
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
                    DespawnOnExit(AppState::InGame),
                ));
            }
            PlayerCameraState::FreeCam => {
                info!("requested freecam -> normal");
                *player_camera_state = PlayerCameraState::Normal;
                info!("player camera state now set to normal");
                for free_cam_entity in free_cam_entity_query {
                    debug!("despawning free cam entity {}", free_cam_entity);
                    commands.entity(free_cam_entity).despawn();
                }
                spawn_player_cameras_message_writer
                    .write(SpawnPlayerCamerasMessage(player_entity));
            }
        }
    }
}

pub fn handle_free_cam_movement(
    mut free_cam_transform: Single<&mut Transform, With<FreeCam>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec3::ZERO;
    let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
        0.10
    } else {
        0.02
    };

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

pub fn weapon_sway(
    time: Res<Time>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut transform: Single<&mut Transform, With<PlayerWeaponModel>>,
) {
    // how fast it will return to initial position
    const DAMPING: f32 = 12.0;

    // how strong the sway is
    const SWAY_MULTIPLIER: f32 = 0.0005;

    let delta = mouse_motion.delta;

    let pitch = -delta.y * SWAY_MULTIPLIER;
    let yaw = delta.x * SWAY_MULTIPLIER;

    let sway_rot = Quat::from_rotation_x(pitch) * Quat::from_rotation_y(yaw);

    transform.rotation *= sway_rot;

    transform.rotation = transform.rotation.slerp(
        Quat::from_rotation_y(FRAC_PI_2),
        DAMPING * time.delta_secs(),
    );
}
