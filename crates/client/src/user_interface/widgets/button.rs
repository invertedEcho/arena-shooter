use bevy::prelude::*;

use crate::user_interface::common::{UI_BORDER, UI_PANEL, UI_TEXT};

pub fn build_common_button(
    button_text: &str,
    font_handle: Handle<Font>,
) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(8.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(4)),
            ..default()
        },
        BackgroundColor(UI_PANEL),
        Button,
        BorderColor::all(UI_BORDER),
        BorderRadius::all(px(4)),
        children![
            Text::new(button_text),
            TextFont {
                font: font_handle,
                ..default()
            },
            TextColor(UI_TEXT),
        ],
    )
}
