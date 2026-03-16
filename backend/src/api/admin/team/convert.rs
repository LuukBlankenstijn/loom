use crate::domain;
use loom_rpc::admin::v1 as admin_pb;

impl From<domain::Team> for admin_pb::Team {
    fn from(t: domain::Team) -> Self {
        Self {
            id: t.id,
            ip: t.ip,
            name: t.name,
        }
    }
}
