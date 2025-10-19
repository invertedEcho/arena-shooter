use bevy::prelude::*;

use crate::{
    game_flow::{states::InGameState, systems::free_mouse},
    player::Player,
    user_interface::common::{CommonUiButton, CommonUiButtonType},
};

pub struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::PlayerDead),
            (spawn_death_screen, free_mouse),
        )
        .add_systems(Update, handle_button_click);
    }
}

#[derive(Component)]
struct DeathScreenButton(DeathScreenButtonType);

#[derive(PartialEq)]
enum DeathScreenButtonType {
    Respawn,
}

fn spawn_death_screen(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(InGameState::PlayerDead),
            Node {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ImageNode::new(
                asset_server.load("Bloody Screen Effects/Effect_3.png"),
            ),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    padding: UiRect::new(
                        Val::ZERO,
                        Val::ZERO,
                        Val::ZERO,
                        Val::Px(16.0),
                    ),
                    ..default()
                })
                .with_child(Text::new("You are dead"));
            parent.spawn(Text::new("Tip:"));
            parent.spawn(Text::new("Take cover when under heavy shooting!"));
            parent
                .spawn((
                    Node { ..default() },
                    Button,
                    DeathScreenButton(DeathScreenButtonType::Respawn),
                ))
                .with_child(Text::new("Respawn"));
            parent
                .spawn((
                    Node {
                        padding: UiRect {
                            top: Val::Px(16.0),
                            ..default()
                        },
                        ..default()
                    },
                    Button,
                    CommonUiButton(CommonUiButtonType::Quit),
                ))
                .with_child(Text::new("Rage Quit"));
        });
}

fn handle_button_click(
    query: Query<(&Interaction, &DeathScreenButton), Changed<Interaction>>,
    mut player: Single<&mut Player>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for (interaction, button) in query {
        match interaction {
            Interaction::Pressed => {
                if button.0 == DeathScreenButtonType::Respawn {
                    player.health = 100.0;
                    next_in_game_state.set(InGameState::Playing);
                }
            }
            _ => {}
        }
    }
}
