use enum_dispatch::enum_dispatch;
use iced::{Point, Theme, Vector, widget::canvas::Frame};
use uuid::Uuid;

mod door;
mod wall;

pub use door::Door;
pub use wall::Wall;

#[derive(Clone, Debug)]
#[enum_dispatch(Drawable, MapElement)]
pub enum MapElement {
    Door(Door),
    Wall(Wall),
}

#[enum_dispatch]
pub trait Drawable {
    fn draw(&self, frame: &mut Frame, theme: &Theme, selected: bool);
    fn get_id(&self) -> Uuid;
    fn is_hit(&self, point: Point) -> bool;
    fn move_by(&mut self, delta: Vector);
    fn duplicate(&self) -> Self;
    fn rotate(&mut self, _rotation: Option<Rotation>) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u16)]
pub enum Rotation {
    #[default]
    Deg0 = 0,
    Deg90 = 90,
    Deg180 = 180,
    Deg270 = 270,
}

impl Rotation {
    fn rotate_cw(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }
}

impl From<Rotation> for iced::Radians {
    fn from(value: Rotation) -> Self {
        match value {
            Rotation::Deg0 => 0.0 * iced::Radians::PI,
            Rotation::Deg90 => 0.5 * iced::Radians::PI,
            Rotation::Deg180 => 1.0 * iced::Radians::PI,
            Rotation::Deg270 => 1.5 * iced::Radians::PI,
        }
    }
}
