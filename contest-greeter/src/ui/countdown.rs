use std::cell::RefCell;
use std::rc::Rc;

use chrono::{DateTime, Local};
use gtk4::glib::{ControlFlow, SourceId, timeout_add_local, timeout_add_seconds_local};
use gtk4::{
    CssProvider, Label, Overlay, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk::Display, prelude::*,
    style_context_add_provider_for_display,
};

use crate::ui::UiConfig;
use types::{CoreName, GreeterMessage, SystemSender};

pub struct CountDown<S: SystemSender + Clone + 'static> {
    overlay: Overlay,
    label: Label,
    state: Rc<RefCell<CountdownState>>,
    tick: Rc<RefCell<Option<SourceId>>>,
    bus: S,
}

struct CountdownState {
    end_time: Option<DateTime<Local>>,
    from_seconds: Option<u64>,
    end_login: bool,
    triggered: bool,
}

const COUNTDOWN_CSS: &str = "
    label.countdown {
        font-size: 128px;
        color: rgb(0, 0, 0);
        padding: 20px;
        font-weight: bold;
    }
";

impl<S: SystemSender + Clone + 'static> CountDown<S> {
    pub fn new(conf: UiConfig, bus: S) -> Self {
        let overlay = Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);

        let label = Label::new(Some(""));
        label.style_context().add_class("countdown");
        label.set_halign(gtk4::Align::Center);
        label.set_valign(gtk4::Align::Center);
        label.set_hexpand(true);
        label.set_vexpand(true);
        overlay.set_child(Some(&label));

        let css = CssProvider::new();
        css.load_from_data(COUNTDOWN_CSS);
        if let Some(display) = Display::default() {
            style_context_add_provider_for_display(
                &display,
                &css,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let end_time = conf.countdown_end_time.map(|dt| dt.with_timezone(&Local));

        let state = Rc::new(RefCell::new(CountdownState {
            end_time,
            from_seconds: conf.countdown_from,
            end_login: conf.countdown_end_login,
            triggered: false,
        }));

        let countdown = Self {
            overlay,
            label,
            state,
            tick: Rc::new(RefCell::new(None)),
            bus,
        };

        countdown.schedule_timers();
        countdown
    }

    pub fn widget(&self) -> &Overlay {
        &self.overlay
    }

    pub fn update_endtime(&self, end_time: Option<DateTime<Local>>) {
        let mut state = self.state.borrow_mut();
        if let Some(dt) = end_time {
            state.end_time = Some(dt);
            state.triggered = false;
        }
        drop(state);
        self.schedule_timers();
    }

    fn schedule_timers(&self) {
        // cancel any existing tick
        if let Some(id) = self.tick.borrow_mut().take() {
            id.remove();
        }

        // update immediately so the label is in a correct state until the countdown starts
        update_label(&self.label, &self.state, &self.bus);

        let state_snapshot = self.state.borrow();
        let Some(end_time) = state_snapshot.end_time else {
            return;
        };
        let threshold_ms = state_snapshot.from_seconds.unwrap_or(0) as i64 * 1000;
        let remaining_ms = (end_time - Local::now()).num_milliseconds();
        let start_after_ms = (remaining_ms - threshold_ms).max(0) as u64;
        drop(state_snapshot);

        let label = self.label.clone();
        let state = self.state.clone();
        let tick_handle = self.tick.clone();
        let bus = self.bus.clone();

        // wait until threshold is reached, then start a steady 1s ticker aligned to that moment
        let start_id = timeout_add_local(
            std::time::Duration::from_millis(start_after_ms),
            move || {
                update_label(&label, &state, &bus);

                let label_tick = label.clone();
                let state_tick = state.clone();
                let tick_ref = tick_handle.clone();
                let bus_tick = bus.clone();
                let tick_id = timeout_add_seconds_local(1, move || {
                    update_label(&label_tick, &state_tick, &bus_tick);
                    ControlFlow::Continue
                });
                *tick_ref.borrow_mut() = Some(tick_id);

                ControlFlow::Break
            },
        );

        *self.tick.borrow_mut() = Some(start_id);
    }
}

fn update_label<S: SystemSender + Clone + 'static>(
    label: &Label,
    state: &Rc<RefCell<CountdownState>>,
    bus: &S,
) {
    let now = Local::now();
    let mut state = state.borrow_mut();
    if let Some(end_time) = state.end_time {
        let remaining = end_time - now;
        let seconds = remaining.num_seconds();
        let show = match state.from_seconds {
            Some(threshold) => seconds <= threshold as i64,
            None => true,
        };

        if show {
            if seconds > 0 {
                label.set_text(&format!("{}", seconds));
            } else {
                label.set_text("Starting...");
                if state.end_login && !state.triggered {
                    bus.send_to(CoreName::Greeter, GreeterMessage::Login());
                    state.triggered = true;
                }
            }
        } else {
            label.set_text("");
        }
    } else {
        label.set_text("");
    }
}
