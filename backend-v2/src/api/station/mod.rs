mod convert;

use std::{pin::Pin, sync::Arc};

use derive_more::derive::Constructor;
use futures::{Stream, StreamExt};
use loom_rpc::station::v1::{self as pb, station_service_server::StationService};
use tonic::{Request, Response, Status, Streaming, metadata::MetadataMap};
use tracing::error;

use crate::{
    domain::{Orchestrator, event::station::StationEvent},
    error::AppError,
};

#[derive(Clone, Constructor)]
pub struct StationsHandler {
    orchestrator: Arc<dyn Orchestrator>,
}

#[tonic::async_trait]
impl StationService for StationsHandler {
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<pb::StationCommand, Status>> + Send>>;

    async fn subscribe(
        &self,
        request: Request<()>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let ip = get_ip(request.metadata())?;
        let domain_stream = self.orchestrator.register_station(&ip).await?;

        let response_stream =
            domain_stream.map(|res| res.map(pb::StationCommand::from).map_err(Status::from));
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn push(
        &self,
        request: Request<Streaming<pb::StationEvent>>,
    ) -> Result<Response<()>, Status> {
        let ip = get_ip(request.metadata())?;
        let mut stream = request.into_inner();

        while let Some(result) = stream.next().await {
            let message = match result {
                Ok(msg) => msg,
                Err(e) => {
                    error!(%ip, "gRPC stream error: {}", e);
                    break;
                }
            };

            match StationEvent::try_from(message) {
                Ok(event) => {
                    self.orchestrator.handle_event((ip.clone(), event).into());
                }
                Err(e) => {
                    error!(%ip, "station sent a bad message: {}", e);
                }
            }
        }
        Ok(Response::new(()))
    }
}

fn get_ip(metadata: &MetadataMap) -> Result<String, AppError> {
    match metadata.get("x-loom-station-ip") {
        Some(ip) => {
            if let Ok(ip) = ip.to_str() {
                Ok(ip.to_string())
            } else {
                Err(AppError::InvalidArgument(
                    "ip metadata could not be decoded".to_string(),
                ))
            }
        }
        None => Err(AppError::InvalidArgument(
            "ip not found on meta data".to_string(),
        )),
    }
}
