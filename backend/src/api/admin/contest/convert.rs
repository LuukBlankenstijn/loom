use crate::{api::to_timestamp, domain};
use loom_rpc::admin::v1 as admin_pb;

impl From<domain::Contest> for admin_pb::Contest {
    fn from(c: domain::Contest) -> Self {
        Self {
            id: c.id,
            name: c.name,
            start_time: Some(to_timestamp(c.start_time)),
            end_time: Some(to_timestamp(c.end_time)),
            map_id: None,
        }
    }
}
