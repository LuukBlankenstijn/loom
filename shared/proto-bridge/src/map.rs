use loom_core::map::{
    MapElement, MapMetadata, Point, Rotation,
    door::Door,
    seat::Seat,
    wall::Wall,
};
use loom_rpc::map::v1 as pb;
use tonic::Status;
use uuid::Uuid;

use crate::{IntoProto, TryIntoCore};

impl IntoProto<pb::MapMetadata> for MapMetadata {
    fn into_proto(self) -> pb::MapMetadata {
        pb::MapMetadata {
            id: self.id,
            name: self.name,
        }
    }
}

fn rotation_to_proto(r: Rotation) -> pb::Rotation {
    match r {
        Rotation::Deg0 => pb::Rotation::Rotation0,
        Rotation::Deg90 => pb::Rotation::Rotation90,
        Rotation::Deg180 => pb::Rotation::Rotation180,
        Rotation::Deg270 => pb::Rotation::Rotation270,
    }
}

fn rotation_from_proto(r: pb::Rotation) -> Rotation {
    match r {
        pb::Rotation::Rotation90 => Rotation::Deg90,
        pb::Rotation::Rotation180 => Rotation::Deg180,
        pb::Rotation::Rotation270 => Rotation::Deg270,
        _ => Rotation::Deg0,
    }
}

impl IntoProto<pb::Element> for &MapElement {
    fn into_proto(self) -> pb::Element {
        match self {
            MapElement::Wall(wall) => pb::Element {
                element: Some(pb::element::Element::Wall(pb::Wall {
                    id: wall.id.to_string(),
                    start: Some(pb::Location {
                        x: wall.start.x as i32,
                        y: wall.start.y as i32,
                    }),
                    end: Some(pb::Location {
                        x: wall.end.x as i32,
                        y: wall.end.y as i32,
                    }),
                })),
            },
            MapElement::Door(door) => pb::Element {
                element: Some(pb::element::Element::Door(pb::Door {
                    id: door.id.to_string(),
                    location: Some(pb::Location {
                        x: door.position.x as i32,
                        y: door.position.y as i32,
                    }),
                    rotation: rotation_to_proto(door.rotation) as i32,
                })),
            },
            MapElement::Seat(seat) => pb::Element {
                element: Some(pb::element::Element::Seat(pb::Seat {
                    id: seat.id.to_string(),
                    location: Some(pb::Location {
                        x: seat.position.x as i32,
                        y: seat.position.y as i32,
                    }),
                    rotation: rotation_to_proto(seat.rotation) as i32,
                })),
            },
        }
    }
}

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
                let start = w.start.as_ref().unwrap_or(&pb::Location { x: 0, y: 0 });
                let end = w.end.as_ref().unwrap_or(&pb::Location { x: 0, y: 0 });
                Ok(Wall {
                    id,
                    start: Point::new(start.x as f32, start.y as f32),
                    end: Point::new(end.x as f32, end.y as f32),
                }
                .into())
            }
            pb::element::Element::Door(d) => {
                let id = Uuid::parse_str(&d.id)
                    .map_err(|_| Status::invalid_argument("invalid door uuid"))?;
                let loc = d.location.as_ref().unwrap_or(&pb::Location { x: 0, y: 0 });
                let rotation = rotation_from_proto(
                    pb::Rotation::try_from(d.rotation).unwrap_or(pb::Rotation::Rotation0),
                );
                Ok(Door {
                    id,
                    position: Point::new(loc.x as f32, loc.y as f32),
                    rotation,
                }
                .into())
            }
            pb::element::Element::Seat(s) => {
                let id = Uuid::parse_str(&s.id)
                    .map_err(|_| Status::invalid_argument("invalid table uuid"))?;
                let loc = s.location.as_ref().unwrap_or(&pb::Location { x: 0, y: 0 });
                let rotation = rotation_from_proto(
                    pb::Rotation::try_from(s.rotation).unwrap_or(pb::Rotation::Rotation0),
                );
                Ok(Seat {
                    id,
                    position: Point::new(loc.x as f32, loc.y as f32),
                    rotation,
                }
                .into())
            }
        }
    }
}
