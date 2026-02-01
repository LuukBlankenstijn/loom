use iced::{
    Alignment, Border, Color, Element, Font, Length, Shadow, Theme, Vector,
    alignment::Vertical,
    border,
    widget::{
        self, button, container, row, scrollable,
        table::{self, column},
        text, tooltip,
    },
};

use crate::{service::Team, ui::body::Message};

pub fn view_teams(teams: Option<Vec<Team>>) -> Element<'static, Message> {
    if let Some(teams) = teams {
        container(widget::column![
            container(scrollable(
                table::table(
                    vec![
                        column(
                            text("ID").size(18).font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            |t: Team| { text(t.id).size(16) }
                        )
                        .width(Length::Fixed(80.0)),
                        column(
                            text("IP address").size(18).font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            |t: Team| -> Element<'_, Message> {
                                match t.ip {
                                    Some(ip) => row![
                                        text(ip).font(Font::MONOSPACE).size(16),
                                        tooltip(
                                            button("󰅚")
                                                .style(delete_ip_button)
                                                .on_press(Message::Unassign(t.id.clone())),
                                            "Unassign ip",
                                            tooltip::Position::Top
                                        )
                                    ]
                                    .align_y(Vertical::Center)
                                    .spacing(10)
                                    .into(),
                                    None => button(text("Assign IP address").size(16))
                                        .on_press(Message::OpenModal {
                                            station_id: None,
                                            team_id: Some(t.id),
                                        })
                                        .into(),
                                }
                            }
                        )
                        .width(Length::Fill),
                        column(
                            text("Name").size(18).font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            |t: Team| { text(t.name).size(16) }
                        )
                        .width(Length::Fixed(200.0)),
                    ],
                    teams,
                )
                .padding_y(15)
                .padding_x(20)
                .separator(1.0)
            ))
            .style(table_card)
            .width(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40)
        .align_x(Alignment::Center)
        .into()
    } else {
        container(text("No stations found").size(24))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(100)
            .into()
    }
}

pub fn table_card(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.base.color.into()),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color {
                a: 0.5,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

pub fn delete_ip_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base_style = button::Style {
        border: border::width(0.0),
        text_color: palette.danger.base.color,
        ..Default::default()
    };

    match status {
        button::Status::Hovered | button::Status::Pressed => button::Style {
            text_color: palette.danger.strong.color,
            ..base_style
        },
        button::Status::Disabled => button::Style {
            text_color: palette.danger.weak.color,
            ..base_style
        },
        _ => base_style,
    }
}
