use bevy::prelude::*;

use crate::{
    game_flow::states::MainMenuState,
    user_interface::{
        common::{
            CommonUiButton, DEFAULT_FONT_SIZE, DEFAULT_GAME_FONT_PATH,
            DEFAULT_ROW_GAP, SMALL_FONT_SIZE, TITLE_FONT_SIZE,
        },
        widgets::{button::build_common_button, text::build_normal_text_block},
    },
};

pub struct CreditsScreenPlugin;

impl Plugin for CreditsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MainMenuState::Credits), spawn_credits_screen);
    }
}

fn spawn_credits_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font_handle = asset_server.load(DEFAULT_GAME_FONT_PATH);

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: DEFAULT_ROW_GAP,
                ..default()
            },
            DespawnOnExit(MainMenuState::Credits),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Credits"),
                TextFont {
                    font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
            ));
            parent.spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: DEFAULT_ROW_GAP,
                    ..default()
                },
                children![
                (build_normal_text_block("3D Models", font_handle.clone(), DEFAULT_FONT_SIZE)),
                (build_normal_text_block("fps/tps Map by theking1322 CC-BY via Poly Pizza (https://poly.pizza/m/wna54gOjL7)", font_handle.clone(), SMALL_FONT_SIZE)),
                (build_normal_text_block("SWAT by Quaternius (https://poly.pizza/m/Btfn3G5Xv4)", font_handle.clone(), SMALL_FONT_SIZE)),
                (build_normal_text_block("'LOWPOLY | FPS | TDM | GAME | MAP' by ResoForge (https://skfb.ly/pxM87) by ResoForge (old profile) is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).", font_handle.clone(), SMALL_FONT_SIZE)),
                ]
            ));

            parent.spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: DEFAULT_ROW_GAP,
                    ..default()
                },
                children![

                (build_normal_text_block(
                    "Music & SFX",
                    font_handle.clone(),
                    DEFAULT_FONT_SIZE,
                )),
                    (build_normal_text_block(
                        "Main Menu Theme by juanjo_sound (https://juanjosound.itch.io/)",
                        font_handle.clone(),
                        SMALL_FONT_SIZE
                    )),
                    (build_normal_text_block(
                        "Snake's Authentic Gun Sounds by SnakeF8 (https://f8studios.itch.io/snakes-authentic-gun-sounds)",
                        font_handle.clone(),
                        SMALL_FONT_SIZE
                    )),
                    (build_normal_text_block(
                        "Sounds by JDSherbert – https://jdsherbert.itch.io",
                        font_handle.clone(),
                        SMALL_FONT_SIZE
                    )),
                ],
            ));

            parent.spawn(build_common_button("Back to Main Menu", font_handle.clone(), CommonUiButton::BackToMainMenu));
        });
}
