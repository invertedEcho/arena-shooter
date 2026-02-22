use bevy::{
    color::palettes::css::{RED, WHITE},
    prelude::*,
};

#[derive(Component)]
pub struct ProgressBar;

pub fn build_progress_bar<T: Component>(
    marker_component: T,
    width: Val,
    height: Val,
) -> impl Bundle {
    (
        Node {
            width,
            height,
            ..default()
        },
        Name::new("Progress Bar Container"),
        BackgroundColor(WHITE.into()),
        ProgressBar,
        children![(
            Node {
                width: percent(100.0),
                height: percent(100.0),
                ..default()
            },
            Name::new("Progress Bar Value"),
            BackgroundColor(RED.into()),
            marker_component,
        )],
    )
}
