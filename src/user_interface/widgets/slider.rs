use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{
        CoreSliderDragState, Slider, SliderRange, SliderThumb, SliderValue,
        TrackClick,
    },
};

const DEFAULT_SLIDER_TRACK: Color = Color::srgb(0.05, 0.05, 0.05);
const DEFAULT_SLIDER_THUMB: Color = Color::srgb(0.35, 0.75, 0.35);

// TODO: move elsewhere
#[derive(Resource)]
pub struct GameSettings {
    pub volume: f32,
}

#[derive(Component)]
pub struct DemoSlider;

#[derive(Component)]
pub struct DemoSliderThumb;

/// Update the widget states based on the changing resource.
pub fn update_slider_values(
    res: Res<GameSettings>,
    mut sliders: Query<Entity, With<DemoSlider>>,
    mut commands: Commands,
) {
    if res.is_changed() {
        for slider_entity in sliders.iter_mut() {
            // we insert as component instead of changing the SliderValue component directly,
            // as SliderValue is internally marked as immutable
            commands
                .entity(slider_entity)
                .insert(SliderValue(res.volume));
        }
    }
}

pub fn build_slider(min: f32, max: f32, value: f32) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Center,
            column_gap: px(4),
            height: px(12),
            width: percent(30),
            ..default()
        },
        Name::new("Slider"),
        Hovered::default(),
        DemoSlider,
        Slider {
            track_click: TrackClick::Snap,
        },
        SliderValue(value),
        SliderRange::new(min, max),
        TabIndex(0),
        Children::spawn((
            // Slider background rail
            Spawn((
                Node {
                    height: px(6),
                    ..default()
                },
                BackgroundColor(DEFAULT_SLIDER_TRACK), // Border color for the slider
                BorderRadius::all(px(3)),
            )),
            // Invisible track to allow absolute placement of thumb entity. This is narrower than
            // the actual slider, which allows us to position the thumb entity using simple
            // percentages, without having to measure the actual width of the slider thumb.
            Spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: px(0),
                    // Track is short by 12px to accommodate the thumb.
                    right: px(12),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                children![(
                    DemoSliderThumb,
                    SliderThumb,
                    Node {
                        display: Display::Flex,
                        width: px(12),
                        height: px(12),
                        position_type: PositionType::Absolute,
                        left: percent(0), // This will be updated by the slider's value
                        ..default()
                    },
                    BorderRadius::MAX,
                    BackgroundColor(DEFAULT_SLIDER_THUMB),
                )],
            )),
        )),
    )
}

/// Update the visuals of the slider based on the slider state.
pub fn update_slider_style(
    sliders: Query<
        (
            Entity,
            &SliderValue,
            &SliderRange,
            &Hovered,
            &CoreSliderDragState,
        ),
        (
            Or<(
                Changed<SliderValue>,
                Changed<SliderRange>,
                Changed<Hovered>,
                Changed<CoreSliderDragState>,
            )>,
            With<DemoSlider>,
        ),
    >,
    children: Query<&Children>,
    mut thumbs: Query<
        (&mut Node, &mut BackgroundColor, Has<DemoSliderThumb>),
        Without<DemoSlider>,
    >,
) {
    for (slider_ent, value, range, hovered, drag_state) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, mut thumb_bg, is_thumb)) =
                thumbs.get_mut(child)
                && is_thumb
            {
                thumb_node.left =
                    percent(range.thumb_position(value.0) * 100.0);
                thumb_bg.0 = thumb_color(hovered.0 | drag_state.dragging);
            }
        }
    }
}

fn thumb_color(hovered: bool) -> Color {
    if hovered {
        DEFAULT_SLIDER_THUMB.lighter(0.3)
    } else {
        DEFAULT_SLIDER_THUMB
    }
}
