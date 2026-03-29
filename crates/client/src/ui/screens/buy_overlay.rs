use bevy::{
    color::palettes::{
        css::{GRAY, WHITE},
        tailwind::{GRAY_600, GRAY_900},
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

use crate::{game_flow::states::InGameState, ui::UiState};

#[derive(Component)]
struct BuyScreenRoot;

pub struct BuyScreenPlugin;

#[derive(Component)]
struct ShopItemButton(GameWeapon);

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
                        parent.spawn(build_buy_list_item(game_weapon));
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

/// Simple marker component so we can filter in queries
#[derive(Component)]
struct ShopItemText;

fn build_buy_list_item(game_weapon: GameWeapon) -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(1)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Button,
        BackgroundColor(GRAY.with_alpha(0.7).into()),
        children![
            (Text::new(game_weapon.kind.to_string()), ShopItemText),
            (
                Text::new(format!("Cost: {}$", game_weapon.cost)),
                ShopItemText
            )
        ],
        ShopItemButton(game_weapon),
    )
}

fn update_disabled_enabled_shop_item_buttons(
    mut commands: Commands,
    player_cash_changed: Single<&PlayerCash, Changed<PlayerCash>>,
    shop_item_buttons: Query<&ShopItemButton>,
    text_color_query: Query<
        (&mut TextColor, &ChildOf, Entity),
        With<ShopItemText>,
    >,
) {
    for (mut text_color, child_of, entity) in text_color_query {
        let Ok(shop_item_button) = shop_item_buttons.get(child_of.0) else {
            continue;
        };
        if shop_item_button.0.cost > player_cash_changed.0 {
            commands.entity(entity).insert(InteractionDisabled);
            *text_color = GRAY_600.into();
        } else {
            commands.entity(entity).remove::<InteractionDisabled>();
            *text_color = WHITE.into();
        }
    }
}

fn handle_pressed_buy_item(
    query: Query<(&Interaction, &ShopItemButton), Changed<Interaction>>,
    player_query: Single<(&mut PlayerCash, &mut PlayerWeapons)>,
) {
    let (mut player_cash, mut player_weapons) = player_query.into_inner();

    for (interaction, shop_item_button) in query {
        if Interaction::Pressed != *interaction {
            continue;
        }

        if shop_item_button.0.cost > player_cash.0 {
            info!("not enough cash to buy item");
            return;
        }

        player_cash.0 -= shop_item_button.0.cost;

        let game_weapon = get_game_weapon_by_kind(&shop_item_button.0.kind);

        let player_weapon = match shop_item_button.0.slot_type {
            WeaponSlotType::Primary => &mut player_weapons.weapons[0],
            WeaponSlotType::Secondary => &mut player_weapons.weapons[1],
        };

        player_weapon.game_weapon = game_weapon.clone();
        player_weapon.state.loaded_ammo = game_weapon.max_loaded_ammo;
        // TODO: function to get this value depending on WeaponKind
        player_weapon.state.carried_ammo = 360;
    }
}
