mod convert;

use std::{pin::Pin, sync::Arc};

use derive_more::derive::Constructor;
use futures::{Stream, StreamExt};
use loom_rpc::broadcast::v1 as pb;
use loom_rpc::broadcast::v1::broadcast_service_server::BroadcastService;
use tokio_stream::{self as stream};
use tonic::{Request, Response, Status, async_trait};

use crate::domain::{Orchestrator, event::broadcast::BroadcastEvent};

#[derive(Constructor)]
pub struct BroadcastHandler {
    orchestrator: Arc<dyn Orchestrator>,
}

#[async_trait]
impl BroadcastService for BroadcastHandler {
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<pb::BroadcastEvent, Status>> + Send>>;

    async fn subscribe(&self, _: Request<()>) -> Result<Response<Self::SubscribeStream>, Status> {
        let state = self.orchestrator.get_state();
        let initial_stream = stream::once(Ok(BroadcastEvent::State(state).into()));

        let broadcast_stream = self
            .orchestrator
            .subscribe_broadcast()
            .map(|result| match result {
                Ok(event) => Ok(event.into()),
                Err(e) => Err(Status::from(e)),
            });

        let response_stream = initial_stream.chain(broadcast_stream);
        Ok(Response::new(Box::pin(response_stream)))
    }
}
