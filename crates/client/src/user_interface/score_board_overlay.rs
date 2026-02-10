use bevy::{color::palettes::css::BLACK, prelude::*};
use lightyear::prelude::*;
use shared::game_score::GameScore;

#[derive(Component)]
struct ScoreBoardOverlay;

pub struct ScoreBoardOverlayPlugin;

impl Plugin for ScoreBoardOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_score_board_overlay);
        app.add_systems(
            Update,
            (change_score_board_overlay_visibility, update_score_board),
        );
    }
}

fn spawn_score_board_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            width: percent(70.0),
            height: percent(60.0),
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(16.0)),
            row_gap: px(16.0),
            ..default()
        },
        BackgroundColor(BLACK.with_alpha(0.7).into()),
        ScoreBoardOverlay,
        Visibility::Hidden,
    ));
}

fn build_score_board_list_item(
    name: &String,
    kills: u64,
    deaths: u64,
) -> impl Bundle {
    (
        Node {
            width: percent(95),
            height: px(16.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        children![
            (Text::new(name)),
            (Text::new(format!("Kills: {}", kills))),
            (Text::new(format!("Deaths: {}", deaths)))
        ],
    )
}

fn change_score_board_overlay_visibility(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_score_board_overlay_state: Single<
        &mut Visibility,
        With<ScoreBoardOverlay>,
    >,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        **next_score_board_overlay_state = Visibility::Visible;
    } else if keyboard_input.just_released(KeyCode::Tab) {
        **next_score_board_overlay_state = Visibility::Hidden;
    }
}

fn update_score_board(
    mut commands: Commands,
    changed_game_score: Single<&GameScore, Changed<GameScore>>,
    score_board_overlay: Single<Entity, With<ScoreBoardOverlay>>,
    own_peer_id: Query<&RemoteId, With<Controlled>>,
) {
    info!("Game score has changed! Updating UI to reflect new values");
    commands.entity(*score_board_overlay).despawn_children();
    for (peer_id, player_stats) in &changed_game_score.players {
        let is_this_us = own_peer_id
            .iter()
            .find(|remote_id| remote_id.0.to_bits() == *peer_id);
        if is_this_us.is_some() {
            info!("YAAAAAAAAAAY IT WORKS!!!!11");
        }

        let score_board_list_item = build_score_board_list_item(
            &player_stats.username,
            player_stats.kills,
            player_stats.deaths,
        );
        let id = commands.spawn(score_board_list_item).id();
        commands.entity(*score_board_overlay).add_child(id);
    }
    for enemy_stats in changed_game_score.enemies.values() {
        let score_board_list_item = build_score_board_list_item(
            &enemy_stats.username,
            enemy_stats.kills,
            enemy_stats.deaths,
        );
        let id = commands.spawn(score_board_list_item).id();
        commands.entity(*score_board_overlay).add_child(id);
    }
}
