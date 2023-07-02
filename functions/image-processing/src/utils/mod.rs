#[cfg(feature = "local")]
pub mod lambda_helper;

mod base64_image;
pub use base64_image::get_image_from_base64;
