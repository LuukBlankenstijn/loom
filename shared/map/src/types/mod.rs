use iced::{Vector, widget::canvas::Frame};
pub use loom_map_types;
use loom_map_types::{MapElement, Rotation};
use uuid::Uuid;

mod door;
pub mod prelude;
mod seat;
mod wall;

macro_rules! dispatch_on_map_element {
    ($target:expr, $inner:ident => $exec:expr) => {
        match $target {
            MapElement::Door($inner) => $exec,
            MapElement::Wall($inner) => $exec,
            MapElement::Seat($inner) => $exec,
        }
    };
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame, scale: f32, selected: bool);
    fn get_id(&self) -> Uuid;
    fn is_hit(&self, point: iced::Point) -> bool;
    fn move_by(&mut self, delta: Vector);
    fn duplicate(&self) -> Self;
    fn rotate(&mut self, _rotation: Option<Rotation>) {}
}

impl Drawable for MapElement {
    fn draw(&self, frame: &mut Frame, scale: f32, selected: bool) {
        dispatch_on_map_element!(self, x => x.draw(frame, scale, selected))
    }

    fn get_id(&self) -> Uuid {
        dispatch_on_map_element!(self, x => x.get_id())
    }

    fn is_hit(&self, point: iced::Point) -> bool {
        dispatch_on_map_element!(self, x => x.is_hit(point))
    }

    fn move_by(&mut self, delta: Vector) {
        dispatch_on_map_element!(self, x => x.move_by(delta))
    }

    fn duplicate(&self) -> Self {
        dispatch_on_map_element!(self, x => x.duplicate().into())
    }

    fn rotate(&mut self, rotation: Option<Rotation>) {
        dispatch_on_map_element!(self, x => x.rotate(rotation))
    }
}
