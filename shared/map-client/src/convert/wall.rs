use crate::convert::prelude::{ConvertError, FromProto, ToProto, TryFromProto};

use loom_map_types::{MapElement, Point, wall::Wall};
use loom_rpc::map::v1::Wall as ProtoWall;

impl TryFromProto<ProtoWall> for Wall {
    fn try_from_proto(val: ProtoWall) -> Result<Self, ConvertError> {
        let id =
            uuid::Uuid::try_parse(&val.id).map_err(|_| ConvertError("Invalid uuid".to_string()))?;
        let start = val
            .start
            .ok_or(ConvertError("No start location".to_string()))?;
        let end = val.end.ok_or(ConvertError("No end location".to_string()))?;

        Ok(Wall::construct(
            id,
            Point::from_proto(start),
            Point::from_proto(end),
        ))
    }
}

impl TryFromProto<ProtoWall> for MapElement {
    fn try_from_proto(val: ProtoWall) -> Result<Self, ConvertError> {
        Ok(Wall::try_from_proto(val)?.into())
    }
}

impl ToProto<ProtoWall> for Wall {
    fn to_proto(self) -> ProtoWall {
        ProtoWall {
            id: self.id.to_string(),
            start: self.start.to_proto().into(),
            end: self.end.to_proto().into(),
        }
    }
}
