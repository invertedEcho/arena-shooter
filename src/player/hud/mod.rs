use bevy::prelude::*;

use crate::{debug_hud::RootNode, player::Player};

#[derive(Component)]
struct PlayerHealthText;

pub struct PlayerHudPlugin;

impl Plugin for PlayerHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_player_health_text, spawn_player_hud));
    }
}

fn spawn_player_hud(
    mut commands: Commands,
    player: Single<&Player>,
    root_node: Single<Entity, Added<RootNode>>,
) {
    info!("spawnimg player hud");
    commands
        .entity(*root_node)
        .with_child(Node {
            align_self: AlignSelf::FlexEnd,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(Text::new("HP"));
            parent.spawn((
                Text::new(player.health.to_string()),
                PlayerHealthText,
            ));
        });
}

fn update_player_health_text(
    player: Single<&Player, Changed<Player>>,
    mut player_health_text: Single<&mut Text, With<PlayerHealthText>>,
) {
    info!("player changed, updating health text");
    info!("new player health: {}", player.health);
    ***player_health_text = player.health.to_string();
}
