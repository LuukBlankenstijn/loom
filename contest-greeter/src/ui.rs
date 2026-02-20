pub mod background;
pub mod countdown;
pub mod form;

use anyhow::Result;
use iced::{Element, Subscription, Task, Theme, widget::Stack};

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
        let (api_poller, api_poller_task) = ApiPoller::new(config.url.clone());

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
            Task::batch(vec![
                background_task.map(Message::Background),
                api_poller_task.map(Message::ApiPoller),
            ]),
        )
    }

    pub fn view(&self) -> Element<'_, Message> {
        let (background, background_label) = self.background.view();
        let (countdown_label, countdown_indicator_fn) = self.countdown.view();
        let form_element = self.form.view();

        let mut layers = vec![background.map(Message::Background)];

        // Add the central label (Countdown takes priority over Background Label)
        if let Some(c_label) = countdown_label {
            layers.push(c_label.map(Message::Countdown));
        } else if let Some(b_label) = background_label {
            layers.push(b_label.map(Message::Background));
        }

        // if there is a generator for the countdown label, add it
        // whether or not the form label is visible decides if the starttime is shown
        if let Some(countdown_indicator_fn) = countdown_indicator_fn {
            layers.push(countdown_indicator_fn(form_element.is_some()).map(Message::Countdown));
        }

        if let Some(f) = form_element {
            layers.push(f.map(Message::Form));
        }

        Stack::with_children(layers).into()
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
