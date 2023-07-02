use image::{imageops::FilterType, ImageFormat, ImageOutputFormat};
use lambda_runtime::Error;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::ResponseError;

const DEFAULT_QUALITY: u8 = 100;
const MAX_WITDH: u32 = 10_000;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FlipImage {
    Vertical,
    Horizontal,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub struct ImageManipulationQuery {
    pub width: Option<u32>,
    pub quality: Option<u8>,
    pub blur: Option<f32>,
    pub flip: Option<FlipImage>,
    pub grayscale: bool,
    pub brightness: Option<i32>,
    pub contrast: Option<f32>,
    pub hue: Option<i32>,
    pub invert: bool,
    pub crop: Option<Rect>,
}

pub struct ImageByteBuffer {
    pub buf: Vec<u8>,
    pub format: ImageFormat,
}

pub async fn process_image(
    buffer: Vec<u8>,
    format: ImageFormat,
    options: ImageManipulationQuery,
) -> Result<ImageByteBuffer, Error> {
    let ImageManipulationQuery {
        width,
        quality,
        grayscale,
        blur,
        flip,
        contrast,
        brightness,
        hue,
        invert,
        crop,
    } = options;

    tracing::info!(
        options.format = format!("{format:?}"),
        options.width = &width,
        options.quality = &quality,
        options.grayscale = &grayscale,
        options.quality = &quality,
        options.flip = format!("{flip:?}"),
        options.contrast = &contrast,
        options.brightness = &brightness,
        options.invert = &invert,
        options.hue = &hue,
        options.crop = format!("{crop:?}"),
    );

    let mut img = image::load(Cursor::new(buffer), format)?;

    if let Some(width) = width {
        if width > MAX_WITDH {
            return Err(ResponseError::new(
                StatusCode::BAD_REQUEST,
                format!("invalid width, max width is {MAX_WITDH}"),
            )
            .into());
        }

        let height_f = (width as f64 / img.width() as f64) * img.height() as f64;
        let height = height_f as u32;
        img = img.resize_exact(width, height, FilterType::Lanczos3);
    }

    if grayscale {
        img = img.grayscale();
    }

    if let Some(blur) = blur {
        img = img.blur(blur)
    }

    if let Some(flip) = flip {
        img = match flip {
            FlipImage::Vertical => img.flipv(),
            FlipImage::Horizontal => img.fliph(),
        };
    }

    if let Some(contrast) = contrast {
        img = img.adjust_contrast(contrast);
    }

    if let Some(brightness) = brightness {
        img = img.brighten(brightness);
    }

    if let Some(hue) = hue {
        img = img.huerotate(hue);
    }

    if invert {
        img.invert();
    }

    if let Some(crop) = crop {
        img = img.crop_imm(crop.x, crop.y, crop.width, crop.height);
    }

    let total_bytes: usize = (img.width() * img.height()).try_into().unwrap_or(0);
    let mut cursor = Cursor::new(Vec::with_capacity(total_bytes));

    img.write_to(
        &mut cursor,
        ImageOutputFormat::Jpeg(quality.unwrap_or(DEFAULT_QUALITY)),
    )?;

    Ok(ImageByteBuffer {
        buf: cursor.get_ref().to_vec(),
        format: ImageFormat::Jpeg,
    })
}
