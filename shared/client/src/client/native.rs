use crate::client::Client;

#[cfg(feature = "broadcast")]
use loom_rpc::broadcast::v1::broadcast_service_client::BroadcastServiceClient;
#[cfg(feature = "map")]
use loom_rpc::map::v1::map_service_client::MapServiceClient;
use tonic::{
    metadata::{Ascii, MetadataValue},
    service::Interceptor,
};

use tonic::{service::interceptor::InterceptedService, transport::Channel};

pub type InnerChannel = InterceptedService<Channel, AuthInterceptor>;

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

#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    token: Option<MetadataValue<Ascii>>,
}

impl AuthInterceptor {
    pub fn new() -> Self {
        Self { token: None }
    }

    pub fn with_auth(&mut self, token: String) -> Result<Self, String> {
        let token_value: MetadataValue<_> = token
            .parse()
            .map_err(|_| "Invalid auth token, could not convert to metadata value".to_string())?;
        let mut new = self.clone();
        new.token = Some(token_value);
        Ok(new)
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let mut req = request;
        if let Some(token) = self.token.clone() {
            req.metadata_mut().insert("authorization", token);
        };

        Ok(req)
    }
}
