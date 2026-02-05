use iced::{
    Border, Color, Element, Length, Point, Task,
    alignment::{Horizontal, Vertical},
    border,
    widget::{button, column, container, row, space, stack, text},
};
use loom_map::{Door, MapElement, Wall};

#[derive(Default, Debug, Clone)]
pub enum MapMode {
    #[default]
    View,
    Edit,
}

#[derive(Debug)]
pub struct Map {
    drawmode: MapMode,
    internal_map: loom_map::Map,
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
            internal_map: loom_map::Map::new(elements),
            drawmode: Default::default(),
            is_colapsed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    ToggleHud,
    Internal(loom_map::Message),
    ToggleMapMode,
}

impl Map {
    pub fn view(&self) -> Element<'_, Message> {
        stack![
            self.internal_map.view().map(Message::Internal),
            self.view_hud()
        ]
        .into()
    }

    fn view_hud(&self) -> Element<'_, Message> {
        let ghost_button = |_: &iced::Theme, _: button::Status| button::Style {
            background: None,
            border: Border::default(),
            text_color: Color::WHITE,
            ..Default::default()
        };

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

        let toggle_label = if self.is_colapsed { "◀" } else { "▼" };

        let toggle_button = button(text(toggle_label).size(16))
            .on_press(Message::ToggleHud)
            .style(ghost_button);

        let content = if self.is_colapsed {
            column![toggle_button]
        } else {
            column![
                row![
                    text("Menu").size(16),
                    space().width(Length::Fill),
                    toggle_button
                ]
                .align_y(Vertical::Center),
                self.view_edit_mode_toggle()
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
            Message::Internal(message) => {
                return self.internal_map.update(message).map(Message::Internal);
            }
            Message::ToggleMapMode => match self.drawmode {
                MapMode::View => self.drawmode = MapMode::Edit,
                MapMode::Edit => self.drawmode = MapMode::View,
            },
        }
        Task::none()
    }

    fn view_edit_mode_toggle(&self) -> Element<'_, Message> {
        let (label, color) = match self.drawmode {
            MapMode::Edit => ("Mode: Editing", Color::from_rgb(0.0, 1.0, 0.0)),
            MapMode::View => ("Mode: Viewing", Color::from_rgb(0.7, 0.7, 0.7)),
        };

        button(
            container(text(label).size(14))
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(Message::ToggleMapMode)
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
