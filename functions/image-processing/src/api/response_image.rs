use http::{header, header::HeaderValue};
use image::ImageFormat;
use lambda_http::{Body, Response};
use lambda_runtime::Error;
use crate::common::{image_handler, ImageHandlerOptions};

pub async fn get_response_image(
    buffer: Vec<u8>,
    format: ImageFormat,
    query: ImageHandlerOptions,
) -> Result<Response<Body>, Error> {
    let options = ImageHandlerOptions {
        quality: query.quality,
        width: query.width,
        grayscale: query.grayscale,
        blur: query.blur,
        flip: query.flip,
        brightness: query.brightness,
        contrast: query.contrast,
        hue: query.hue,
        invert: query.invert,
        crop: query.crop,
    };

    let image_buffer = image_handler(buffer, format, options).await?;
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
