use gtk4::Window;
use gtk4::gdk;
use gtk4::glib::idle_add_local;
use gtk4::glib::{ControlFlow, MainLoop};
use gtk4::prelude::*;

mod background;
mod chain_listener;
mod config;
mod countdown;
mod login_ui;
use chain_listener::register_chain_listener;
use lightdm_contest_rs_greeter::CoreName;
use log::info;
use login_ui::LoginUi;
use tokio::sync::mpsc;
use types::GreeterMessage;
use types::SystemBus;
use types::SystemSender;
use types::UiMessage;

use crate::ui::background::Background;
use crate::ui::countdown::CountDown;
pub use config::UiConfig;

pub async fn run_ui(bus: impl SystemBus, conf: UiConfig) {
    gtk4::init().expect("init gtk");
    let (tx, rx) = mpsc::channel::<UiMessage>(16);
    bus.register(CoreName::UI, tx);

    build_ui(bus, rx, conf);

    info!("[UI] running main loop");
    let main_loop = MainLoop::new(None, false);
    main_loop.run();
}

fn build_ui(bus: impl SystemSender, mut rx: mpsc::Receiver<UiMessage>, conf: UiConfig) {
    let window = Window::builder().title("lightdm-contest-greeter").build();

    size_to_first_monitor(&window);
    window.set_decorated(false);

    let background = Background::new();
    if let Some(source) = conf.background_source.clone() {
        background.set_image(&source)
    }
    let background_overlay = background.get_overlay();
    window.set_child(Some(background_overlay));

    let countdown = CountDown::new(conf.clone(), bus.clone());
    let countdown_overlay = countdown.widget();

    let login_ui = build_login_ui(bus);

    background_overlay.add_overlay(countdown_overlay);
    background_overlay.add_overlay(login_ui.widget());
    let login_ui_clone = login_ui.clone();
    register_chain_listener(&window, conf.chain.chars().collect(), {
        let login_ui = login_ui_widget_closure(login_ui_clone);
        move || login_ui()
    });

    idle_add_local(move || {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                UiMessage::SetWallpaper(path_option) => match path_option {
                    Some(path) => {
                        background.set_image(&path.to_string());
                    }
                    None => {
                        background.set_empty();
                    }
                },
                UiMessage::SetError(error) => {
                    login_ui.set_error_text(&error.to_string());
                }
                UiMessage::SetCountdownEndtime { end_time } => {
                    countdown.update_endtime(end_time);
                }
            }
        }
        ControlFlow::Continue
    });

    window.present();
    window.fullscreen();
}

fn build_login_ui(bus: impl SystemSender) -> LoginUi {
    let login_ui = LoginUi::new(Box::new(move |username, password| {
        bus.send_to(
            CoreName::Greeter,
            GreeterMessage::LoginWithCreds(username, password),
        );
    }));
    login_ui.init();
    login_ui
}

fn login_ui_widget_closure(login_ui: LoginUi) -> impl Fn() {
    move || {
        login_ui.toggle();
    }
}

// since we cannot the the primary monitor we size to the first one
fn size_to_first_monitor(window: &Window) {
    if let Some(display) = gdk::Display::default() {
        let monitors = display.monitors();
        if let Some(monitor) = monitors
            .item(0)
            .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        {
            let geometry = monitor.geometry();
            window.set_default_size(geometry.width(), geometry.height());
        }
    }
}
