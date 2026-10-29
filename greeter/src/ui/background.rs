mod media;

use log::{error, warn};
use std::{path::Path, str::FromStr};

use iced::{
    Color, Element, Font, Length, Padding, Task, alignment,
    font::Weight,
    widget::{container, stack, text},
};
use url::Url;

use crate::ui::{
    Message,
    background::media::{Media, MediaMessage, MediaSource},
};

#[derive(Debug, Clone)]
pub struct Label {
    text: String,
    color: iced::Color,
}

#[derive(Debug, Clone)]
pub enum Fetched {
    Local(Option<MediaSource>),
    Remote(Option<MediaSource>, Option<Label>),
}

#[derive(Debug, Default)]
pub struct Background {
    media: Media,
    label: Option<Label>,
    request: u64,
}

#[derive(Debug, Clone)]
pub enum BackgroundMessage {
    Source(Option<String>),
    Fetched(u64, Fetched),
    Media(MediaMessage),
    GotColor(Option<Color>),
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
        let task = Task::done(BackgroundMessage::Source(source));

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
        let media_element = self.media.view().map(BackgroundMessage::Media);

        let label_element = if self.media.is_showing() {
            self.label.as_ref().map(|label| {
                container(outlined_text(
                    &label.text,
                    70.0,
                    label.color,
                    Color::BLACK,
                    2.0,
                ))
                .center(Length::Fill)
                .into()
            })
        } else {
            None
        };

        (media_element, label_element)
    }

    pub fn update(&mut self, msg: BackgroundMessage) -> Task<BackgroundMessage> {
        match msg {
            BackgroundMessage::Source(source) => {
                self.request += 1;
                let Some(source) = source else {
                    return self.update_media(MediaMessage::Clear);
                };
                let request = self.request;
                let fetch = Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || fetch(&source))
                            .await
                            .unwrap_or(Fetched::Local(None))
                    },
                    move |fetched| BackgroundMessage::Fetched(request, fetched),
                );
                Task::batch(vec![self.update_media(MediaMessage::Loading), fetch])
            }
            BackgroundMessage::Fetched(request, fetched) if request == self.request => {
                match fetched {
                    Fetched::Local(source) => self.load(source),
                    Fetched::Remote(source, label) => {
                        let color = label.as_ref().map(|l| l.color);
                        self.label = label;
                        Task::batch(vec![
                            self.load(source),
                            Task::done(BackgroundMessage::GotColor(color)),
                        ])
                    }
                }
            }
            BackgroundMessage::Media(msg) => self.update_media(msg),
            BackgroundMessage::Fetched(..) | BackgroundMessage::GotColor(_) => Task::none(),
        }
    }

    fn load(&mut self, source: Option<MediaSource>) -> Task<BackgroundMessage> {
        self.update_media(match source {
            Some(source) => MediaMessage::Load(source),
            None => MediaMessage::Invalid,
        })
    }

    fn update_media(&mut self, msg: MediaMessage) -> Task<BackgroundMessage> {
        self.media.update(msg).map(BackgroundMessage::Media)
    }
}

fn outlined_text<'a>(
    content: &'a str,
    size: f32,
    color: Color,
    outline: Color,
    border: f32,
) -> Element<'a, BackgroundMessage> {
    let layer = |dx: f32, dy: f32, color: Color| {
        container(
            text(content)
                .color(color)
                .size(size)
                .font(Font {
                    weight: Weight::ExtraBold,
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .padding(Padding {
            top: dy,
            right: 2.0 * border - dx,
            bottom: 2.0 * border - dy,
            left: dx,
        })
        .width(Length::Fill)
        .height(Length::Fill)
    };

    let (w, d) = (border, border * 2.0);
    let mut layers: Vec<Element<'a, BackgroundMessage>> = [
        (0.0, 0.0),
        (w, 0.0),
        (d, 0.0),
        (0.0, w),
        (d, w),
        (0.0, d),
        (w, d),
        (d, d),
    ]
    .into_iter()
    .map(|(dx, dy)| layer(dx, dy, outline).into())
    .collect();
    layers.push(layer(w, w, color).into());

    stack(layers).into()
}

fn is_http_url(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

fn fetch(source: &str) -> Fetched {
    if is_http_url(source) {
        let (media, label) = fetch_remote(source);
        Fetched::Remote(media, label)
    } else {
        Fetched::Local(fetch_local_image(source).map(MediaSource::Image))
    }
}

fn fetch_local_image(path: &str) -> Option<Vec<u8>> {
    if !Path::new(path).exists() {
        error!("Path does not exist: {}", path);
        return None;
    }

    match std::fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            error!("Failed to read file {}: {}", path, e);
            None
        }
    }
}

fn fetch_remote(source: &str) -> (Option<MediaSource>, Option<Label>) {
    let response = match ureq::get(source).call() {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to fetch {}: {}", source, e);
            return (None, None);
        }
    };

    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };

    let label = header("X-Wallpaper-Text").map(|text| {
        let color = header("X-Wallpaper-Text-Color")
            .and_then(|h| Color::from_str(&h).ok())
            .unwrap_or(Color::WHITE);

        Label { text, color }
    });

    if let Some(stream) = header("X-Wallpaper-Stream") {
        return match Url::parse(&stream) {
            Ok(url) => (Some(MediaSource::Stream(url)), label),
            Err(e) => {
                warn!("Invalid wallpaper stream url {}: {}", stream, e);
                (None, label)
            }
        };
    }

    match response.into_body().read_to_vec() {
        Ok(bytes) => (Some(MediaSource::Image(bytes)), label),
        Err(e) => {
            error!("Failed to read bytes from {}: {}", source, e);
            (None, None)
        }
    }
}
