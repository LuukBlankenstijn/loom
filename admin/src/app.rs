use std::sync::Arc;

use crate::{
    service::AdminService,
    ui::body::{self, Body},
    ui::header::{self, Header},
};
use anyhow::Result;
use iced::{
    Element, Task, Theme,
    widget::column,
    window::{Settings, settings::PlatformSpecific},
};

#[derive(Debug)]
struct AdminApp {
    service: Arc<dyn AdminService>,
    body: Body,
    header: Header,
}

#[derive(Debug, Clone)]
pub enum Message {
    Body(body::Message),
    Header(header::Message),
}

impl AdminApp {
    fn new(service: Arc<dyn AdminService>) -> (Self, Task<Message>) {
        let (body, body_task) = Body::new();
        let (header, header_task) = Header::new();
        let state = Self {
            service: service.clone(),
            body,
            header,
        };

        // load inital data
        let task = Task::batch(vec![
            body_task.map(Message::Body),
            header_task.map(Message::Header),
        ]);

        (state, task)
    }

    fn view(&self) -> Element<'_, Message> {
        let header = self.header.view().map(Message::Header);
        let body = self
            .body
            .view(self.header.get_active_tab())
            .map(Message::Body);
        column![header, body].into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let service = self.service.clone();
        match message {
            Message::Body(message) => self.body.update(message, service).map(Message::Body),
            Message::Header(message) => self.header.udpate(message, service).map(Message::Header),
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        self.header.subscription().map(Message::Header)
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
    .window(Settings {
        platform_specific: PlatformSpecific {
            application_id: String::from("nl.luukblankenstijn.loom-admin"),
            ..Default::default()
        },
        ..Default::default()
    })
    .run()?;
    Ok(())
}
