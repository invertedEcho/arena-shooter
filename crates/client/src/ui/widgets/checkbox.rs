use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::hover::Hovered,
    prelude::*,
    ui::{Checked, InteractionDisabled},
    ui_widgets::Checkbox,
};

use crate::ui::common::{ELEMENT_FILL, ELEMENT_OUTLINE};

pub fn build_checkbox<T: Component>(
    font_handle: Handle<Font>,
    caption: &str,
    marker_component: T,
) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            column_gap: px(4),
            ..default()
        },
        Name::new("Checkbox"),
        Hovered::default(),
        marker_component,
        Checkbox,
        TabIndex(0),
        Children::spawn((
            Spawn((
                // Checkbox outer
                Node {
                    display: Display::Flex,
                    width: px(16),
                    height: px(16),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BorderColor::all(ELEMENT_OUTLINE), // Border color for the checkbox
                children![
                    // Checkbox inner
                    (
                        Node {
                            display: Display::Flex,
                            width: px(8),
                            height: px(8),
                            position_type: PositionType::Absolute,
                            left: px(2),
                            top: px(2),
                            ..default()
                        },
                        BackgroundColor(ELEMENT_FILL),
                    ),
                ],
            )),
            Spawn((
                Text::new(caption),
                TextFont {
                    font: font_handle,
                    ..default()
                },
            )),
        )),
    )
}

type AnyCheckboxInteraction = Or<(
    With<Checkbox>,
    (
        Added<Checkbox>,
        Changed<Hovered>,
        Added<Checked>,
        Added<InteractionDisabled>,
    ),
)>;

// Update the element's styles.
pub fn update_checkbox_style(
    mut checkboxes: Query<
        (Has<Checked>, &Hovered, &Children),
        AnyCheckboxInteraction,
    >,
    mut border_colors: Query<
        (&mut BorderColor, &mut Children),
        Without<Checkbox>,
    >,
    mut background_colors: Query<
        &mut BackgroundColor,
        (Without<Checkbox>, Without<Children>),
    >,
) {
    for (checked, Hovered(is_hovering), children) in checkboxes.iter_mut() {
        let Some(border_id) = children.first() else {
            continue;
        };

        let Ok((mut border_color, border_children)) =
            border_colors.get_mut(*border_id)
        else {
            continue;
        };

        let Some(mark_id) = border_children.first() else {
            warn!("Checkbox does not have a mark entity.");
            continue;
        };

        let Ok(mut mark_bg) = background_colors.get_mut(*mark_id) else {
            warn!("Checkbox mark entity lacking a background color.");
            continue;
        };

        set_checkbox_style(
            *is_hovering,
            checked,
            &mut border_color,
            &mut mark_bg,
        );
    }
}

pub fn update_checkbox_style2(
    mut q_checkbox: Query<(Has<Checked>, &Hovered, &Children), With<Checkbox>>,
    mut q_border_color: Query<
        (&mut BorderColor, &mut Children),
        Without<Checkbox>,
    >,
    mut q_bg_color: Query<
        &mut BackgroundColor,
        (Without<Checkbox>, Without<Children>),
    >,
    mut removed_checked: RemovedComponents<Checked>,
) {
    removed_checked.read().for_each(|entity| {
        if let Ok((checked, Hovered(is_hovering), children)) =
            q_checkbox.get_mut(entity)
        {
            let Some(border_id) = children.first() else {
                return;
            };

            let Ok((mut border_color, border_children)) =
                q_border_color.get_mut(*border_id)
            else {
                return;
            };

            let Some(mark_id) = border_children.first() else {
                warn!("Checkbox does not have a mark entity.");
                return;
            };

            let Ok(mut mark_bg) = q_bg_color.get_mut(*mark_id) else {
                warn!("Checkbox mark entity lacking a background color.");
                return;
            };

            set_checkbox_style(
                *is_hovering,
                checked,
                &mut border_color,
                &mut mark_bg,
            );
        }
    });
}

fn set_checkbox_style(
    hovering: bool,
    checked: bool,
    border_color: &mut BorderColor,
    mark_bg: &mut BackgroundColor,
) {
    let color: Color = if hovering {
        // If hovering, use a lighter color
        ELEMENT_OUTLINE.lighter(0.2)
    } else {
        // Default color for the element
        ELEMENT_OUTLINE
    };

    // Update the background color of the element
    border_color.set_all(color);

    let mark_color: Color = if checked {
        ELEMENT_FILL
    } else {
        Srgba::NONE.into()
    };

    if mark_bg.0 != mark_color {
        // Update the color of the element
        mark_bg.0 = mark_color;
    }
}
