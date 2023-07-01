use crate::{error::ResponseError, process_image, ProcessImageError, ProcessingOptions};
use image::ImageFormat;
use lambda_http::{http::HeaderValue, RequestExt};
use lambda_http::{Body, Error, Request, Response};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageManipulationQuery {
    pub source_url: Option<String>,
    pub source_base64: Option<String>,
    pub width: Option<u32>,
    pub quality: Option<u8>,
    pub blur: Option<f32>,

    #[serde(default)]
    pub grayscale: bool,
}

pub async fn get_image_endpoint(request: Request) -> Result<Response<Body>, ResponseError> {
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
        ));
    }

    if query.source_base64.is_some() && query.source_url.is_some() {
        return Err(ResponseError::new(
            StatusCode::BAD_REQUEST,
            "query string cannot contains both, `source_url` and `source_base64`",
        ));
    }

    let image_data_result = if let Some(url) = query.source_url.take() {
        get_image_bytes_from_url(url).await
    } else if let Some(base64) = query.source_base64.take() {
        get_image_from_base64(base64).await
    } else {
        unreachable!()
    };

    let (buffer, format) = image_data_result.map_err(ResponseError::from_error)?;

    match get_response_image(buffer, format, query).await {
        Ok(x) => Ok(x),
        Err(err) => {
            if err.is::<ProcessImageError>() {
                let err = *err.downcast::<ProcessImageError>().unwrap();
                Err(ResponseError::new(StatusCode::BAD_REQUEST, err.0))
            } else {
                Err(ResponseError::from_error(err))
            }
        }
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
async fn get_image_from_base64(base64: String) -> Result<(Vec<u8>, ImageFormat), Error> {
    todo!()
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