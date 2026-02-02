use iced::{Theme, widget::canvas::Frame};

pub mod door;
pub mod wall;

#[derive(Clone, Debug)]
pub enum MapElement {
    Door(door::Door),
    Wall(wall::Wall),
}

impl Drawable for MapElement {
    fn draw(&self, frame: &mut Frame, theme: &Theme) {
        match self {
            MapElement::Wall(door) => door.draw(frame, theme),
            MapElement::Door(wall) => wall.draw(frame, theme),
        }
    }
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame, theme: &Theme);
}
