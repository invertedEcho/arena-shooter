use std::num::NonZero;

use avian3d::prelude::*;
use bevy::{
    color::palettes::{css::RED, tailwind::BLUE_700},
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext},
    egui,
};
use bevy_landmass::debug::{EnableLandmassDebug, Landmass3dDebugPlugin};
use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyling, TextAtlas};
use shared::{
    SpawnDebugSphereMessage,
    components::Health,
    enemy::{
        ENEMY_FOV, ENEMY_VISION_RANGE,
        components::{Enemy, EnemyState},
    },
    player::Player,
};

use crate::{
    enemy_visuals::HealthBarCamera, game_flow::states::AppState,
    gameplay_debug::states_overlay::DebugOverlayPlugin,
};

mod states_overlay;

#[derive(Resource, Eq, Debug, PartialEq, Hash, Clone, Default, Copy)]
pub struct AppDebugState {
    show_physics_gizmos: bool,
    show_nav_mesh: bool,
    show_enemy_debug_info: bool,
    show_states_overlay: bool,
    invincibility: bool,
}

/// This plugin adds functionality related to debugging the game itself, like having debug gizmos
/// for the navigation mesh visible, or having a debug text above enemies visible, to see their
/// state. Note that this plugin only runs when running a debug build
pub struct GameplayDebugPlugin;

impl Plugin for GameplayDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Landmass3dDebugPlugin {
            draw_on_start: false,
            ..default()
        });
        app.add_plugins(PhysicsDebugPlugin);

        app.init_resource::<AppDebugState>();
        app.add_plugins(DebugOverlayPlugin);
        app.add_plugins(Text3dPlugin {
            load_system_fonts: true,
            ..Default::default()
        });

        app.add_message::<SpawnDebugSphereMessage>();

        app.add_systems(
            Update,
            (
                draw_gizmos,
                draw_enemy_fov,
                add_enemy_state_text,
                update_enemy_debug_text,
                tick_despawn_timer_debug_gizmo_lines,
                handle_spawn_debug_points_message,
                do_invicibility,
                ensure_egui_context_exists,
            ),
        );
        app.add_systems(
            Update,
            (
                update_physics_debug_enabled,
                update_landmass_debug_enabled,
                update_enemy_debug_text_visible,
            )
                .run_if(resource_changed::<AppDebugState>),
        );
        // app.add_systems(EguiPrimaryContextPass, player_inspector);
        app.add_systems(EguiPrimaryContextPass, developer_menu);

        app.insert_resource(DebugGizmos(Vec::new()));
    }
}

fn update_physics_debug_enabled(
    mut store: ResMut<GizmoConfigStore>,
    current_app_debug_state: Res<AppDebugState>,
) {
    let (config, _) = store.config_mut::<PhysicsGizmos>();
    config.enabled = current_app_debug_state.show_physics_gizmos;
}

pub struct DebugGizmoLine {
    pub start: Vec3,
    pub end: Vec3,
    pub despawn_timer: Timer,
}

#[derive(Resource)]
pub struct DebugGizmos(pub Vec<DebugGizmoLine>);

fn draw_gizmos(mut gizmos: Gizmos, debug_gizmos: Res<DebugGizmos>) {
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

pub fn draw_enemy_fov(
    enemy_transforms: Query<&Transform, With<Enemy>>,
    mut gizmos: Gizmos,
    current_app_debug_state: Res<AppDebugState>,
) {
    if !current_app_debug_state.show_enemy_debug_info {
        return;
    }

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

        gizmos.ray(pos, left_dir * range, BLUE_700.with_alpha(0.5));
        gizmos.ray(pos, right_dir * range, BLUE_700.with_alpha(0.5));
    }
}

fn update_enemy_debug_text(
    mut query: Query<(&mut Text3d, &EnemyDebugText)>,
    changed_enemies: Query<
        (Entity, &EnemyState, &Health),
        Or<(Changed<EnemyState>, Changed<Health>)>,
    >,
) {
    for (enemy_entity, enemy_state, enemy_health) in changed_enemies {
        let Some((mut text, _)) =
            query.iter_mut().find(|e| e.1.0 == enemy_entity)
        else {
            continue;
        };
        *text = Text3d::new(format!(
            "{} | {:?} | {} HP",
            enemy_entity, enemy_state, enemy_health.0
        ));
    }
}

#[derive(Component)]
struct EnemyDebugText(pub Entity);

fn add_enemy_state_text(
    mut commands: Commands,
    enemy_query: Query<(Entity, &EnemyState, &Health), Added<EnemyState>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, enemy_state, enemy_health) in enemy_query {
        let mat = materials.add(StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
            alpha_mode: AlphaMode::Mask(0.5),
            unlit: true,
            cull_mode: None,
            ..Default::default()
        });

        commands.entity(entity).with_child((
            Text3d::new(format!(
                "{:?} | {:?} | {} HP",
                entity, enemy_state, enemy_health.0
            )),
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
                translation: Vec3::new(0.0, 1.3, 0.0),
                rotation: Quat::from_euler(
                    EulerRot::XYZ,
                    180_f32.to_radians(),
                    0.0,
                    180_f32.to_radians(),
                ),
                scale: Vec3::ONE,
            },
            EnemyDebugText(entity),
            Name::new("Enemy Debug Text"),
        ));
    }
}

fn _player_inspector(world: &mut World) {
    let mut ui_ctx = match world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single_mut(world)
    {
        Ok(ctx) => ctx.clone(),
        _ => return,
    };

    egui::Window::new("Player Inspector").show(ui_ctx.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Ok(player_entity) =
                world.query_filtered::<Entity, With<Player>>().single(world)
            {
                ui.label(format!("Player Entity ID: {:?}", player_entity));

                bevy_inspector_egui::bevy_inspector::ui_for_entity(
                    world,
                    player_entity,
                    ui,
                );
            }
        })
    });
}

fn update_enemy_debug_text_visible(
    query: Query<&mut Visibility, With<EnemyDebugText>>,
    current_app_debug_state: Res<AppDebugState>,
) {
    let new_visibility = if current_app_debug_state.show_enemy_debug_info {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for mut visibility in query {
        *visibility = new_visibility;
    }
}

#[derive(Component)]
pub struct DebugSphere;

fn handle_spawn_debug_points_message(
    mut commands: Commands,
    mut message_reader: MessageReader<SpawnDebugSphereMessage>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for message in message_reader.read() {
        commands.spawn((
            Transform::from_translation(message.location),
            Mesh3d(meshes.add(Sphere::new(message.radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: message.color,
                ..Default::default()
            })),
            DebugSphere,
            DespawnOnExit(AppState::InGame),
        ));
    }
}

fn update_landmass_debug_enabled(
    mut land_mass_debug: ResMut<EnableLandmassDebug>,
    current_app_debug_state: Res<AppDebugState>,
) {
    let app_debug_state_enabled = current_app_debug_state.show_nav_mesh;
    land_mass_debug.0 = app_debug_state_enabled;
}

fn developer_menu(
    mut ui_context: Single<&mut EguiContext, With<PrimaryEguiContext>>,
    mut app_debug_state: ResMut<AppDebugState>,
) {
    egui::Window::new("Developer Menu").show(ui_context.get_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Show physics gizmos");
            ui.checkbox(&mut app_debug_state.show_physics_gizmos, "");
        });
        ui.horizontal(|ui| {
            ui.label("Show nav mesh");
            ui.checkbox(&mut app_debug_state.show_nav_mesh, "");
        });
        ui.horizontal(|ui| {
            ui.label("Show enemy debug info");
            ui.checkbox(&mut app_debug_state.show_enemy_debug_info, "");
        });
        ui.horizontal(|ui| {
            ui.label("Show states overlay");
            ui.checkbox(&mut app_debug_state.show_states_overlay, "");
        });
        ui.horizontal(|ui| {
            ui.label("Invincibility");
            ui.checkbox(&mut app_debug_state.invincibility, "");
        });
    });
}

fn do_invicibility(
    mut changed_health: Single<&mut Health, (Changed<Health>, With<Player>)>,
    current_app_debug_state: Res<AppDebugState>,
) {
    if current_app_debug_state.invincibility {
        changed_health.0 = 100.0;
    }
}

fn ensure_egui_context_exists(
    mut commands: Commands,
    existing_egui_contexts: Query<&PrimaryEguiContext>,
    camera_query: Query<Entity, (With<Camera>, Without<HealthBarCamera>)>,
) {
    if existing_egui_contexts.count() == 0 {
        let Some(first_camera) = camera_query.iter().next() else {
            return;
        };

        debug!("Inserting PrimaryEguiContext into first camera found");
        commands.entity(first_camera).insert(PrimaryEguiContext);
    }
}
