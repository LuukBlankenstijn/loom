use crate::convert::map::prelude::{ConvertError, FromProto, ToProto, TryFromProto};
use loom_map_types::{MapElement, Point, Rotation, door::Door};
use loom_rpc::map::v1::Door as ProtoDoor;

impl TryFromProto<ProtoDoor> for Door {
    fn try_from_proto(val: ProtoDoor) -> Result<Self, ConvertError> {
        let id =
            uuid::Uuid::try_parse(&val.id).map_err(|_| ConvertError("Invalid uuid".to_string()))?;
        let location = val
            .location
            .ok_or(ConvertError("No location".to_string()))?;
        Ok(Door {
            id,
            position: Point::from_proto(location),
            rotation: Rotation::from_proto(val.rotation()),
        })
    }
}

impl TryFromProto<ProtoDoor> for MapElement {
    fn try_from_proto(val: ProtoDoor) -> Result<Self, ConvertError> {
        Ok(Door::try_from_proto(val)?.into())
    }
}

impl ToProto<ProtoDoor> for Door {
    fn to_proto(self) -> ProtoDoor {
        ProtoDoor {
            id: self.id.to_string(),
            location: self.position.to_proto().into(),
            rotation: self.rotation.to_proto().into(),
        }
    }
}
