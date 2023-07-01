use image::{imageops::FilterType, ImageFormat, ImageOutputFormat};
use lambda_runtime::Error;
use std::io::Cursor;

const DEFAULT_QUALITY: u8 = 100;

#[derive(Debug)]
pub struct ProcessingOptions {
    pub buffer: Vec<u8>,
    pub format: ImageFormat,
    pub width: Option<u32>,
    pub quality: Option<u8>,
}
pub struct ImageByteBuffer {
    pub buf: Vec<u8>,
    pub format: ImageFormat,
}

pub async fn process_image(options: ProcessingOptions) -> Result<ImageByteBuffer, Error> {
    let ProcessingOptions {
        buffer,
        format,
        width,
        quality,
    } = options;

    let img = image::load(Cursor::new(buffer), format)?;

    if let Some(width) = width {
        let height_f = img.width() as f64 / width as f64;
        let height = height_f as u32;
        img.resize(width, height, FilterType::Lanczos3);
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
