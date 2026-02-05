use iced::{
    Border, Color, Element, Length, Point, Task,
    alignment::{Horizontal, Vertical},
    border,
    widget::{
        button, column, container, row,
        rule::{self, Style},
        space, stack, text,
    },
};
use loom_map::{Door, MapElement, MapMode, Wall};

#[derive(Debug)]
pub struct Map {
    map_mode: MapMode,
    map: loom_map::Map,
    is_colapsed: bool,
}

impl Default for Map {
    fn default() -> Self {
        let elements = get_doors()
            .into_iter()
            .map(MapElement::Door)
            .chain(get_walls().into_iter().map(MapElement::Wall))
            .collect();
        Self {
            map: loom_map::Map::new(elements),
            map_mode: Default::default(),
            is_colapsed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    ToggleHud,
    Map(loom_map::Message),
    ToggleMapMode,
}

impl Map {
    pub fn view(&self) -> Element<'_, Message> {
        stack![
            self.map.view(self.map_mode.clone()).map(Message::Map),
            self.view_hud()
        ]
        .into()
    }

    fn view_hud(&self) -> Element<'_, Message> {
        let hud_style = |_: &iced::Theme| container::Style {
            background: Some(Color::from_rgba(0.95, 0.95, 0.95, 0.05).into()),
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
                        space().height(1),
                        self.view_hud_button(
                            "Clear Selection",
                            Color::from_rgb(1.0, 0.8, 0.2),
                            Message::Map(loom_map::Message::ClearSelection)
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
        match message {
            Message::ToggleHud => self.is_colapsed = !self.is_colapsed,
            Message::Map(message) => {
                return self.map.update(message).map(Message::Map);
            }
            Message::ToggleMapMode => match self.map_mode {
                MapMode::View => self.map_mode = MapMode::Edit,
                MapMode::Edit => self.map_mode = MapMode::View,
            },
        }
        Task::none()
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

        let toggle_label = if self.is_colapsed { "◀" } else { "▼" };

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

fn get_doors() -> Vec<Door> {
    vec![
        Door::new(Point::new(0.0, 0.0), false),
        Door::new(Point::new(200.0, 200.0), true),
    ]
}

fn get_walls() -> Vec<Wall> {
    vec![
        Wall::new(Point::new(50.0, 0.0), Point::new(200.0, 0.0)),
        Wall::new(Point::new(200.0, 0.0), Point::new(200.0, 150.0)),
    ]
}
