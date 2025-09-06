use bevy::prelude::*;

use crate::debug_hud::RootNode;

pub fn spawn_debug_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            RootNode,
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Press g to toggle grab mouse"));
            parent.spawn(Text::new(
                "Press v to switch between first and third person",
            ));
            parent.spawn(Text::new(
                "Press m to disable mouse motion roll/pitch/yaw",
            ));
        });
}
