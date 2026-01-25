use crate::app::Message;
use crate::service::Contest;
use chrono::Duration;
use iced::widget::space::horizontal;
use iced::widget::{button, container, row, text};
use iced::{Background, Color, Element, Font, Length, Shadow, Vector, alignment};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Stations,
    Teams,
}

pub fn view(
    contest: Option<Contest>,
    time_remaining: Option<Duration>,
) -> Element<'static, Message> {
    let mut nav_bar = row![
        tab_button("Stations", Tab::Stations),
        tab_button("Teams", Tab::Teams),
        horizontal(),
    ]
    .spacing(20);
    if let Some(contest) = contest
        && let Some(time_remaining) = time_remaining
    {
        nav_bar = nav_bar.push(contest_info(contest.name.clone(), time_remaining));
    } else {
        nav_bar = nav_bar.push(text("No contest found").font(Font::MONOSPACE).size(32))
    }

    container(nav_bar)
        .width(Length::Fill)
        .height(Length::Shrink)
        .padding(10)
        .style(header_style)
        .into()
}

fn tab_button(label: &str, tab: Tab) -> Element<'static, Message> {
    button(
        text(label.to_string())
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .height(Length::Fill),
    )
    .on_press(Message::TabChanged(tab))
    .padding([5, 5])
    .height(Length::Fill)
    .width(150)
    .into()
}

fn header_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        shadow: Shadow {
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.25,
            },
            offset: Vector::new(0.0, 6.0),
            blur_radius: 14.0,
        },
        ..container::Style::default()
    }
}

fn contest_info(contest_name: String, time_remaining: Duration) -> Element<'static, Message> {
    let total_seconds = time_remaining.num_seconds();

    let mut label = if total_seconds <= 0 {
        String::from("00:00")
    } else {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let mins = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;

        match (days, hours) {
            (d, _) if d > 0 => format!("{d}d {hours:02}:{mins:02}:{secs:02}"),
            (0, h) if h > 0 => format!("{h:02}:{mins:02}:{secs:02}"),
            _ => format!("{mins:02}:{secs:02}"),
        }
    };

    label.push_str(format!(" | {contest_name}").as_str());

    text(label).font(Font::MONOSPACE).size(32).into()
}
