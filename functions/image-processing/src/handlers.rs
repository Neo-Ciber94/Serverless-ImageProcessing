use image::ImageFormat;
use image_processing::{process_image, ProcessingOptions};
use lambda_http::{http::HeaderValue, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use reqwest::header;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageManipulationQuery {
    pub source_url: Option<String>,
    pub source_base64: Option<String>,
    pub width: Option<u32>,
    pub quality: Option<u8>,
}

pub async fn image_handler(request: Request) -> Result<Response<Body>, Error> {
    let query_map = request
        .query_string_parameters_ref()
        .ok_or_else(|| Error::from("missing image query params"))?;

    let query_str = query_map.to_query_string();
    let mut query: ImageManipulationQuery = serde_qs::from_str(&query_str).map_err(Box::new)?;

    if query.source_base64.is_none() && query.source_url.is_none() {
        return Err(Error::from(
            "query string should contains `source_url` or `source_base64`",
        ));
    } else if query.source_base64.is_some() && query.source_url.is_some() {
        return Err(Error::from(
            "query string cannot contains both, `source_url` and `source_base64`",
        ));
    } else if let Some(url) = query.source_url.take() {
        handler_image_from_url(url, query).await
    } else if let Some(base64) = query.source_base64.take() {
        handler_image_from_base64(base64, query).await
    } else {
        unreachable!()
    }
}

async fn handler_image_from_url(
    url: String,
    query: ImageManipulationQuery,
) -> Result<Response<Body>, Error> {
    let res = reqwest::get(url).await?;

    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .ok_or_else(|| Error::from("content type not found"))?;

    let buffer = res.bytes().await?.to_vec();
    let content_type_str = content_type.to_str().map_err(Error::from)?;
    let format = ImageFormat::from_mime_type(content_type_str)
        .ok_or_else(|| Error::from("failed to read format"))?;

    let options = ProcessingOptions {
        buffer,
        format,
        quality: query.quality,
        width: query.width,
    };

    let image_buffer = process_image(options).await?;
    let image_format: ImageFormat = image_buffer.format.into();
    let body = Body::Binary(image_buffer.buf);
    let res_content_type = format!("image/{}", image_format.extensions_str()[0]);

    Response::builder()
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&res_content_type).unwrap(),
        )
        .body(body)
        .map_err(Error::from)
}

async fn handler_image_from_base64(
    base64: String,
    query: ImageManipulationQuery,
) -> Result<Response<Body>, Error> {
    todo!()
}
