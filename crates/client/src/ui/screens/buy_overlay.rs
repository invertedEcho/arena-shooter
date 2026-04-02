use bevy::{
    color::palettes::{
        css::{GRAY, WHITE},
        tailwind::{GRAY_600, GRAY_900, RED_500},
    },
    prelude::*,
    ui::InteractionDisabled,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use shared::{
    player::PlayerCash,
    shooting::{
        ALL_GAME_WEAPONS, GameWeapon, PlayerWeapons, WeaponSlotType,
        get_game_weapon_by_kind,
    },
};

use crate::{
    game_flow::states::InGameState,
    player::camera::messages::UpdatePlayerWeaponModel, ui::UiState,
};

#[derive(Component)]
struct BuyScreenRoot;

pub struct BuyScreenPlugin;

#[derive(Component)]
struct ShopItemButton {
    game_weapon: GameWeapon,
    // TODO: maybe better name for this
    text_entity: Entity,
    cost_text_entity: Entity,
}

impl Plugin for BuyScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_buy_screen)
            .add_systems(
                Update,
                (update_mouse_mode, update_visibility).run_if(
                    resource_changed::<UiState>
                        .and(not(resource_added::<UiState>))
                        .and(in_state(InGameState::Playing)),
                ),
            )
            .add_systems(
                Update,
                (
                    handle_input,
                    handle_pressed_buy_item,
                    update_disabled_enabled_shop_item_buttons,
                    update_texts_on_player_weapons_change,
                )
                    .run_if(in_state(InGameState::Playing)),
            );
    }
}

fn spawn_buy_screen(mut commands: Commands) {
    commands
        .spawn((
            BuyScreenRoot,
            Name::new("Buy Screen"),
            Node {
                width: percent(90),
                height: percent(90),
                padding: UiRect::all(px(16)),
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(GRAY.with_alpha(0.6).into()),
            ZIndex(1000),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_grow: 1.0,
                        padding: UiRect::all(px(8)),
                        border: UiRect {
                            left: px(1),
                            ..default()
                        },
                        justify_self: JustifySelf::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BorderColor::all(GRAY_900),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Weapons"),
                        Node {
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                    ));
                    for game_weapon in ALL_GAME_WEAPONS {
                        build_buy_list_item(parent, game_weapon);
                    }
                });
            parent
                .spawn((
                    Node {
                        flex_grow: 1.0,
                        padding: UiRect::all(px(8)),
                        border: UiRect {
                            left: px(1),
                            ..default()
                        },
                        ..default()
                    },
                    BorderColor::all(GRAY_900),
                ))
                .with_child(Text::new("Explosives"));
            parent
                .spawn((
                    Node {
                        flex_grow: 1.0,
                        padding: UiRect::all(px(8)),
                        border: UiRect {
                            left: px(1),
                            right: px(1),
                            ..default()
                        },
                        ..default()
                    },
                    BorderColor::all(GRAY_900),
                ))
                .with_child(Text::new("Accessories"));
        });
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UiState>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        ui_state.buy_overlay_visibile = !ui_state.buy_overlay_visibile;
    }
}

fn update_visibility(
    mut buy_screen_visibility: Single<&mut Visibility, With<BuyScreenRoot>>,
    ui_state: Res<UiState>,
) {
    **buy_screen_visibility = if ui_state.buy_overlay_visibile {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn update_mouse_mode(
    ui_state: Res<UiState>,
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    primary_cursor_options.visible = ui_state.buy_overlay_visibile;
    primary_cursor_options.grab_mode = if ui_state.buy_overlay_visibile {
        CursorGrabMode::None
    } else {
        CursorGrabMode::Locked
    };
}

#[derive(Component)]
struct ShopItemText;

#[derive(Component)]
struct ShopItemCostText;

fn build_buy_list_item(
    parent_builder: &mut ChildSpawnerCommands,
    game_weapon: GameWeapon,
) {
    let shop_item_text = parent_builder
        .spawn((Text::new("Cost:"), ShopItemText))
        .id();

    let cost_text = parent_builder
        .spawn((
            Text::new(format!("{}$", game_weapon.cost)),
            ShopItemCostText,
        ))
        .id();

    let mut button_entity = parent_builder.spawn((
        Node {
            border: UiRect::all(px(1)),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(8)),
            ..default()
        },
        Button,
        ShopItemButton {
            game_weapon: game_weapon.clone(),
            text_entity: shop_item_text,
            cost_text_entity: cost_text,
        },
        BackgroundColor(GRAY.with_alpha(0.7).into()),
    ));

    button_entity.with_children(|parent| {
        parent.spawn(Text::new(game_weapon.kind.to_string()));
        parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![(),],
            ))
            .add_children(&[shop_item_text, cost_text]);
    });
}

fn update_disabled_enabled_shop_item_buttons(
    mut commands: Commands,
    player_cash_changed: Single<&PlayerCash, Changed<PlayerCash>>,
    shop_item_buttons: Query<(Entity, &ShopItemButton)>,
    mut text_color_query: Query<&mut TextColor, With<ShopItemCostText>>,
) {
    for (button_entity, shop_item_button) in shop_item_buttons {
        let Ok(mut text_color) =
            text_color_query.get_mut(shop_item_button.cost_text_entity)
        else {
            continue;
        };
        if shop_item_button.game_weapon.cost > player_cash_changed.0 {
            commands.entity(button_entity).insert(InteractionDisabled);
            *text_color = RED_500.into();
        } else {
            commands
                .entity(button_entity)
                .remove::<InteractionDisabled>();
            *text_color = WHITE.into();
        }
    }
}

fn handle_pressed_buy_item(
    shop_item_button_query: Query<
        (&Interaction, &ShopItemButton),
        (Changed<Interaction>, Without<InteractionDisabled>),
    >,
    player_query: Single<(&mut PlayerCash, &mut PlayerWeapons)>,
    mut update_player_weapon_model_message_writer: MessageWriter<
        UpdatePlayerWeaponModel,
    >,
) {
    let (mut player_cash, mut player_weapons) = player_query.into_inner();

    for (interaction, shop_item_button) in shop_item_button_query {
        if Interaction::Pressed != *interaction {
            continue;
        }

        if shop_item_button.game_weapon.cost > player_cash.0 {
            info!("not enough cash to buy item");
            return;
        }

        player_cash.0 -= shop_item_button.game_weapon.cost;

        let game_weapon =
            get_game_weapon_by_kind(&shop_item_button.game_weapon.kind);

        let player_weapon = match shop_item_button.game_weapon.slot_type {
            WeaponSlotType::Primary => &mut player_weapons.weapons[0],
            WeaponSlotType::Secondary => &mut player_weapons.weapons[1],
        };

        player_weapon.game_weapon = game_weapon.clone();
        player_weapon.state.loaded_ammo = game_weapon.max_loaded_ammo;
        // TODO: function to get this value depending on WeaponKind
        player_weapon.state.carried_ammo = 360;

        update_player_weapon_model_message_writer
            .write(UpdatePlayerWeaponModel);
    }
}

fn update_texts_on_player_weapons_change(
    mut commands: Commands,
    player_weapons: Single<&PlayerWeapons, Changed<PlayerWeapons>>,
    shop_item_buttons: Query<(Entity, &ShopItemButton)>,
    mut shop_item_text_query: Query<
        (&mut Text, &mut TextColor),
        With<ShopItemText>,
    >,
    mut shop_item_cost_text_query: Query<
        &mut Visibility,
        With<ShopItemCostText>,
    >,
) {
    for (button_entity, shop_item_button) in shop_item_buttons {
        let game_weapon = &shop_item_button.game_weapon;

        // check if weapon of current weapon can be found in current player weapons
        let weapon_equipped = player_weapons
            .weapons
            .iter()
            .find(|weapon| weapon.game_weapon.kind == game_weapon.kind)
            .is_some();

        let Ok((mut shop_item_text, mut shop_item_text_color)) =
            shop_item_text_query.get_mut(shop_item_button.text_entity)
        else {
            continue;
        };

        let Ok(mut shop_item_cost_text_visibility) = shop_item_cost_text_query
            .get_mut(shop_item_button.cost_text_entity)
        else {
            continue;
        };

        if weapon_equipped {
            commands.entity(button_entity).insert(InteractionDisabled);

            **shop_item_text = "Equipped".to_string();
            **shop_item_text_color = GRAY_600.into();

            *shop_item_cost_text_visibility = Visibility::Hidden;
        } else {
            commands
                .entity(button_entity)
                .remove::<InteractionDisabled>();

            **shop_item_text = "Cost:".to_string();

            *shop_item_cost_text_visibility = Visibility::Inherited;
        }
    }
}
