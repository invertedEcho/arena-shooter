use bevy::prelude::*;
use game_core::ServerLoadingState;
use lightyear::prelude::*;
use shared::{ServerMode, game_score::GameScore};

use crate::{
    game_flow::states::{AppState, MainMenuState},
    user_interface::{
        map_selection::MapSelectionButton,
        settings_menu::TabButtonSettings,
        widgets::{
            checkbox::{update_checkbox_style, update_checkbox_style2},
            slider::update_slider_style,
        },
    },
};

pub const DEFAULT_GAME_FONT_PATH: &str = "fonts/Exo_2/static/Exo2-Regular.ttf";
pub const ITALIC_GAME_FONT_PATH: &str = "fonts/Exo_2/static/Exo2-Italic.ttf";
pub const TITLE_FONT_SIZE: f32 = 64.0;
pub const DEFAULT_FONT_SIZE: f32 = 32.0;
pub const SMALL_FONT_SIZE: f32 = 16.0;

pub const ELEMENT_OUTLINE: Color = Color::srgb(0.45, 0.45, 0.45);
pub const ELEMENT_FILL: Color = Color::srgb(0.35, 0.75, 0.35);

/// Background color
pub const UI_BACKGROUND: Color =
    Color::Srgba(Srgba::new(0.055, 0.067, 0.086, 1.0));

/// Color for visually elevated elements
pub const UI_PANEL: Color = Color::Srgba(Srgba::new(0.086, 0.106, 0.137, 1.0));

/// Color for text of hovered buttons
pub const UI_HOVER: Color = Color::Srgba(Srgba::new(0.298, 0.788, 0.941, 1.0));

/// default text color
pub const UI_TEXT: Color = Color::Srgba(Srgba::new(0.918, 0.918, 0.918, 1.0));

/// default border color
pub const UI_BORDER: Color = Color::Srgba(Srgba::new(0.165, 0.196, 0.251, 1.0));

// TODO: make MapSelectionButton visually a button too, right now its just text
/// Color for selected elements, for example MapSelectionButton or SettingsChangeTabButton
pub const UI_SELECTED: Color = Color::Srgba(Srgba::new(0.22, 0.55, 0.75, 1.0));

pub const DEFAULT_ROW_GAP: Val = Val::Px(8.0);

pub struct CommonUiPlugin;

impl Plugin for CommonUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_common_ui_button_press,
                handle_button_hover_text_color,
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

// TODO: This system does way too many things and especially things that aren't relevant for
// `user_interface` module.
fn handle_common_ui_button_press(
    mut commands: Commands,
    query: Query<(&Interaction, &CommonUiButton), Changed<Interaction>>,
    mut app_exit_message_writer: MessageWriter<AppExit>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    own_client: Query<Entity, With<Client>>,
    server_mode: Res<State<ServerMode>>,
    game_score: Query<Entity, With<GameScore>>,
    mut next_server_loading_state: ResMut<NextState<ServerLoadingState>>,
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

                let Ok(own_client) = own_client.single() else {
                    continue;
                };

                debug!("Triggering disconnect and despawning our client");
                commands.trigger(Disconnect { entity: own_client });
                commands.entity(own_client).despawn();
                if *server_mode.get() == ServerMode::LocalServerSinglePlayer
                    && let Ok(game_score) = game_score.single()
                {
                    commands.entity(game_score).despawn();
                    next_server_loading_state.set(ServerLoadingState::Initial);
                };
            }
            CommonUiButton::ToGameModeSelection => {
                next_main_menu_state.set(MainMenuState::GameModeSelection);
            }
        }
    }
}

pub type AnyButtonInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static Children),
    (
        Changed<Interaction>,
        With<Button>,
        Without<MapSelectionButton>,
        Without<TabButtonSettings>,
    ),
>;

fn handle_button_hover_text_color(
    query: AnyButtonInteractionQuery,
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
