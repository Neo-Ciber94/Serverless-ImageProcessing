use http::{header, header::HeaderValue};
use image::ImageFormat;
use lambda_http::{Body, Response};
use lambda_runtime::Error;

use crate::{
    process_image::{process_image, FlipImage},
    ProcessingOptions,
};

#[derive(Debug)]
pub struct ImageManipulationQuery {
    pub width: Option<u32>,
    pub quality: Option<u8>,
    pub blur: Option<f32>,
    pub flip: Option<FlipImage>,
    pub grayscale: bool,
}

pub async fn get_response_image(
    buffer: Vec<u8>,
    format: ImageFormat,
    query: ImageManipulationQuery,
) -> Result<Response<Body>, Error> {
    let options = ProcessingOptions {
        buffer,
        format,
        quality: query.quality,
        width: query.width,
        grayscale: query.grayscale,
        blur: query.blur,
        flip: query.flip,
    };

    let image_buffer = process_image(options).await?;
    let image_format: ImageFormat = image_buffer.format.into();
    let res_content_type = format!("image/{}", image_format.extensions_str()[0]);

    let body = Body::Binary(image_buffer.buf);
    Response::builder()
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&res_content_type).unwrap(),
        )
        .body(body)
        .map_err(Error::from)
}
