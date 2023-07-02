use http::StatusCode;
use image::ImageFormat;
use lambda_runtime::Error;
use once_cell::sync::Lazy;
use regex::Regex;
use base64::Engine as _;
use crate::error::ResponseError;

pub async fn get_image_from_base64(base64_text: String) -> Result<(Vec<u8>, ImageFormat), Error> {
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
