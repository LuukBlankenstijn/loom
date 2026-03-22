use loom_core::map::{
    MapElement, MapMetadata, Point, Rotation, door::Door, seat::Seat, wall::Wall,
};
use loom_rpc::map::v1 as pb;
use tonic::Status;
use uuid::Uuid;

use crate::{FromProto, IntoProto, TryIntoCore};

// -- Point --

impl FromProto<pb::Location> for Point {
    fn from_proto(val: pb::Location) -> Self {
        Point::new(val.x, val.y)
    }
}

impl IntoProto<pb::Location> for Point {
    fn into_proto(self) -> pb::Location {
        pb::Location {
            x: self.x,
            y: self.y,
        }
    }
}

// -- Rotation --

impl FromProto<pb::Rotation> for Rotation {
    fn from_proto(val: pb::Rotation) -> Self {
        match val {
            pb::Rotation::Rotation90 => Rotation::Deg90,
            pb::Rotation::Rotation180 => Rotation::Deg180,
            pb::Rotation::Rotation270 => Rotation::Deg270,
            _ => Rotation::Deg0,
        }
    }
}

impl IntoProto<pb::Rotation> for Rotation {
    fn into_proto(self) -> pb::Rotation {
        match self {
            Rotation::Deg0 => pb::Rotation::Rotation0,
            Rotation::Deg90 => pb::Rotation::Rotation90,
            Rotation::Deg180 => pb::Rotation::Rotation180,
            Rotation::Deg270 => pb::Rotation::Rotation270,
        }
    }
}

// -- MapMetadata --

impl IntoProto<pb::MapMetadata> for MapMetadata {
    fn into_proto(self) -> pb::MapMetadata {
        pb::MapMetadata {
            id: self.id,
            name: self.name,
        }
    }
}

// -- Wall --

impl TryIntoCore<Wall> for pb::Wall {
    type Error = String;

    fn try_into_core(self) -> Result<Wall, String> {
        let id = Uuid::try_parse(&self.id).map_err(|_| "invalid uuid".to_string())?;
        let start = self.start.ok_or("no start location")?;
        let end = self.end.ok_or("no end location")?;
        Ok(Wall::construct(
            id,
            Point::from_proto(start),
            Point::from_proto(end),
        ))
    }
}

impl IntoProto<pb::Wall> for Wall {
    fn into_proto(self) -> pb::Wall {
        pb::Wall {
            id: self.id.to_string(),
            start: Some(self.start.into_proto()),
            end: Some(self.end.into_proto()),
        }
    }
}

// -- Door --

impl TryIntoCore<Door> for pb::Door {
    type Error = String;

    fn try_into_core(self) -> Result<Door, String> {
        let id = Uuid::try_parse(&self.id).map_err(|_| "invalid uuid".to_string())?;
        let location = self.location.ok_or("no location")?;
        let rotation = Rotation::from_proto(
            pb::Rotation::try_from(self.rotation).unwrap_or(pb::Rotation::Rotation0),
        );
        Ok(Door {
            id,
            position: Point::from_proto(location),
            rotation,
        })
    }
}

impl IntoProto<pb::Door> for Door {
    fn into_proto(self) -> pb::Door {
        pb::Door {
            id: self.id.to_string(),
            location: Some(self.position.into_proto()),
            rotation: self.rotation.into_proto() as i32,
        }
    }
}

// -- Seat --

impl TryIntoCore<Seat> for pb::Seat {
    type Error = String;

    fn try_into_core(self) -> Result<Seat, String> {
        let id = Uuid::try_parse(&self.id).map_err(|_| "invalid uuid".to_string())?;
        let location = self.location.ok_or("no location")?;
        let rotation = Rotation::from_proto(
            pb::Rotation::try_from(self.rotation).unwrap_or(pb::Rotation::Rotation0),
        );
        Ok(Seat {
            id,
            position: Point::from_proto(location),
            rotation,
        })
    }
}

impl IntoProto<pb::Seat> for Seat {
    fn into_proto(self) -> pb::Seat {
        pb::Seat {
            id: self.id.to_string(),
            location: Some(self.position.into_proto()),
            rotation: self.rotation.into_proto() as i32,
        }
    }
}

// -- MapElement (owned) --

impl TryIntoCore<MapElement> for pb::element::Element {
    type Error = String;

    fn try_into_core(self) -> Result<MapElement, String> {
        Ok(match self {
            pb::element::Element::Wall(w) => w.try_into_core()?.into(),
            pb::element::Element::Door(d) => d.try_into_core()?.into(),
            pb::element::Element::Seat(s) => s.try_into_core()?.into(),
        })
    }
}

impl IntoProto<pb::element::Element> for MapElement {
    fn into_proto(self) -> pb::element::Element {
        match self {
            MapElement::Door(d) => pb::element::Element::Door(d.into_proto()),
            MapElement::Wall(w) => pb::element::Element::Wall(w.into_proto()),
            MapElement::Seat(s) => pb::element::Element::Seat(s.into_proto()),
        }
    }
}

impl IntoProto<pb::Element> for MapElement {
    fn into_proto(self) -> pb::Element {
        pb::Element {
            element: Some(IntoProto::<pb::element::Element>::into_proto(self)),
        }
    }
}

// -- MapElement (borrowed, for server-side streaming) --

impl IntoProto<pb::Element> for &MapElement {
    fn into_proto(self) -> pb::Element {
        let inner = match self {
            MapElement::Wall(w) => pb::element::Element::Wall(pb::Wall {
                id: w.id.to_string(),
                start: Some(w.start.into_proto()),
                end: Some(w.end.into_proto()),
            }),
            MapElement::Door(d) => pb::element::Element::Door(pb::Door {
                id: d.id.to_string(),
                location: Some(d.position.into_proto()),
                rotation: d.rotation.into_proto() as i32,
            }),
            MapElement::Seat(s) => pb::element::Element::Seat(pb::Seat {
                id: s.id.to_string(),
                location: Some(s.position.into_proto()),
                rotation: s.rotation.into_proto() as i32,
            }),
        };
        pb::Element {
            element: Some(inner),
        }
    }
}

// -- MapElement (server-side, from wrapper with tonic::Status error) --

impl TryIntoCore<MapElement> for pb::Element {
    type Error = Status;

    fn try_into_core(self) -> Result<MapElement, Status> {
        let element = self
            .element
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("null element"))?;
        match element {
            pb::element::Element::Wall(w) => {
                let id = Uuid::parse_str(&w.id)
                    .map_err(|_| Status::invalid_argument("invalid wall uuid"))?;
                let start = w.start.as_ref().unwrap_or(&pb::Location { x: 0.0, y: 0.0 });
                let end = w.end.as_ref().unwrap_or(&pb::Location { x: 0.0, y: 0.0 });
                Ok(Wall {
                    id,
                    start: Point::new(start.x, start.y),
                    end: Point::new(end.x, end.y),
                }
                .into())
            }
            pb::element::Element::Door(d) => {
                let id = Uuid::parse_str(&d.id)
                    .map_err(|_| Status::invalid_argument("invalid door uuid"))?;
                let loc = d
                    .location
                    .as_ref()
                    .unwrap_or(&pb::Location { x: 0.0, y: 0.0 });
                let rotation = Rotation::from_proto(
                    pb::Rotation::try_from(d.rotation).unwrap_or(pb::Rotation::Rotation0),
                );
                Ok(Door {
                    id,
                    position: Point::new(loc.x, loc.y),
                    rotation,
                }
                .into())
            }
            pb::element::Element::Seat(s) => {
                let id = Uuid::parse_str(&s.id)
                    .map_err(|_| Status::invalid_argument("invalid table uuid"))?;
                let loc = s
                    .location
                    .as_ref()
                    .unwrap_or(&pb::Location { x: 0.0, y: 0.0 });
                let rotation = Rotation::from_proto(
                    pb::Rotation::try_from(s.rotation).unwrap_or(pb::Rotation::Rotation0),
                );
                Ok(Seat {
                    id,
                    position: Point::new(loc.x, loc.y),
                    rotation,
                }
                .into())
            }
        }
    }
}
