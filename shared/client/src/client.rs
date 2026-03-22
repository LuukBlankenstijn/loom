#[cfg(feature = "broadcast")]
pub mod broadcast;
#[cfg(feature = "map")]
pub mod map;

#[cfg(feature = "broadcast")]
use loom_rpc::broadcast::v1::broadcast_service_client::BroadcastServiceClient;
#[cfg(feature = "map")]
use loom_rpc::map::v1::map_service_client::MapServiceClient;

#[cfg(not(target_arch = "wasm32"))]
use tonic::transport::Channel;
#[cfg(target_arch = "wasm32")]
use tonic_web_wasm_client::Client as Channel;

#[derive(Debug, Clone)]
pub struct Client {
    #[cfg(feature = "map")]
    map_client: MapServiceClient<Channel>,
    #[cfg(feature = "broadcast")]
    broadcast_client: BroadcastServiceClient<Channel>,
}

impl Client {
    pub fn new(channel: Channel) -> Self {
        Self {
            #[cfg(feature = "map")]
            map_client: MapServiceClient::new(channel.clone()),
            #[cfg(feature = "broadcast")]
            broadcast_client: BroadcastServiceClient::new(channel),
        }
    }
}
