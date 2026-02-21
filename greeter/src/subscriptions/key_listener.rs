use std::collections::VecDeque;

use iced::{Subscription, Task};

#[derive(Debug)]
pub struct KeyListener {
    buffer: VecDeque<char>,
    chain: String,
}

#[derive(Clone, Debug)]
pub enum KeyListenerMessage {
    Event(iced::event::Event),
    ChainTriggered,
}

impl KeyListener {
    pub fn new(chain: String) -> Self {
        Self {
            buffer: VecDeque::new(),
            chain,
        }
    }
    pub fn update(&mut self, msg: KeyListenerMessage) -> Task<KeyListenerMessage> {
        match msg {
            KeyListenerMessage::Event(event) => {
                if let iced::event::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    physical_key,
                    ..
                }) = event
                    && let Some(key) = key.to_latin(physical_key)
                {
                    self.buffer.push_back(key);
                    if self.buffer.len() != self.chain.len() {
                        return Task::none();
                    }
                    let buffer_string = String::from_iter(self.buffer.clone());
                    self.buffer.pop_front();
                    if buffer_string == self.chain {
                        return Task::done(KeyListenerMessage::ChainTriggered);
                    }
                }
            }
            KeyListenerMessage::ChainTriggered => {
                // handled by parent
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<KeyListenerMessage> {
        iced::event::listen().map(KeyListenerMessage::Event)
    }
}
