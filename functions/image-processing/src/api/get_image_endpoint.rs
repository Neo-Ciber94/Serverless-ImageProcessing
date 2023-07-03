use super::get_response_image;
use crate::common::ImageHandlerOptions;
use crate::error::ResponseError;
use image::ImageFormat;
use lambda_http::RequestExt;
use lambda_http::{Body, Error, Request, Response};
use reqwest::{header, StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GetImageQuery {
    pub source_url: Option<String>,
    pub source_base64: Option<String>,

    #[serde(flatten)]
    pub options: ImageHandlerOptions,
}

pub async fn get_image_endpoint(request: Request) -> Result<Response<Body>, Error> {
    tracing::info!("url: {:?}", request.uri().path_and_query());

    let query_map = request
        .query_string_parameters_ref()
        .ok_or_else(|| ResponseError::new(StatusCode::BAD_REQUEST, "missing image query params"))?;

    tracing::info!("query: {query_map:#?}");

    let query_str = query_map.to_query_string();
    let mut query: GetImageQuery = serde_qs::from_str(&query_str)
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
        get_response_image(buffer, format, query.options).await
    } else if let Some(base64) = query.source_base64.take() {
        let (buffer, format) = get_image_bytes_from_base64(base64).await?;
        get_response_image(buffer, format, query.options).await
    } else {
        unreachable!()
    }
}

#[tracing::instrument(level = "INFO")]
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

#[tracing::instrument(level = "INFO")]
async fn get_image_bytes_from_base64(base64_text: String) -> Result<(Vec<u8>, ImageFormat), Error> {
    crate::utils::get_image_from_base64(base64_text).await
}
