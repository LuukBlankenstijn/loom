use std::sync::Arc;

use loom_rpc::stations::v1 as pb;
use loom_rpc::stations::v1::station_service_server::StationService;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

use crate::api::middleware::RequestExt;
use crate::convert::ClientAction;
use crate::domain::{ContestRepository, StationRepository};
use crate::hub::{StationCommand, StationsHub};

pub struct StationsHandler {
    hub: Arc<StationsHub>,
    contest_repo: Arc<dyn ContestRepository>,
    station_repo: Arc<dyn StationRepository>,
    contest_api_base_url: Option<String>,
}

impl StationsHandler {
    pub fn new(
        hub: Arc<StationsHub>,
        contest_repo: Arc<dyn ContestRepository>,
        station_repo: Arc<dyn StationRepository>,
        contest_api_base_url: Option<String>,
    ) -> Self {
        Self {
            hub,
            contest_repo,
            station_repo,
            contest_api_base_url,
        }
    }
}

#[tonic::async_trait]
impl StationService for StationsHandler {
    type SubscribeStream = ReceiverStream<Result<pb::ServerMessage, Status>>;

    async fn subscribe(
        &self,
        request: Request<Streaming<pb::ClientMessage>>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let client_meta = request.client_meta()?;
        let ip = client_meta.clone().ip.to_string();
        let host = client_meta.host.clone();

        let client_stream = request.into_inner();

        // Record in DB
        self.station_repo
            .upsert(&ip)
            .await
            .map_err(|e| Status::internal(format!("failed to upsert station: {e}")))?;

        // Register with hub (deregisters on drop of registration)
        let registration = self.hub.register(&ip)?;

        let (tx, rx) = mpsc::channel(32);

        // Send wallpaper source and contest URL
        let wallpaper_url = format!("http://{host}/wallpaper");
        self.hub
            .send_command(StationCommand::SetWallpaperSource(wallpaper_url), &[&ip]);

        if let Some(base_url) = &self.contest_api_base_url
            && let Ok(Some(contest)) = self.contest_repo.get_next_contest().await
        {
            let contest_url = format!("{base_url}/api/v4/contests/{}", contest.id);
            self.hub
                .send_command(StationCommand::SetContestUrl(contest_url), &[&ip]);
        }

        // Single task handles both directions with select!
        let hub = self.hub.clone();
        let station_repo = self.station_repo.clone();
        let ip_clone = ip.clone();

        tokio::spawn(async move {
            let mut registration = registration;
            let mut client_stream = client_stream;

            loop {
                tokio::select! {
                    Some(cmd) = registration.commands.recv() => {
                        let msg: pb::ServerMessage = cmd.into();
                        if tx.send(Ok(msg)).await.is_err() {
                            break;
                        }
                    }
                    msg = client_stream.message() => {
                        match msg {
                            Ok(Some(msg)) => {
                                if let Ok(action) = ClientAction::try_from(msg) {
                                    match action {
                                        ClientAction::LoggedIn => {
                                            hub.set_login_status(&ip_clone, true);
                                        }
                                        ClientAction::LoggedOut => {
                                            hub.set_login_status(&ip_clone, false);
                                        }
                                        ClientAction::CommandOutput { .. } => {
                                            // TODO: forward command output to admin
                                        }
                                    }
                                }
                            }
                            _ => break,
                        }
                    }
                }
            }

            // Station disconnected — registration is dropped here (deregisters from hub)
            let _ = station_repo.update_disconnected_at(&ip_clone).await;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
