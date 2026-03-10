use std::sync::Arc;

use futures::future::try_join_all;
use loom_rpc::admin::v1::admin_service_server::AdminService;
use loom_rpc::admin::v1::{self as pb, AssignTeamRequest, ClientCommand, DeleteStationRequest};
use loom_rpc::map::v1 as map_pb;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{convert::CommandOutput, domain::*, hub::StationsHub};

pub struct AdminHandler {
    contest_repo: Arc<dyn ContestRepository>,
    team_repo: Arc<dyn TeamRepository>,
    station_repo: Arc<dyn StationRepository>,
    wallpaper_repo: Arc<dyn WallpaperRepository>,
    map_repo: Arc<dyn MapRepository>,
    hub: Arc<StationsHub>,
    client_broadcast: broadcast::Sender<CommandOutput>,
}

impl AdminHandler {
    pub fn new(
        contest_repo: Arc<dyn ContestRepository>,
        team_repo: Arc<dyn TeamRepository>,
        station_repo: Arc<dyn StationRepository>,
        wallpaper_repo: Arc<dyn WallpaperRepository>,
        map_repo: Arc<dyn MapRepository>,
        hub: Arc<StationsHub>,
        client_broadcast: broadcast::Sender<CommandOutput>,
    ) -> Self {
        Self {
            contest_repo,
            team_repo,
            station_repo,
            wallpaper_repo,
            map_repo,
            hub,
            client_broadcast,
        }
    }
}

#[tonic::async_trait]
impl AdminService for AdminHandler {
    async fn get_next_contest(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::Contest>, Status> {
        let contest = self
            .contest_repo
            .get_next_contest()
            .await?
            .ok_or_else(|| Status::not_found("no upcoming contest"))?;

        let map_id = self
            .map_repo
            .get_by_contest(&contest.id)
            .await
            .ok()
            .flatten()
            .map(|m| m.id);

        let mut pb_contest: pb::Contest = contest.into();
        pb_contest.map_id = map_id;
        Ok(Response::new(pb_contest))
    }

    async fn get_active_teams(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::TeamsResponse>, Status> {
        let teams = match self.contest_repo.get_next_contest().await? {
            Some(contest) => self.team_repo.get_all(&contest.id).await?,
            None => vec![],
        };
        Ok(Response::new(pb::TeamsResponse {
            teams: teams.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_stations(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::StationsResponse>, Status> {
        let stations = self.station_repo.get_all().await?;
        Ok(Response::new(pb::StationsResponse {
            stations: stations.into_iter().map(Into::into).collect(),
        }))
    }

    async fn set_ip(&self, request: Request<pb::SetIpRequest>) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let maybe_ip = self
            .team_repo
            .set_ip(&req.team_id, req.ip.as_deref())
            .await?;
        if let Some(ip) = maybe_ip {
            self.hub.sync_stations(&[&ip]);
        }
        Ok(Response::new(()))
    }

    async fn set_wallpaper(
        &self,
        request: Request<pb::UploadWallpaperRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        match req.image_data.filter(|d| !d.is_empty()) {
            Some(data) => {
                let mime_type = validate_image(&data).map_err(Status::invalid_argument)?;
                self.wallpaper_repo
                    .set_wallpaper_data(&req.contest_id, &data, mime_type)
                    .await?;
            }
            None => {
                self.wallpaper_repo
                    .delete_wallpaper(&req.contest_id)
                    .await?;
            }
        }
        // update stations
        self.hub.sync_stations(&[]);
        Ok(Response::new(()))
    }

    async fn set_wallpaper_text_color(
        &self,
        request: Request<pb::SetWallpaperTextColorRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        self.wallpaper_repo
            .set_wallpaper_text_color(&req.contest_id, &req.color)
            .await?;
        // update stations
        self.hub.sync_stations(&[]);
        Ok(Response::new(()))
    }

    async fn get_wallpaper(
        &self,
        request: Request<pb::GetWallpaperRequest>,
    ) -> Result<Response<pb::WallpaperResponse>, Status> {
        let contest_id = match request.into_inner().contest_id {
            Some(id) => id,
            None => match self.contest_repo.get_next_contest().await? {
                Some(c) => c.id,
                None => {
                    return Ok(Response::new(pb::WallpaperResponse {
                        image_data: None,
                        color: None,
                    }));
                }
            },
        };

        match self.wallpaper_repo.get_wallpaper(&contest_id).await? {
            Some(wp) => Ok(Response::new(pb::WallpaperResponse {
                image_data: Some(wp.data),
                color: Some(wp.text_color),
            })),
            None => Ok(Response::new(pb::WallpaperResponse {
                image_data: None,
                color: None,
            })),
        }
    }

    async fn get_all_maps(
        &self,
        _request: Request<()>,
    ) -> Result<Response<pb::GetAllMapsResponse>, Status> {
        let maps = self.map_repo.get_all().await?;
        Ok(Response::new(pb::GetAllMapsResponse {
            maps: maps.into_iter().map(Into::into).collect(),
        }))
    }

    async fn set_map(&self, request: Request<pb::SetMapRequest>) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        self.map_repo.set_map(req.map_id, &req.contest_id).await?;
        Ok(Response::new(()))
    }

    async fn create_map(
        &self,
        request: Request<pb::CreateMapRequest>,
    ) -> Result<Response<pb::MapResponse>, Status> {
        let name = request.into_inner().name;
        let id = self.map_repo.create_map(&name).await?;
        Ok(Response::new(pb::MapResponse {
            map: Some(map_pb::Map { id, name }),
            elements: vec![],
        }))
    }

    async fn get_map(
        &self,
        request: Request<pb::GetMapRequest>,
    ) -> Result<Response<pb::MapResponse>, Status> {
        let full_map = self
            .map_repo
            .get_map(request.into_inner().id)
            .await?
            .ok_or_else(|| Status::not_found("map not found"))?;
        Ok(Response::new(full_map.into()))
    }

    async fn update_map(
        &self,
        request: Request<pb::UpdateMapRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let deleted_ids: Vec<Uuid> = req
            .deleted
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        if !deleted_ids.is_empty() {
            self.map_repo.delete_elements(&deleted_ids).await?;
        }

        let mut walls = Vec::new();
        let mut doors = Vec::new();
        let mut tables = Vec::new();

        for el in &req.updated {
            match &el.element {
                Some(map_pb::element::Element::Wall(_)) => {
                    walls.push(Wall::try_from(el)?);
                }
                Some(map_pb::element::Element::Door(_)) => {
                    doors.push(Door::try_from(el)?);
                }
                Some(map_pb::element::Element::Table(_)) => {
                    tables.push(Table::try_from(el)?);
                }
                None => {}
            }
        }

        if !walls.is_empty() || !doors.is_empty() || !tables.is_empty() {
            self.map_repo
                .upsert_elements(req.id, &walls, &doors, &tables)
                .await?;
        }

        Ok(Response::new(()))
    }

    type SubscribeStream = ReceiverStream<Result<pb::SubscribtionMessage, Status>>;

    async fn subscribe(&self, _: Request<()>) -> Result<Response<Self::SubscribeStream>, Status> {
        let (tx, rx) = mpsc::channel(32);
        let mut hub_channel = self.hub.subscribe_state();
        let mut broadcast_receiver = self.client_broadcast.subscribe();

        // send initial state
        if tx
            .send(Ok(self.hub.connected_stations().into()))
            .await
            .is_err()
        {
            return Err(Status::internal("failed to send state"));
        };

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = hub_channel.recv() => {
                        match msg {
                            Ok(stations) => {
                                if tx.send(Ok(stations.into())).await.is_err() {
                                    break;
                                };
                            }
                            _ => break,
                        }
                    }
                    msg = broadcast_receiver.recv() => {
                        match msg {
                            Ok(msg) => {
                                if tx.send(Ok(msg.into())).await.is_err() {
                                    break;
                                }
                            }
                            _ => break,
                        }
                    }
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn send_command(&self, request: Request<ClientCommand>) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        if let Some(command) = req.command {
            let ips: Vec<&str> = req.ips.iter().map(|s| s.as_str()).collect();
            self.hub.send_command(command.into(), &ips);
        }
        Ok(Response::new(()))
    }

    async fn delete_station(
        &self,
        request: Request<DeleteStationRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let station = self.station_repo.get_by_id(req.id).await?;
        if let Some(team) = self.team_repo.get_by_ip(&station.ip).await? {
            self.team_repo.set_ip(&team.id, None).await?;
        }
        self.station_repo.delete(req.id).await?;

        Ok(Response::new(()))
    }

    async fn assign_team(
        &self,
        request: Request<AssignTeamRequest>,
    ) -> Result<Response<()>, Status> {
        let ids = request.into_inner().ids;

        let (contest_opt, all_stations) = tokio::try_join!(
            self.contest_repo.get_next_contest(),
            self.station_repo.get_all(),
        )
        .map_err(|e| Status::internal(format!("Dependency error: {}", e)))?;

        let contest = match contest_opt {
            Some(c) => c,
            None => return Ok(Response::new(())),
        };

        let mut teams: Vec<_> = self
            .team_repo
            .get_all(&contest.id)
            .await?
            .into_iter()
            .filter(|t| t.ip.is_none())
            .collect();

        let mut stations: Vec<_> = all_stations
            .into_iter()
            .filter(|s| ids.contains(&s.id))
            .collect();

        let mut update_futures = Vec::new();
        let mut updated_ips = Vec::new();
        let pair_count = std::cmp::min(stations.len(), teams.len());

        for _ in 0..pair_count {
            if let (Some(station), Some(team)) = (stations.pop(), teams.pop()) {
                let team_id = team.id.clone();
                let station_ip = station.ip.clone();
                updated_ips.push(station_ip.clone());

                let fut = async move { self.team_repo.set_ip(&team_id, Some(&station_ip)).await };
                update_futures.push(Box::pin(fut));
            }
        }

        if !update_futures.is_empty() {
            try_join_all(update_futures)
                .await
                .map_err(|e| Status::internal(format!("Failed to batch update teams: {}", e)))?;
        }
        let sync_refs: Vec<&str> = updated_ips.iter().map(|s| s.as_str()).collect();
        self.hub.sync_stations(&sync_refs);

        Ok(Response::new(()))
    }
}

fn validate_image(data: &[u8]) -> Result<&'static str, String> {
    use image::ImageFormat;
    use std::io::Cursor;

    let format = image::guess_format(data).map_err(|_| "unsupported image format".to_string())?;

    let reader = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| format!("failed to read image: {e}"))?;

    reader
        .into_dimensions()
        .map_err(|e| format!("invalid image data: {e}"))?;

    let mime = match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Gif => "image/gif",
        ImageFormat::WebP => "image/webp",
        ImageFormat::Bmp => "image/bmp",
        ImageFormat::Tiff => "image/tiff",
        _ => return Err("unsupported image format".to_string()),
    };

    Ok(mime)
}
