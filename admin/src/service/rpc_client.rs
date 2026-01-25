use super::{AdminService, Contest, Station, Team};
use anyhow::Result;
use chrono::{DateTime, Utc};
use loom_rpc::admin::v1::{
    Contest as RpcContest, SetIpRequest, Station as RpcStation, Team as RpcTeam,
    UploadImageRequest, admin_service_client::AdminServiceClient,
};
use tonic::{async_trait, transport::Channel};

fn ts_to_utc(ts: prost_types::Timestamp) -> Option<DateTime<Utc>> {
    let st: std::time::SystemTime = ts.try_into().ok()?;
    Some(DateTime::<Utc>::from(st))
}

#[derive(Debug)]
pub struct RpcProvider {
    channel: Channel,
}

impl RpcProvider {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl From<RpcTeam> for Team {
    fn from(value: RpcTeam) -> Self {
        Self {
            id: value.id,
            ip: value.ip,
            name: value.name,
        }
    }
}

impl From<RpcStation> for Station {
    fn from(value: RpcStation) -> Self {
        Self {
            id: value.id,
            ip: value.ip,
            connected_at: value
                .connected_at
                .and_then(ts_to_utc)
                .unwrap_or_else(Utc::now),
            disconnected_at: value.diconnected_at.and_then(ts_to_utc),
        }
    }
}

impl From<RpcContest> for Contest {
    fn from(value: RpcContest) -> Self {
        Self {
            id: value.id,
            name: value.name,
            start_time: value
                .start_time
                .and_then(ts_to_utc)
                .unwrap_or_else(Utc::now),
            end_time: value.end_time.and_then(ts_to_utc).unwrap_or_else(Utc::now),
        }
    }
}

#[async_trait]
impl AdminService for RpcProvider {
    async fn fetch_stations(&self) -> Result<Vec<Station>> {
        let mut client = AdminServiceClient::new(self.channel.clone());

        let response = client.get_stations(()).await?;

        Ok(response
            .into_inner()
            .stations
            .into_iter()
            .map(Station::from)
            .collect())
    }

    async fn fetch_teams(&self) -> Result<Vec<Team>> {
        let mut client = AdminServiceClient::new(self.channel.clone());

        let response = client.get_active_teams(()).await?;

        Ok(response
            .into_inner()
            .teams
            .into_iter()
            .map(Team::from)
            .collect())
    }

    async fn fetch_contest(&self) -> Result<Option<Contest>> {
        let mut client = AdminServiceClient::new(self.channel.clone());

        let response_result = client.get_next_contest(()).await;
        let inner = match response_result {
            Ok(response) => Some(response.into_inner().into()),
            Err(err) => {
                if err.code() == tonic::Code::NotFound {
                    None
                } else {
                    return Err(err.into());
                }
            }
        };

        Ok(inner)
    }

    async fn set_ip(&self, team_id: String, ip: Option<String>) -> Result<()> {
        let mut client = AdminServiceClient::new(self.channel.clone());

        client.set_ip(SetIpRequest { team_id, ip }).await?;

        Ok(())
    }
    async fn set_wallpaper(&self, contest_id: String, image_data: Option<Vec<u8>>) -> Result<()> {
        let mut client = AdminServiceClient::new(self.channel.clone());

        client
            .set_wallpaper(UploadImageRequest {
                contest_id,
                image_data,
            })
            .await?;

        Ok(())
    }
}
