mod bus;
mod messages;

pub use bus::{CoreName, ServiceChannel, SystemBus, SystemHandle, SystemMsg, SystemSender};
pub use messages::{GreeterMessage, UiMessage};
