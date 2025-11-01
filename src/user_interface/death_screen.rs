use bevy::prelude::*;

use crate::{
    game_flow::{
        game_mode::GameStateWave, states::InGameState, systems::free_mouse,
    },
    player::Player,
    user_interface::{
        DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH,
        common::{CommonUiButton, CommonUiButtonType},
    },
};

pub struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::PlayerDead),
            (spawn_wave_game_mode_death_screen, free_mouse),
        )
        .add_systems(Update, handle_button_click);
    }
}

#[derive(Component)]
struct DeathScreenButton(WaveGameModeDeathScreen);

#[derive(PartialEq)]
enum WaveGameModeDeathScreen {
    Restart,
}

fn spawn_wave_game_mode_death_screen(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    game_state_wave: Res<State<GameStateWave>>,
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
                        game_state_wave.get().current_wave
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
                        game_state_wave.get().enemies_killed
                    )),
                    TextFont {
                        font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        ..default()
                    },
                ));
            parent
                .spawn(Node {
                    row_gap: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            DeathScreenButton(WaveGameModeDeathScreen::Restart),
                        ))
                        .with_child((
                            Text::new("Retry"),
                            TextFont {
                                font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                                font_size: DEFAULT_FONT_SIZE,
                                ..default()
                            },
                        ));
                    parent
                        .spawn((
                            Button,
                            CommonUiButton(CommonUiButtonType::BackToMainMenu),
                        ))
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

fn handle_button_click(
    query: Query<(&Interaction, &DeathScreenButton), Changed<Interaction>>,
    mut player: Single<&mut Player>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for (interaction, button) in query {
        if interaction == &Interaction::Pressed
            && button.0 == WaveGameModeDeathScreen::Restart
        {
            player.health = 100.0;
            next_in_game_state.set(InGameState::Playing);
        }
    }
}
