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
            border: UiRect::all(Val::Px(2.)),
            border_radius: BorderRadius::all(Val::Px(10.)),
            ..default()
        },
        Name::new("Progress Bar Container"),
        BackgroundColor(WHITE.into()),
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
