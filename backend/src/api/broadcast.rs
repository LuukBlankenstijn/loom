use std::{pin::Pin, sync::Arc};

use derive_more::derive::Constructor;
use futures::{Stream, StreamExt};
use loom_core::event::broadcast::BroadcastEvent;
use loom_proto_bridge::IntoProto;
use loom_rpc::broadcast::v1::broadcast_service_server::BroadcastService;
use loom_rpc::broadcast::v1::{self as pb, SubscribeBroadcastRequest};
use tonic::{Request, Response, Status, async_trait};

use crate::domain::{MapRepository, Orchestrator};

#[derive(Constructor)]
pub struct BroadcastHandler {
    orchestrator: Arc<dyn Orchestrator>,
    map_repo: Arc<dyn MapRepository>,
}

#[async_trait]
impl BroadcastService for BroadcastHandler {
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<pb::BroadcastEvent, Status>> + Send>>;

    async fn subscribe(
        &self,
        request: Request<SubscribeBroadcastRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let mut broadcast_stream: Self::SubscribeStream = self
            .orchestrator
            .subscribe_broadcast()
            .map(|result| match result {
                Ok(event) => Ok(event.into_proto()),
                Err(e) => Err(Status::from(e)),
            })
            .boxed();

        let req = request.into_inner();
        if req
            .types
            .contains(&(pb::BroadcastType::ConnectionState as i32))
        {
            let state = self.orchestrator.get_state();
            let stream = tokio_stream::once(Ok(BroadcastEvent::Connection(state).into_proto()));
            broadcast_stream = stream.chain(broadcast_stream).boxed();
        }
        if req
            .types
            .contains(&(pb::BroadcastType::StationAssignments as i32))
        {
            let state = self.map_repo.get_all_station_assignments(None).await?;
            let stream = tokio_stream::once(Ok(BroadcastEvent::Assignment(state).into_proto()));
            broadcast_stream = stream.chain(broadcast_stream).boxed();
        }

        Ok(Response::new(broadcast_stream))
    }
}
