use bevy::{
    color::palettes::{css::WHITE, tailwind::ORANGE_400},
    prelude::*,
};

use crate::{
    game_flow::states::{AppState, MainMenuState},
    user_interface::{
        map_selection::MapSelectionButton,
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
pub struct CommonUiButton(pub CommonUiButtonType);

pub enum CommonUiButtonType {
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
        match common_ui_button.0 {
            CommonUiButtonType::Quit => {
                app_exit_message_writer.write(AppExit::Success);
            }
            CommonUiButtonType::BackToMainMenu => {
                next_app_state.set(AppState::MainMenu);
                next_main_menu_state.set(MainMenuState::Root);
            }
            CommonUiButtonType::ToGameModeSelection => {
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
            Interaction::Hovered => *text_color = ORANGE_400.into(),
            Interaction::None => *text_color = WHITE.into(),
            _ => {}
        }
    }
}
