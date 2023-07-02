use super::get_response_image;
use crate::common::{FlipImage, ImageHandlerOptions, CropRect};
use crate::error::ResponseError;
use crate::utils::get_image_from_base64;
use image::ImageFormat;
use lambda_http::RequestExt;
use lambda_http::{Body, Error, Request, Response};
use multer::parse_boundary;
use reqwest::{header, StatusCode};
use serde::Deserialize;
use std::convert::Infallible;

#[derive(Debug, Deserialize)]
struct InputQuery {
    pub width: Option<u32>,
    pub quality: Option<u8>,
    pub blur: Option<f32>,
    pub flip: Option<FlipImage>,
    pub brightness: Option<i32>,
    pub contrast: Option<f32>,
    pub hue: Option<i32>,

    #[serde(flatten)]
    pub crop: Option<CropRect>,

    #[serde(default)]
    pub grayscale: bool,

    #[serde(default)]
    pub invert: bool,
}

impl From<InputQuery> for ImageHandlerOptions {
    fn from(value: InputQuery) -> Self {
        ImageHandlerOptions {
            width: value.width,
            quality: value.quality,
            flip: value.flip,
            grayscale: value.grayscale,
            blur: value.blur,
            brightness: value.brightness,
            contrast: value.contrast,
            hue: value.hue,
            invert: value.invert,
            crop: value.crop,
        }
    }
}

#[allow(dead_code)]
struct FormFile {
    file_name: String,
    bytes: Vec<u8>,
    content_type: String,
}

pub async fn post_image_endpoint(request: Request) -> Result<Response<Body>, Error> {
    let query_map = request.query_string_parameters();
    let query_str = query_map.to_query_string();
    let query: InputQuery = serde_qs::from_str(&query_str)
        .map_err(|e| ResponseError::new(StatusCode::BAD_REQUEST, e.to_string()))?;

    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .ok_or_else(|| {
            ResponseError::new(StatusCode::BAD_REQUEST, "missing content-type boundary")
        })?
        .to_str()
        .map_err(ResponseError::from_error)?;

    let mime: mime::Mime = content_type.parse().map_err(ResponseError::from_error)?;
    let bytes = request.body().to_vec();

    let (buffer, format) = if mime == mime::APPLICATION_JSON {
        get_body_base64_bytes(bytes).await?
    } else {
        get_form_file_bytes(bytes, content_type).await?
    };

    get_response_image(buffer, format, query.into()).await
}

async fn get_body_base64_bytes(body: Vec<u8>) -> Result<(Vec<u8>, ImageFormat), Error> {
    #[derive(Debug, Deserialize)]
    struct Data {
        base64_data: String,
    }

    let data = serde_json::from_slice::<Data>(&body)?;
    get_image_from_base64(data.base64_data).await
}

async fn get_form_file_bytes(
    body: Vec<u8>,
    content_type: &str,
) -> Result<(Vec<u8>, ImageFormat), Error> {
    let boundary = parse_boundary(content_type).map_err(ResponseError::from_error)?;
    let mut multipart = multer::Multipart::new(
        futures::stream::once(async move { Ok::<_, Infallible>(body) }),
        boundary,
    );

    let mut form_file: Option<FormFile> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(file_name) = field.file_name() {
            if form_file.is_some() {
                return Err(ResponseError::new(
                    StatusCode::BAD_REQUEST,
                    "expected 1 file but received more than 1",
                )
                .into());
            }

            let file_name = file_name.to_owned();
            let mime_type = field.content_type().ok_or_else(|| {
                ResponseError::new(StatusCode::BAD_REQUEST, "unable to get file content-type")
            })?;

            let content_type = mime_type.essence_str().to_owned();
            let bytes = field.bytes().await?.to_vec();
            form_file = Some(FormFile {
                file_name,
                bytes,
                content_type,
            });
        }
    }

    match form_file {
        Some(file) => {
            let buffer = file.bytes;
            let format = ImageFormat::from_mime_type(&file.content_type)
                .ok_or_else(|| ResponseError::new(StatusCode::BAD_REQUEST, "expected image"))?;

            Ok((buffer, format))
        }
        None => Err(ResponseError::new(StatusCode::BAD_REQUEST, "no file").into()),
    }
}
