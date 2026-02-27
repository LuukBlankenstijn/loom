use iced::Point;
use loom_map::{Door, MapElement, Rotation, Station, Wall};
use loom_rpc::map::v1::{
    Door as ProtoDoor, Element as ProtoElement, Location as ProtoLocation,
    Rotation as ProtoRotation, Table as ProtoTable, Wall as ProtoWall,
    element::Element as InnerElement,
};

pub trait FromProto<T> {
    fn from_proto(val: T) -> Self;
}

pub trait ToProto<T> {
    fn to_proto(self) -> T;
}

impl FromProto<ProtoWall> for Option<MapElement> {
    fn from_proto(val: ProtoWall) -> Self {
        let id = uuid::Uuid::try_parse(&val.id).ok()?;

        Some(Wall::construct(id, Point::from_proto(val.start), Point::from_proto(val.end)).into())
    }
}

impl ToProto<ProtoWall> for &Wall {
    fn to_proto(self) -> ProtoWall {
        ProtoWall {
            id: self.id.to_string(),
            start: self.start.to_proto().into(),
            end: self.end.to_proto().into(),
        }
    }
}

impl FromProto<ProtoDoor> for Option<MapElement> {
    fn from_proto(val: ProtoDoor) -> Self {
        let id = uuid::Uuid::try_parse(&val.id).ok()?;
        Some(
            Door::construct(
                id,
                Point::from_proto(val.location),
                Rotation::from_proto(val.rotation()),
            )
            .into(),
        )
    }
}

impl ToProto<ProtoDoor> for &Door {
    fn to_proto(self) -> ProtoDoor {
        ProtoDoor {
            id: self.id.to_string(),
            location: self.position.to_proto().into(),
            rotation: self.rotation.to_proto().into(),
        }
    }
}

impl FromProto<ProtoTable> for Option<MapElement> {
    fn from_proto(val: ProtoTable) -> Self {
        let id = uuid::Uuid::try_parse(&val.id).ok()?;
        Some(
            Station::construct(
                id,
                Point::from_proto(val.location),
                Rotation::from_proto(val.rotation()),
            )
            .into(),
        )
    }
}

impl ToProto<ProtoTable> for &Station {
    fn to_proto(self) -> ProtoTable {
        ProtoTable {
            id: self.id.to_string(),
            location: self.position.to_proto().into(),
            rotation: self.rotation.to_proto().into(),
        }
    }
}

impl FromProto<Option<ProtoLocation>> for Point {
    fn from_proto(val: Option<ProtoLocation>) -> Self {
        match val {
            Some(location) => (location.x as f32, location.y as f32).into(),
            None => Default::default(),
        }
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

impl FromProto<ProtoElement> for Option<MapElement> {
    fn from_proto(val: ProtoElement) -> Self {
        val.element.and_then(|e| match e {
            InnerElement::Wall(wall) => FromProto::from_proto(wall),
            InnerElement::Door(door) => FromProto::from_proto(door),
            InnerElement::Table(table) => FromProto::from_proto(table),
        })
    }
}

impl ToProto<ProtoElement> for &MapElement {
    fn to_proto(self) -> ProtoElement {
        let inner = match self {
            MapElement::Door(door) => InnerElement::Door(door.to_proto()),
            MapElement::Wall(wall) => InnerElement::Wall(wall.to_proto()),
            MapElement::Station(station) => InnerElement::Table(station.to_proto()),
        };
        ProtoElement {
            element: inner.into(),
        }
    }
}
