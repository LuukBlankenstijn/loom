use std::sync::Arc;

use iced::widget::{
    button, center, column, combo_box, container, mouse_area, opaque, row, space, text,
};
use iced::{Color, Element, Length, Task, Theme, alignment};

use crate::service::{AdminService, Station, Team};

#[derive(Debug, Default)]
pub struct Modal {
    selected_team: Option<Team>,
    teams_state: combo_box::State<Team>,

    selected_station: Option<Station>,
    stations_state: combo_box::State<Station>,

    error: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    // message when closing without doing anything
    Cancel,
    // message when closing after updating a team
    Close,
    TeamSelected(Team),
    StationSelected(Station),
    SetError(Option<String>),
    Submit,
}

impl Modal {
    pub fn new(
        team_id: Option<String>,
        station_id: Option<i32>,
        teams: Option<Vec<Team>>,
        stations: Option<Vec<Station>>,
    ) -> Self {
        let team =
            team_id.and_then(|id| teams.as_ref()?.iter().find(|team| team.id == id).cloned());
        let station = station_id.and_then(|id| {
            stations
                .as_ref()?
                .iter()
                .find(|station| station.id == id)
                .cloned()
        });
        Self {
            selected_team: team,
            selected_station: station,
            teams_state: combo_box::State::new(
                teams
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|t| t.ip.is_none())
                    .collect(),
            ),
            stations_state: combo_box::State::new(stations.unwrap_or_default()),
            error: None,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sumbit_enabled = self.selected_team.is_some() && self.selected_station.is_some();
        opaque(
            mouse_area(
                center(opaque(
                    container(
                        column![
                            // header
                            row![
                                text("Assign").size(20).width(Length::Fill),
                                button(text("✕"))
                                    .style(button::secondary)
                                    .on_press(Message::Cancel)
                            ]
                            .align_y(alignment::Vertical::Center),
                            space::vertical(),
                            // body
                            column![
                                combo_box(
                                    &self.stations_state,
                                    "Select a station",
                                    self.selected_station.as_ref(),
                                    Message::StationSelected
                                ),
                                combo_box(
                                    &self.teams_state,
                                    "Select a team",
                                    self.selected_team.as_ref(),
                                    Message::TeamSelected
                                ),
                                if let Some(message) = self.error.clone() {
                                    text(message).style(|theme: &Theme| {
                                        let palette = theme.extended_palette();
                                        text::Style {
                                            color: Some(palette.danger.base.color),
                                        }
                                    })
                                } else {
                                    text("")
                                }
                            ]
                            .spacing(15),
                            // footer
                            row![
                                space::horizontal(),
                                button("Cancel")
                                    .style(button::secondary)
                                    .on_press(Message::Cancel),
                                button("Submit")
                                    .style(button::primary)
                                    .on_press_maybe(sumbit_enabled.then_some(Message::Submit))
                            ]
                            .spacing(10)
                        ]
                        .spacing(20),
                    )
                    .width(400)
                    .height(Length::Shrink)
                    .padding(25)
                    .style(container::rounded_box),
                ))
                .style(|_theme| container::Style {
                    background: Some(
                        Color {
                            a: 0.5,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..Default::default()
                }),
            )
            .on_press(Message::Cancel),
        )
    }

    pub fn update(&mut self, message: Message, service: Arc<dyn AdminService>) -> Task<Message> {
        let service = service.clone();
        match message {
            Message::Submit => {
                if let Some(station) = self.selected_station.clone()
                    && let Some(team) = self.selected_team.clone()
                {
                    self.error = None;
                    return Task::perform(
                        async move {
                            service
                                .set_ip(team.id, Some(station.ip))
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |r| match r {
                            Ok(_) => Message::Close,
                            Err(e) => Message::SetError(Some(e)),
                        },
                    );
                }
            }
            Message::TeamSelected(team) => self.selected_team = Some(team),
            Message::StationSelected(station) => self.selected_station = Some(station),
            Message::SetError(e) => self.error = e,
            _ => {
                //handled by parent
            }
        };
        Task::none()
    }
}
