mod convert;

use std::{pin::Pin, sync::Arc};

use derive_more::derive::Constructor;
use futures::{Stream, StreamExt};
use loom_rpc::station::v1::{self as pb, station_service_server::StationService};
use tonic::{Request, Response, Status, Streaming, metadata::MetadataMap};
use tracing::error;

use crate::{
    domain::{Orchestrator, StationRepository, event::station::StationEvent},
    error::AppError,
};

#[derive(Clone, Constructor)]
pub struct StationHandler {
    station_repo: Arc<dyn StationRepository>,
    orchestrator: Arc<dyn Orchestrator>,
}

#[tonic::async_trait]
impl StationService for StationHandler {
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<pb::StationCommand, Status>> + Send>>;

    async fn subscribe(
        &self,
        request: Request<Streaming<pb::StationEvent>>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        // get ip and save in database
        let ip = get_ip(request.metadata())?;
        self.station_repo.upsert(&ip).await?;

        let domain_stream = self.orchestrator.register_station(&ip).await?;

        // listen to client stream
        let mut stream = request.into_inner();

        let orchestrator = self.orchestrator.clone();

        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                let message = match result {
                    Ok(msg) => msg,
                    Err(_) => {
                        break;
                    }
                };

                match StationEvent::try_from(message) {
                    Ok(event) => {
                        orchestrator.handle_event((ip.clone(), event).into());
                    }
                    Err(e) => {
                        error!(%ip, "station sent a bad message: {}", e);
                    }
                }
            }
        });

        // map stream
        let response_stream =
            domain_stream.map(|res| res.map(pb::StationCommand::from).map_err(Status::from));
        Ok(Response::new(Box::pin(response_stream)))
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
