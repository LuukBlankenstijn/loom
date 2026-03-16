mod convert;

use std::{pin::Pin, sync::Arc};

use derive_more::derive::Constructor;
use futures::{Stream, StreamExt};
use loom_rpc::station::v1::{self as pb, station_service_server::StationService};
use tonic::{Request, Response, Status, Streaming, metadata::MetadataMap};
use tracing::error;

use crate::{
    domain::{ContestRepository, Orchestrator, StationRepository, event::station::StationEvent},
    error::AppError,
};

#[derive(Clone, Constructor)]
pub struct StationHandler {
    contests_repo: Arc<dyn ContestRepository>,
    station_repo: Arc<dyn StationRepository>,
    orchestrator: Arc<dyn Orchestrator>,
}

#[tonic::async_trait]
impl StationService for StationHandler {
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<pb::StationCommand, Status>> + Send>>;

    async fn subscribe(
        &self,
        request: Request<()>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        // get ip and save in database
        let ip = get_ip(request.metadata())?;
        self.station_repo.upsert(&ip).await?;

        let domain_stream = self.orchestrator.register_station(&ip).await?;

        // send state
        let contest = self.contests_repo.get_next_contest().await?;
        if let Some(contest) = contest {
            self.orchestrator.sync_api_url(&[&ip], contest.id);
        }
        self.orchestrator.sync_wallpaper(&[&ip]);

        // map stream
        let response_stream =
            domain_stream.map(|res| res.map(pb::StationCommand::from).map_err(Status::from));
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn push(
        &self,
        request: Request<Streaming<pb::StationEvent>>,
    ) -> Result<Response<()>, Status> {
        // get ip and save in database
        let ip = get_ip(request.metadata())?;
        self.station_repo.upsert(&ip).await?;

        let mut stream = request.into_inner();

        while let Some(result) = stream.next().await {
            let message = match result {
                Ok(msg) => msg,
                Err(_) => {
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
