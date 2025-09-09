use bevy::prelude::*;

use crate::player::Player;

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
    player: Single<&Player, Added<Player>>,
) {
    info!("Spawning player hud");
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::End,
            column_gap: Val::Px(16.0),
            padding: UiRect::all(Val::Px(16.0)),
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
