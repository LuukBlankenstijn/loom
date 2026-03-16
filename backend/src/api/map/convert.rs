use crate::domain;
use loom_rpc::map::v1 as pb;
use uuid::Uuid;

impl From<domain::MapMetadata> for pb::MapMetadata {
    fn from(m: domain::MapMetadata) -> Self {
        Self {
            id: m.id,
            name: m.name,
        }
    }
}

impl From<domain::Rotation> for pb::Rotation {
    fn from(r: domain::Rotation) -> Self {
        match r {
            domain::Rotation::R0 => Self::Rotation0,
            domain::Rotation::R90 => Self::Rotation90,
            domain::Rotation::R180 => Self::Rotation180,
            domain::Rotation::R270 => Self::Rotation270,
        }
    }
}

impl From<pb::Rotation> for domain::Rotation {
    fn from(r: pb::Rotation) -> Self {
        match r {
            pb::Rotation::Rotation90 => Self::R90,
            pb::Rotation::Rotation180 => Self::R180,
            pb::Rotation::Rotation270 => Self::R270,
            _ => Self::R0,
        }
    }
}

impl From<&domain::MapElement> for pb::Element {
    fn from(value: &domain::MapElement) -> Self {
        match value {
            domain::MapElement::Wall(wall) => pb::Element {
                element: Some(pb::element::Element::Wall(pb::Wall {
                    id: wall.id.to_string(),
                    start: Some(pb::Location {
                        x: wall.x_start,
                        y: wall.y_start,
                    }),
                    end: Some(pb::Location {
                        x: wall.x_end,
                        y: wall.y_end,
                    }),
                })),
            },
            domain::MapElement::Door(door) => pb::Element {
                element: Some(pb::element::Element::Door(pb::Door {
                    id: door.id.to_string(),
                    location: Some(pb::Location {
                        x: door.x,
                        y: door.y,
                    }),
                    rotation: pb::Rotation::from(door.rotation) as i32,
                })),
            },
            domain::MapElement::Seat(seat) => pb::Element {
                element: Some(pb::element::Element::Seat(pb::Seat {
                    id: seat.id.to_string(),
                    location: Some(pb::Location {
                        x: seat.x,
                        y: seat.y,
                    }),
                    rotation: pb::Rotation::from(seat.rotation) as i32,
                })),
            },
        }
    }
}

impl TryFrom<pb::Element> for domain::MapElement {
    type Error = tonic::Status;

    fn try_from(value: pb::Element) -> Result<Self, Self::Error> {
        let element = value
            .element
            .as_ref()
            .ok_or(tonic::Status::invalid_argument("null element"))?;
        match element {
            pb::element::Element::Wall(w) => {
                let id = Uuid::parse_str(&w.id)
                    .map_err(|_| tonic::Status::invalid_argument("invalid wall uuid"))?;
                let start = w.start.as_ref().unwrap_or(&pb::Location { x: 0, y: 0 });
                let end = w.end.as_ref().unwrap_or(&pb::Location { x: 0, y: 0 });
                Ok(domain::Wall {
                    id,
                    x_start: start.x,
                    y_start: start.y,
                    x_end: end.x,
                    y_end: end.y,
                }
                .into())
            }
            pb::element::Element::Door(door) => {
                let id = Uuid::parse_str(&door.id)
                    .map_err(|_| tonic::Status::invalid_argument("invalid door uuid"))?;
                let loc = door
                    .location
                    .as_ref()
                    .unwrap_or(&pb::Location { x: 0, y: 0 });
                let rotation = pb::Rotation::try_from(door.rotation)
                    .unwrap_or(pb::Rotation::Rotation0)
                    .into();
                Ok(domain::Door {
                    id,
                    x: loc.x,
                    y: loc.y,
                    rotation,
                }
                .into())
            }
            pb::element::Element::Seat(s) => {
                let id = Uuid::parse_str(&s.id)
                    .map_err(|_| tonic::Status::invalid_argument("invalid table uuid"))?;
                let loc = s.location.as_ref().unwrap_or(&pb::Location { x: 0, y: 0 });
                let rotation = pb::Rotation::try_from(s.rotation)
                    .unwrap_or(pb::Rotation::Rotation0)
                    .into();
                Ok(domain::Seat {
                    id,
                    x: loc.x,
                    y: loc.y,
                    rotation,
                }
                .into())
            }
        }
    }
}
