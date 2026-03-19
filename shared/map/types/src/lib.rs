use derive_more::{Constructor, From};

use crate::{door::Door, seat::Seat, wall::Wall};

pub mod door;
pub mod seat;
pub mod wall;

#[derive(Clone, Debug, PartialEq, From)]
pub enum MapElement {
    Door(Door),
    Wall(Wall),
    Seat(Seat),
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
    pub fn rotate_cw(self) -> Self {
        match self {
            Rotation::Deg0 => Rotation::Deg90,
            Rotation::Deg90 => Rotation::Deg180,
            Rotation::Deg180 => Rotation::Deg270,
            Rotation::Deg270 => Rotation::Deg0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Constructor, Copy, Default)]
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T> From<(T, T)> for Point<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
