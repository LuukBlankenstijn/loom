use futures::{StreamExt, stream::BoxStream};
use loom_rpc::broadcast::v1::SubscribeBroadcastRequest;

use crate::{
    client::Client,
    convert::broadcast::types::{BroadcastEvent, BroadcastType},
};
use loom_rpc::broadcast::v1 as pb;

pub trait BroadcastClient {
    fn subscribe(
        &self,
        event_types: &[BroadcastType],
    ) -> impl Future<Output = Result<BoxStream<'static, BroadcastEvent>, String>>;
}

impl BroadcastClient for Client {
    async fn subscribe(
        &self,
        event_types: &[BroadcastType],
    ) -> Result<BoxStream<'static, BroadcastEvent>, String> {
        let request = SubscribeBroadcastRequest {
            types: event_types
                .iter()
                .map(|&t| pb::BroadcastType::from(t) as i32)
                .collect(),
        };

        let response = self
            .broadcast_client
            .clone()
            .subscribe(request)
            .await
            .map_err(|e| e.to_string())?;

        let stream = response.into_inner().filter_map(|res| async move {
            res.ok()
                .and_then(|proto| BroadcastEvent::try_from(proto).ok())
        });

        Ok(Box::pin(stream))
    }
}
