use bevy::prelude::*;

use crate::{
    game_flow::states::{AppState, MainMenuState},
    user_interface::{
        map_selection::MapSelectionButton,
        settings_menu::SettingsChangeTabButton,
        shared::{UI_HOVER, UI_TEXT},
        widgets::{
            checkbox::{update_checkbox_style, update_checkbox_style2},
            slider::update_slider_style,
        },
    },
};

pub struct CommonUiPlugin;

impl Plugin for CommonUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_common_ui_button_press,
                handle_any_button_hover,
                update_slider_style,
                update_checkbox_style,
                update_checkbox_style2,
            ),
        );
    }
}

#[derive(Component)]
pub enum CommonUiButton {
    ToGameModeSelection,
    BackToMainMenu,
    Quit,
}

fn handle_common_ui_button_press(
    query: Query<(&Interaction, &CommonUiButton), Changed<Interaction>>,
    mut app_exit_message_writer: MessageWriter<AppExit>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
) {
    for (interaction, common_ui_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match common_ui_button {
            CommonUiButton::Quit => {
                app_exit_message_writer.write(AppExit::Success);
            }
            CommonUiButton::BackToMainMenu => {
                next_app_state.set(AppState::MainMenu);
                next_main_menu_state.set(MainMenuState::Root);
            }
            CommonUiButton::ToGameModeSelection => {
                next_main_menu_state.set(MainMenuState::GameModeSelection);
            }
        }
    }
}

type AnyButtonHoveredQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static Children),
    (
        Changed<Interaction>,
        With<Button>,
        Without<MapSelectionButton>,
        Without<SettingsChangeTabButton>,
    ),
>;

fn handle_any_button_hover(
    query: AnyButtonHoveredQuery,
    mut text_color_query: Query<&mut TextColor>,
) {
    for (interaction, children) in query {
        let Ok(mut text_color) = text_color_query.get_mut(children[0]) else {
            continue;
        };
        match interaction {
            Interaction::Hovered => **text_color = UI_HOVER,
            Interaction::None => **text_color = UI_TEXT,
            _ => {}
        }
    }
}
