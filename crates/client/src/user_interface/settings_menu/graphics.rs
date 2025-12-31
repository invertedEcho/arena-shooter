use bevy::prelude::*;

#[derive(Component)]
pub struct GraphicsCheckbox(pub GraphicsCheckboxType);

#[derive(PartialEq)]
pub enum GraphicsCheckboxType {
    Fullscreen,
}
