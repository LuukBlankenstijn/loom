use std::sync::Arc;

use chrono::{Duration, Utc};
use iced::{
    Background, Color, Element, Font, Length, Shadow, Task, Vector, alignment, time,
    widget::{button, container, row, space::horizontal, text},
};

use crate::service::{AdminService, Contest};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Stations,
    Teams,
}

#[derive(Debug, Default)]
pub struct Header {
    active_tab: Tab,
    contest: Option<Contest>,
    time_remaining: Option<chrono::Duration>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadContest,
    ContestLoaded(Result<Option<Contest>, String>),
    Tick,
    TabChanged(Tab),
}

impl Header {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::done(Message::LoadContest))
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut nav_bar = row![
            tab_button("Stations", Tab::Stations),
            tab_button("Teams", Tab::Teams),
            horizontal(),
        ]
        .spacing(20);
        if let Some(contest) = self.contest.clone()
            && let Some(time_remaining) = self.time_remaining
        {
            nav_bar = nav_bar.push(contest_info(contest.name.clone(), time_remaining));
        } else {
            nav_bar = nav_bar.push(text("No contest found").font(Font::MONOSPACE).size(32))
        }

        container(nav_bar)
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding(10)
            .style(header_style)
            .into()
    }

    pub fn udpate(&mut self, message: Message, service: Arc<dyn AdminService>) -> Task<Message> {
        match message {
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
            Message::Tick => self.handle_tick(),
            Message::TabChanged(tab) => {
                self.active_tab = tab;
                Task::none()
            }
        }
    }

    fn handle_tick(&mut self) -> Task<Message> {
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

    pub fn subscription(&self) -> iced::Subscription<Message> {
        time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    }

    pub fn get_active_tab(&self) -> Tab {
        self.active_tab
    }
}

fn header_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        shadow: Shadow {
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.25,
            },
            offset: Vector::new(0.0, 6.0),
            blur_radius: 14.0,
        },
        ..container::Style::default()
    }
}

fn tab_button(label: &str, tab: Tab) -> Element<'static, Message> {
    button(
        text(label.to_string())
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .height(Length::Fill),
    )
    .on_press(Message::TabChanged(tab))
    .padding([5, 5])
    .height(Length::Fill)
    .width(150)
    .into()
}

fn contest_info(contest_name: String, time_remaining: Duration) -> Element<'static, Message> {
    let total_seconds = time_remaining.num_seconds();

    let mut label = if total_seconds <= 0 {
        String::from("00:00")
    } else {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let mins = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;

        match (days, hours) {
            (d, _) if d > 0 => format!("{d}d {hours:02}:{mins:02}:{secs:02}"),
            (0, h) if h > 0 => format!("{h:02}:{mins:02}:{secs:02}"),
            _ => format!("{mins:02}:{secs:02}"),
        }
    };

    label.push_str(format!(" | {contest_name}").as_str());

    text(label).font(Font::MONOSPACE).size(32).into()
}
