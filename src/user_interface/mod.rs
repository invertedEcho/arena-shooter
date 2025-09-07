use bevy::prelude::*;

use crate::user_interface::{
    common::CommonUiPlugin, pause_menu::PauseMenuPlugin,
};

mod common;
mod pause_menu;

pub struct UserInterfacePlugin;

impl Plugin for UserInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PauseMenuPlugin).add_plugins(CommonUiPlugin);
    }
}
