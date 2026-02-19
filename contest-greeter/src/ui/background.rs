use image::GenericImageView;
use log::error;
use std::{path::Path, str::FromStr};

use iced::{
    Color, ContentFit, Element, Font, Length, Task,
    font::Weight,
    widget::{container, space, text},
};

use crate::ui::Message;

#[derive(Debug, Clone, Default)]
enum ImageStatus {
    Loading,
    Ready,
    #[default]
    Invalid,
    Empty,
}

#[derive(Debug, Clone)]
pub struct Label {
    text: String,
    color: iced::Color,
}

#[derive(Debug, Default)]
pub struct Background {
    handle: Option<iced::widget::image::Handle>,
    image_status: ImageStatus,
    label: Option<Label>,
}

#[derive(Debug, Clone)]
pub enum BackgroundMessage {
    SetSource(Option<String>),
    SetData(Option<(iced::widget::image::Handle, Option<Label>)>),
}

impl From<BackgroundMessage> for Message {
    fn from(value: BackgroundMessage) -> Self {
        Message::Background(value)
    }
}

impl Background {
    pub fn new(
        source: Option<String>,
        label: Option<String>,
        color: Option<String>,
    ) -> (Self, Task<BackgroundMessage>) {
        let task = Task::done(BackgroundMessage::SetSource(source));

        let label = label.map(|t| {
            let color = color
                .and_then(|h| Color::from_str(&h).ok())
                .unwrap_or(Color::WHITE);

            Label { text: t, color }
        });

        (
            Self {
                label,
                ..Default::default()
            },
            task,
        )
    }

    pub fn view(
        &self,
    ) -> (
        Element<'_, BackgroundMessage>,
        Option<Element<'_, BackgroundMessage>>,
    ) {
        let image_element = match self.image_status {
            ImageStatus::Loading => no_background_container("Loading...".to_string()),
            ImageStatus::Empty => no_background_container("No background configured".to_string()),
            ImageStatus::Ready => {
                if let Some(handle) = self.handle.clone() {
                    container(
                        iced::widget::image(handle)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .content_fit(ContentFit::Contain),
                    )
                    .into()
                } else {
                    container(space()).into()
                }
            }
            ImageStatus::Invalid => no_background_container("Invalid background...".to_string()),
        };
        let label_element = self.label.clone().map(|label| {
            container(text(label.text).color(label.color).size(40).font(Font {
                weight: Weight::Bold,
                ..Default::default()
            }))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        });
        (image_element, label_element)
    }

    pub fn update(&mut self, msg: BackgroundMessage) -> Task<BackgroundMessage> {
        match msg {
            BackgroundMessage::SetSource(source) => {
                if let Some(source) = source {
                    self.image_status = ImageStatus::Loading;
                    if is_http_url(&source) {
                        return Task::perform(
                            async move { fetch_remote_image(&source).and_then(create_handle) },
                            BackgroundMessage::SetData,
                        );
                    } else {
                        return Task::perform(
                            async move { fetch_local_image(&source).and_then(create_handle) },
                            BackgroundMessage::SetData,
                        );
                    }
                } else {
                    self.image_status = ImageStatus::Empty;
                }
            }
            BackgroundMessage::SetData(result) => match result {
                Some((handle, label)) => {
                    self.handle = Some(handle);
                    self.label = label;
                    self.image_status = ImageStatus::Ready
                }
                None => {
                    self.handle = None;
                    self.image_status = ImageStatus::Invalid;
                }
            },
        }
        Task::none()
    }
}

fn no_background_container<'a>(label: String) -> Element<'a, BackgroundMessage> {
    container(text(label).size(24).color(Color::WHITE))
        .height(Length::Fill)
        .center(Length::Fill)
        .into()
}

fn is_http_url(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

fn fetch_local_image(path: &str) -> Option<(Vec<u8>, Option<Label>)> {
    if !Path::new(path).exists() {
        error!("Path does not exist: {}", path);
        return None;
    }

    match std::fs::read(path) {
        Ok(bytes) => Some((bytes, None)),
        Err(e) => {
            error!("Failed to read file {}: {}", path, e);
            None
        }
    }
}

fn fetch_remote_image(source: &str) -> Option<(Vec<u8>, Option<Label>)> {
    if !is_http_url(source) {
        error!("Invalid URL format: {}", source);
        return None;
    }

    let response = match ureq::get(source).call() {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to fetch {}: {}", source, e);
            return None;
        }
    };

    let label_text = response.headers().get("X-Wallpaper-Text");
    let color = response.headers().get("X-Wallpaper-Text-Color");

    let label = label_text.map(|t| {
        let color = color
            .and_then(|h| Color::from_str(h.to_str().unwrap_or_default()).ok())
            .unwrap_or(Color::WHITE);

        Label {
            text: t.to_str().unwrap_or_default().to_string(),
            color,
        }
    });

    match response.into_body().read_to_vec() {
        Ok(bytes) => Some((bytes, label)),
        Err(e) => {
            error!("Failed to read bytes from {}: {}", source, e);
            None
        }
    }
}

fn create_handle(
    (bytes, label): (Vec<u8>, Option<Label>),
) -> Option<(iced::widget::image::Handle, Option<Label>)> {
    let img = image::load_from_memory(&bytes)
        .map_err(|e| {
            error!("Failed to decode image: {}", e);
            e
        })
        .ok()?;

    let resized = img.thumbnail(1920, 1080);
    let (width, height) = resized.dimensions();

    let rgba_bytes = resized.to_rgba8().into_raw();

    Some((
        iced::widget::image::Handle::from_rgba(width, height, rgba_bytes),
        label,
    ))
}
