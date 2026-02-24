use bevy::prelude::*;

use crate::ui::screens::{
    death_screen::DeathScreenPlugin, disconnect_screen::DisconnectScreenPlugin,
    loading_screen::LoadingScreenPlugin, pause_screen::PauseScreenPlugin,
    score_board_overlay::ScoreBoardOverlayPlugin,
};

mod death_screen;
mod disconnect_screen;
mod loading_screen;
mod pause_screen;
mod score_board_overlay;

pub struct UIScreensPlugin;

impl Plugin for UIScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeathScreenPlugin)
            .add_plugins(DisconnectScreenPlugin)
            .add_plugins(LoadingScreenPlugin)
            .add_plugins(PauseScreenPlugin)
            .add_plugins(ScoreBoardOverlayPlugin);
    }
}
