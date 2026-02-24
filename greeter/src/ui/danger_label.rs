use std::time::Duration;

use iced::{
    Alignment, Color, Element, Font, Length, Subscription, Task, Theme,
    font::Weight,
    keyboard::{self, Event},
    task::Handle,
    widget::{column, container, text},
};

#[derive(Debug, Default)]
pub struct DangerLabel {
    handle: Option<Handle>,
}

#[derive(Debug, Clone)]
pub enum DangerLabelMessage {
    ShowLabel,
    HideLabel,
}

impl DangerLabel {
    pub fn view(&self) -> Option<Element<'_, DangerLabelMessage>> {
        self.handle.is_some().then(|| {
            column![
                iced::widget::space::vertical().height(Length::FillPortion(8)),
                container(
                    text("Do not touch this machine")
                        .color(Color::WHITE)
                        .size(32)
                        .font(Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        })
                        .center()
                )
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .height(Length::FillPortion(2))
                .style(|_: &Theme| Color::from_rgb(1.0, 0.0, 0.0).into()),
            ]
            .into()
        })
    }

    pub fn update(&mut self, message: DangerLabelMessage) -> Task<DangerLabelMessage> {
        match message {
            DangerLabelMessage::ShowLabel => {
                if let Some(handle) = self.handle.take() {
                    handle.abort();
                }
                let (task, handle) = Task::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    },
                    |_| DangerLabelMessage::HideLabel,
                )
                .abortable();
                self.handle = Some(handle);
                return task;
            }
            DangerLabelMessage::HideLabel => {
                self.handle = None;
            }
        }
        Task::none()
    }

    pub fn subscribe(&self) -> Subscription<DangerLabelMessage> {
        keyboard::listen().filter_map(|message| {
            if matches!(message, Event::KeyPressed { .. }) {
                return DangerLabelMessage::ShowLabel.into();
            }
            None
        })
    }
}
