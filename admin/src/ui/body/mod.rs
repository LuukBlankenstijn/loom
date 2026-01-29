mod stations;
mod teams;

use std::{collections::HashMap, sync::Arc};

use iced::{Element, Length, Task, widget::container};

use crate::{
    service::{AdminService, Station, Team},
    ui::header::Tab,
};

#[derive(Debug, Default)]
pub struct Body {
    teams: Option<Vec<Team>>,
    ip_team_name_map: HashMap<String, String>,
    stations: Option<Vec<Station>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadTeams,
    TeamsLoaded(Result<Vec<Team>, String>),
    LoadStations,
    StationsLoaded(Result<Vec<Station>, String>),
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
        };
        container(page).height(Length::Fill).into()
    }

    pub fn update(&mut self, message: Message, service: Arc<dyn AdminService>) -> Task<Message> {
        let service = service.clone();
        match message {
            Message::LoadTeams => Task::perform(
                async move { service.fetch_teams().await.map_err(|e| e.to_string()) },
                Message::TeamsLoaded,
            ),
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
                Task::none()
            }
            Message::LoadStations => Task::perform(
                async move { service.fetch_stations().await.map_err(|e| e.to_string()) },
                Message::StationsLoaded,
            ),

            Message::StationsLoaded(stations) => {
                match stations {
                    Ok(stations) => {
                        self.stations = Some(stations);
                    }
                    Err(msg) => {
                        println!("failed to load stations: {msg}");
                    }
                };
                Task::none()
            }
        }
    }
}
