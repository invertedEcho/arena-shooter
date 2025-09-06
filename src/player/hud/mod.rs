use bevy::prelude::*;

pub struct PlayerHudPlugin;

impl Plugin for PlayerHudPlugin {
    fn build(&self, app: &mut App) {}
}

fn spawn_player_hud(mut commands: Commands) {
    commands
        .spawn(
            (Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            }),
        )
        .with_children(|parent| {
            parent.spawn();
        });
}
