use bevy::{
    color::palettes::tailwind::{BLUE_500, RED_500},
    prelude::*,
};

use crate::{
    common::components::DespawnTimer,
    game_flow::{
        game_mode::{GameMode, GameStateWave},
        score::GameScore,
        states::{AppState, InGameState},
    },
    player::{
        Player,
        hud::{
            WHITE_CROSSHAIR_PATH,
            components::{
                CurrentWaveText, EnemiesLeftText, EnemyScoreText,
                PlayerCarriedAmmoText, PlayerHealthText, PlayerHud,
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
    player_weapon: Query<&PlayerWeapon>,
) {
    info!(
        "Do we have access to player weapon?: {}",
        player_weapon.iter().len() != 0
    );

    info!("Spawning player hud");
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
            StateScoped(InGameState::Playing),
            PlayerHud,
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
            StateScoped(AppState::InGame),
            PlayerHud,
        ))
        .with_child(ImageNode::new(asset_server.load(WHITE_CROSSHAIR_PATH)));
}

pub fn update_player_health_text(
    player: Single<&Player, Changed<Player>>,
    mut player_health_text: Single<&mut Text, With<PlayerHealthText>>,
) {
    info!(
        "Updating player health text, new text: {}",
        player.health.to_string()
    );
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
    info!(
        "Updating player loaded ammo text, new text: {}",
        player_weapon.loaded_ammo.to_string()
    );
    ***player_loaded_ammo_text = player_weapon.loaded_ammo.to_string();
    info!(
        "Updating player carried ammo text, new text: {}",
        player_weapon.carried_ammo.to_string()
    );
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
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            StateScoped(AppState::InGame),
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

pub fn spawn_wave_info_hud(mut commands: Commands) {
    commands
        .spawn((
            StateScoped(GameMode::Waves),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Current wave:"));
            parent.spawn((Text::new(""), CurrentWaveText));
            parent.spawn(Text::new("Enemies left:"));
            parent.spawn((Text::new(""), EnemiesLeftText));
        });
}

pub fn update_wave_info_hud(
    game_state_wave: Res<State<GameStateWave>>,
    mut current_wave_text: Single<
        &mut Text,
        (With<CurrentWaveText>, Without<EnemiesLeftText>),
    >,
    mut enemies_left_text: Single<&mut Text, With<EnemiesLeftText>>,
) {
    if game_state_wave.is_changed() {
        **current_wave_text =
            Text::new((game_state_wave.current_wave).to_string());
        **enemies_left_text = Text::new(
            game_state_wave.enemies_left_from_current_wave.to_string(),
        );
    }
}
