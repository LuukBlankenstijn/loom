use image::ImageFormat;
use std::io::Cursor;

pub(super) fn mime_type(data: &[u8]) -> Result<&'static str, String> {
    let unsupported = || "unsupported image format, use png, jpeg, webp or gif".to_string();
    let mime_type = match image::guess_format(data).map_err(|_| unsupported())? {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Gif => "image/gif",
        ImageFormat::WebP => "image/webp",
        _ => return Err(unsupported()),
    };

    image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| format!("failed to read image: {e}"))?
        .into_dimensions()
        .map_err(|e| format!("invalid image data: {e}"))?;

    Ok(mime_type)
}
