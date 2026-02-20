use iced::{
    Alignment, Color, Element, Length, Task,
    widget::{container, text},
};
use local_ip_address::local_ip;
use log::error;

#[derive(Debug)]
pub struct IpLabel {
    ip_label: String,
}

#[derive(Debug, Clone)]
pub enum IpLabelMessage {
    SetIp(String),
}

impl IpLabel {
    pub fn new() -> (Self, Task<IpLabelMessage>) {
        (
            Self {
                ip_label: "Loading ip".to_string(),
            },
            Task::perform(get_ip_async(), IpLabelMessage::SetIp),
        )
    }

    pub fn view(&self) -> Element<'_, IpLabelMessage> {
        container(text(&self.ip_label).color(Color::WHITE).size(16))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Start)
            .align_y(Alignment::Start)
            .padding(10)
            .into()
    }

    pub fn update(&mut self, msg: IpLabelMessage) -> Task<IpLabelMessage> {
        match msg {
            IpLabelMessage::SetIp(ip_label) => self.ip_label = ip_label,
        };
        Task::none()
    }
}

async fn get_ip_async() -> String {
    match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(err) => {
            error!("failed to get ip address: {:?}", err);
            "No ip found".to_string()
        }
    }
}
