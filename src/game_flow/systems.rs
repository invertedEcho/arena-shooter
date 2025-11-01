use avian3d::prelude::ColliderConstructorHierarchyReady;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_rerecast::{Navmesh, prelude::NavmeshReady};

use crate::{
    enemy::Enemy,
    game_flow::{
        AppState,
        game_mode::{GameModeState, StartGameModeMessage},
        states::{GameLoadingState, InGameState, SelectedMapState},
    },
    nav_mesh_pathfinding::NavMeshHandle,
    player::{Player, camera::components::FreeCam},
    user_interface::main_menu::MainMenuCamera,
    world::resources::WorldSceneHandle,
};

pub fn grab_mouse(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    info!("Grabbing mouse");
    primary_cursor_options.visible = false;
    primary_cursor_options.grab_mode = CursorGrabMode::Locked;
}

pub fn free_mouse(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    primary_cursor_options.visible = true;
    primary_cursor_options.grab_mode = CursorGrabMode::None;
}

pub fn handle_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_app_state: Res<State<AppState>>,
    current_in_game_state: Res<State<InGameState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match *current_app_state.get() {
            AppState::InGame => match *current_in_game_state.get() {
                InGameState::None => {}
                InGameState::Playing => {
                    next_in_game_state.set(InGameState::Paused);
                }
                InGameState::PlayerDead => {}
                InGameState::Paused => {
                    next_in_game_state.set(InGameState::Playing)
                }
            },
            AppState::MainMenu => {}
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
    mut next_game_loading_state: ResMut<NextState<GameLoadingState>>,
    selected_map_state: Res<State<SelectedMapState>>,
) {
    for asset_event in asset_event_message_reader.read() {
        if let AssetEvent::LoadedWithDependencies { id } = asset_event
            && let Some(ref world_scene_handle) = maybe_world_scene_handle
            && *id == world_scene_handle.0.id()
        {
            info!("Map assets loaded!");
            next_game_loading_state
                .set(GameLoadingState::WorldLoadedWithDependencies);
            if *selected_map_state.get() == SelectedMapState::TinyTown {
                next_game_loading_state.set(GameLoadingState::CollidersReady);
            }
        }
    }
}

// this is only relevant for Map::MediumPlastic, because tiny town map has colliders in skein
pub fn check_collider_constructor_hierarchy_ready(
    _: On<ColliderConstructorHierarchyReady>,
    mut next_game_loading_state: ResMut<NextState<GameLoadingState>>,
) {
    info!("Collider collider_constructor_hierarchy_ready is now!");
    next_game_loading_state.set(GameLoadingState::CollidersReady);
}

pub fn check_navmesh_ready(
    trigger: On<NavmeshReady>,
    mut commands: Commands,
    mut next_game_loading_state: ResMut<NextState<GameLoadingState>>,
    mut nav_meshes: ResMut<Assets<Navmesh>>,
) {
    let Some(nav_mesh_handle) = nav_meshes.get_strong_handle(trigger.0) else {
        panic!(
            "Got navmeshready event but the Handle could not be found using \
             the asset id from the trigger"
        );
    };

    info!("Navmesh is now ready!");
    next_game_loading_state.set(GameLoadingState::NavMeshReady);

    // need to store it in our own resource so we can call regenerate when a new map is selected
    commands.insert_resource(NavMeshHandle(nav_mesh_handle));
    info!("Stored navmesh handle in our own resource, `NavMeshHandle`");
}

pub fn on_game_loading_state_nav_mesh_ready(
    mut start_game_mode_message_writer: MessageWriter<StartGameModeMessage>,
    game_mode_state: Res<State<GameModeState>>,
) {
    info!(
        "Entered nav_mesh_ready loading state, everything is ready now. \
         Writing StartGameModeMessage"
    );
    start_game_mode_message_writer
        .write(StartGameModeMessage(game_mode_state.get().clone()));
}

pub fn handle_playing_state_enter(
    mut commands: Commands,
    main_menu_camera: Single<Entity, With<MainMenuCamera>>,
) {
    info!("Entered playing state, despawning main menu camera");
    commands.entity(*main_menu_camera).despawn();
}
