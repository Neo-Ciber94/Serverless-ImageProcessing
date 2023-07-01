use image::{imageops::FilterType, ImageFormat, ImageOutputFormat};
use lambda_runtime::tower::BoxError;
use std::io::Cursor;
use thiserror::Error;

const DEFAULT_QUALITY: u8 = 100;
const MAX_WITDH: u32 = 10_000;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ProcessImageError(pub String);
impl ProcessImageError {
    pub fn msg(message: impl Into<String>) -> Self {
        ProcessImageError(message.into())
    }
}

#[derive(Debug)]
pub struct ProcessingOptions {
    pub buffer: Vec<u8>,
    pub format: ImageFormat,
    pub width: Option<u32>,
    pub quality: Option<u8>,
    pub grayscale: bool,
    pub blur: Option<f32>,
}

pub struct ImageByteBuffer {
    pub buf: Vec<u8>,
    pub format: ImageFormat,
}

pub async fn process_image(options: ProcessingOptions) -> Result<ImageByteBuffer, BoxError> {
    let ProcessingOptions {
        buffer,
        format,
        width,
        quality,
        grayscale,
        blur,
    } = options;

    tracing::info!(format = "{format:?}", width = &width, quality = &quality);

    let mut img = image::load(Cursor::new(buffer), format)?;

    if let Some(width) = width {
        if width > MAX_WITDH {
            return Err(
                ProcessImageError::msg(format!("invalid width, max width is {MAX_WITDH}")).into(),
            );
        }

        let height_f = (width as f64 / img.width() as f64) * img.height() as f64;
        let height = height_f as u32;
        img = img.resize(width, height, FilterType::Lanczos3);
    }

    if grayscale {
        img = img.grayscale();
    }

    if let Some(blur) = blur {
        img = img.blur(blur)
    }

    let mut cursor = Cursor::new(vec![]);
    img.write_to(
        &mut cursor,
        ImageOutputFormat::Jpeg(quality.unwrap_or(DEFAULT_QUALITY)),
    )?;

    Ok(ImageByteBuffer {
        buf: cursor.get_ref().to_vec(),
        format: ImageFormat::Jpeg,
    })
}
