#[cfg(feature = "broadcast")]
pub mod broadcast;
#[cfg(feature = "map")]
pub mod map;
#[cfg(feature = "broadcast")]
use loom_rpc::broadcast::v1::broadcast_service_client::BroadcastServiceClient;
#[cfg(feature = "map")]
use loom_rpc::map::v1::map_service_client::MapServiceClient;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use self::native::*;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::*;

#[derive(Debug, Clone)]
pub struct Client {
    #[cfg(feature = "map")]
    map_client: MapServiceClient<InnerChannel>,
    #[cfg(feature = "broadcast")]
    broadcast_client: BroadcastServiceClient<InnerChannel>,
}
