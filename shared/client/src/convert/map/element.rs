use loom_map_types::{MapElement, door::Door, seat::Seat, wall::Wall};
use loom_rpc::map::v1::{Element as ProtoWrapperElement, element::Element as ProtoElement};

use crate::convert::map::prelude::{ToProto, TryFromProto};

impl TryFromProto<ProtoElement> for MapElement {
    fn try_from_proto(val: ProtoElement) -> Result<Self, super::prelude::ConvertError> {
        let element = match val {
            ProtoElement::Wall(wall) => Wall::try_from_proto(wall)?.into(),
            ProtoElement::Door(door) => Door::try_from_proto(door)?.into(),
            ProtoElement::Seat(seat) => Seat::try_from_proto(seat)?.into(),
        };
        Ok(element)
    }
}

impl ToProto<ProtoElement> for MapElement {
    fn to_proto(self) -> ProtoElement {
        match self {
            MapElement::Door(door) => ProtoElement::Door(door.to_proto()),
            MapElement::Wall(wall) => ProtoElement::Wall(wall.to_proto()),
            MapElement::Seat(seat) => ProtoElement::Seat(seat.to_proto()),
        }
    }
}

impl ToProto<ProtoWrapperElement> for MapElement {
    fn to_proto(self) -> ProtoWrapperElement {
        ProtoWrapperElement {
            element: Some(self.to_proto()),
        }
    }
}
