mod map;
mod modal;
mod stations;
mod teams;

use std::{collections::HashMap, sync::Arc};

use iced::{
    Element, Length, Task,
    widget::{container, stack},
};

use crate::{
    service::{AdminService, Station, Team},
    ui::{
        body::{map::MapApp, modal::Modal},
        header::Tab,
    },
};

#[derive(Debug, Default)]
pub struct Body {
    teams: Option<Vec<Team>>,
    ip_team_name_map: HashMap<String, String>,
    stations: Option<Vec<Station>>,
    modal_state: Option<Modal>,
    map: MapApp,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadTeams,
    TeamsLoaded(Result<Vec<Team>, String>),
    LoadStations,
    StationsLoaded(Result<Vec<Station>, String>),
    OpenModal {
        team_id: Option<String>,
        station_id: Option<i32>,
    },
    Modal(modal::Message),
    Unassign(String),
    Unassigned(Result<(), String>),
    Map(map::Message),
}

impl Body {
    pub fn new() -> (Self, Task<Message>) {
        let task = Task::batch(vec![
            Task::done(Message::LoadTeams),
            Task::done(Message::LoadStations),
        ]);

        (Self::default(), task)
    }

    pub fn view(&self, active_tab: Tab) -> Element<'_, Message> {
        let page = match active_tab {
            Tab::Stations => {
                stations::view_stations(self.stations.clone(), self.ip_team_name_map.clone())
            }
            Tab::Teams => teams::view_teams(self.teams.clone()),
            Tab::Map => self.map.view().map(Message::Map),
        };
        let base = container(page).height(Length::Fill);
        if let Some(modal) = &self.modal_state {
            stack![base, modal.view().map(Message::Modal)].into()
        } else {
            base.into()
        }
    }

    pub fn update(&mut self, message: Message, service: Arc<dyn AdminService>) -> Task<Message> {
        let service = service.clone();
        match message {
            Message::LoadTeams => {
                return Task::perform(
                    async move { service.fetch_teams().await.map_err(|e| e.to_string()) },
                    Message::TeamsLoaded,
                );
            }
            Message::TeamsLoaded(teams) => {
                match teams {
                    Ok(teams) => {
                        self.teams = Some(teams.clone());
                        self.ip_team_name_map = teams
                            .into_iter()
                            .filter_map(|t| {
                                if let Some(ip) = t.ip {
                                    Some((ip, t.name))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                    Err(msg) => {
                        println!("failed to load teams: {msg}");
                    }
                };
            }
            Message::LoadStations => {
                return Task::perform(
                    async move { service.fetch_stations().await.map_err(|e| e.to_string()) },
                    Message::StationsLoaded,
                );
            }
            Message::StationsLoaded(stations) => {
                match stations {
                    Ok(stations) => {
                        self.stations = Some(stations);
                    }
                    Err(msg) => {
                        println!("failed to load stations: {msg}");
                    }
                };
            }
            Message::Modal(message) => {
                match message {
                    modal::Message::Cancel => {
                        self.modal_state = None;
                    }
                    modal::Message::Close => {
                        self.modal_state = None;
                        return Task::done(Message::LoadTeams);
                    }
                    _ => {
                        if let Some(modal) = &mut self.modal_state {
                            return modal.update(message, service).map(Message::Modal);
                        }
                    }
                };
            }
            Message::OpenModal {
                team_id,
                station_id,
            } => {
                self.modal_state = Some(modal::Modal::new(
                    team_id,
                    station_id,
                    self.teams.clone(),
                    self.stations.clone(),
                ));
            }
            Message::Unassign(team_id) => {
                return Task::perform(
                    async move {
                        service
                            .set_ip(team_id, None)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::Unassigned,
                );
            }
            Message::Unassigned(result) => match result {
                Ok(_) => return Task::done(Message::LoadTeams),
                Err(msg) => {
                    println!("Failed to unassign team: {msg}")
                }
            },
            Message::Map(msg) => return self.map.update(msg).map(Message::Map),
        };
        Task::none()
    }
}
