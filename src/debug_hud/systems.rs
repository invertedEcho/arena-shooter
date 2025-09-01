use bevy::prelude::*;

pub fn spawn_debug_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(Text::new("Press g to toggle grab mouse"));
        });
}
