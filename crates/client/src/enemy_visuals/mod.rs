use std::f32::consts::{FRAC_PI_2, PI};

use avian3d::prelude::LinearVelocity;
use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    prelude::*,
    render::render_resource::{Extent3d, TextureUsages},
};
use shared::{
    EnemyKilledMessage,
    components::Health,
    enemy::components::{Enemy, EnemyState},
    player::Player,
};

use crate::{
    enemy_visuals::animate::{AnimateEnemyPlugin, ENEMY_MODEL_PATH},
    game_flow::states::AppState,
    ui::widgets::progressbar::build_progress_bar,
};

mod animate;

pub struct EnemyVisualsPlugin;

impl Plugin for EnemyVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimateEnemyPlugin);
        app.add_systems(
            Update,
            (
                spawn_health_bar_for_new_enemy,
                spawn_enemy_model_for_new_enemies,
                update_health_bar_of_enemies,
                rotate_enemy_toward_direction,
                rotate_health_bar_to_player,
                health_bar_follow_enemy,
                despawn_health_bar_for_killed_enemies,
            ),
        );
    }
}

/// 0: The enemy entity this health bar belongs to
/// We need this to make the health bar follow the corresponding enemy
#[derive(Component)]
struct HealthBar(pub Entity);

#[derive(Component)]
pub struct HealthBarCamera;

fn spawn_health_bar_for_new_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    added_enemy_query: Query<Entity, Added<Enemy>>,
    mut camera_order: Local<isize>,
) {
    *camera_order -= 1;
    for enemy_entity in added_enemy_query {
        let size = Extent3d {
            width: 512,
            height: 512,
            ..default()
        };

        // this is the texture that will be rendered to
        let mut image = Image::new_fill(
            size,
            bevy::render::render_resource::TextureDimension::D2,
            &[0, 0, 0, 0],
            bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );

        // we need to set these flags in order to use the image as a render target
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;

        let image_handle = images.add(image);

        let texture_camera = commands
            .spawn((
                Camera2d,
                Camera {
                    order: *camera_order,
                    ..default()
                },
                RenderTarget::Image(image_handle.clone().into()),
                HealthBarCamera,
                DespawnOnExit(AppState::InGame),
            ))
            .id();

        let mesh_handle = meshes.add(Plane3d {
            normal: Dir3::X,
            half_size: vec2(0.5, 0.03),
        });

        let material_handle = materials.add(StandardMaterial {
            cull_mode: None,
            alpha_mode: AlphaMode::Blend,
            base_color_texture: Some(image_handle),
            reflectance: 1.0,
            unlit: true,
            ..default()
        });

        commands
            .spawn((
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                UiTargetCamera(texture_camera),
                DespawnOnExit(AppState::InGame),
            ))
            .with_children(|parent| {
                parent.spawn((build_progress_bar(
                    HealthBarUINode(enemy_entity),
                    percent(100),
                    percent(100),
                ),));
            });

        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::default(),
            Name::new("Health Bar"),
            HealthBar(enemy_entity),
            DespawnOnExit(AppState::InGame),
        ));
    }
}

#[derive(Component)]
struct HealthBarUINode(Entity);

type EnemiesWithChangedHealth = (Changed<Health>, With<Enemy>);
fn update_health_bar_of_enemies(
    mut progress_bar_query: Query<(&mut Node, &HealthBarUINode)>,
    changed_health: Query<(Entity, &Health), EnemiesWithChangedHealth>,
) {
    for (entity, new_health) in changed_health {
        if let Some((mut progress_bar, _)) = progress_bar_query
            .iter_mut()
            .find(|(_, health_bar)| health_bar.0 == entity)
        {
            progress_bar.width = percent(new_health.0);
        } else {
            warn!(
                "Health changed of an enemy but couldnt find health bar to \
                 update the value"
            );
        }
    }
}

fn spawn_enemy_model_for_new_enemies(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    enemy_query: Query<Entity, Added<Enemy>>,
) {
    const ENEMY_MODEL_Y_OFFSET: f32 = -1.0;

    for added_enemy in enemy_query {
        let enemy_model = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(ENEMY_MODEL_PATH));

        commands.entity(added_enemy).with_child((
            Transform {
                translation: Vec3::new(
                    0.0,
                    // center enemy model -> in blender, feet are at bottom, so in
                    // bevy model feet are at center of collider, 0.0
                    ENEMY_MODEL_Y_OFFSET,
                    0.0,
                ),
                // enemy model needs to be rotated 180 degrees
                rotation: Quat::from_rotation_y(PI),
                ..default()
            },
            SceneRoot(enemy_model),
            Visibility::Visible,
        ));
    }
}

/// Rotates all enemies toward the direction the velocity is going
fn rotate_enemy_toward_direction(
    enemy_query: Query<
        (&mut Transform, &LinearVelocity, &EnemyState),
        With<Enemy>,
    >,
) {
    for (mut transform, velocity, enemy_state) in enemy_query {
        if *enemy_state != EnemyState::GoToAgentTarget {
            continue;
        }
        if velocity.length_squared() < 0.0001 {
            continue;
        }
        if let Some(mut direction) = velocity.0.try_normalize() {
            direction.y = 0.0;
            let yaw = direction.x.atan2(direction.z);

            // i really dont get this. the enemy model is initially rotated 180 degree, so the
            // models forward matches the bevy forward. and when using draw_enemy_fov debug gizmo
            // system, we can also see the forward is now correct. but here we need to rotate again
            // 180 degrees? if we remove this + 180 degree and the initial 180 degree rotation in
            // enemy model, then this would match but all usages of transform.forward(), like in
            // RotateTowardsPlayer and draw_enemy_fov would be wrong way around.
            transform.rotation = Quat::from_rotation_y(yaw + PI);
        }
    }
}

fn rotate_health_bar_to_player(
    health_bar_query: Query<&mut Transform, With<HealthBar>>,
    player_transform: Single<&Transform, (With<Player>, Without<HealthBar>)>,
) {
    for mut transform in health_bar_query {
        let offset =
            Quat::from_euler(EulerRot::XYZ, FRAC_PI_2, 0.0, -FRAC_PI_2);
        transform.look_at(player_transform.translation, Vec3::Y);
        transform.rotation *= offset;
    }
}

// TODO: we wouldnt need this if the health bar was a child of the enemy,
// but right now i dont understand how we can take the parent rotation into account, so we get
// global rotation of the health bar. we probably need to take the inverse of the enemy parent
// rotation or something like this. this works so whatever
fn health_bar_follow_enemy(
    enemy_transform: Query<&Transform, With<Enemy>>,
    health_bar_query: Query<(&mut Transform, &HealthBar), Without<Enemy>>,
) {
    for (mut health_bar_transform, health_bar) in health_bar_query {
        match enemy_transform.get(health_bar.0) {
            Ok(enemy_transform) => {
                health_bar_transform.translation = enemy_transform.translation;
                // so its above the head
                health_bar_transform.translation.y += 1.0;
            }
            Err(error) => {
                warn!(
                    "Failed to update health bar position to enemy position: \
                     {}",
                    error
                );
            }
        };
    }
}

fn despawn_health_bar_for_killed_enemies(
    mut commands: Commands,
    mut message_reader: MessageReader<EnemyKilledMessage>,
    health_bars: Query<(Entity, &HealthBar)>,
) {
    for message in message_reader.read() {
        if let Some(health_bar_of_killed_enemy) = health_bars
            .iter()
            .find(|(_, health_bar)| message.0 == health_bar.0)
        {
            commands.entity(health_bar_of_killed_enemy.0).despawn();
        }
    }
}
