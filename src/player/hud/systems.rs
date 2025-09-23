use bevy::{
    color::palettes::tailwind::{BLUE_500, RED_500},
    prelude::*,
};

use crate::{
    common::components::DespawnTimer,
    game_flow::{GameState, score::GameScore},
    player::{
        Player,
        hud::{
            WHITE_CROSSHAIR_PATH,
            components::{
                EnemyScoreText, PlayerCarriedAmmoText, PlayerHealthText,
                PlayerLoadedAmmoText, PlayerScoreText,
            },
        },
        shooting::{
            components::PlayerWeapon, events::PlayerBulletHitEnemyEvent,
        },
    },
};

pub fn spawn_player_hud(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
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

pub fn update_player_health_text(
    player: Single<&Player, Changed<Player>>,
    mut player_health_text: Single<&mut Text, With<PlayerHealthText>>,
) {
    ***player_health_text = player.health.to_string();
}

pub fn update_player_ammo_text(
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

pub fn spawn_bullet_hit_crosshair(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut player_bullet_hit_enemy_event_reader: EventReader<
        PlayerBulletHitEnemyEvent,
    >,
) {
    for _ in player_bullet_hit_enemy_event_reader.read() {
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_child((
                ImageNode::new(
                    asset_server.load(
                        "kenney_crosshair-pack/PNG/White/crosshair002.png",
                    ),
                ),
                DespawnTimer(Timer::from_seconds(0.05, TimerMode::Once)),
            ));
    }
}

pub fn spawn_score_hud(mut commands: Commands, game_score: Res<GameScore>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Start,
                column_gap: Val::Px(32.0),
                ..default()
            },
            StateScoped(GameState::InGame),
        ))
        .with_children(|parent| {
            parent.spawn(Node { ..default() }).with_child((
                Text::new(game_score.player.to_string()),
                TextColor(BLUE_500.into()),
                PlayerScoreText,
            ));
            parent
                .spawn(Node { ..default() })
                .with_child(Text::new("Score"));
            parent.spawn(Node { ..default() }).with_child((
                Text::new(game_score.enemy.to_string()),
                TextColor(RED_500.into()),
                EnemyScoreText,
            ));
        });
}

pub fn update_score_hud(
    mut player_score_text: Single<
        &mut Text,
        (With<PlayerScoreText>, Without<EnemyScoreText>),
    >,
    mut enemy_score_text: Single<
        &mut Text,
        (With<EnemyScoreText>, Without<PlayerScoreText>),
    >,
    game_score: Res<GameScore>,
) {
    if game_score.is_changed() {
        info!("Game score changed, updating score hud");
        **player_score_text = Text::new(game_score.player.to_string());
        **enemy_score_text = Text::new(game_score.enemy.to_string());
    }
}
