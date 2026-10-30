use std::{sync::Arc, time::Duration};

use ::image::GenericImageView;
use gstreamer::{self as gst, prelude::*};
use gstreamer_app::AppSink;
use iced::{
    Color, ContentFit, Element, Length, Task,
    widget::{container, image, text},
};
use iced_video_player::{Video, VideoPlayer};
use log::{error, info};
use url::Url;

const STREAM_RETRY_DELAY: Duration = Duration::from_secs(5);
const STREAM_PIPELINE: &str = "udpsrc name=source buffer-size=4194304 \
    ! queue max-size-buffers=0 max-size-time=0 max-size-bytes=16777216 \
    ! decodebin name=decode ! videoconvert ! videoscale \
    ! appsink name=video drop=true caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1";
const STREAM_AUDIO_BRANCH: &str = " decode. ! audioconvert ! audioresample ! queue ! autoaudiosink";

#[derive(Debug, Clone)]
pub enum MediaSource {
    Image(Vec<u8>),
    Stream(Stream),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub url: Url,
    pub audio: bool,
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Empty,
    Loading,
    Invalid,
    Image(image::Handle),
    WaitingForStream(Stream),
    Video(Stream, Arc<Video>),
}

#[derive(Debug, Default)]
pub struct Media {
    state: State,
    generation: u64,
}

#[derive(Debug, Clone)]
pub enum MediaMessage {
    Clear,
    Loading,
    Invalid,
    Load(MediaSource),
    ImageDecoded(u64, Option<image::Handle>),
    RetryStream(u64),
    StreamOpened(u64, Result<Arc<Video>, String>),
    VideoFailed(u64, String),
}

impl Media {
    pub fn is_showing(&self) -> bool {
        matches!(self.state, State::Image(_) | State::Video(..))
    }

    pub fn view(&self) -> Element<'_, MediaMessage> {
        match &self.state {
            State::Image(handle) => container(
                image(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Cover),
            )
            .into(),
            State::Video(_, video) => {
                let generation = self.generation;
                VideoPlayer::new(video)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Cover)
                    .on_error(move |e| MediaMessage::VideoFailed(generation, e.to_string()))
                    .on_end_of_stream(MediaMessage::VideoFailed(
                        generation,
                        "end of stream".to_string(),
                    ))
                    .into()
            }
            State::Empty => placeholder("No background configured"),
            State::Loading => placeholder("Loading..."),
            State::Invalid => placeholder("Invalid background..."),
            State::WaitingForStream(_) => placeholder("Waiting for video stream..."),
        }
    }

    pub fn update(&mut self, msg: MediaMessage) -> Task<MediaMessage> {
        match msg {
            MediaMessage::Clear => self.replace(State::Empty),
            MediaMessage::Loading => {
                if !self.is_showing() {
                    self.replace(State::Loading);
                }
            }
            MediaMessage::Invalid => self.replace(State::Invalid),
            MediaMessage::Load(MediaSource::Image(bytes)) => {
                self.replace(State::Loading);
                let generation = self.generation;
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || create_handle(bytes))
                            .await
                            .unwrap_or(None)
                    },
                    move |handle| MediaMessage::ImageDecoded(generation, handle),
                );
            }
            MediaMessage::Load(MediaSource::Stream(stream)) => {
                if let State::Video(current, _) = &self.state
                    && *current == stream
                {
                    return Task::none();
                }
                self.replace(State::WaitingForStream(stream));
                return self.open_stream();
            }
            MediaMessage::ImageDecoded(generation, handle) if generation == self.generation => {
                self.state = match handle {
                    Some(handle) => State::Image(handle),
                    None => State::Invalid,
                };
            }
            MediaMessage::RetryStream(generation) if generation == self.generation => {
                return self.open_stream();
            }
            MediaMessage::StreamOpened(generation, result) if generation == self.generation => {
                let State::WaitingForStream(stream) = &self.state else {
                    return Task::none();
                };
                match result {
                    Ok(video) => {
                        info!("Playing video stream {}", stream.url);
                        self.state = State::Video(stream.clone(), video);
                    }
                    Err(e) => {
                        error!("Failed to open video stream {}: {e}", stream.url);
                        return self.retry_later();
                    }
                }
            }
            MediaMessage::VideoFailed(generation, e) if generation == self.generation => {
                let State::Video(stream, _) = &self.state else {
                    return Task::none();
                };
                error!("Video stream {} failed: {e}", stream.url);
                self.state = State::WaitingForStream(stream.clone());
                return self.retry_later();
            }
            MediaMessage::ImageDecoded(..)
            | MediaMessage::RetryStream(_)
            | MediaMessage::StreamOpened(..)
            | MediaMessage::VideoFailed(..) => {}
        }
        Task::none()
    }

    fn replace(&mut self, state: State) {
        self.generation += 1;
        self.state = state;
    }

    fn open_stream(&self) -> Task<MediaMessage> {
        let State::WaitingForStream(stream) = &self.state else {
            return Task::none();
        };
        let stream = stream.clone();
        let generation = self.generation;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || open_video(&stream).map(Arc::new))
                    .await
                    .unwrap_or_else(|_| Err("Task panicked".to_string()))
            },
            move |result| MediaMessage::StreamOpened(generation, result),
        )
    }

    fn retry_later(&self) -> Task<MediaMessage> {
        let generation = self.generation;
        Task::perform(tokio::time::sleep(STREAM_RETRY_DELAY), move |_| {
            MediaMessage::RetryStream(generation)
        })
    }
}

fn placeholder<'a>(label: &'a str) -> Element<'a, MediaMessage> {
    container(text(label).size(24).color(Color::WHITE))
        .height(Length::Fill)
        .center(Length::Fill)
        .into()
}

fn open_video(stream: &Stream) -> Result<Video, String> {
    gst::init().map_err(|e| e.to_string())?;
    let description = if stream.audio {
        format!("{STREAM_PIPELINE}{STREAM_AUDIO_BRANCH}")
    } else {
        STREAM_PIPELINE.to_string()
    };
    let pipeline = gst::parse::launch(&description)
        .map_err(|e| e.to_string())?
        .downcast::<gst::Pipeline>()
        .map_err(|_| "stream pipeline is not a gst::Pipeline".to_string())?;
    let source = pipeline
        .by_name("source")
        .ok_or("stream pipeline has no source")?;
    source.set_property("uri", stream.url.as_str());
    let sink = pipeline
        .by_name("video")
        .and_then(|sink| sink.downcast::<AppSink>().ok())
        .ok_or("stream pipeline has no video appsink")?;
    Video::from_gst_pipeline(pipeline, sink, None).map_err(|e| e.to_string())
}

fn create_handle(bytes: Vec<u8>) -> Option<image::Handle> {
    let img = ::image::load_from_memory(&bytes)
        .map_err(|e| error!("Failed to decode image: {}", e))
        .ok()?;

    let resized = img.thumbnail(1920, 1080);
    let (width, height) = resized.dimensions();

    Some(image::Handle::from_rgba(
        width,
        height,
        resized.to_rgba8().into_raw(),
    ))
}
