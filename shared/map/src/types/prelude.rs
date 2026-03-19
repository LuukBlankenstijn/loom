pub use loom_map_types;
use loom_map_types::{Point, Rotation};

pub trait IntoIced<T>: Sized {
    fn to_iced(&self) -> T;
}

pub trait FromIced<T>: Sized {
    fn from_iced(value: T) -> Self;
}

pub trait AddIcedVector {
    fn add_vector(&mut self, vector: iced::Vector);
}

impl IntoIced<iced::Radians> for Rotation {
    fn to_iced(&self) -> iced::Radians {
        match self {
            Rotation::Deg0 => 0.0 * iced::Radians::PI,
            Rotation::Deg90 => 0.5 * iced::Radians::PI,
            Rotation::Deg180 => 1.0 * iced::Radians::PI,
            Rotation::Deg270 => 1.5 * iced::Radians::PI,
        }
    }
}

impl IntoIced<iced::Point> for Point {
    fn to_iced(&self) -> iced::Point {
        iced::Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl AddIcedVector for Point {
    fn add_vector(&mut self, vector: iced::Vector) {
        self.x += vector.x;
        self.y += vector.y;
    }
}

impl FromIced<iced::Point> for Point {
    fn from_iced(value: iced::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
