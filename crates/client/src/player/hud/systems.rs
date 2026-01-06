use bevy::{
    color::palettes::tailwind::{BLUE_500, RED_500},
    prelude::*,
};
use lightyear::prelude::Controlled;
use shared::player::{AimType, Health};

use crate::{
    game_flow::{
        game_mode::GameStateWave,
        score::GameScore,
        states::{AppState, InGameState},
    },
    player::{
        Player, PlayerReady,
        hud::{
            CROSSHAIR_BULLET_HIT_PATH, MAIN_CROSSHAIR_PATH,
            components::{
                CurrentWaveText, EnemiesLeftText, EnemyScoreText,
                PlayerCarriedAmmoText, PlayerCrosshair, PlayerHealthText,
                PlayerHud, PlayerLoadedAmmoText, PlayerScoreText,
                PlayerWeaponText,
            },
        },
        shooting::{
            components::PlayerWeapons,
            messages::{
                PlayerBulletHitEnemyMessage, PlayerWeaponSlotChangeMessage,
            },
        },
    },
    shared::components::DespawnTimer,
    user_interface::{ITALIC_GAME_FONT_PATH, shared::PRIMARY_COLOR},
};

pub fn spawn_player_hud(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    player_query: Query<
        (&Health, &PlayerWeapons),
        (Added<PlayerReady>, With<Controlled>),
    >,
) {
    let Ok((player_health, player_weapons)) = player_query.single() else {
        return;
    };

    info!(
        "Spawning player hud because PlayerReady on our own player was \
         inserted"
    );

    let weapon_state =
        &player_weapons.weapons[player_weapons.active_slot].state;

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
            PlayerHud,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("HP"),
                        TextFont {
                            font: asset_server.load(ITALIC_GAME_FONT_PATH),
                            ..default()
                        },
                    ));
                    parent.spawn((
                        Text::new(player_health.0.to_string()),
                        PlayerHealthText,
                        TextFont {
                            font: asset_server.load(ITALIC_GAME_FONT_PATH),
                            ..default()
                        },
                    ));
                });
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    row_gap: px(8.0),
                    ..default()
                })
                .with_children(|parent| {
                    for (index, player_weapon) in
                        player_weapons.weapons.iter().enumerate()
                    {
                        let text_color = if player_weapons.active_slot == index
                        {
                            PRIMARY_COLOR
                        } else {
                            Color::WHITE.to_srgba()
                        };

                        parent.spawn((
                            Text::new(format!(
                                "{}: {}",
                                index + 1,
                                player_weapon.stats.weapon_type
                            )),
                            TextFont {
                                font: asset_server.load(ITALIC_GAME_FONT_PATH),
                                ..default()
                            },
                            TextColor(text_color.into()),
                            PlayerWeaponText(index),
                        ));
                    }
                    parent
                        .spawn(Node {
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(weapon_state.loaded_ammo.to_string()),
                                PlayerLoadedAmmoText,
                                TextFont {
                                    font: asset_server
                                        .load(ITALIC_GAME_FONT_PATH),
                                    ..default()
                                },
                            ));
                            parent.spawn((
                                Text::new("/"),
                                TextFont {
                                    font: asset_server
                                        .load(ITALIC_GAME_FONT_PATH),
                                    ..default()
                                },
                            ));
                            parent.spawn((
                                Text::new(
                                    weapon_state.carried_ammo.to_string(),
                                ),
                                PlayerCarriedAmmoText,
                                TextFont {
                                    font: asset_server
                                        .load(ITALIC_GAME_FONT_PATH),
                                    ..default()
                                },
                            ));
                        });
                });
        });
}

pub fn spawn_player_crosshair(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    player_query: Query<Entity, Added<Player>>,
) {
    for _ in player_query {
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PlayerCrosshair,
                DespawnOnExit(AppState::InGame),
            ))
            .with_child(ImageNode::new(asset_server.load(MAIN_CROSSHAIR_PATH)));
    }
}

pub fn update_player_crosshair_visibility(
    player_aim_type: Single<&AimType, Changed<AimType>>,
    mut player_cross_hair: Single<&mut Visibility, With<PlayerCrosshair>>,
) {
    match *player_aim_type {
        AimType::Normal => **player_cross_hair = Visibility::Visible,
        AimType::Scoped => **player_cross_hair = Visibility::Hidden,
    }
}

pub fn update_player_health_text(
    player_health: Single<&Health, (Changed<Health>, With<Controlled>)>,
    mut player_health_text: Single<&mut Text, With<PlayerHealthText>>,
) {
    info!("Updated player health text");
    player_health_text.0 = player_health.0.to_string();
}

pub fn update_player_ammo_text(
    player_weapons: Single<&PlayerWeapons, Changed<PlayerWeapons>>,
    mut player_loaded_ammo_text: Single<
        &mut Text,
        (With<PlayerLoadedAmmoText>, Without<PlayerCarriedAmmoText>),
    >,
    mut player_carried_ammo_text: Single<
        &mut Text,
        (With<PlayerCarriedAmmoText>, Without<PlayerLoadedAmmoText>),
    >,
) {
    let active_weapon = &player_weapons.weapons[player_weapons.active_slot];

    ***player_loaded_ammo_text = active_weapon.state.loaded_ammo.to_string();
    ***player_carried_ammo_text = active_weapon.state.carried_ammo.to_string();
}

pub fn spawn_bullet_hit_crosshair(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut player_bullet_hit_enemy_message_reader: MessageReader<
        PlayerBulletHitEnemyMessage,
    >,
) {
    for _ in player_bullet_hit_enemy_message_reader.read() {
        commands
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_child((
                ImageNode::new(asset_server.load(CROSSHAIR_BULLET_HIT_PATH)),
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
            DespawnOnExit(InGameState::Playing),
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
        **player_score_text = Text::new(game_score.player.to_string());
        **enemy_score_text = Text::new(game_score.enemy.to_string());
    }
}

pub fn spawn_wave_info_hud(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(InGameState::Playing),
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

pub fn hide_player_hud(
    mut player_hud_visibility: Single<&mut Visibility, With<PlayerHud>>,
) {
    **player_hud_visibility = Visibility::Hidden;
}

pub fn show_player_hud(
    mut player_hud_visibility: Single<&mut Visibility, With<PlayerHud>>,
) {
    **player_hud_visibility = Visibility::Visible;
}

pub fn hide_player_crosshair(
    mut player_crosshair_visibility: Single<
        &mut Visibility,
        With<PlayerCrosshair>,
    >,
) {
    debug!("Hiding player crosshair");
    **player_crosshair_visibility = Visibility::Hidden;
}

pub fn show_player_crosshair(
    mut player_crosshair_visibility: Single<
        &mut Visibility,
        With<PlayerCrosshair>,
    >,
) {
    **player_crosshair_visibility = Visibility::Visible;
}

pub fn update_selected_weapon(
    mut message_reader: MessageReader<PlayerWeaponSlotChangeMessage>,
    mut player_weapon_texts: Query<(&mut TextColor, &PlayerWeaponText)>,
) {
    for message in message_reader.read() {
        let weapon_slot = message.0;
        for (mut text_color, player_weapon_text) in
            player_weapon_texts.iter_mut()
        {
            if player_weapon_text.0 == weapon_slot {
                text_color.0 = PRIMARY_COLOR.into();
            } else {
                text_color.0 = Color::WHITE;
            }
        }
    }
}
