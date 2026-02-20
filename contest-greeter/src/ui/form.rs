use std::ops::Not;

use iced::keyboard::key::Named;
use iced::widget::operation::focus;
use iced::widget::{Id, button, column, container, text, text_input};
use iced::{
    Background, Border, Color, Event, Shadow, Subscription, Task, Theme, Vector, event, keyboard,
};
use iced::{Element, Length};

use crate::ui::Message;

#[derive(Debug)]
pub struct Form {
    // form state
    username: String,
    username_id: Id,
    password: String,
    password_id: Id,
    error: Option<String>,

    visible: bool,
    is_logging_in: bool,
}

#[derive(Debug, Clone)]
pub enum FormMessage {
    UsernameChanged(String),
    PasswordChanged(String),
    Login,
    LoginWithCredentials(String, String),
    ToggleVisible,
    FocusUsername,
    FocusPassword,
    SetError(String),
}

impl From<FormMessage> for Message {
    fn from(value: FormMessage) -> Self {
        Message::Form(value)
    }
}

impl Form {
    pub fn new() -> Self {
        Self {
            username: Default::default(),
            password: Default::default(),
            username_id: Id::new("username"),
            password_id: Id::new("password"),
            error: Default::default(),

            visible: false,
            is_logging_in: false,
        }
    }

    pub fn view(&self) -> Option<Element<'_, FormMessage>> {
        if !self.visible {
            return None;
        }
        let mut content = column![
            text("Username"),
            text_input("Enter username", &self.username)
                .id(self.username_id.clone())
                .on_input(FormMessage::UsernameChanged)
                .padding(10)
                .style(input_style),
            text("Password"),
            text_input("Enter password", &self.password)
                .id(self.password_id.clone())
                .on_input(FormMessage::PasswordChanged)
                .secure(true)
                .padding(10)
                .style(input_style),
            button(if !self.is_logging_in {
                "Login"
            } else {
                "Logging in..."
            })
            .on_press_maybe(self.is_logging_in.not().then_some(FormMessage::Login))
            .padding(10)
            .style(button_style)
        ]
        .spacing(10)
        .padding(20);

        if let Some(error) = &self.error {
            content = content.push(text(error).color(Color::from_rgb(1.0, 0.4, 0.4)));
        }

        Some(
            container(
                container(content)
                    .center_x(400)
                    .center_y(Length::Shrink)
                    .style(container_style),
            )
            .center(Length::Fill)
            .into(),
        )
    }

    pub fn update(&mut self, msg: FormMessage) -> Task<FormMessage> {
        match msg {
            FormMessage::UsernameChanged(username) => self.username = username,
            FormMessage::PasswordChanged(password) => self.password = password,
            FormMessage::Login => {
                if self.username.is_empty() {
                    self.error = Some(String::from("Username can not be empty"));
                    return Task::none();
                }
                if self.password.is_empty() {
                    self.error = Some(String::from("Password can not be empty"));
                    return Task::none();
                }
                self.error = None;
                self.is_logging_in = true;
                return Task::done(FormMessage::LoginWithCredentials(
                    self.username.clone(),
                    self.password.clone(),
                ));
            }
            FormMessage::ToggleVisible => {
                self.visible = !self.visible;
                if self.visible {
                    return focus(self.username_id.clone());
                }
            }
            FormMessage::FocusPassword => return focus(self.password_id.clone()),
            FormMessage::FocusUsername => return focus(self.username_id.clone()),
            FormMessage::SetError(error) => {
                self.error = Some(error);
                self.is_logging_in = false;
            }
            _ => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<FormMessage> {
        event::listen_with(|event, status, _id| {
            if status == event::Status::Captured {
                return None;
            }

            if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                match key {
                    keyboard::Key::Named(Named::Tab) => {
                        if modifiers.shift() {
                            Some(FormMessage::FocusUsername)
                        } else {
                            Some(FormMessage::FocusPassword)
                        }
                    }
                    keyboard::Key::Named(Named::Enter) => Some(FormMessage::Login),
                    _ => None,
                }
            } else {
                None
            }
        })
    }
}

fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.border.radius = 8.0.into();
    style
}

fn button_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = 8.0.into();
    style
}

fn container_style(theme: &Theme) -> container::Style {
    let color_palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(color_palette.background.strong.color)),
        border: Border {
            radius: 12.0.into(),
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 10.0,
        },
        ..Default::default()
    }
}
