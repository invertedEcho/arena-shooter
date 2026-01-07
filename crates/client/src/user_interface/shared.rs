use bevy::color::Srgba;
use bevy::prelude::*;

pub const DEFAULT_GAME_FONT_PATH: &str = "fonts/Exo_2/static/Exo2-Regular.ttf";
pub const ITALIC_GAME_FONT_PATH: &str = "fonts/Exo_2/static/Exo2-Italic.ttf";
pub const TITLE_FONT_SIZE: f32 = 64.0;
pub const NORMAL_FONT_SIZE: f32 = 32.0;

pub const ELEMENT_OUTLINE: Color = Color::srgb(0.45, 0.45, 0.45);
pub const ELEMENT_FILL: Color = Color::srgb(0.35, 0.75, 0.35);

pub const UI_BG: Color = Color::Srgba(Srgba::new(0.055, 0.067, 0.086, 1.0)); // #0E1116
pub const UI_PANEL: Color = Color::Srgba(Srgba::new(0.086, 0.106, 0.137, 1.0)); // #161B22
pub const UI_PRIMARY: Color =
    Color::Srgba(Srgba::new(0.902, 0.224, 0.275, 1.0)); // #E63946
pub const UI_HOVER: Color = Color::Srgba(Srgba::new(0.298, 0.788, 0.941, 1.0)); // #4CC9F0
pub const UI_TEXT: Color = Color::Srgba(Srgba::new(0.918, 0.918, 0.918, 1.0)); // #EAEAEA
pub const UI_BORDER: Color = Color::Srgba(Srgba::new(0.165, 0.196, 0.251, 1.0)); // ~ #2A3240
pub const UI_SELECTED: Color = Color::Srgba(Srgba::new(0.22, 0.55, 0.75, 1.0)); // ~ #388CBF

pub fn build_common_button(
    asset_server: Res<AssetServer>,
    button_text: String,
) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(8.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::left(px(8)),
            ..default()
        },
        BackgroundColor(UI_PRIMARY),
        Button,
        BorderColor::all(UI_BORDER),
        BorderRadius::all(px(4)),
        children![
            Text::new(button_text),
            TextFont {
                font: asset_server.load(DEFAULT_GAME_FONT_PATH),
                ..default()
            },
            TextColor(UI_TEXT),
        ],
    )
}
