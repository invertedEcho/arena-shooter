use avian3d::prelude::*;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    game_flow::states::{AppState, InGameState, LoadingGameState},
    user_interface::main_menu::{
        MainMenuCamera, get_main_menu_camera_transform,
    },
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

pub fn manual_free_mouse(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        primary_cursor_options.grab_mode = CursorGrabMode::None;
        primary_cursor_options.visible = true;
    }
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
            InGameState::Paused => next_in_game_state.set(InGameState::Playing),
            InGameState::PlayerDead => {}
            InGameState::Disconnected => {}
        }
    }
}

pub fn spawn_main_menu_camera(
    mut commands: Commands,
    existing_main_menu_camera: Query<&MainMenuCamera>,
) {
    // TODO: optimally this couldnt happen in first place
    if existing_main_menu_camera.count() != 0 {
        info!("Not spawning Main Menu Camera, already exists");
        return;
    }
    info!("Spawning Main Menu Camera");
    commands.spawn((
        Name::new("Main Menu Camera"),
        Camera::default(),
        Camera3d::default(),
        get_main_menu_camera_transform(),
        MainMenuCamera,
        bevy_inspector_egui::bevy_egui::PrimaryEguiContext,
        // we still need mainmenucamera during loading screen
        DespawnOnExit(AppState::LoadingGame),
    ));
}

pub fn check_world_scene_loaded(
    mut asset_event_message_reader: MessageReader<AssetEvent<Scene>>,
    maybe_world_scene_handle: Option<Res<WorldSceneHandle>>,
    mut next_game_loading_state: ResMut<NextState<LoadingGameState>>,
) {
    for asset_event in asset_event_message_reader.read() {
        if let AssetEvent::LoadedWithDependencies { id } = asset_event
            && let Some(ref world_scene_handle) = maybe_world_scene_handle
            && *id == world_scene_handle.0.id()
        {
            info!(
                "Map assets loaded!, setting LoadingGameSubState to \
                 MapLoadedWithDependencies"
            );
            next_game_loading_state
                .set(LoadingGameState::MapLoadedWithDependencies);
        }
    }
}

pub fn check_collider_constructor_hierarchy_ready(
    _trigger: On<ColliderConstructorHierarchyReady>,
    mut next_game_loading_state: ResMut<NextState<LoadingGameState>>,
) {
    // next_game_loading_state.set(LoadingGameSubState::CollidersReady);
    // FIXME: add navmesh generation again
    next_game_loading_state.set(LoadingGameState::NavMeshReady);
}

// pub fn check_navmesh_ready(
//     trigger: On<NavmeshReady>,
//     mut commands: Commands,
//     mut next_game_loading_state: ResMut<NextState<LoadingGameSubState>>,
//     mut nav_meshes: ResMut<Assets<Navmesh>>,
// ) {
//     let Some(nav_mesh_handle) = nav_meshes.get_strong_handle(trigger.0) else {
//         panic!(
//             "Got navmeshready event but the Handle could not be found using \
//              the asset id from the trigger"
//         );
//     };
//
//     info!("Navmesh is now ready!");
//     next_game_loading_state.set(LoadingGameSubState::NavMeshReady);
//
//     commands.insert_resource(NavMeshHandle(nav_mesh_handle));
//     info!("NavMesh Handle stored");
// }

pub fn pause_all_animations(animation_players: Query<&mut AnimationPlayer>) {
    for mut animation_player in animation_players {
        animation_player.pause_all();
    }
}

pub fn resume_all_animations(animation_players: Query<&mut AnimationPlayer>) {
    for mut animation_player in animation_players {
        animation_player.resume_all();
    }
}
