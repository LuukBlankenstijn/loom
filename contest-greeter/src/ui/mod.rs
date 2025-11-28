use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::prelude::*;
use log::debug;

mod background;
mod chain_listener;
mod login_ui;
use chain_listener::register_chain_listener;
use login_ui::LoginUi;

use crate::ui::background::Background;

pub fn run_ui() {
    let application = Application::builder()
        .application_id("nl.luukblankenstijn.lightdm-greeter")
        .build();

    application.connect_activate(build_ui);

    application.run();
}

fn build_ui(application: &Application) {
    let window = ApplicationWindow::builder()
        .application(application)
        .title("lightdm-contest-greeter")
        .build();

    let background = Background::new();
    let background_overlay = background.get_overlay();
    window.set_child(Some(background_overlay));

    let login_ui = build_login_ui();

    background_overlay.add_overlay(login_ui.widget());
    register_chain_listener(&window, vec!['n', 'i', 'a', 'h', 'c'], {
        let login_ui = login_ui_widget_closure(login_ui);
        move || login_ui()
    });

    // window.fullscreen();
    window.set_decorated(false);
    window.present();
}

fn build_login_ui() -> LoginUi {
    let login_ui = LoginUi::new(Box::new(|username, password| {
        //TODO: call core with login request
        debug!("username: {}, password: {}", username, password)
    }));
    login_ui.init();
    login_ui
}

fn login_ui_widget_closure(login_ui: LoginUi) -> impl Fn() {
    move || {
        login_ui.toggle();
    }
}
