use std::collections::HashMap;

use iced::{
    Alignment, Border, Color, Element, Font, Length, Shadow, Theme, Vector,
    widget::{
        self, container, row, scrollable,
        table::{self, column},
        text,
    },
};

#[derive(Clone, Debug)]
struct ExtendedStation {
    base: Station,
    team_name: Option<String>,
}

use crate::{service::Station, ui::body::Message};

pub fn view_stations(
    stations: Option<Vec<Station>>,
    ip_team_name_map: HashMap<String, String>,
) -> Element<'static, Message> {
    if let Some(mut stations) = stations {
        container(widget::column![
            container(scrollable(
                table::table(
                    vec![
                        column(
                            text("ID").size(18).font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            |s: ExtendedStation| { text(s.base.id).size(16) }
                        )
                        .width(Length::Fixed(80.0)),
                        column(
                            text("IP Address").size(18).font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            |s: ExtendedStation| { text(s.base.ip).font(Font::MONOSPACE).size(16) }
                        )
                        .width(Length::Fill),
                        column(
                            text("Team").size(18).font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            |s: ExtendedStation| {
                                if let Some(team_name) = s.team_name {
                                    text(team_name).size(16)
                                } else {
                                    text("No team assigned").size(16)
                                }
                            }
                        )
                        .width(Length::Fill),
                        column(
                            text("Status").size(18).font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            |s: ExtendedStation| {
                                let connected = s
                                    .base
                                    .disconnected_at
                                    .map(|d| s.base.connected_at > d)
                                    .unwrap_or(true);

                                row![
                                    text(if connected { "●" } else { "○" }).size(20).style(
                                        move |theme: &Theme| {
                                            let palette = theme.extended_palette();
                                            text::Style {
                                                color: Some(if connected {
                                                    palette.success.base.color
                                                } else {
                                                    palette.danger.base.color
                                                }),
                                            }
                                        }
                                    ),
                                    text(if connected {
                                        " Connected"
                                    } else {
                                        " Disconnected"
                                    })
                                    .size(16)
                                ]
                                .align_y(Alignment::Center)
                            }
                        )
                        .width(Length::Fixed(200.0)),
                    ],
                    stations
                        .iter_mut()
                        .map(|original| ExtendedStation {
                            base: original.clone(),
                            team_name: ip_team_name_map.get(&original.ip).cloned(),
                        })
                        .collect::<Vec<ExtendedStation>>(),
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
