use iced::{Point, Theme, Vector, widget::canvas::Frame};
use uuid::Uuid;

mod door;
mod wall;

pub use door::Door;
pub use wall::Wall;

#[derive(Clone, Debug)]
pub enum MapElement {
    Door(Door),
    Wall(Wall),
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

    fn move_by(&mut self, delta: Vector) {
        match self {
            MapElement::Door(door) => door.move_by(delta),
            MapElement::Wall(wall) => wall.move_by(delta),
        }
    }

    fn duplicate(&self) -> Self {
        match self {
            MapElement::Door(door) => MapElement::Door(door.duplicate()),
            MapElement::Wall(wall) => MapElement::Wall(wall.duplicate()),
        }
    }
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame, theme: &Theme, selected: bool);
    fn get_id(&self) -> Uuid;
    fn is_hit(&self, point: Point) -> bool;
    fn move_by(&mut self, delta: Vector);
    fn duplicate(&self) -> Self;
}
