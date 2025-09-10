use bevy::prelude::*;

use crate::player::{Player, shooting::components::PlayerWeapon};

#[derive(Component)]
struct PlayerHealthText;

#[derive(Component)]
struct PlayerLoadedAmmoText;

#[derive(Component)]
struct PlayerCarriedAmmoText;

pub struct PlayerHudPlugin;

impl Plugin for PlayerHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_player_health_text,
                spawn_player_hud,
                update_player_ammo_text,
            ),
        );
    }
}

fn spawn_player_hud(
    mut commands: Commands,
    player: Single<&Player, Added<Player>>,
    player_weapon: Single<&PlayerWeapon, Added<PlayerWeapon>>,
) {
    let _ = player_weapon;
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::End,
            // column_gap: Val::Px(16.0),
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(Text::new("HP"));
                    parent.spawn((
                        Text::new(player.health.to_string()),
                        PlayerHealthText,
                    ));
                });
            parent
                .spawn(Node {
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(player_weapon.loaded_ammo.to_string()),
                        PlayerLoadedAmmoText,
                    ));
                    parent.spawn(Text::new("/"));
                    parent.spawn((
                        Text::new(player_weapon.carried_ammo.to_string()),
                        PlayerCarriedAmmoText,
                    ));
                });
        });
}

fn update_player_health_text(
    player: Single<&Player, Changed<Player>>,
    mut player_health_text: Single<&mut Text, With<PlayerHealthText>>,
) {
    ***player_health_text = player.health.to_string();
}

fn update_player_ammo_text(
    player_weapon: Single<&PlayerWeapon, Changed<PlayerWeapon>>,
    mut player_loaded_ammo_text: Single<
        &mut Text,
        (With<PlayerLoadedAmmoText>, Without<PlayerCarriedAmmoText>),
    >,
    mut player_carried_ammo_text: Single<
        &mut Text,
        (With<PlayerCarriedAmmoText>, Without<PlayerLoadedAmmoText>),
    >,
) {
    ***player_loaded_ammo_text = player_weapon.loaded_ammo.to_string();
    ***player_carried_ammo_text = player_weapon.carried_ammo.to_string();
}
