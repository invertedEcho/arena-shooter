use bevy::{
    color::palettes::{css::WHITE, tailwind::ORANGE_400},
    prelude::*,
};

pub struct CommonUiPlugin;

impl Plugin for CommonUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_common_ui_button_interaction, handle_any_button_hover),
        );
    }
}

#[derive(Component)]
pub struct CommonUiButton {
    pub common_ui_button_type: CommonUiButtonType,
}

pub enum CommonUiButtonType {
    Quit,
}

fn handle_common_ui_button_interaction(
    query: Query<(&Interaction, &CommonUiButton), Changed<Interaction>>,
    mut app_exit_event_writer: EventWriter<AppExit>,
) {
    for (interaction, common_ui_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match common_ui_button.common_ui_button_type {
            CommonUiButtonType::Quit => {
                app_exit_event_writer.write(AppExit::Success);
            }
        }
    }
}

fn handle_any_button_hover(
    query: Query<
        (&Interaction, &Children),
        (Changed<Interaction>, With<Button>),
    >,
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
