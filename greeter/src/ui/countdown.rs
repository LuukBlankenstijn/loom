use chrono::{DateTime, Local};
use iced::{
    Color, Element, Font, Length, Subscription, Task,
    alignment::{Horizontal, Vertical},
    font::Weight,
    widget::{container, text, tooltip},
    window,
};

use crate::ui::Message;
#[derive(Debug, Default)]
pub struct Countdown {
    start_time: Option<DateTime<Local>>,
    now: DateTime<Local>,
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

pub type IndicatorBuilder<'a, Message> = Box<dyn Fn(bool) -> Element<'a, Message> + 'a>;

impl Countdown {
    pub fn view<'a>(
        &'a self,
    ) -> (
        Option<Element<'a, CountdownMessage>>,
        Option<IndicatorBuilder<'a, CountdownMessage>>,
    ) {
        let Some(start_time) = self.start_time else {
            return (None, None);
        };

        let remaining = start_time - self.now;
        let ms_total = remaining.num_milliseconds();

        let main_label = if ms_total > 0 && ms_total <= 10000 {
            let pulse = (ms_total as f32 * std::f32::consts::PI / 1000.0)
                .sin()
                .abs();
            let dynamic_size = 80.0 + (pulse * 20.0);

            // We calculate display seconds as floor(ms / 1000)
            // but for a countdown, showing "0.x" is usually preferred at the end.
            let display_secs = ms_total / 1000;
            let display_ms = (ms_total % 1000) / 10;
            let display_text = format!("{}.{:02}", display_secs, display_ms);

            Some(
                container(
                    text(display_text)
                        .size(dynamic_size)
                        .font(Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        })
                        .color(Color::WHITE),
                )
                .center(Length::Fill)
                .into(),
            )
        } else {
            None
        };

        let time_format = if start_time.date_naive() == self.now.date_naive() {
            "%H:%M:%S"
        } else {
            "%d/%m/%Y %H:%M:%S"
        };

        let indicator_fn = move |show_tooltip: bool| {
            let mut element: Element<'a, _, _, _> =
                container(text("●").color(Color::from_rgb(0.0, 1.0, 0.0)))
                    .padding(10)
                    .into();

            if show_tooltip {
                element = tooltip(
                    element,
                    text(format!("Starts at: {}", start_time.format(time_format))),
                    tooltip::Position::Bottom,
                )
                .into()
            }

            container(element)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Right)
                .align_y(Vertical::Top)
                .into()
        };

        (main_label, Some(Box::new(indicator_fn)))
    }

    pub fn update(&mut self, msg: CountdownMessage) -> Task<CountdownMessage> {
        match msg {
            CountdownMessage::SetStartTime(date_time) => self.start_time = Some(date_time),
            CountdownMessage::Tick => {
                self.now = Local::now();
                if let Some(start_time) = self.start_time
                    && self.now >= start_time
                {
                    self.start_time = None;
                    return Task::done(CountdownMessage::Start);
                }
            }
            _ => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<CountdownMessage> {
        if self.start_time.is_some() {
            window::frames().map(|_| CountdownMessage::Tick)
        } else {
            Subscription::none()
        }
    }
}
