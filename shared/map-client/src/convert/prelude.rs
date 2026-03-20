use std::{error::Error, fmt::Display};

use loom_map_types::{Point, Rotation};
use loom_rpc::map::v1::{Location as ProtoLocation, Rotation as ProtoRotation};

pub trait FromProto<T> {
    fn from_proto(val: T) -> Self;
}

pub trait TryFromProto<T>: Sized {
    fn try_from_proto(val: T) -> Result<Self, ConvertError>;
}

pub trait ToProto<T> {
    fn to_proto(self) -> T;
}

impl FromProto<ProtoLocation> for Point {
    fn from_proto(val: ProtoLocation) -> Self {
        (val.x as f32, val.y as f32).into()
    }
}

impl ToProto<ProtoLocation> for Point {
    fn to_proto(self) -> ProtoLocation {
        ProtoLocation {
            x: self.x as i32,
            y: self.y as i32,
        }
    }
}

impl FromProto<ProtoRotation> for Rotation {
    fn from_proto(val: ProtoRotation) -> Self {
        match val {
            ProtoRotation::Unspecified => Rotation::Deg0,
            ProtoRotation::Rotation0 => Rotation::Deg0,
            ProtoRotation::Rotation90 => Rotation::Deg90,
            ProtoRotation::Rotation180 => Rotation::Deg180,
            ProtoRotation::Rotation270 => Rotation::Deg270,
        }
    }
}

impl ToProto<ProtoRotation> for Rotation {
    fn to_proto(self) -> ProtoRotation {
        match self {
            Rotation::Deg0 => ProtoRotation::Rotation0,
            Rotation::Deg90 => ProtoRotation::Rotation90,
            Rotation::Deg180 => ProtoRotation::Rotation180,
            Rotation::Deg270 => ProtoRotation::Rotation270,
        }
    }
}

#[derive(Debug)]
pub struct ConvertError(pub String);
impl Error for ConvertError {}

impl Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
