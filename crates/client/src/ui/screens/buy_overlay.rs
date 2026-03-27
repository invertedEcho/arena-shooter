use bevy::{
    color::palettes::{css::GRAY, tailwind::GRAY_900},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use shared::{GAME_ITEMS, GameItem};

use crate::{game_flow::states::InGameState, ui::UiState};

#[derive(Component)]
struct BuyScreenRoot;

pub struct BuyScreenPlugin;

#[derive(Component)]
struct ShopItemButton {
    item: GameItem,
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
                handle_input.run_if(in_state(InGameState::Playing)),
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
                        ..default()
                    },
                    BorderColor::all(GRAY_900),
                ))
                .with_child(Node {
                    justify_self: JustifySelf::Center,
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|parent| {
                    for game_item in GAME_ITEMS {
                        parent.spawn(build_buy_list_item(
                            game_item.kind.to_string(),
                            game_item.cost,
                        ));
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
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::Locked
    };
}

fn build_buy_list_item(item_name: String, cost: usize) -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(1)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Button,
        BackgroundColor(GRAY.with_alpha(0.7).into()),
        children![Text::new(item_name), Text::new(format!("Cost: {}$", cost))],
    )
}
