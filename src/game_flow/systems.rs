use avian3d::prelude::ColliderConstructorHierarchyReady;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_rerecast::{Navmesh, prelude::NavmeshReady};

use crate::{
    enemy::Enemy,
    game_flow::{
        game_mode::{GameModeState, StartWaveGameModeMessage},
        states::{
            AppState, InGameState, LoadingGameSubState, SelectedMapState,
        },
    },
    nav_mesh_pathfinding::NavMeshHandle,
    player::{Player, camera::components::FreeCam, spawn::SpawnPlayerMessage},
    user_interface::main_menu::MainMenuCamera,
    world::resources::WorldSceneHandle,
};

pub fn grab_mouse(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    primary_cursor_options.visible = false;
    primary_cursor_options.grab_mode = CursorGrabMode::Locked;
}

pub fn free_mouse(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    primary_cursor_options.visible = true;
    primary_cursor_options.grab_mode = CursorGrabMode::None;
}

pub fn handle_escape_in_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_in_game_state: If<Res<State<InGameState>>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    let escape_just_pressed = keyboard_input.just_pressed(KeyCode::Escape);
    let current_in_game_state = current_in_game_state.get();

    if escape_just_pressed {
        match current_in_game_state {
            InGameState::Playing => {
                next_in_game_state.set(InGameState::Paused);
            }
            InGameState::PlayerDead => {}
            InGameState::Paused => next_in_game_state.set(InGameState::Playing),
        }
    }
}

pub fn spawn_main_menu_camera(mut commands: Commands) {
    info!("Spawning Main Menu Camera");
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
    ));
}

pub fn handle_exit_in_game(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    players: Query<Entity, (With<Player>, Without<Enemy>)>,
    free_cam_cameras: Query<Entity, With<FreeCam>>,
) {
    info!("Despawning all enemies");
    for enemy in enemies {
        commands.entity(enemy).despawn();
    }
    info!("Despawning all players");
    for player in players {
        commands.entity(player).despawn();
    }
    for free_cam_camera in free_cam_cameras {
        info!("Despawning free cam");
        commands.entity(free_cam_camera).despawn();
    }

    info!("Spawning Main Menu Camera");
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
    ));
}

pub fn check_world_scene_loaded(
    mut asset_event_message_reader: MessageReader<AssetEvent<Scene>>,
    maybe_world_scene_handle: Option<Res<WorldSceneHandle>>,
    mut next_game_loading_state: ResMut<NextState<LoadingGameSubState>>,
    selected_map_state: If<Res<State<SelectedMapState>>>,
) {
    for asset_event in asset_event_message_reader.read() {
        if let AssetEvent::LoadedWithDependencies { id } = asset_event
            && let Some(ref world_scene_handle) = maybe_world_scene_handle
            && *id == world_scene_handle.0.id()
        {
            info!("Map assets loaded!");
            next_game_loading_state
                .set(LoadingGameSubState::MapLoadedWithDependencies);
            if *selected_map_state.get() == SelectedMapState::TinyTown {
                next_game_loading_state
                    .set(LoadingGameSubState::CollidersReady);
            }
        }
    }
}

// this is only relevant for Map::MediumPlastic, because tiny town map has colliders in skein
pub fn check_collider_constructor_hierarchy_ready(
    _: On<ColliderConstructorHierarchyReady>,
    mut next_game_loading_state: ResMut<NextState<LoadingGameSubState>>,
) {
    info!("Collider_constructor_hierarchy_ready is now!");
    next_game_loading_state.set(LoadingGameSubState::CollidersReady);
}

pub fn check_navmesh_ready(
    trigger: On<NavmeshReady>,
    mut commands: Commands,
    mut next_game_loading_state: ResMut<NextState<LoadingGameSubState>>,
    mut nav_meshes: ResMut<Assets<Navmesh>>,
) {
    let Some(nav_mesh_handle) = nav_meshes.get_strong_handle(trigger.0) else {
        panic!(
            "Got navmeshready event but the Handle could not be found using \
             the asset id from the trigger"
        );
    };

    info!("Navmesh is now ready!");
    next_game_loading_state.set(LoadingGameSubState::NavMeshReady);

    // need to store it in our own resource so we can call regenerate when a new map is selected
    commands.insert_resource(NavMeshHandle(nav_mesh_handle));
    info!("Stored navmesh handle in our own resource, `NavMeshHandle`");
}

pub fn on_game_loading_state_nav_mesh_ready(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut start_game_mode_message_writer: MessageWriter<StartWaveGameModeMessage>,
    game_mode_state: Res<State<GameModeState>>,
) {
    info!(
        "Entered nav_mesh_ready loading state, everything is ready now. \
         Writing StartGameModeMessage.",
    );
    next_app_state.set(AppState::InGame);
    match game_mode_state.get() {
        GameModeState::Waves => {
            start_game_mode_message_writer.write(StartWaveGameModeMessage);
        }
        GameModeState::FreePlay => {}
    }
}

pub fn on_exit_main_menu(
    mut commands: Commands,
    main_menu_camera: Single<Entity, With<MainMenuCamera>>,
) {
    info!("Entered playing state, despawning main menu camera");
    commands.entity(*main_menu_camera).despawn();
}

pub fn on_enter_app_state_in_game(
    mut spawn_player_message_writer: MessageWriter<SpawnPlayerMessage>,
) {
    spawn_player_message_writer.write(SpawnPlayerMessage);
}
