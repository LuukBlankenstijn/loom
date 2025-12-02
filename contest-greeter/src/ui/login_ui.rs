use std::rc::Rc;

use gtk4::{
    Align, Box as GTBox, Button, CssProvider, Entry, EventControllerKey, InputPurpose, Label,
    Orientation, Overlay, PropagationPhase, STYLE_PROVIDER_PRIORITY_APPLICATION,
    gdk::{Display, Key},
    glib::Propagation,
    pango::WrapMode,
    prelude::*,
    style_context_add_provider_for_display,
};

#[derive(Clone)]
pub struct LoginUi {
    container: Overlay,
    username: Entry,
    password: Entry,
    label: Label,
    login_callback: Rc<Box<dyn Fn(String, String)>>,
}

const LOGIN_CSS: &str = "
    .login-container {
        background-color: rgba(30, 30, 30, 0.9);
        border-radius: 12px;
        padding: 24px;
        padding-top: 48px;
        box-shadow: 0 8px 32px rgba(0,0,0,0.18);
    }

    entry {
        margin: 6px 0;
        border-radius: 6px;
    }

    .error-label {
        color: #EE4B2B;
    }
";

impl LoginUi {
    pub fn new(login_callback: Box<dyn Fn(String, String)>) -> Self {
        let container = build_container();
        container.set_hexpand(true);
        container.set_vexpand(true);
        let content = build_content();
        load_css();

        let username = build_entry("username", false);
        let password = build_entry("password", true);
        let label = build_label();
        content.append(&username);
        content.append(&password);
        content.append(&label);
        container.set_child(Some(&content));
        let login_ui = Self {
            container,
            username,
            password,
            label,
            login_callback: Rc::new(login_callback),
        };

        let close_button = build_close_button(&login_ui);
        login_ui.container.add_overlay(&close_button);
        login_ui.container.set_visible(false);

        login_ui
    }

    pub fn init(&self) {
        attach_submit_handler(
            &self.username,
            &self.username,
            &self.password,
            &self.label,
            &self.login_callback,
        );
        attach_submit_handler(
            &self.password,
            &self.username,
            &self.password,
            &self.label,
            &self.login_callback,
        );
    }

    pub fn widget(&self) -> &Overlay {
        &self.container
    }

    pub fn toggle(&self) {
        let visible = self.container.is_visible();
        self.container.set_visible(!visible);

        // grab focus if the previous state was hidden
        if !visible {
            self.username.grab_focus();
        }
    }

    pub fn set_error_text(&self, label: &str) {
        self.label.set_label(label);
        self.label.set_visible(true);
    }
}

fn build_container() -> Overlay {
    let container = Overlay::new();
    container.set_halign(Align::Center);
    container.set_valign(Align::Center);
    container
}

fn load_css() {
    let css = CssProvider::new();
    css.load_from_data(LOGIN_CSS);

    if let Some(display) = Display::default() {
        style_context_add_provider_for_display(&display, &css, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

fn build_content() -> GTBox {
    let content = GTBox::new(Orientation::Vertical, 6);
    content.set_halign(Align::Center);
    content.set_valign(Align::Center);
    content.set_width_request(300);
    content.style_context().add_class("login-container");
    content
}

fn build_entry(placeholder: &str, secret: bool) -> Entry {
    let entry = Entry::new();
    entry.set_placeholder_text(Some(placeholder));
    if secret {
        entry.set_visibility(false);
        entry.set_input_purpose(InputPurpose::Password);
    }
    entry
}

fn build_label() -> Label {
    let label = Label::builder()
        .visible(false)
        .wrap_mode(WrapMode::Word)
        .wrap(true)
        .max_width_chars(30)
        .build();
    label.style_context().add_class("error-label");
    label
}

fn attach_submit_handler(
    trigger: &Entry,
    username_entry: &Entry,
    password_entry: &Entry,
    label: &Label,
    callback: &Rc<Box<dyn Fn(String, String)>>,
) {
    let controller = EventControllerKey::new();
    controller.set_propagation_phase(PropagationPhase::Capture);

    let username_clone = username_entry.clone();
    let password_clone = password_entry.clone();
    let label_clone = label.clone();
    let callback = callback.clone();

    controller.connect_key_pressed(move |_, key, _, _| {
        if key == Key::Return {
            let username_val = username_clone.text().to_string();
            let password_val = password_clone.text().to_string();
            if !username_val.is_empty() && !password_val.is_empty() {
                label_clone.set_visible(false);
                callback(username_val, password_val);
            }
        }
        Propagation::Proceed
    });

    trigger.add_controller(controller);
}

fn build_close_button(login_ui: &LoginUi) -> Button {
    let button = Button::builder()
        .icon_name("window-close-symbolic")
        .focus_on_click(false)
        .has_frame(false)
        .build();
    button.add_css_class("titlebutton");
    button.add_css_class("close");
    button.set_halign(Align::End);
    button.set_valign(Align::Start);
    button.set_margin_end(6);
    button.set_margin_top(6);
    let login_ui = login_ui.clone();
    button.connect_clicked(move |_| {
        login_ui.toggle();
    });
    button
}
