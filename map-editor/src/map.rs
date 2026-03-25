use loom_client::client::map::MapClient;
use loom_core::map::{MapElement, Point, door::Door, seat::Seat};
use std::sync::Arc;

use iced::{
    Border, Color, Element, Length, Task,
    alignment::{Horizontal, Vertical},
    border,
    futures::FutureExt,
    widget::{
        button, column, container, row,
        rule::{self, Style},
        space, stack, text,
    },
};
use loom_client::client::Client;
use loom_map::MapMode;

#[derive(Debug)]
pub struct Map {
    client: Arc<Client>,
    map_id: i32,
    map_mode: MapMode,
    map: loom_map::Map,
    is_colapsed: bool,
    error: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    ToggleHud,
    Map(loom_map::Message),
    ToggleMapMode,
    FetchMap,
    MapFetched(Result<Vec<MapElement>, String>),
    UpdateMap,
    MapUpdated(Option<String>),
    ClearError,
}

impl Map {
    pub fn new(client: Arc<Client>, map_id: i32) -> (Self, Task<Message>) {
        (
            Self {
                client,
                map_id,
                map_mode: MapMode::default(),
                map: loom_map::Map::new(Vec::new()),
                is_colapsed: true,
                error: None,
            },
            Task::done(Message::FetchMap),
        )
    }
    pub fn view(&self) -> Element<'_, Message> {
        stack![
            self.map.view(self.map_mode.clone()).map(Message::Map),
            self.view_hud(),
            self.view_error_banner()
        ]
        .into()
    }

    fn view_hud(&self) -> Element<'_, Message> {
        let hud_style = |_: &iced::Theme| container::Style {
            background: Some(Color::from_rgb(0.15, 0.15, 0.15).into()),
            text_color: Some(Color::WHITE),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                width: 1.0,
                radius: border::radius(8),
            },
            ..Default::default()
        };

        let content = if self.is_colapsed {
            column![self.view_hud_toggle_button()]
        } else {
            column![
                row![
                    text("Menu").size(16),
                    space().width(Length::Fill),
                    self.view_hud_toggle_button()
                ]
                .align_y(Vertical::Center),
                rule::horizontal(1).style(|t| {
                    Style {
                        color: Color::WHITE,
                        ..rule::default(t)
                    }
                }),
                self.view_edit_mode_toggle(),
                if matches!(self.map_mode, MapMode::Edit) {
                    // Action Buttons
                    column![
                        rule::horizontal(1).style(|t| {
                            Style {
                                color: Color::WHITE,
                                ..rule::default(t)
                            }
                        }),
                        self.view_hud_button(
                            "Delete Selected",
                            Color::from_rgb(1.0, 0.3, 0.3),
                            Message::Map(loom_map::Message::DeleteSelection)
                        ),
                        self.view_hud_button(
                            "Clear Selection",
                            Color::from_rgb(1.0, 0.8, 0.2),
                            Message::Map(loom_map::Message::ClearSelection)
                        ),
                        self.view_hud_button(
                            "Duplicate Selection",
                            Color::from_rgb(0.2, 0.8, 1.0),
                            Message::Map(loom_map::Message::DuplicateSelection)
                        ),
                        self.view_hud_button(
                            "Rotate Selection",
                            Color::from_rgb(0.2, 0.8, 1.0),
                            Message::Map(loom_map::Message::RotateSelection)
                        ),
                        rule::horizontal(1).style(|t| {
                            Style {
                                color: Color::WHITE,
                                ..rule::default(t)
                            }
                        }),
                        self.view_hud_button(
                            "New Door",
                            Color::from_rgb(0.0, 1.0, 0.0),
                            Message::Map(loom_map::Message::AddElement(|point| {
                                Door::new(
                                    Point {
                                        x: point.x,
                                        y: point.y,
                                    },
                                    None,
                                )
                                .into()
                            }))
                        ),
                        self.view_hud_button(
                            "New Seat",
                            Color::from_rgb(0.0, 1.0, 0.0),
                            Message::Map(loom_map::Message::AddElement(|point| {
                                Seat::new(
                                    Point {
                                        x: point.x,
                                        y: point.y,
                                    },
                                    None,
                                    None,
                                )
                                .into()
                            }))
                        ),
                        rule::horizontal(1).style(|t| {
                            Style {
                                color: Color::WHITE,
                                ..rule::default(t)
                            }
                        }),
                        self.view_hud_button(
                            "Save",
                            Color::from_rgb(0.0, 1.0, 0.0),
                            Message::UpdateMap
                        ),
                    ]
                    .spacing(5)
                } else {
                    column![]
                },
            ]
            .spacing(10)
        };

        let hud_box = container(content)
            .width(if self.is_colapsed {
                Length::Shrink
            } else {
                220.0.into()
            })
            .padding(15)
            .style(hud_style);

        container(hud_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Top)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let client = self.client.clone();
        let map_id = self.map_id;
        match message {
            Message::ToggleHud => self.is_colapsed = !self.is_colapsed,
            Message::Map(message) => {
                return self.map.update(message).map(Message::Map);
            }
            Message::ToggleMapMode => match self.map_mode {
                MapMode::View => self.map_mode = MapMode::Edit,
                MapMode::Edit => {
                    self.map_mode = MapMode::View;
                    return Task::done(Message::Map(loom_map::Message::ClearSelection));
                }
            },
            Message::FetchMap => {
                return Task::perform(
                    async move {
                        let elements = client
                            .get_map_elements(map_id)
                            .boxed_local()
                            .await
                            .map_err(|e| e.to_string())?;

                        Ok(elements)
                    },
                    Message::MapFetched,
                );
            }
            Message::MapFetched(result) => match result {
                Ok(elements) => {
                    self.error = None;
                    self.map.update_elements(elements);
                }
                Err(error) => self.error = Some(error),
            },
            Message::UpdateMap => {
                let (deleted, updated) = self.map.get_changes();
                return Task::perform(
                    async move {
                        let result = client
                            .update_map(map_id, deleted, updated)
                            .boxed_local()
                            .await;
                        match result {
                            Ok(_) => None,
                            Err(e) => Some(e.to_string()),
                        }
                    },
                    Message::MapUpdated,
                );
            }
            Message::MapUpdated(error) => {
                if error.is_some() {
                    self.error = error
                } else {
                    self.error = None;
                    return Task::done(Message::FetchMap);
                }
            }
            Message::ClearError => self.error = None,
        }
        Task::none()
    }

    fn view_error_banner(&self) -> Element<'_, Message> {
        if let Some(error_msg) = &self.error {
            let banner_style = |_: &iced::Theme| container::Style {
                background: Some(Color::from_rgba(0.8, 0.1, 0.1, 0.8).into()), // 80% Red
                text_color: Some(Color::WHITE),
                border: Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                    width: 1.0,
                    radius: border::radius(0),
                },
                ..Default::default()
            };

            container(
                row![
                    text(format!("Error: {}", error_msg))
                        .size(14)
                        .width(Length::Fill),
                    button("Close")
                        .padding([2, 8])
                        .on_press(Message::ClearError)
                        .style(|_, _| button::Style {
                            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.1).into()),
                            text_color: Color::WHITE,
                            ..Default::default()
                        })
                ]
                .spacing(20)
                .align_y(Vertical::Center),
            )
            .width(Length::Fill)
            .padding(10)
            .style(banner_style)
            .into()
        } else {
            space().width(0).height(0).into()
        }
    }

    fn view_edit_mode_toggle(&self) -> Element<'_, Message> {
        let (label, color) = match self.map_mode {
            MapMode::Edit => ("Mode: Editing", Color::from_rgb(0.0, 1.0, 0.0)),
            MapMode::View => ("Mode: Viewing", Color::from_rgb(0.7, 0.7, 0.7)),
        };
        self.view_hud_button(label, color, Message::ToggleMapMode)
    }

    fn view_hud_toggle_button(&self) -> Element<'_, Message> {
        let ghost_button = |_: &iced::Theme, _: button::Status| button::Style {
            background: None,
            border: Border::default(),
            text_color: Color::WHITE,
            ..Default::default()
        };

        let toggle_label = if self.is_colapsed { "<" } else { "v" };

        button(text(toggle_label).size(16))
            .on_press(Message::ToggleHud)
            .style(ghost_button)
            .into()
    }

    fn view_hud_button<'a>(
        &self,
        label: &'a str,
        color: Color,
        msg: Message,
    ) -> Element<'a, Message> {
        button(
            container(text(label).size(14))
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(msg)
        .style(move |_, _| button::Style {
            background: Some(Color::from_rgba(color.r, color.g, color.b, 0.1).into()),
            text_color: color,
            border: Border {
                color: Color::from_rgba(color.r, color.g, color.b, 0.4),
                width: 1.0,
                radius: border::radius(4),
            },
            ..Default::default()
        })
        .into()
    }
}
