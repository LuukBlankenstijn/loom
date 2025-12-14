mod bus;
mod messages;

pub use bus::{CoreName, ServiceChannel, SystemHandle, SystemMsg, SystemReceiver, SystemSender};
pub use messages::{GreeterMessage, UiMessage};
