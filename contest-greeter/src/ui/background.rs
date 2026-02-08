use log::{debug, info};
use std::{fs::File, io::BufReader, path::Path};

use iced::{
    Color, ContentFit, Element, Length, Task,
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

#[derive(Debug, Default)]
pub struct Background {
    handle: Option<iced::widget::image::Handle>,
    image_status: ImageStatus,
}

#[derive(Debug, Clone)]
pub enum BackgroundMessage {
    SetSource(Option<String>),
    GetHandle(String),
    SetHandle(Option<iced::widget::image::Handle>),
}

impl From<BackgroundMessage> for Message {
    fn from(value: BackgroundMessage) -> Self {
        Message::Background(value)
    }
}

impl Background {
    pub fn new(source: Option<String>) -> (Self, Task<BackgroundMessage>) {
        let task = Task::done(BackgroundMessage::SetSource(source));

        (Self::default(), task)
    }

    pub fn view(&self) -> Element<'_, BackgroundMessage> {
        match self.image_status {
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
        }
    }

    pub fn update(&mut self, msg: BackgroundMessage) -> Task<BackgroundMessage> {
        match msg {
            BackgroundMessage::SetSource(source) => {
                if let Some(source) = source {
                    self.image_status = ImageStatus::Loading;
                    if !is_local(&source) {
                        return Task::perform(
                            async move { fetch_remote_image(source) },
                            BackgroundMessage::GetHandle,
                        );
                    }
                    return Task::done(BackgroundMessage::GetHandle(source));
                } else {
                    self.image_status = ImageStatus::Empty;
                }
            }
            BackgroundMessage::GetHandle(filepath) => {
                return Task::perform(
                    async move { get_handle(&filepath) },
                    BackgroundMessage::SetHandle,
                );
            }
            BackgroundMessage::SetHandle(result) => match result {
                Some(handle) => {
                    self.handle = Some(handle);
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

fn get_handle(path: &str) -> Option<iced::widget::image::Handle> {
    if !Path::new(path).exists() {
        return None;
    }
    File::open(path)
        .ok()
        .and_then(|file| {
            let reader = BufReader::new(file);
            image::ImageReader::new(reader).with_guessed_format().ok()
        })
        .and_then(|reader| {
            let format = reader.format()?;
            let ext = format.extensions_str().first()?;
            let path_with_ext = format!("{}.{}", path, ext);
            std::os::unix::fs::symlink(path, &path_with_ext).ok()?;
            Some(iced::widget::image::Handle::from_path(path_with_ext))
        })
}

fn is_local(source: &str) -> bool {
    let path = Path::new(source);
    path.exists() && path.is_file()
}

fn fetch_remote_image(source: String) -> String {
    if !source.starts_with("http://") && !source.starts_with("https://") {
        debug!("Invalid URL format: {}", source);
        return String::new();
    }
    let response = match ureq::get(&source).call() {
        Ok(resp) => resp,
        Err(e) => {
            debug!("Failed to fetch {}: {}", source, e);
            return String::new();
        }
    };
    let bytes = match response.into_body().read_to_vec() {
        Ok(bytes) => bytes,
        Err(e) => {
            debug!("Failed to read bytes from {}: {}", source, e);
            return String::new();
        }
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let filename = format!("/tmp/download_{}", timestamp);

    if std::fs::write(&filename, bytes).is_err() {
        debug!("Failed to write file for {}", source);
        return String::new();
    }

    filename
}
