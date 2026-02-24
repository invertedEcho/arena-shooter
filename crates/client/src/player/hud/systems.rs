use bevy::{color::palettes::css::WHITE, prelude::*};
use game_core::GameStateWave;
use lightyear::prelude::{Controlled, MessageReceiver};
use shared::{
    PlayerHitMessage,
    components::{DespawnTimer, Health},
    player::{AimType, PlayerState},
};

use crate::{
    game_flow::states::{AppState, InGameState},
    player::{
        Player, PlayerReady,
        camera::components::WorldCamera,
        hud::{
            CROSSHAIR_BULLET_HIT_PATH, MAIN_CROSSHAIR_PATH,
            components::{
                CurrentWaveText, DamageIndicator, EnemiesLeftText,
                PlayerCarriedAmmoText, PlayerCrosshair, PlayerHealthText,
                PlayerHud, PlayerLoadedAmmoText, PlayerWeaponText,
            },
        },
        shooting::{
            components::PlayerWeapons,
            messages::{
                PlayerBulletHitEnemyMessage, PlayerWeaponSlotChangeMessage,
            },
        },
    },
    shared::components::OnlyVisibleInGame,
    user_interface::{
        UiState,
        common::{ITALIC_GAME_FONT_PATH, UI_SELECTED, UI_TEXT},
    },
    utils::query_filters::OurPlayerFilter,
};

pub fn spawn_player_hud(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    player_query: Query<
        (&Health, &PlayerWeapons, &PlayerState),
        (Added<PlayerReady>, With<Controlled>),
    >,
) {
    let Ok((player_health, player_weapons, player_state)) =
        player_query.single()
    else {
        return;
    };

    debug!(
        "Spawning player hud because PlayerReady on our own player was \
         inserted"
    );

    let weapon_state =
        &player_weapons.weapons[player_state.active_weapon_slot].state;

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
            Name::new("PlayerHud"),
            OnlyVisibleInGame,
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
                        let text_color =
                            if player_state.active_weapon_slot == index {
                                UI_SELECTED
                            } else {
                                UI_TEXT
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
                            TextColor(text_color),
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
    player_query: Query<Entity, (Added<Player>, With<Controlled>)>,
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
                Name::new("PlayerCrosshair"),
                OnlyVisibleInGame,
            ))
            .with_child(ImageNode::new(asset_server.load(MAIN_CROSSHAIR_PATH)));
    }
}

pub fn update_player_crosshair_visibility(
    player_aim_type: Single<&AimType, Changed<AimType>>,
    mut player_cross_hair: Single<&mut Visibility, With<PlayerCrosshair>>,
    ui_state: Res<UiState>,
) {
    if ui_state.score_board_overlay_visible {
        return;
    }

    match *player_aim_type {
        AimType::Normal => **player_cross_hair = Visibility::Visible,
        AimType::Scoped => **player_cross_hair = Visibility::Hidden,
    }
}

// this is a seperate system because update_player_crosshair_visibility only runs when AimType
// changes, and this system should only run if UiState changed
pub fn on_ui_state_change(
    ui_state: Res<UiState>,
    mut player_cross_hair: Single<&mut Visibility, With<PlayerCrosshair>>,
    player_aim_type: Single<&AimType, With<AimType>>,
) {
    if ui_state.score_board_overlay_visible {
        **player_cross_hair = Visibility::Hidden;
    // only switch back to visible cross hair if player not currently scoping
    } else if **player_aim_type != AimType::Scoped {
        **player_cross_hair = Visibility::Visible;
    }
}

pub fn update_player_health_text(
    player_health: Single<&Health, (Changed<Health>, With<Controlled>)>,
    mut player_health_text: Single<&mut Text, With<PlayerHealthText>>,
) {
    debug!("Updated player health text");
    player_health_text.0 = player_health.0.to_string();
}

pub fn update_player_ammo_text(
    player_query: Single<
        (&PlayerWeapons, &PlayerState),
        Changed<PlayerWeapons>,
    >,
    mut player_loaded_ammo_text: Single<
        &mut Text,
        (With<PlayerLoadedAmmoText>, Without<PlayerCarriedAmmoText>),
    >,
    mut player_carried_ammo_text: Single<
        &mut Text,
        (With<PlayerCarriedAmmoText>, Without<PlayerLoadedAmmoText>),
    >,
) {
    let (player_weapons, player_state) = player_query.into_inner();
    let active_weapon =
        &player_weapons.weapons[player_state.active_weapon_slot];

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

pub fn spawn_wave_hud(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(AppState::InGame),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            Name::new("WaveHud"),
            OnlyVisibleInGame,
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Current wave:"));
            parent.spawn((Text::new(""), CurrentWaveText));
            parent.spawn(Text::new("Enemies left:"));
            parent.spawn((Text::new(""), EnemiesLeftText));
        });
}

pub fn update_wave_hud(
    game_state_wave: Res<GameStateWave>,
    mut current_wave_text: Single<
        &mut Text,
        (With<CurrentWaveText>, Without<EnemiesLeftText>),
    >,
    mut enemies_left_text: Single<&mut Text, With<EnemiesLeftText>>,
) {
    **current_wave_text = Text::new((game_state_wave.current_wave).to_string());
    **enemies_left_text =
        Text::new(game_state_wave.enemies_left_from_current_wave.to_string());
}

// TODO: do we need a message for this? cant we just watch for change in PlayerWeapons?
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
                text_color.0 = UI_SELECTED;
            } else {
                text_color.0 = UI_TEXT;
            }
        }
    }
}

pub fn spawn_damage_indicator(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut message_reader: MessageReader<PlayerHitMessage>,
    player_transform: Single<&Transform, OurPlayerFilter>,
    camera_transform: Single<&Transform, With<WorldCamera>>,
    mut network_message_reader: Single<&mut MessageReceiver<PlayerHitMessage>>,
) {
    let internal_messages = message_reader.read().copied();
    let network_messages = network_message_reader.receive();

    let combined_messages = internal_messages.chain(network_messages);

    for message in combined_messages {
        if let Some(world_direction) =
            (message.origin - player_transform.translation).try_normalize()
        {
            let direction_flat =
                Vec3::new(world_direction.x, 0.0, world_direction.z)
                    .normalize();
            let forward = camera_transform.forward();
            let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize();

            let angle = forward_flat.angle_between(direction_flat);

            // the cross product gives us the area of the parallelogram, that we get after applying
            // the transformation on two given vectors.
            // but the cross product actually gives a vector. the length of that vector is the said
            // area
            // and the resulting vector will be a vector which is perpendicular to the
            // parallelogram
            let cross = forward_flat.cross(direction_flat);

            // this is relevant as if the cross product is negative, it means the orienation
            // changed during the transformation
            let signed_angle = if cross.y < 0.0 { -angle } else { angle };

            commands.spawn((
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ImageNode {
                    image: asset_server.load("hud/damage_indicator.png"),
                    color: WHITE.with_alpha(0.7).into(),
                    ..default()
                },
                UiTransform::from_rotation(Rot2::radians(-signed_angle)),
                DamageIndicator(Timer::from_seconds(
                    0.05,
                    TimerMode::Repeating,
                )),
                DespawnOnExit(InGameState::Playing),
            ));
        }
    }
}

pub fn fade_out_damage_indicator(
    mut commands: Commands,
    time: Res<Time>,
    damage_indicators: Query<(&mut DamageIndicator, &mut ImageNode, Entity)>,
) {
    for (mut damage_indicator, mut image_node, entity) in damage_indicators {
        damage_indicator.0.tick(time.delta());
        if damage_indicator.0.is_finished() {
            let current_alpha = image_node.color.alpha();
            let new_alpha = current_alpha - 0.05;
            if new_alpha == 0.0 {
                commands.entity(entity).despawn();
                continue;
            }
            image_node.color = WHITE.with_alpha(new_alpha).into();
        }
    }
}
