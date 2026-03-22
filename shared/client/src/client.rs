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

use tonic::service::interceptor::InterceptedService;

#[cfg(feature = "map")]
use crate::auth::AuthInterceptor;

type InterceptedChannel = InterceptedService<Channel, AuthInterceptor>;

#[derive(Debug, Clone)]
pub struct Client {
    #[cfg(feature = "map")]
    map_client: MapServiceClient<InterceptedChannel>,
    #[cfg(feature = "broadcast")]
    broadcast_client: BroadcastServiceClient<InterceptedChannel>,
}

impl Client {
    pub fn new(channel: Channel) -> Self {
        let interceptor = AuthInterceptor::new();
        Self::new_with_interceptor(channel, interceptor)
    }

    pub fn with_auth_token(channel: Channel, auth_token: String) -> Result<Self, String> {
        let interceptor = AuthInterceptor::new().with_auth(auth_token)?;
        Ok(Self::new_with_interceptor(channel, interceptor))
    }

    fn new_with_interceptor(channel: Channel, interceptor: AuthInterceptor) -> Self {
        Self {
            #[cfg(feature = "map")]
            map_client: MapServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
            #[cfg(feature = "broadcast")]
            broadcast_client: BroadcastServiceClient::with_interceptor(
                channel.clone(),
                interceptor.clone(),
            ),
        }
    }
}
