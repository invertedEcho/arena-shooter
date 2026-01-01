use avian3d::prelude::ColliderConstructorHierarchyReady;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_rerecast::{Navmesh, prelude::NavmeshReady};

use crate::{
    game_flow::{
        game_mode::StartGameModeMessage,
        states::{
            AppState, InGameState, LoadingGameSubState, SelectedMapState,
        },
    },
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
            InGameState::PlayerDead => {}
            InGameState::Paused => next_in_game_state.set(InGameState::Playing),
        }
    }
}

pub fn spawn_main_menu_camera(mut commands: Commands) {
    info!("Spawning Main Menu Camera");
    commands.spawn((
        Name::new("Main Menu Camera"),
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuCamera,
        bevy_inspector_egui::bevy_egui::PrimaryEguiContext,
        DespawnOnExit(AppState::MainMenu),
    ));
}

pub fn check_world_scene_loaded(
    mut asset_event_message_reader: MessageReader<AssetEvent<Scene>>,
    maybe_world_scene_handle: Option<Res<WorldSceneHandle>>,
    mut next_game_loading_state: ResMut<NextState<LoadingGameSubState>>,
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
                .set(LoadingGameSubState::MapLoadedWithDependencies);
        }
    }
}

pub fn handle_map_loaded_with_dependencies(
    selected_map_state: Res<State<SelectedMapState>>,
    mut next_game_loading_state: ResMut<NextState<LoadingGameSubState>>,
) {
    // if the map is tiny town and MapLoadedWithDependencies, the colliders are already
    // spawned, because they are part of the map itself, so we can just set CollidersReady
    // immediately
    if *selected_map_state.get() == SelectedMapState::TinyTown {
        next_game_loading_state.set(LoadingGameSubState::CollidersReady);
    }
}

// FIXME: lets just do this for tinytown map too
// this is only relevant for Map::MediumPlastic, because tiny town map has colliders in skein
pub fn check_collider_constructor_hierarchy_ready(
    _: On<ColliderConstructorHierarchyReady>,
    mut next_game_loading_state: ResMut<NextState<LoadingGameSubState>>,
) {
    info!("Collider_constructor_hierarchy_ready is now!");
    // next_game_loading_state.set(LoadingGameSubState::CollidersReady);
    // FIXME: add navmesh generation again
    next_game_loading_state.set(LoadingGameSubState::NavMeshReady);
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

pub fn on_game_loading_state_nav_mesh_ready(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut start_game_mode_message_writer: MessageWriter<StartGameModeMessage>,
) {
    info!(
        "Entered nav_mesh_ready loading state, everything is ready now. \
         Writing StartGameModeMessage.",
    );
    next_app_state.set(AppState::InGame);
    start_game_mode_message_writer.write(StartGameModeMessage);
}

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
