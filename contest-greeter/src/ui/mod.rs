use gio::glib::idle_add_local;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::prelude::*;

mod background;
mod chain_listener;
mod login_ui;
use chain_listener::register_chain_listener;
use login_ui::LoginUi;
use tokio::sync::mpsc;

use crate::ui::background::Background;
use crate::{CoreUICommand, UICoreCommand};

pub fn run_ui(
    core_tx: mpsc::UnboundedSender<UICoreCommand>,
    ui_rx: mpsc::UnboundedReceiver<CoreUICommand>,
) {
    let application = Application::builder()
        .application_id("nl.luukblankenstijn.lightdm-greeter")
        .build();

    let ui_rx_cell = std::cell::RefCell::new(Some(ui_rx));
    application.connect_activate(move |app| {
        let ui_rx = ui_rx_cell
            .borrow_mut()
            .take()
            .expect("take called more then once");

        build_ui(app, core_tx.clone(), ui_rx)
    });

    application.run();
}

fn build_ui(
    application: &Application,
    core_tx: mpsc::UnboundedSender<UICoreCommand>,
    mut ui_rx: mpsc::UnboundedReceiver<CoreUICommand>,
) {
    let window = ApplicationWindow::builder()
        .application(application)
        .title("lightdm-contest-greeter")
        .build();

    let background = Background::new();
    let background_overlay = background.get_overlay();
    window.set_child(Some(background_overlay));

    let login_ui = build_login_ui(core_tx);

    background_overlay.add_overlay(login_ui.widget());
    let login_ui_clone = login_ui.clone();
    register_chain_listener(&window, vec!['n', 'i', 'a', 'h', 'c'], {
        let login_ui = login_ui_widget_closure(login_ui_clone);
        move || login_ui()
    });

    idle_add_local(move || {
        while let Ok(ev) = ui_rx.try_recv() {
            match ev {
                CoreUICommand::SetWallpaper(path_option) => match path_option {
                    Some(path) => {
                        background.set_image(&path.to_string());
                    }
                    None => {
                        background.set_empty();
                    }
                },
                CoreUICommand::SetError(error) => {
                    login_ui.set_error_text(&error.to_string());
                }
            }
        }
        gio::glib::ControlFlow::Continue
    });

    // window.fullscreen();
    window.set_decorated(false);
    window.present();
}

fn build_login_ui(core_tx: mpsc::UnboundedSender<UICoreCommand>) -> LoginUi {
    let login_ui = LoginUi::new(Box::new(move |username, password| {
        let _ = core_tx.send(UICoreCommand::Login(username, password));
    }));
    login_ui.init();
    login_ui
}

fn login_ui_widget_closure(login_ui: LoginUi) -> impl Fn() {
    move || {
        login_ui.toggle();
    }
}
