use std::{cell::RefCell, rc::Rc};

use gtk4::{EventControllerKey, Window, prelude::WidgetExt};

pub fn register_chain_listener(
    window: &Window,
    sequence: Vec<char>,
    callback: impl Fn() + 'static,
) {
    let buffer_size = sequence.len();
    let key_buffer: Rc<RefCell<Vec<char>>> = Rc::new(RefCell::new(Vec::with_capacity(buffer_size)));

    let event_controller = EventControllerKey::new();

    event_controller.connect_key_pressed(move |_, key, _, _| {
        if let Some(ch) = key.to_unicode() {
            let mut buf = key_buffer.borrow_mut();
            if buf.len() == buffer_size {
                buf.remove(0);
            }
            buf.push(ch);

            if *buf == sequence {
                callback();
            }
        }

        gtk4::glib::Propagation::Proceed
    });

    window.add_controller(event_controller);
}
