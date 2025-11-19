use std::num::NonZero;

use bevy::{color::palettes::{css::RED, tailwind::BLUE_700}, prelude::*};
use bevy_rich_text3d::{LoadFonts, Text3d, Text3dPlugin, Text3dStyling, TextAtlas};

use crate::enemy::{ai::{ENEMY_FOV, ENEMY_VISION_RANGE}, Enemy};

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
        app.add_systems(Update, (draw_gizmos, draw_enemy_fov, add_enemy_state_text)); 

        // https://github.com/mintlu8/bevy_rich_text3d/issues/19
        app.add_systems(PreUpdate, update_enemy_debug_text);

        app.insert_resource(DebugGizmos(Vec::new()));
    }
}

#[derive(Resource)]
pub struct DebugGizmos(pub Vec<(Vec3, Vec3)>);

pub fn draw_gizmos(mut gizmos: Gizmos, debug_gizmos: Res<DebugGizmos>) {
    for gizmo in &debug_gizmos.0 {
        let start = gizmo.0;
        let end = gizmo.1;
        let color = RED.with_alpha(0.5);
        gizmos.line(start, end, color);
    }
}

pub fn draw_enemy_fov(
    enemy_transforms: Query<&Transform, With<Enemy>>,
    mut gizmos: Gizmos,
) {
    for transform in enemy_transforms {
        let pos = transform.translation;
        let forward = transform.forward();
        let range = ENEMY_VISION_RANGE;

        // Cone edges
        let half_angle = ENEMY_FOV.to_radians() / 2.0;
        let left_dir: Vec3 =
            (Quat::from_rotation_y(half_angle) * forward).normalize();
        let right_dir: Vec3 =
            (Quat::from_rotation_y(-half_angle) * forward).normalize();

        gizmos.ray(pos, left_dir * range, BLUE_700);
        gizmos.ray(pos, right_dir * range, BLUE_700);
    }
}

fn update_enemy_debug_text(mut query: Query<(&mut Text3d, &EnemyDebugText)>, changed_enemies: Query<(Entity, &Enemy), Changed<Enemy>>) {
    for (enemy_entity, enemy) in changed_enemies {
        let Some((mut text, _)) = query.iter_mut().find(|e| e.1.0 == enemy_entity) else {
            continue;
        };
        *text = Text3d::new(format!("{} | {:?}", enemy_entity, enemy.state));
    }
}


#[derive(Component)]
struct EnemyDebugText(pub Entity);

fn add_enemy_state_text(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Enemy), Added<Enemy>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, enemy) in enemy_query {
        let mat = materials.add(StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
            alpha_mode: AlphaMode::Mask(0.5),
            unlit: true,
            cull_mode: None,
            ..Default::default()
        });

        commands.entity(entity).with_child((
            Text3d::new(format!(" | {:?}", enemy.state)),
            Text3dStyling {
                size: 64.,
                stroke: NonZero::new(10),
                color: Srgba::new(1., 0., 0., 1.),
                stroke_color: Srgba::BLACK,
                world_scale: Some(Vec2::splat(0.25)),
                layer_offset: 0.001,
                ..Default::default()
            },
            Mesh3d::default(),
            MeshMaterial3d(mat.clone()),
            Transform {
                translation: Vec3::new(0.0, 1.0, 0.0),
                rotation: Quat::from_euler(
                    EulerRot::XYZ,
                    180_f32.to_radians(),
                    0.0,
                    180_f32.to_radians(),
                ),
                scale: Vec3::ONE,
            },
            EnemyDebugText(entity),
        ));
    }
}
