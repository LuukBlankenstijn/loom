use iced::{Point, Theme, widget::canvas::Frame};
use uuid::Uuid;

pub mod door;
pub mod wall;

#[derive(Clone, Debug)]
pub enum MapElement {
    Door(door::Door),
    Wall(wall::Wall),
}

impl Drawable for MapElement {
    fn draw(&self, frame: &mut Frame, theme: &Theme, selected: bool) {
        match self {
            MapElement::Door(wall) => wall.draw(frame, theme, selected),
            MapElement::Wall(door) => door.draw(frame, theme, selected),
        }
    }

    fn get_id(&self) -> Uuid {
        match self {
            MapElement::Door(door) => door.get_id(),
            MapElement::Wall(door) => door.get_id(),
        }
    }

    fn is_hit(&self, point: Point) -> bool {
        match self {
            MapElement::Door(door) => door.is_hit(point),
            MapElement::Wall(door) => door.is_hit(point),
        }
    }
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame, theme: &Theme, selected: bool);
    fn get_id(&self) -> Uuid;
    fn is_hit(&self, point: Point) -> bool;
}
