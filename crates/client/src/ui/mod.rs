use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::ui::{
    common::CommonUiPlugin, hud::PlayerHudPlugin, menus::UIMenusPlugin,
    screens::UIScreensPlugin,
};

pub mod common;
mod hud;
mod menus;
mod screens;
pub mod widgets;

#[derive(Resource, Default, Debug)]
pub struct UiState {
    pub score_board_overlay_visible: bool,
    pub buy_overlay_visible: bool,
    pub crosshair_visible: bool,
    pub cursor_visible: bool,
}

pub struct UserInterfacePlugin;

impl Plugin for UserInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerHudPlugin)
            .add_plugins(UIMenusPlugin)
            .add_plugins(CommonUiPlugin)
            .add_plugins(UIScreensPlugin);

        app.insert_resource(UiState::default());

        app.add_systems(
            Update,
            update_mouse_mode.run_if(resource_changed::<UiState>),
        );
    }
}

fn update_mouse_mode(
    ui_state: Res<UiState>,
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    debug!(
        "Updating mouse mode because UiState resource changed! New ui state \
         {:?}",
        ui_state
    );

    primary_cursor_options.visible = ui_state.cursor_visible;
    primary_cursor_options.grab_mode = if ui_state.cursor_visible {
        CursorGrabMode::None
    } else {
        CursorGrabMode::Locked
    };
}
