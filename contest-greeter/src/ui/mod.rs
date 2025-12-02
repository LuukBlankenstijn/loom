use gtk4::Window;
use gtk4::gdk;
use gtk4::glib::idle_add_local;
use gtk4::glib::{ControlFlow, MainLoop};
use gtk4::prelude::*;

mod background;
mod chain_listener;
mod login_ui;
use chain_listener::register_chain_listener;
use log::debug;
use login_ui::LoginUi;
use tokio::sync::mpsc;

use crate::ui::background::Background;
use crate::{CoreUICommand, UICoreCommand};

pub fn run_ui(
    core_tx: mpsc::UnboundedSender<UICoreCommand>,
    ui_rx: mpsc::UnboundedReceiver<CoreUICommand>,
) {
    gtk4::init().expect("init gtk");

    let ui_rx_cell = std::cell::RefCell::new(Some(ui_rx));
    build_ui(
        core_tx,
        ui_rx_cell.borrow_mut().take().expect("ui_rx already taken"),
    );

    debug!("running ui main loop");
    let main_loop = MainLoop::new(None, false);
    main_loop.run();
}

fn build_ui(
    core_tx: mpsc::UnboundedSender<UICoreCommand>,
    mut ui_rx: mpsc::UnboundedReceiver<CoreUICommand>,
) {
    let window = Window::builder().title("lightdm-contest-greeter").build();

    size_to_first_monitor(&window);
    window.set_decorated(false);

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
        ControlFlow::Continue
    });

    window.present();
    window.fullscreen();
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
