//! Encoding of WebP images.
///
/// Uses the simple encoding API from the [libwebp] library.
///
/// [libwebp]: https://developers.google.com/speed/webp/docs/api#simple_encoding_api
use std::io::Write;

use libwebp::{Encoder, PixelLayout, WebPMemory};

use crate::error::EncodingError;
use crate::ImageFormat::WebP;
use crate::{ColorType, ImageEncoder};
use crate::{ImageError, ImageFormat, ImageResult};

/// WebP Encoder.
pub struct WebPEncoder<W> {
    inner: W,
    quality: WebPQuality,
}

#[derive(Debug, Copy, Clone)]
enum Quality {
    Lossless,
    Lossy(u8),
}

/// WebP encoder quality.
#[derive(Debug, Copy, Clone)]
pub struct WebPQuality(Quality);

impl WebPQuality {
    /// Minimum lossy quality value (0).
    pub const MIN: u8 = 0;
    /// Maximum lossy quality value (100).
    pub const MAX: u8 = 100;
    /// Default lossy quality, providing reasonable quality and file size (80).
    pub const DEFAULT: u8 = 80;

    /// Lossless encoding.
    pub fn lossless() -> Self {
        Self(Quality::Lossless)
    }

    /// Lossy quality. 0 = low quality, small size; 100 = high quality, large size.
    ///
    /// Values are clamped from 0 to 100.
    pub fn lossy(quality: u8) -> Self {
        Self(Quality::lossy(quality.clamp(Self::MIN, Self::MAX)))
    }
}

impl Default for WebPQuality {
    fn default() -> Self {
        Self::lossy(WebPQuality::DEFAULT)
    }
}

impl<W: Write> WebPEncoder<W> {
    /// Create a new encoder that writes its output to `w`.
    ///
    /// Defaults to lossy encoding, see [`WebPQuality::DEFAULT`].
    pub fn new(w: W) -> Self {
        WebPEncoder::new_with_quality(w, WebPQuality::default())
    }

    /// Create a new encoder with the specified quality, that writes its output to `w`.
    pub fn new_with_quality(w: W, quality: WebPQuality) -> Self {
        Self { inner: w, quality }
    }

    /// Encode image data with the indicated color type.
    ///
    /// The encoder requires all data to be RGB8 or RGBA8, it will be converted
    /// internally if necessary.
    pub fn encode(
        mut self,
        data: &[u8],
        width: u32,
        height: u32,
        color: ColorType,
    ) -> ImageResult<()> {
        // TODO: convert color types internally.
        let layout: PixelLayout = match color {
            ColorType::Rgb8 => PixelLayout::Rgb,
            ColorType::Rgba8 => PixelLayout::Rgba,
            _ => unimplemented!("Color type not yet supported"),
        };

        // Call the native libwebp library to encode the image.
        let encoder = Encoder::new(data, layout, width, height);
        let encoded: WebPMemory = match self.quality.0 {
            Quality::Lossless => encoder.encode_lossless(),
            Quality::Lossy(quality) => encoder.encode(quality as f32),
        };

        // TODO: how to check if any errors occurred? Can errors occur?
        if encoded.is_empty() {
            return Err(ImageError::Encoding(EncodingError::new(
                ImageFormat::WebP.into(),
                "encoding failed, output empty",
            )));
        }

        self.inner.write_all(&encoded)?;
        Ok(())
    }
}

impl<W: Write> ImageEncoder for WebPEncoder<W> {
    fn write_image(
        self,
        buf: &[u8],
        width: u32,
        height: u32,
        color_type: ColorType,
    ) -> ImageResult<()> {
        self.encode(buf, width, height, color_type)
    }
}
