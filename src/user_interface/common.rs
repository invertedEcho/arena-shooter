use bevy::prelude::*;

pub struct CommonUiPlugin;

impl Plugin for CommonUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_common_ui_button_interaction);
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
