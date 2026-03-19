use uuid::Uuid;

use crate::{Point, Rotation};

#[derive(Clone, Debug, PartialEq)]
pub struct Door {
    pub id: Uuid,
    pub position: Point,
    pub rotation: Rotation,
}

impl Door {
    pub const WIDTH: f32 = 100.0;
    pub fn new(position: Point, rotation: Option<Rotation>) -> Self {
        Self {
            id: Uuid::now_v7(),
            position,
            rotation: rotation.unwrap_or_default(),
        }
    }
    pub fn construct(id: Uuid, position: impl Into<Point>, rotation: impl Into<Rotation>) -> Self {
        Self {
            id,
            position: position.into(),
            rotation: rotation.into(),
        }
    }
}
