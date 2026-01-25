use std::sync::Arc;

use crate::{
    service::{AdminService, Contest, Team},
    ui::header::{self, Tab},
};
use anyhow::Result;
use chrono::Utc;
use iced::{
    Element, Length, Task, Theme, time,
    widget::{column, container, text},
};

#[derive(Debug)]
struct AdminApp {
    active_tab: Tab,
    service: Arc<dyn AdminService>,
    contest: Option<Contest>,
    time_remaining: Option<chrono::Duration>,
    teams: Option<Vec<Team>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabChanged(Tab),
    LoadContest,
    ContestLoaded(Result<Option<Contest>, String>),
    LoadTeams,
    TeamsLoaded(Result<Vec<Team>, String>),
    Tick,
}

impl AdminApp {
    fn new(service: Arc<dyn AdminService>) -> (Self, Task<Message>) {
        let state = Self {
            service: service.clone(),
            active_tab: Tab::Stations,
            contest: None,
            time_remaining: None,
            teams: None,
        };

        // load inital data
        let task = Task::batch(vec![
            Task::done(Message::LoadContest),
            Task::done(Message::LoadTeams),
        ]);

        (state, task)
    }

    fn view(&self) -> Element<'_, Message> {
        let header = header::view(self.contest.clone(), self.time_remaining);
        let label = if self.active_tab == Tab::Stations {
            String::from("stations")
        } else if let Some(teams) = self.teams.clone() {
            format!("teams: {:?}", teams)
        } else {
            String::from("No teams loaded")
        };
        let body = container(text(label)).height(Length::Fill);
        column![header, body].into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabChanged(tab) => {
                self.active_tab = tab;
                if tab == Tab::Stations {
                    Task::none()
                } else {
                    Task::done(Message::LoadTeams)
                }
            }
            Message::LoadContest => {
                let service = self.service.clone();
                Task::perform(
                    async move { service.fetch_contest().await.map_err(|e| e.to_string()) },
                    Message::ContestLoaded,
                )
            }
            Message::ContestLoaded(contest) => match contest {
                Ok(result) => {
                    self.contest = result.clone();
                    if let Some(contest) = result {
                        let now = Utc::now();
                        self.time_remaining = Some(contest.start_time.signed_duration_since(now))
                    }
                    Task::none()
                }
                Err(msg) => {
                    println!("failed to load contest: {msg}");
                    Task::none()
                }
            },
            Message::Tick => {
                if let Some(contest) = &mut self.contest {
                    let now = Utc::now();
                    self.time_remaining = Some(contest.start_time.signed_duration_since(now))
                } else {
                    self.time_remaining = None
                }
                Task::none()
            }
            Message::LoadTeams => {
                let service = self.service.clone();
                Task::perform(
                    async move { service.fetch_teams().await.map_err(|e| e.to_string()) },
                    Message::TeamsLoaded,
                )
            }
            Message::TeamsLoaded(teams) => {
                match teams {
                    Ok(result) => self.teams = Some(result.clone()),
                    Err(msg) => println!("failed to load teams: {msg}"),
                };
                Task::none()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    }

    fn style(&self) -> Theme {
        iced::Theme::TokyoNightStorm
    }
}

pub fn run_app(service: Arc<dyn AdminService>) -> Result<()> {
    iced::application(
        move || AdminApp::new(service.clone()),
        AdminApp::update,
        AdminApp::view,
    )
    .subscription(AdminApp::subscription)
    .theme(AdminApp::style)
    .run()?;
    Ok(())
}
