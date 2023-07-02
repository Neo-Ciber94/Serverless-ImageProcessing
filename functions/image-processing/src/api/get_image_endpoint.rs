use crate::process_image::FlipImage;
use crate::{error::ResponseError, process_image, ProcessingOptions};
use base64::Engine as _;
use image::ImageFormat;
use lambda_http::{http::HeaderValue, RequestExt};
use lambda_http::{Body, Error, Request, Response};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageManipulationQuery {
    pub source_url: Option<String>,
    pub source_base64: Option<String>,
    pub width: Option<u32>,
    pub quality: Option<u8>,
    pub blur: Option<f32>,
    pub flip: Option<FlipImage>,

    #[serde(default)]
    pub grayscale: bool,
}

pub async fn get_image_endpoint(request: Request) -> Result<Response<Body>, Error> {
    let query_map = request
        .query_string_parameters_ref()
        .ok_or_else(|| ResponseError::new(StatusCode::BAD_REQUEST, "missing image query params"))?;

    let query_str = query_map.to_query_string();
    let mut query: ImageManipulationQuery = serde_qs::from_str(&query_str)
        .map_err(|e| ResponseError::new(StatusCode::BAD_REQUEST, e.to_string()))?;

    if query.source_base64.is_none() && query.source_url.is_none() {
        return Err(ResponseError::new(
            StatusCode::BAD_REQUEST,
            "query string should contains `source_url` or `source_base64`",
        )
        .into());
    }

    if query.source_base64.is_some() && query.source_url.is_some() {
        return Err(ResponseError::new(
            StatusCode::BAD_REQUEST,
            "query string cannot contains both, `source_url` and `source_base64`",
        )
        .into());
    }

    if let Some(url) = query.source_url.take() {
        let (buffer, format) = get_image_bytes_from_url(url).await?;
        get_response_image(buffer, format, query).await
    } else if let Some(base64) = query.source_base64.take() {
        let (buffer, format) = get_image_from_base64(base64).await?;
        get_response_image(buffer, format, query).await
    } else {
        unreachable!()
    }
}

#[tracing::instrument]
async fn get_image_bytes_from_url(url: String) -> Result<(Vec<u8>, ImageFormat), Error> {
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

    Ok((buffer, format))
}

#[tracing::instrument]
async fn get_image_from_base64(base64_text: String) -> Result<(Vec<u8>, ImageFormat), Error> {
    static ERROR_MSG : &str = "failed to get base64 data, expected format: data:image/type;base64,ABCDEFGHIJKLMNOPQRStuvwxyz";
    static DATA_IMAGE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^data:image/?P<type>;base64,?P<data>").expect("failed to build regex")
    });

    let captures = DATA_IMAGE_REGEX
        .captures(&base64_text)
        .ok_or_else(|| ResponseError::new(StatusCode::BAD_REQUEST, ERROR_MSG))?;

    let image_type = captures
        .name("type")
        .ok_or_else(|| ResponseError::new(StatusCode::BAD_REQUEST, ERROR_MSG))?
        .as_str();

    let data = captures
        .name("data")
        .ok_or_else(|| ResponseError::new(StatusCode::BAD_REQUEST, ERROR_MSG))?
        .as_str();

    let format = ImageFormat::from_extension(image_type)
        .ok_or_else(|| Error::from("failed to read format"))?;

    let buffer = base64::engine::general_purpose::STANDARD.decode(data)?;

    Ok((buffer, format))
}

async fn get_response_image(
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
