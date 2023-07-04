use super::{CropRect, FlipImage};
use crate::error::ResponseError;
use image::{imageops::FilterType, ImageFormat, ImageOutputFormat};
use lambda_runtime::Error;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_aux::prelude::*;
use std::io::Cursor;

const DEFAULT_QUALITY: u8 = 100;
const MAX_WITDH: u32 = 10_000;

#[derive(Debug, Deserialize)]
pub struct ImageHandlerOptions {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub width: Option<u32>,

    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub quality: Option<u8>,

    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub blur: Option<f32>,

    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub brightness: Option<i32>,

    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub contrast: Option<f32>,

    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub hue: Option<i32>,

    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub sharp: Option<f32>,

    pub flip: Option<FlipImage>,

    pub flop: Option<FlipImage>,

    #[serde(default, deserialize_with = "deserialize_bool_from_anything")]
    pub grayscale: bool,

    #[serde(default, deserialize_with = "deserialize_bool_from_anything")]
    pub invert: bool,

    #[serde(flatten)]
    pub crop: Option<CropRect>,
}

pub struct ImageByteBuffer {
    pub buf: Vec<u8>,
    pub format: ImageFormat,
}

#[tracing::instrument(skip(image_buffer), level = "INFO")]
pub async fn image_handler(
    image_buffer: Vec<u8>,
    image_format: ImageFormat,
    options: ImageHandlerOptions,
) -> Result<ImageByteBuffer, Error> {
    let ImageHandlerOptions {
        width,
        quality,
        grayscale,
        blur,
        flip,
        flop,
        contrast,
        brightness,
        sharp,
        hue,
        invert,
        crop,
    } = options;

    let mut img = image::load(Cursor::new(image_buffer), image_format)?;

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

    if let Some(sharp) = sharp {
        img = img.unsharpen(sharp, 1);
    }

    if let Some(flip) = flip {
        img = match flip {
            FlipImage::Vertical => img.flipv(),
            FlipImage::Horizontal => img.fliph(),
        };
    }

    if let Some(flop) = flop {
        img = match flop {
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
        img = img.crop_imm(crop.crop_x, crop.crop_y, crop.crop_width, crop.crop_height);
    }

    let total_bytes: usize = (img.width() * img.height()).try_into().unwrap_or(0);
    let mut cursor = Cursor::new(Vec::with_capacity(total_bytes));

    img.write_to(
        &mut cursor,
        ImageOutputFormat::Jpeg(quality.unwrap_or(DEFAULT_QUALITY)),
    )?;

    Ok(ImageByteBuffer {
        buf: cursor.into_inner(),
        format: ImageFormat::Jpeg,
    })
}
