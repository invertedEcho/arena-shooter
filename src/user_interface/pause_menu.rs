use bevy::prelude::*;

use crate::{
    game_flow::GameState,
    user_interface::common::{CommonUiButton, CommonUiButtonType},
};

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_pause_menu)
            .add_systems(
                Update,
                (handle_pause_menu_button_pressed)
                    .run_if(in_state(GameState::Paused)),
            );
    }
}

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct PauseMenuButton {
    pub pause_menu_button_type: PauseMenuButtonType,
}

pub enum PauseMenuButtonType {
    Resume,
}

fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Paused"));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    PauseMenuButton {
                        pause_menu_button_type: PauseMenuButtonType::Resume,
                    },
                ))
                .with_child(Text::new("Resume"));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    CommonUiButton {
                        common_ui_button_type: CommonUiButtonType::Quit,
                    },
                ))
                .with_child(Text::new("Quit"));
        });
}

fn despawn_pause_menu(
    mut commands: Commands,
    pause_menu_entity: Single<Entity, With<PauseMenuRoot>>,
) {
    commands.entity(*pause_menu_entity).despawn();
}

fn handle_pause_menu_button_pressed(
    mut next_game_state: ResMut<NextState<GameState>>,
    query: Query<(&Interaction, &PauseMenuButton), Changed<Interaction>>,
) {
    for (interaction, pause_menu_button) in query {
        let Interaction::Pressed = interaction else {
            continue;
        };
        match pause_menu_button.pause_menu_button_type {
            PauseMenuButtonType::Resume => {
                next_game_state.set(GameState::InGame)
            }
        }
    }
}
