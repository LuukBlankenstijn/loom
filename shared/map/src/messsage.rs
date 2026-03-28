use iced::{Point, Vector};
use uuid::Uuid;

use crate::types::Drawable;

#[derive(Clone, Debug)]
pub enum GridMessage {
    MapPanned(Vector<f32>),
    MapZoomed { factor: f32, cursor: Point },
    DrawFinish(Point, Point),
    RequestSelect(Point),
}

impl<T: Drawable> From<GridMessage> for Message<T> {
    fn from(val: GridMessage) -> Self {
        Message::Grid(val)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Message<T> {
    #[doc(hidden)]
    Grid(GridMessage),
    /// Insert an element directly into the map.
    Insert(T),
    /// Fired when the user finishes a draw gesture (shift+drag). Handle this
    /// in your consumer to create and insert whatever element you want.
    DrawFinish(Point, Point),
    ToggleSelect(Uuid),
    ClearSelection,
    DeleteSelection,
    DuplicateSelection,
    MoveSelection(Vector),
    RotateSelection,
    AddElement(fn(Point) -> T),
}
