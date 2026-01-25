use std::sync::Arc;

use iced::{
    Element, Length, Task,
    widget::{container, text},
};

use crate::{
    service::{AdminService, Station, Team},
    ui::header::Tab,
};

#[derive(Debug, Default)]
pub struct Body {
    teams: Option<Vec<Team>>,
    teams_loading: bool,
    stations: Option<Vec<Station>>,
    stations_loading: bool,
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
        let label = if active_tab == Tab::Stations {
            if let Some(stations) = self.stations.clone() {
                format!("stations: {:?}", stations)
            } else {
                String::from("No stations loaded")
            }
        } else if let Some(teams) = self.teams.clone() {
            format!("teams: {:?}", teams)
        } else {
            String::from("No teams loaded")
        };
        container(text(label)).height(Length::Fill).into()
    }

    pub fn update(&mut self, message: Message, service: Arc<dyn AdminService>) -> Task<Message> {
        let service = service.clone();
        match message {
            Message::LoadTeams => {
                self.teams_loading = true;
                Task::perform(
                    async move { service.fetch_teams().await.map_err(|e| e.to_string()) },
                    Message::TeamsLoaded,
                )
            }
            Message::TeamsLoaded(teams) => {
                match teams {
                    Ok(teams) => {
                        self.teams = Some(teams);
                    }
                    Err(msg) => {
                        println!("failed to load teams: {msg}");
                    }
                };
                self.teams_loading = false;
                Task::none()
            }
            Message::LoadStations => {
                self.stations_loading = true;
                Task::perform(
                    async move { service.fetch_stations().await.map_err(|e| e.to_string()) },
                    Message::StationsLoaded,
                )
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
                self.stations_loading = false;
                Task::none()
            }
        }
    }
}
