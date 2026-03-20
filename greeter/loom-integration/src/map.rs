use tonic::transport::Channel;

use loom_rpc::map::v1::{
    map_service_client::MapServiceClient, FullMap, GetMapForContestRequest,
};

#[derive(Clone)]
pub struct MapClient {
    inner: MapServiceClient<Channel>,
}

impl MapClient {
    /// Connect to the loom backend at `endpoint` (e.g. `"http://localhost:8080"`).
    pub async fn connect(endpoint: &str) -> Result<Self, tonic::transport::Error> {
        let inner = MapServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self { inner })
    }

    /// Fetch the full map assigned to the given contest, including seat assignments.
    pub async fn get_map_for_contest(
        &mut self,
        contest_id: &str,
    ) -> Result<FullMap, tonic::Status> {
        let resp = self
            .inner
            .get_map_for_contest(GetMapForContestRequest {
                contest_id: contest_id.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }
}
