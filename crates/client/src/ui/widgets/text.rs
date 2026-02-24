use bevy::prelude::*;

pub fn build_normal_text_block(
    text: &str,
    font_handle: Handle<Font>,
    font_size: f32,
) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: font_handle,
            font_size,
            ..default()
        },
    )
}
