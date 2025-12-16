use std::{
    io::Cursor,
    path::Path,
    sync::mpsc::{self, TryRecvError},
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use gtk4::glib::{ControlFlow, timeout_add_local};
use gtk4::{
    CssProvider, Label, Overlay, Picture, STYLE_PROVIDER_PRIORITY_APPLICATION,
    gdk::{Display, Texture},
    gdk_pixbuf::Pixbuf,
    prelude::*,
    style_context_add_provider_for_display,
};
use log::debug;

pub struct Background {
    overlay: Overlay,
    _empty: Label,
    _invalid: Label,
}

const BACKGROUND_CSS: &str = "
    window { 
        background-color: #808080; 
    }

    label.watermark {
        font-size: 96px;
        color: rgba(255, 255, 255, 0.15);
        padding: 20px;
    }
";

impl Background {
    pub fn new() -> Self {
        let overlay = Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);
        let _empty = Label::new(Some("No Wallpaper"));
        _empty.style_context().add_class("watermark");

        let _invalid = Label::new(Some("Invalid Wallpaper"));
        _invalid.style_context().add_class("watermark");

        let css = CssProvider::new();
        css.load_from_data(BACKGROUND_CSS);

        if let Some(display) = Display::default() {
            style_context_add_provider_for_display(
                &display,
                &css,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        overlay.set_child(Some(&_empty));
        Self {
            overlay,
            _empty,
            _invalid,
        }
    }

    pub fn get_overlay(&self) -> &Overlay {
        &self.overlay
    }

    pub fn set_empty(&self) {
        self.overlay.set_child(Some(&self._empty));
    }

    pub fn set_image(&self, path: &str) {
        if path.starts_with("http://") || path.starts_with("https://") {
            self.fetch_remote_image(path);
        } else {
            match picture_from_filename(path) {
                Ok(pic) => {
                    self.overlay.set_child(Some(&pic));
                }
                Err(e) => {
                    self.overlay.set_child(Some(&self._invalid));
                    debug!("[UI] invalid file source ({path}): {e}");
                    debug!("[UI] reset background to default background");
                }
            }
        }
    }

    fn fetch_remote_image(&self, url: &str) {
        let (sender, receiver) = mpsc::channel::<std::result::Result<Vec<u8>, String>>();
        let url_string = url.to_string();

        std::thread::spawn(move || {
            let res = fetch_image_bytes(&url_string).map_err(|e| e.to_string());
            let _ = sender.send(res);
        });

        let overlay = self.overlay.clone();
        let invalid = self._invalid.clone();
        timeout_add_local(Duration::from_millis(50), move || {
            match receiver.try_recv() {
                Ok(Ok(bytes)) => match picture_from_bytes(bytes) {
                    Ok(pic) => overlay.set_child(Some(&pic)),
                    Err(e) => {
                        overlay.set_child(Some(&invalid));
                        debug!("[UI] failed to decode remote image: {e}");
                    }
                },
                Ok(Err(e)) => {
                    overlay.set_child(Some(&invalid));
                    debug!("[UI] failed to fetch remote image: {e}");
                }
                Err(TryRecvError::Empty) => return ControlFlow::Continue,
                Err(TryRecvError::Disconnected) => {
                    overlay.set_child(Some(&invalid));
                    debug!("[UI] remote fetch task disconnected");
                }
            }

            ControlFlow::Break
        });
    }
}

fn picture_from_filename(source: &str) -> Result<Picture> {
    let p = Path::new(source);

    const EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

    let image_is_valid = p.is_file()
        && p.extension()
            .and_then(|e| e.to_str())
            .map(|ext| EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false);

    if image_is_valid {
        let pic = Picture::for_filename(p);
        pic.set_vexpand(true);
        pic.set_hexpand(true);
        Ok(pic)
    } else {
        Err(anyhow!("invalid filepath {source}"))
    }
}

fn fetch_image_bytes(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::blocking::get(url).context("Failed to connect to the URL")?;

    let bytes = response.bytes().context("Failed to download image bytes")?;

    Ok(bytes.to_vec())
}

fn picture_from_bytes(bytes: Vec<u8>) -> Result<Picture> {
    // 2. Decode the data
    let cursor = Cursor::new(bytes);
    let pixbuf = Pixbuf::from_read(cursor)
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to decode image data (invalid format?)")?;

    let texture = Texture::for_pixbuf(&pixbuf);

    let picture = Picture::for_paintable(&texture);
    picture.set_vexpand(true);
    picture.set_hexpand(true);

    Ok(picture)
}
