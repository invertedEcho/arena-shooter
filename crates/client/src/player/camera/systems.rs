use avian3d::math::FRAC_PI_2;
use bevy::{
    camera::visibility::RenderLayers, core_pipeline::Skybox,
    input::mouse::AccumulatedMouseMotion, prelude::*,
};
use lightyear::prelude::Controlled;
use shared::{
    components::DespawnTimer,
    player::{AimType, Player},
};

use crate::{
    game_flow::states::AppState,
    player::{
        camera::{
            PLAYER_CAMERA_Y_OFFSET,
            components::{
                FreeCam, MuzzleFlash, PlayerCameraState, PlayerWeaponModel,
                ViewModelCamera, WorldCamera,
            },
            weapon_positions::{
                get_muzzle_flash_position_for_weapon, get_position_for_weapon,
            },
        },
        shooting::{
            asset_paths::get_asset_path_for_weapon_type,
            components::PlayerWeapons,
            messages::{
                PlayerWeaponFiredMessage, PlayerWeaponSlotChangeMessage,
            },
        },
    },
    shared::WeaponType,
};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

pub fn setup_player_cameras(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    // using With<Controlled> ensures we only add cameras to our own player
    added_players: Query<Entity, (Added<Player>, With<Controlled>)>,
) {
    for added_player in added_players {
        debug!(
            "A new player was added and it has Controlled, e.g. its our \
             player. Spawning cameras as children..."
        );

        debug!("Inserting PlayerCameraState into player");
        commands
            .entity(added_player)
            .insert(PlayerCameraState::Normal);

        commands.entity(added_player).with_children(|parent| {
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
    player_query: Single<(&mut PlayerWeapons, &mut AimType), With<Controlled>>,
) {
    let (player_weapons, mut aim_type) = player_query.into_inner();

    if player_weapons.reloading {
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

        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            new_yaw_camera,
            new_pitch_camera,
            current_roll_camera,
        );
    }
}

// TODO: is this even needed? cant we just query for With<Camera>?
// 'w -> the world borrow lifetime, e.g. how long this query can read/write world data
// 'a -> the system lifetime, e.g. how long this query is valid inside a system function
// type AnyCamEntityQuery<'w, 's> = Query<
//     'w,
//     's,
//     Entity,
//     Or<(
//         With<ViewModelCamera>,
//         With<WorldCamera>,
//         With<PlayerWeaponModel>,
//     )>,
// >;

// FIXME: Reintroduce
//
// pub fn toggle_freecam(
//     client_local_player: Option<ResMut<ClientLocalPlayer>>,
//     player_query: Single<(&Transform, &mut PlayerCameraState)>,
//     mut commands: Commands,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     camera_entities: AnyCamEntityQuery,
//     free_cam_entity_query: Query<Entity, With<FreeCam>>,
//     mut spawn_player_cameras_message_writer: MessageWriter<
//         SpawnPlayerCamerasMessage,
//     >,
// ) {
//     info!("HELLO?");
//     if keyboard_input.just_pressed(KeyCode::KeyC) {
//         let Some(client_local_player) = client_local_player else {
//             return;
//         };
//
//         let (player_transform, mut player_camera_state) =
//             player_query.into_inner();
//         info!("okay freecam key pressed");
//
//         match *player_camera_state {
//             PlayerCameraState::Normal => {
//                 *player_camera_state = PlayerCameraState::FreeCam;
//                 for player_camera_entity in camera_entities {
//                     commands.entity(player_camera_entity).despawn();
//                 }
//                 commands.spawn((
//                     Camera3d::default(),
//                     Projection::from(PerspectiveProjection {
//                         fov: 80.0_f32.to_radians(),
//                         ..default()
//                     }),
//                     Transform::from_xyz(
//                         player_transform.translation.x,
//                         player_transform.translation.y + 2.0,
//                         player_transform.translation.z,
//                     ),
//                     FreeCam,
//                     DespawnOnExit(AppState::InGame),
//                 ));
//             }
//             PlayerCameraState::FreeCam => {
//                 info!("requested freecam -> normal");
//                 *player_camera_state = PlayerCameraState::Normal;
//                 info!("player camera state now set to normal");
//                 for free_cam_entity in free_cam_entity_query {
//                     debug!("despawning free cam entity {}", free_cam_entity);
//                     commands.entity(free_cam_entity).despawn();
//                 }
//                 spawn_player_cameras_message_writer
//                     .write(SpawnPlayerCamerasMessage(client_local_player.0));
//             }
//         }
//     }
// }

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
    player_query: Single<(&PlayerWeapons, &AimType)>,
) {
    let (player_weapons, aim_type) = player_query.into_inner();

    for _ in player_shot_message_reader.read() {
        let active_weapon = player_weapons.active_slot;
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
                base_color_texture: Some(asset_server.load("muzzle_flash.png")),
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
    player_query: Single<(&PlayerWeapons, &AimType)>,
    mut player_weapon_model_transform: Single<
        &mut Transform,
        With<PlayerWeaponModel>,
    >,
    time: Res<Time>,
) {
    const SPEED: f32 = 20.0;

    let player_weapons = player_query.0;
    let reloading = player_weapons.reloading;

    let mut target_destination = get_position_for_weapon(
        &player_weapons.weapons[player_weapons.active_slot]
            .stats
            .weapon_type,
        player_query.1,
    );

    if reloading {
        target_destination = target_destination.with_y(-1.0);
    }

    let speed = if reloading { 10.0 } else { SPEED };

    player_weapon_model_transform.translation = player_weapon_model_transform
        .translation
        .lerp(target_destination, time.delta_secs() * speed);
}

pub fn do_weapon_kickback(
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
