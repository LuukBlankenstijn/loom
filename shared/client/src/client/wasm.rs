#[cfg(feature = "broadcast")]
use loom_rpc::broadcast::v1::broadcast_service_client::BroadcastServiceClient;
#[cfg(feature = "map")]
use loom_rpc::map::v1::map_service_client::MapServiceClient;

use tonic_web_wasm_client::Client as Channel;

use crate::client::Client;

pub type InnerChannel = Channel;

impl Client {
    pub fn new(channel: Channel) -> Self {
        Self {
            #[cfg(feature = "map")]
            map_client: MapServiceClient::new(channel.clone()),
            #[cfg(feature = "broadcast")]
            broadcast_client: BroadcastServiceClient::new(channel.clone()),
        }
    }
}
