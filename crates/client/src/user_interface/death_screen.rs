use bevy::prelude::*;
use game_core::GameStateWave;
use lightyear::prelude::MessageSender;
use shared::{ClientRespawnRequest, protocol::OrderedReliableChannel};

use crate::{
    game_flow::{states::InGameState, systems::free_mouse},
    user_interface::common::{
        CommonUiButton, DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH,
    },
};

pub struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::PlayerDead),
            (spawn_wave_game_mode_death_screen, free_mouse),
        )
        .add_systems(Update, handle_button_press);
    }
}

#[derive(Component, PartialEq)]
enum DeathScreenButton {
    Restart,
}

fn spawn_wave_game_mode_death_screen(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    game_state_wave: Option<Res<GameStateWave>>,
) {
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
                asset_server.load("hud/blood_screen_effects/Effect_3.png"),
            ),
            Name::new("Death Screen Root"),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    padding: UiRect::new(
                        Val::ZERO,
                        Val::ZERO,
                        Val::ZERO,
                        Val::Px(64.0),
                    ),
                    ..default()
                })
                .with_child((
                    Text::new("You are dead"),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
            if let Some(ref game_state_wave) = game_state_wave {
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
                    .with_child((
                        Text::new(format!(
                            "You survived until wave {}",
                            game_state_wave.current_wave
                        )),
                        TextFont {
                            font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                            font_size: DEFAULT_FONT_SIZE,
                            ..default()
                        },
                    ));
                parent
                    .spawn(Node {
                        padding: UiRect::new(
                            Val::ZERO,
                            Val::ZERO,
                            Val::ZERO,
                            Val::Px(64.0),
                        ),
                        ..default()
                    })
                    .with_child((
                        Text::new(format!(
                            "Enemies killed: {}",
                            game_state_wave.enemies_killed
                        )),
                        TextFont {
                            font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                            font_size: DEFAULT_FONT_SIZE,
                            ..default()
                        },
                    ));
            }
            parent
                .spawn(Node {
                    row_gap: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    let restart_button_text = if game_state_wave.is_some() {
                        "Retry"
                    } else {
                        "Respawn"
                    };

                    parent
                        .spawn((Button, DeathScreenButton::Restart))
                        .with_child((
                            Text::new(restart_button_text),
                            TextFont {
                                font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            },
                        ));
                    parent
                        .spawn((Button, CommonUiButton::BackToMainMenu))
                        .with_child((
                            Text::new("Exit to Main Menu"),
                            TextFont {
                                font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            },
                        ));
                });
        });
}

fn handle_button_press(
    query: Query<(&Interaction, &DeathScreenButton), Changed<Interaction>>,
    mut respawn_request_message_sender: Single<
        &mut MessageSender<ClientRespawnRequest>,
    >,
) {
    for (interaction, button) in query {
        if interaction != &Interaction::Pressed {
            continue;
        }
        match button {
            DeathScreenButton::Restart => {
                respawn_request_message_sender
                    .send::<OrderedReliableChannel>(ClientRespawnRequest);
            }
        }
    }
}
