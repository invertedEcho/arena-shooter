use std::num::NonZero;

use bevy::{
    color::palettes::{css::RED, tailwind::BLUE_700},
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext},
    egui,
};
use bevy_rich_text3d::{
    LoadFonts, Text3d, Text3dPlugin, Text3dStyling, TextAtlas,
};

pub struct GameplayDebugPlugin;

impl Plugin for GameplayDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Text3dPlugin {
            load_system_fonts: true,
            ..Default::default()
        });

        app.insert_resource(LoadFonts {
            font_paths: vec![
                "assets/fonts/Exo_2/static/Exo2-Regular.ttf".to_owned(),
            ],
            font_directories: vec!["assets/fonts/Exo_2/static".to_owned()],
            ..Default::default()
        });
        app.add_systems(
            Update,
            (
                draw_gizmos,
                // draw_enemy_fov,
                // add_enemy_state_text,
                // update_enemy_debug_text,
                tick_despawn_timer_debug_gizmo_lines,
            ),
        );
        // app.add_systems(EguiPrimaryContextPass, player_inspector);

        app.insert_resource(DebugGizmos(Vec::new()));
    }
}

pub struct DebugGizmoLine {
    pub start: Vec3,
    pub end: Vec3,
    pub despawn_timer: Timer,
}

#[derive(Resource)]
pub struct DebugGizmos(pub Vec<DebugGizmoLine>);

pub fn draw_gizmos(mut gizmos: Gizmos, debug_gizmos: Res<DebugGizmos>) {
    for gizmo in debug_gizmos.0.iter() {
        let start = gizmo.start;
        let end = gizmo.end;
        let color = RED.with_alpha(0.5);
        gizmos.line(start, end, color);
    }
}

fn tick_despawn_timer_debug_gizmo_lines(
    mut debug_gizmos: ResMut<DebugGizmos>,
    time: Res<Time>,
) {
    debug_gizmos.0.retain_mut(|gizmo| {
        gizmo.despawn_timer.tick(time.delta());
        !gizmo.despawn_timer.is_finished()
    });
}

// pub fn draw_enemy_fov(
//     enemy_transforms: Query<&Transform, With<Enemy>>,
//     mut gizmos: Gizmos,
// ) {
//     for transform in enemy_transforms {
//         let pos = transform.translation;
//         let forward = transform.forward();
//         let range = ENEMY_VISION_RANGE;
//
//         // Cone edges
//         let half_angle = ENEMY_FOV.to_radians() / 2.0;
//         let left_dir: Vec3 =
//             (Quat::from_rotation_y(half_angle) * forward).normalize();
//         let right_dir: Vec3 =
//             (Quat::from_rotation_y(-half_angle) * forward).normalize();
//
//         gizmos.ray(pos, left_dir * range, BLUE_700.with_alpha(0.5));
//         gizmos.ray(pos, right_dir * range, BLUE_700.with_alpha(0.5));
//     }
// }
//
// fn update_enemy_debug_text(
//     mut query: Query<(&mut Text3d, &EnemyDebugText)>,
//     changed_enemies: Query<(Entity, &EnemyState), Changed<EnemyState>>,
// ) {
//     for (enemy_entity, enemy_state) in changed_enemies {
//         let Some((mut text, _)) =
//             query.iter_mut().find(|e| e.1.0 == enemy_entity)
//         else {
//             continue;
//         };
//         *text = Text3d::new(format!("{} | {:?}", enemy_entity, enemy_state));
//     }
// }
//
// #[derive(Component)]
// struct EnemyDebugText(pub Entity);
//
// fn add_enemy_state_text(
//     mut commands: Commands,
//     enemy_query: Query<(Entity, &EnemyState), Added<EnemyState>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     for (entity, enemy_state) in enemy_query {
//         let mat = materials.add(StandardMaterial {
//             base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
//             alpha_mode: AlphaMode::Mask(0.5),
//             unlit: true,
//             cull_mode: None,
//             ..Default::default()
//         });
//
//         commands.entity(entity).with_child((
//             Text3d::new(format!(" | {:?}", enemy_state)),
//             Text3dStyling {
//                 size: 64.,
//                 stroke: NonZero::new(10),
//                 color: Srgba::new(1., 0., 0., 1.),
//                 stroke_color: Srgba::BLACK,
//                 world_scale: Some(Vec2::splat(0.25)),
//                 layer_offset: 0.001,
//                 ..Default::default()
//             },
//             Mesh3d::default(),
//             MeshMaterial3d(mat.clone()),
//             Transform {
//                 translation: Vec3::new(0.0, 1.0, 0.0),
//                 rotation: Quat::from_euler(
//                     EulerRot::XYZ,
//                     180_f32.to_radians(),
//                     0.0,
//                     180_f32.to_radians(),
//                 ),
//                 scale: Vec3::ONE,
//             },
//             EnemyDebugText(entity),
//         ));
//     }
// }

// fn player_inspector(world: &mut World) {
//     let mut ui_ctx = match world
//         .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
//         .single_mut(world)
//     {
//         Ok(ctx) => ctx.clone(),
//         _ => return,
//     };
//
//     egui::Window::new("Player Inspector").show(ui_ctx.get_mut(), |ui| {
//         egui::ScrollArea::vertical().show(ui, |ui| {
//             if let Ok(player_entity) =
//                 world.query_filtered::<Entity, With<Player>>().single(world)
//             {
//                 ui.label(format!("Player Entity ID: {:?}", player_entity));
//
//                 bevy_inspector_egui::bevy_inspector::ui_for_entity(
//                     world,
//                     player_entity,
//                     ui,
//                 );
//             }
//         })
//     });
// }
