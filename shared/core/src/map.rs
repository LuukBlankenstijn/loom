pub mod door;
pub mod seat;
pub mod wall;

use derive_more::{Constructor, From};

use door::Door;
use seat::Seat;
use wall::Wall;

#[derive(Debug, Clone)]
pub struct Map {
    pub id: i32,
    pub name: String,
    pub elements: Vec<MapElement>,
}

#[derive(Debug, Clone)]
pub struct MapMetadata {
    pub id: i32,
    pub name: String,
}

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

impl Rotation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deg0 => "0",
            Self::Deg90 => "90",
            Self::Deg180 => "180",
            Self::Deg270 => "270",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "90" => Self::Deg90,
            "180" => Self::Deg180,
            "270" => Self::Deg270,
            _ => Self::Deg0,
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
