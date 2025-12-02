use std::path::Path;

use gtk4::{
    CssProvider, Label, Overlay, Picture, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk::Display,
    prelude::*, style_context_add_provider_for_display,
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
        let p = Path::new(path);

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
            self.overlay.set_child(Some(&pic));
        } else {
            self.overlay.set_child(Some(&self._invalid));
            debug!("invalid file at path {}, set default background", path)
        }
    }
}
