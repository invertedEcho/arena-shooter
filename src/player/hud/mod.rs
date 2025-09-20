use bevy::prelude::*;

use crate::{
    game_flow::GameState,
    player::{Player, shooting::components::PlayerWeapon},
};

const WHITE_CROSSHAIR_PATH: &str =
    "kenney_crosshair-pack/PNG/White/crosshair086.png";

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
            (update_player_health_text, update_player_ammo_text),
        )
        .add_systems(OnEnter(GameState::InGame), spawn_player_hud);
    }
}

fn spawn_player_hud(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::End,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            StateScoped(GameState::InGame),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(Text::new("HP"));
                    parent.spawn((Text::new(""), PlayerHealthText));
                });
            parent
                .spawn(Node {
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((Text::new(""), PlayerLoadedAmmoText));
                    parent.spawn(Text::new("/"));
                    parent.spawn((Text::new(""), PlayerCarriedAmmoText));
                });
        });
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            StateScoped(GameState::InGame),
        ))
        .with_child(ImageNode::new(asset_server.load(WHITE_CROSSHAIR_PATH)));
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
