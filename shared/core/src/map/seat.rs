use uuid::Uuid;

use super::{Point, Rotation};

#[derive(Clone, Debug, PartialEq)]
pub struct Seat {
    pub id: Uuid,
    pub position: Point,
    pub rotation: Rotation,
}

impl Seat {
    pub const TABLE_W: f32 = 200.0;
    pub const TABLE_H: f32 = 90.0;
    pub const CHAIR_ARC_RADIUS: f32 = 20.0;
    pub const CHAIR_PROTRUSION: f32 = 25.0;

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

    pub fn get_total_bounds(&self) -> (f32, f32) {
        let total_w = Self::TABLE_W;
        let total_h = Self::TABLE_H + Self::CHAIR_PROTRUSION;

        match self.rotation {
            Rotation::Deg0 | Rotation::Deg180 => (total_w, total_h),
            Rotation::Deg90 | Rotation::Deg270 => (total_h, total_w),
        }
    }
}
