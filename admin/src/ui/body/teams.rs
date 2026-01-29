use iced::{
    Alignment, Border, Color, Element, Font, Length, Shadow, Theme, Vector,
    widget::{
        self, container, scrollable,
        table::{self, column},
        text,
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
                            text("IP Address").size(18).font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            |t: Team| match t.ip {
                                Some(ip) => text(ip).font(Font::MONOSPACE).size(16),
                                None => text("None"),
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
