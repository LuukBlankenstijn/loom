use std::sync::Arc;

use crate::{
    service::{AdminService, Contest},
    ui::body::{self, Body},
    ui::header::{self, Tab},
};
use anyhow::Result;
use chrono::Utc;
use iced::{Element, Task, Theme, time, widget::column};

#[derive(Debug)]
struct AdminApp {
    active_tab: Tab,
    service: Arc<dyn AdminService>,
    contest: Option<Contest>,
    time_remaining: Option<chrono::Duration>,
    body: Body,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabChanged(Tab),
    LoadContest,
    ContestLoaded(Result<Option<Contest>, String>),
    Body(body::Message),
    Tick,
}

impl AdminApp {
    fn new(service: Arc<dyn AdminService>) -> (Self, Task<Message>) {
        let (body, body_task) = Body::new();
        let state = Self {
            service: service.clone(),
            active_tab: Tab::Stations,
            contest: None,
            time_remaining: None,
            body,
        };

        // load inital data
        let task = Task::batch(vec![
            Task::done(Message::LoadContest),
            body_task.map(Message::Body),
        ]);

        (state, task)
    }

    fn view(&self) -> Element<'_, Message> {
        let header = header::view(self.contest.clone(), self.time_remaining);
        let body = self.body.view(self.active_tab).map(Message::Body);
        column![header, body].into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let service = self.service.clone();
        match message {
            Message::TabChanged(tab) => {
                self.active_tab = tab;
                Task::none()
            }
            Message::LoadContest => Task::perform(
                async move { service.fetch_contest().await.map_err(|e| e.to_string()) },
                Message::ContestLoaded,
            ),
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
                    self.time_remaining = Some(contest.start_time.signed_duration_since(now));
                    if contest.end_time.lt(&now) {
                        // if contest is over, fetch the next one
                        return Task::done(Message::LoadContest);
                    }
                } else {
                    self.time_remaining = None
                }
                Task::none()
            }
            Message::Body(message) => self.body.update(message, service).map(Message::Body),
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
