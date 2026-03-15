use crate::domain;
use loom_rpc::admin::v1 as admin_pb;

impl From<domain::Station> for admin_pb::Station {
    fn from(s: domain::Station) -> Self {
        Self { ip: s.ip }
    }
}
