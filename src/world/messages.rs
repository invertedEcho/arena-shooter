use bevy::prelude::*;

#[derive(Message)]
pub struct SpawnDebugPointMessage {
    pub point: Vec3,
    pub color: Color,
}

impl SpawnDebugPointMessage {
    pub fn _new<T: Into<Vec3>, U: Into<Color>>(point: T, color: U) -> Self {
        Self {
            point: point.into(),
            color: color.into(),
        }
    }
}
