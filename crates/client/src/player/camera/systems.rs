use avian3d::{math::FRAC_PI_2, prelude::LinearVelocity};
use bevy::{
    camera::visibility::RenderLayers, core_pipeline::Skybox,
    input::mouse::AccumulatedMouseMotion, prelude::*,
};
use lightyear::prelude::Controlled;
use shared::{
    components::DespawnTimer,
    player::{AimType, Player, PlayerState},
    shooting::{PlayerWeapons, WeaponType},
};

use crate::{
    game_flow::states::AppState,
    player::{
        camera::{
            PLAYER_CAMERA_Y_OFFSET, SpawnPlayerCamera,
            components::{
                FreeCam, MainMenuCamera, MuzzleFlash, PlayerCameraState,
                PlayerWeaponModel, ViewModelCamera, WorldCamera,
            },
            weapon_positions::{
                get_muzzle_flash_position_for_weapon, get_position_for_weapon,
            },
        },
        shooting::{
            asset_paths::get_asset_path_for_weapon_type,
            components::ShootRecoil,
            messages::{
                PlayerWeaponFiredMessage, PlayerWeaponSlotChangeMessage,
            },
        },
    },
};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

pub fn setup_player_cameras(
    // using With<Controlled> ensures we only add cameras to our own player
    added_players: Query<Entity, (Added<Player>, With<Controlled>)>,
    mut message_writer: MessageWriter<SpawnPlayerCamera>,
) {
    for added_player in added_players {
        message_writer.write(SpawnPlayerCamera(added_player));
    }
}

pub fn handle_spawn_player_camera_message(
    mut message_reader: MessageReader<SpawnPlayerCamera>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    main_menu_camera: Query<Entity, With<MainMenuCamera>>,
) {
    for message in message_reader.read() {
        info!(
            "Spawning new player camera, received SpawnPlayerCamera message!"
        );

        for main_menu_camera in main_menu_camera {
            info!("Despawning main menu camera before spawning player camera");
            commands.entity(main_menu_camera).despawn();
        }
        commands.entity(message.0).insert(PlayerCameraState::Normal);

        commands.entity(message.0).with_children(|parent| {
            parent.spawn((
                Name::new("WorldCamera"),
                WorldCamera,
                Camera {
                    order: 0,
                    ..default()
                },
                Camera3d::default(),
                Transform::from_xyz(0.0, PLAYER_CAMERA_Y_OFFSET, 0.0),
                Projection::from(PerspectiveProjection {
                    fov: 80.0_f32.to_radians(),
                    ..default()
                }),
                DespawnOnExit(AppState::InGame),
                EnvironmentMapLight {
                    diffuse_map: asset_server
                        .load("lightmaps/voortrekker_interior_1k_diffuse.ktx2"),
                    specular_map: asset_server.load(
                        "lightmaps/voortrekker_interior_1k_specular.ktx2",
                    ),
                    intensity: 1500.0,
                    ..default()
                },
                Skybox {
                    image: asset_server.load("skyboxes/skybox_main.ktx2"),
                    brightness: 1000.0,
                    ..default()
                },
            ));

            let weapon_model_path =
                get_asset_path_for_weapon_type(&WeaponType::AssaultRifle);
            let weapon_model = asset_server
                .load(GltfAssetLabel::Scene(0).from_asset(weapon_model_path));
            let weapon_position = get_position_for_weapon(
                &WeaponType::AssaultRifle,
                &AimType::Normal,
            );
            parent
                .spawn((
                    Name::new("ViewModelCamera"),
                    ViewModelCamera,
                    Camera3d::default(),
                    Camera {
                        order: 1,
                        ..default()
                    },
                    RenderLayers::layer(1),
                    Transform::from_xyz(0.0, PLAYER_CAMERA_Y_OFFSET, 0.0),
                ))
                .with_child((
                    Name::new("PlayerWeaponModel"),
                    SceneRoot(weapon_model),
                    Transform {
                        translation: weapon_position,
                        scale: Vec3::splat(2.0),
                        rotation: Quat::from_rotation_y(FRAC_PI_2),
                    },
                    PlayerWeaponModel,
                    Visibility::Visible,
                    RenderLayers::layer(1),
                ));
        });
    }
}

pub fn handle_player_scope_aim(
    mouse_input: Res<ButtonInput<MouseButton>>,
    player_query: Single<(&mut AimType, &PlayerState), With<Controlled>>,
) {
    let (mut aim_type, player_state) = player_query.into_inner();

    if player_state.reloading {
        return;
    }

    if mouse_input.just_pressed(MouseButton::Right) {
        *aim_type = AimType::Scoped;
    } else if mouse_input.just_released(MouseButton::Right) {
        *aim_type = AimType::Normal;
    }
}

type AnyCamera = Or<(With<WorldCamera>, With<ViewModelCamera>)>;

pub fn update_yaw_pitch_on_mouse_motion(
    mouse_motion: Res<AccumulatedMouseMotion>,
    camera_transforms: Query<&mut Transform, AnyCamera>,
    mut shoot_recoil: Single<&mut ShootRecoil>,
) {
    let delta = mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    let delta_pitch = -delta.y * 0.001;
    let delta_yaw = -delta.x * 0.002;

    for mut transform in camera_transforms {
        let (current_yaw_camera, current_pitch_camera, current_roll_camera) =
            transform.rotation.to_euler(EulerRot::YXZ);

        let new_yaw_camera = delta_yaw + current_yaw_camera;

        let new_pitch_camera = (delta_pitch + current_pitch_camera)
            .clamp(-PITCH_LIMIT, PITCH_LIMIT);

        let new_rotation = Quat::from_euler(
            EulerRot::YXZ,
            new_yaw_camera,
            new_pitch_camera,
            current_roll_camera,
        );
        transform.rotation = new_rotation;
        shoot_recoil.original_rotation = new_rotation;
    }
}

pub fn toggle_freecam(
    player_query: Single<(
        Entity,
        &Transform,
        &mut PlayerCameraState,
        &mut LinearVelocity,
    )>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_entities: Query<Entity, With<Camera>>,
    mut message_writer: MessageWriter<SpawnPlayerCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        let (
            player_entity,
            player_transform,
            mut player_camera_state,
            mut player_velocity,
        ) = player_query.into_inner();
        player_velocity.0 = Vec3::ZERO;

        for camera_entity in camera_entities {
            commands.entity(camera_entity).despawn();
        }

        match *player_camera_state {
            PlayerCameraState::Normal => {
                info!("PlayerCameraState updated to FreeCam");
                *player_camera_state = PlayerCameraState::FreeCam;

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
                    Name::new("Free Cam"),
                    DespawnOnExit(AppState::InGame),
                ));
            }
            PlayerCameraState::FreeCam => {
                info!("PlayerCameraState updated to Normal");
                *player_camera_state = PlayerCameraState::Normal;

                message_writer.write(SpawnPlayerCamera(player_entity));
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
        0.25
    } else {
        0.10
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

pub fn update_player_weapon_model(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut message_reader: MessageReader<PlayerWeaponSlotChangeMessage>,
    player_weapon_model_query: Single<
        (Entity, &mut Transform),
        With<PlayerWeaponModel>,
    >,
    player_query: Single<(&PlayerWeapons, &AimType)>,
) {
    let (player_weapon_model_entity, mut player_weapon_model_transform) =
        player_weapon_model_query.into_inner();

    for message in message_reader.read() {
        let player_weapons = player_query.0;
        let aim_type = player_query.1;

        let new_slot_index = message.0;

        let weapon_type =
            &player_weapons.weapons[new_slot_index].stats.weapon_type;

        let weapon_position = get_position_for_weapon(weapon_type, aim_type);

        let model_path = get_asset_path_for_weapon_type(weapon_type);

        let weapon_model =
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_path));

        player_weapon_model_transform.translation = weapon_position;
        commands
            .entity(player_weapon_model_entity)
            .insert((SceneRoot(weapon_model),));
    }
}

pub fn spawn_muzzle_flash(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_shot_message_reader: MessageReader<PlayerWeaponFiredMessage>,
    player_weapon_model_entity: Single<Entity, With<PlayerWeaponModel>>,
    player_query: Single<(&PlayerWeapons, &AimType, &PlayerState)>,
) {
    let (player_weapons, aim_type, player_state) = player_query.into_inner();

    for _ in player_shot_message_reader.read() {
        let active_weapon = player_state.active_weapon_slot;
        let muzzle_flash_position = get_muzzle_flash_position_for_weapon(
            &player_weapons.weapons[active_weapon].stats.weapon_type,
            aim_type,
        );

        commands.entity(*player_weapon_model_entity).with_child((
            Transform {
                translation: muzzle_flash_position,
                rotation: Quat::from_euler(EulerRot::XYZ, 0.0, -FRAC_PI_2, 0.0),
                scale: Vec3::splat(2.0),
            },
            MuzzleFlash,
            Mesh3d(meshes.add(Plane3d {
                half_size: Vec2::splat(0.1),
                normal: Dir3::Z,
            })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("hud/muzzle_flash.png"),
                ),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            DespawnTimer(Timer::from_seconds(0.05, TimerMode::Once)),
            RenderLayers::layer(1),
        ));
    }
}

pub fn interpolate_weapon_position(
    player_query: Single<(&PlayerWeapons, &AimType, &PlayerState)>,
    mut player_weapon_model_transform: Single<
        &mut Transform,
        With<PlayerWeaponModel>,
    >,
    time: Res<Time>,
) {
    const SPEED: f32 = 20.0;

    let (player_weapons, aim_type, player_state) = player_query.into_inner();

    let reloading = player_state.reloading;

    let weapon_type = &player_weapons.weapons[player_state.active_weapon_slot]
        .stats
        .weapon_type;

    let mut target_destination = get_position_for_weapon(weapon_type, aim_type);

    if reloading {
        match weapon_type {
            WeaponType::Pistol => {
                target_destination.y -= 0.25;
            }
            WeaponType::AssaultRifle => {
                target_destination.y -= 0.3;
            }
        }
    }

    let speed = if reloading { 10.0 } else { SPEED };

    player_weapon_model_transform.translation = player_weapon_model_transform
        .translation
        .lerp(target_destination, time.delta_secs() * speed);
}

pub fn weapon_model_kickback(
    mut player_weapon_model_transform: Single<
        &mut Transform,
        With<PlayerWeaponModel>,
    >,
    mut player_shot_message_reader: MessageReader<PlayerWeaponFiredMessage>,
) {
    for _ in player_shot_message_reader.read() {
        player_weapon_model_transform.rotation *= Quat::from_rotation_z(0.15);
    }
}

pub fn recoil_camera_kickback(
    mut world_camera: Single<&mut Transform, With<WorldCamera>>,
    mut player_shot_message_reader: MessageReader<PlayerWeaponFiredMessage>,
) {
    for _ in player_shot_message_reader.read() {
        world_camera.rotation *= Quat::from_rotation_x(0.025);
    }
}

pub fn recoil_slerp_back(
    shoot_recoil: Single<&mut ShootRecoil>,
    mut world_camera: Single<&mut Transform, With<WorldCamera>>,
    time: Res<Time>,
) {
    world_camera.rotation = world_camera
        .rotation
        .slerp(shoot_recoil.original_rotation, 3.5 * time.delta_secs());
}
