#[cfg(feature = "local")]
pub mod lambda_helper;

mod get_base64_image;
pub use get_base64_image::get_image_from_base64;
