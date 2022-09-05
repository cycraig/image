//! Encoding of WebP images.
///
/// Uses the simple encoding API from the [libwebp] library.
///
/// [libwebp]: https://developers.google.com/speed/webp/docs/api#simple_encoding_api
use std::io::Write;

use libwebp::{Encoder, PixelLayout, WebPMemory};

use crate::ImageResult;
use crate::{ColorType, ImageEncoder};

/// WebP Encoder.
pub struct WebpEncoder<W> {
    inner: W,
    quality: WebpQuality,
}

/// WebP encoder quality.
#[derive(Debug, Copy, Clone)]
pub enum WebpQuality {
    /// Lossless encoding.
    Lossless,
    /// Lossy quality from 0 to 100. 0 = low quality, small size; 100 = high quality, large size.
    Lossy(f32),
}

impl WebpQuality {
    /// Minimum lossy quality value (0).
    pub const MIN: f32 = 0f32;
    /// Maximum lossy quality value (100).
    pub const MAX: f32 = 100f32;

    /// Clamps lossy quality between 0 and 100.
    fn clamp(self) -> Self {
        match self {
            WebpQuality::Lossy(quality) => WebpQuality::Lossy(quality.clamp(Self::MIN, Self::MAX)),
            lossless => lossless,
        }
    }
}

impl<W: Write> WebpEncoder<W> {
    /// Create a new encoder that writes its output to `w`.
    ///
    /// Defaults to lossy encoding with maximum quality.
    pub fn new(w: W) -> Self {
        WebpEncoder::new_with_quality(w, WebpQuality::Lossy(WebpQuality::MAX))
    }

    /// Create a new encoder with specified quality, that writes its output to `w`.
    ///
    /// Lossy qualities are clamped between 0 and 100.
    pub fn new_with_quality(w: W, quality: WebpQuality) -> Self {
        Self {
            inner: w,
            quality: quality.clamp(),
        }
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

        // Call the library to encode the image.
        let encoder = Encoder::new(data, layout, width, height);
        let output: WebPMemory = match self.quality {
            WebpQuality::Lossless => encoder.encode_lossless(),
            WebpQuality::Lossy(quality) => encoder.encode(quality),
        };

        // TODO: how to check if any errors occurred, maybe if the memory is empty?
        //       Can errors occur?
        // let data = result.map_err(|err| {
        //     ImageError::Encoding(EncodingError::new(ImageFormat::WebP.into(), err))
        // })?;

        self.inner.write_all(&output)?;
        Ok(())
    }
}

impl<W: Write> ImageEncoder for WebpEncoder<W> {
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
