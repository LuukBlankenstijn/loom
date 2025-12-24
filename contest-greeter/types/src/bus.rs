use std::any::Any;

use log::{debug, warn};
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CoreName {
    Greeter,
    UI,
}

pub enum SystemMsg {
    Register {
        name: CoreName,
        channel: Box<dyn ServiceChannel>,
    },
    Route {
        to: CoreName,
        payload: Box<dyn Any + Send>,
    },
}

pub trait ServiceChannel: Send {
    fn send_any(&self, msg: Box<dyn Any + Send>);
}

impl<T: 'static + Send> ServiceChannel for Sender<T> {
    fn send_any(&self, msg: Box<dyn Any + Send>) {
        if let Ok(typed_msg) = msg.downcast::<T>() {
            if let Err(e) = self.try_send(*typed_msg) {
                warn!("[Bus] failed to dispatch message: {}", e)
            }
        } else {
            debug!("[Bus] wrong message type sent to channel")
        }
    }
}

pub trait SystemSender: Clone + Send + Sync + 'static {
    fn send_to<T: Send + 'static>(&self, target: CoreName, msg: T);
}

pub trait SystemBus: SystemSender {
    fn register<T: Send + 'static>(&self, name: CoreName, tx: Sender<T>);
}

#[derive(Clone)]
pub struct SystemHandle {
    tx: Sender<SystemMsg>,
}

impl SystemHandle {
    pub fn new(tx: Sender<SystemMsg>) -> Self {
        Self { tx }
    }
}

impl SystemSender for SystemHandle {
    fn send_to<T: Send + 'static>(&self, target: CoreName, msg: T) {
        let to = target.clone();
        if let Err(e) = self.tx.try_send(SystemMsg::Route {
            to,
            payload: Box::new(msg),
        }) {
            warn!("[Bus] failed to enqueue message for {:?}: {}", target, e)
        }
    }
}

impl SystemBus for SystemHandle {
    fn register<T: Send + 'static>(&self, name: CoreName, tx: Sender<T>) {
        if let Err(e) = self.tx.try_send(SystemMsg::Register {
            name: name.clone(),
            channel: Box::new(tx),
        }) {
            warn!("[Bus] failed to register {:?}: {}", name, e)
        }
    }
}
