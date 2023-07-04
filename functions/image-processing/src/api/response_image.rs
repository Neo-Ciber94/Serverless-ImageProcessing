use crate::common::{image_handler, ImageHandlerOptions};
use http::{header, header::HeaderValue};
use image::ImageFormat;
use lambda_http::{Body, Response};
use lambda_runtime::Error;

pub async fn get_response_image(
    buffer: Vec<u8>,
    format: ImageFormat,
    options: ImageHandlerOptions,
) -> Result<Response<Body>, Error> {
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
