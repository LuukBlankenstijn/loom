use std::time::Duration;

use chrono::{DateTime, Local, Timelike, Utc};
use iced::{
    Color, Element, Font, Length, Subscription, Task,
    font::Weight,
    futures,
    widget::{container, text},
};

use crate::ui::Message;
#[derive(Debug, Default)]
pub struct Countdown {
    start_time: Option<DateTime<Local>>,
}

#[derive(Debug, Clone)]
pub enum CountdownMessage {
    SetStartTime(DateTime<Local>),
    Tick,
    Start,
}

impl From<CountdownMessage> for Message {
    fn from(value: CountdownMessage) -> Self {
        Message::Countdown(value)
    }
}

impl Countdown {
    pub fn view(&self) -> Option<Element<'_, CountdownMessage>> {
        let start_time = self.start_time?;
        let seconds = (start_time - Local::now()).num_seconds();

        if (1..=10).contains(&seconds) {
            Some(
                container(
                    text(seconds.to_string())
                        .size(80)
                        .font(Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        })
                        .color(Color::WHITE),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
            )
        } else {
            None
        }
    }

    pub fn update(&mut self, msg: CountdownMessage) -> Task<CountdownMessage> {
        match msg {
            CountdownMessage::SetStartTime(date_time) => self.start_time = Some(date_time),
            CountdownMessage::Tick => {
                let now = Local::now();
                if let Some(start_time) = self.start_time
                    && now >= start_time
                    && now
                        .checked_add_signed(chrono::Duration::seconds(1))
                        .expect("Now +1 minute is out of range")
                        < start_time
                {
                    return Task::done(CountdownMessage::Start);
                }
            }
            _ => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<CountdownMessage> {
        if let Some(start_time) = self.start_time {
            every_second_until(start_time.to_utc()).map(|_| CountdownMessage::Tick)
        } else {
            Subscription::none()
        }
    }
}

pub fn every_second_until(target: DateTime<Utc>) -> Subscription<()> {
    Subscription::run_with(target, |target| {
        let target = *target;
        futures::stream::unfold((), move |_| async move {
            let now = Utc::now();

            if now >= target {
                return None;
            }

            let next_second = now
                .with_nanosecond(0)
                .unwrap()
                .checked_add_signed(chrono::Duration::seconds(1))
                .unwrap();

            let duration_until_next = (next_second - now)
                .to_std()
                .unwrap_or(Duration::from_millis(1));

            tokio::time::sleep(duration_until_next).await;

            Some(((), ()))
        })
    })
}
