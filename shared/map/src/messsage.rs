use iced::{Point, Vector};
use uuid::Uuid;

use crate::MapElement;

#[derive(Clone, Debug)]
pub enum GridMessage {
    MapPanned(Vector<f32>),
    MapZoomed { factor: f32, cursor: Point },
    DrawFinish(Point, Point),
    RequestSelect(Point),
}

impl From<GridMessage> for Message {
    fn from(val: GridMessage) -> Self {
        Message::Grid(val)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum SystemMessage {
    AddElement(MapElement),
}

impl From<SystemMessage> for Message {
    fn from(value: SystemMessage) -> Self {
        Message::System(value)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Message {
    #[doc(hidden)]
    Grid(GridMessage),
    #[doc(hidden)]
    System(SystemMessage),
    ToggleSelect(Uuid),
    ClearSelection,
    DeleteSelection,
    DuplicateSelection,
    MoveSelection(Vector),
    RotateSelection,
    AddElement(fn(Point) -> MapElement),
}
