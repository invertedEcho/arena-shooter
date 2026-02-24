use bevy::prelude::*;

use crate::ui::common::{UI_BORDER, UI_PANEL, UI_TEXT};

pub fn build_common_button<T: Component>(
    button_text: &str,
    font_handle: Handle<Font>,
    marker_component: T,
) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(8.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(4)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        marker_component,
        BackgroundColor(UI_PANEL),
        Button,
        BorderColor::all(UI_BORDER),
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
