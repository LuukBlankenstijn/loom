pub mod background;
pub mod countdown;
pub mod form;

use anyhow::Result;
use iced::{
    Element, Subscription, Task, Theme,
    widget::{space, stack},
};

use crate::{
    conf::Conf,
    ipc::greeter_client::{GreeterClient, GreeterClientMessage},
    subscriptions::{
        api_poller::{ApiPoller, ApiPollerMessage},
        dbus::{DbusMessage, dbus_service_subscription},
        key_listener::{KeyListener, KeyListenerMessage},
    },
    ui::{
        background::{Background, BackgroundMessage},
        countdown::{Countdown, CountdownMessage},
        form::{Form, FormMessage},
    },
};

pub struct Greeter {
    // ui
    background: Background,
    form: Form,
    countdown: Countdown,

    // subscriptions
    key_listener: KeyListener,
    api_poller: ApiPoller,

    greeter_client: GreeterClient,

    config: Conf,
}

pub enum Message {
    Background(BackgroundMessage),
    Form(FormMessage),
    KeyListener(KeyListenerMessage),
    GreeterClient(GreeterClientMessage),
    ApiPoller(ApiPollerMessage),
    Countdown(CountdownMessage),
    Dbus(DbusMessage),
}

impl Greeter {
    pub fn new(config: Conf) -> (Self, Task<Message>) {
        let (background, background_task) = Background::new(
            config.background_source.clone(),
            config.background_label.clone(),
            config.background_label_color.clone(),
        );
        let form = Form::new();
        let countdown = Countdown::default();

        let key_listener = KeyListener::new(config.chain.clone());
        let api_poller = ApiPoller::new(config.url.clone());

        let greeter_client = GreeterClient::new(
            config.session.clone(),
            config.username.clone(),
            config.password.clone(),
        );
        (
            Self {
                background,
                form,
                countdown,
                key_listener,
                api_poller,
                greeter_client,
                config,
            },
            background_task.map(Message::Background),
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let (background, background_label) = self.background.view();

        let countdown_overlay = self.countdown.view().map(|c| c.map(Message::Countdown));

        let overlay = countdown_overlay
            .or_else(|| background_label.map(|b| b.map(Message::Background)))
            .unwrap_or_else(|| space().into());

        stack![
            background.map(Message::Background),
            overlay,
            self.form.view().map(Message::Form)
        ]
        .into()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Background(background_message) => self
                .background
                .update(background_message)
                .map(Message::Background),
            Message::Form(form_message) => {
                if let FormMessage::LoginWithCredentials(username, password) = form_message {
                    return Task::done(
                        GreeterClientMessage::LoginWithCredentials(username, password).into(),
                    );
                }
                self.form.update(form_message).map(Message::Form)
            }
            Message::KeyListener(msg) => {
                if let KeyListenerMessage::ChainTriggered = msg {
                    return Task::done(FormMessage::ToggleVisible.into());
                }
                self.key_listener.update(msg).map(Message::KeyListener)
            }
            Message::GreeterClient(msg) => match msg {
                GreeterClientMessage::LoginError(error) => {
                    Task::done(FormMessage::SetError(error).into())
                }
                _ => self.greeter_client.update(msg).map(Message::GreeterClient),
            },
            Message::ApiPoller(api_poller_message) => {
                if let ApiPollerMessage::SetStartime(starttime) = api_poller_message {
                    return Task::done(CountdownMessage::SetStartTime(starttime).into());
                }
                self.api_poller
                    .update(api_poller_message)
                    .map(Message::ApiPoller)
            }
            Message::Countdown(countdown_message) => {
                if let CountdownMessage::Start = countdown_message {
                    return Task::done(GreeterClientMessage::Login.into());
                }
                self.countdown
                    .update(countdown_message)
                    .map(Message::Countdown)
            }
            Message::Dbus(dbus_message) => match dbus_message {
                DbusMessage::SetWallpaper(source) => {
                    Task::done(BackgroundMessage::SetSource(Some(source)).into())
                }
                DbusMessage::Login => Task::done(GreeterClientMessage::Login.into()),
                DbusMessage::SetApiUrl(url) => {
                    Task::done(ApiPollerMessage::SetUrl(Some(url)).into())
                }
            },
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            self.key_listener.subscription().map(Message::KeyListener),
            self.form.subscription().map(Message::Form),
            self.api_poller.subscription().map(Message::ApiPoller),
            self.countdown.subscription().map(Message::Countdown),
        ];
        if self.config.enable_dbus {
            subscriptions.push(dbus_service_subscription().map(Message::Dbus));
        }
        Subscription::batch(subscriptions)
    }

    fn style(&self) -> Theme {
        iced::Theme::TokyoNightStorm
    }
}

pub fn run_greeter(config: Conf) -> Result<()> {
    iced::application(
        move || Greeter::new(config.clone()),
        Greeter::update,
        Greeter::view,
    )
    .subscription(Greeter::subscription)
    .theme(Greeter::style)
    .run()?;
    Ok(())
}
