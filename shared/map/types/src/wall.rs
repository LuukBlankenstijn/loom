use uuid::Uuid;

use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub struct Wall {
    pub id: Uuid,
    pub start: Point,
    pub end: Point,
}

impl Wall {
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            id: Uuid::now_v7(),
            start,
            end,
        }
    }
    pub fn construct(id: Uuid, start: impl Into<Point>, end: impl Into<Point>) -> Self {
        Self {
            id,
            start: start.into(),
            end: end.into(),
        }
    }
    pub fn get_test() -> Vec<Self> {
        vec![
            Self::new(Point::new(50.0, 0.0), Point::new(200.0, 0.0)),
            Self::new(Point::new(200.0, 0.0), Point::new(200.0, 150.0)),
        ]
    }
}
