use crate::convert::prelude::{ConvertError, FromProto, ToProto, TryFromProto};

use loom_map_types::{MapElement, Point, Rotation, seat::Seat};
use loom_rpc::map::v1::Seat as ProtoSeat;

impl TryFromProto<ProtoSeat> for Seat {
    fn try_from_proto(val: ProtoSeat) -> Result<Self, ConvertError> {
        let id =
            uuid::Uuid::try_parse(&val.id).map_err(|_| ConvertError("Invalid uuid".to_string()))?;
        let location = val
            .location
            .ok_or(ConvertError("No location".to_string()))?;
        Ok(Seat {
            id,
            position: Point::from_proto(location),
            rotation: Rotation::from_proto(val.rotation()),
        })
    }
}

impl TryFromProto<ProtoSeat> for MapElement {
    fn try_from_proto(val: ProtoSeat) -> Result<Self, ConvertError> {
        Ok(Seat::try_from_proto(val)?.into())
    }
}

impl ToProto<ProtoSeat> for &Seat {
    fn to_proto(self) -> ProtoSeat {
        ProtoSeat {
            id: self.id.to_string(),
            location: self.position.to_proto().into(),
            rotation: self.rotation.to_proto().into(),
        }
    }
}
