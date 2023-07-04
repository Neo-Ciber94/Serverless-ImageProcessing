use serde_aux::prelude::*;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlipImage {
    Vertical,
    Horizontal,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct CropRect {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub crop_x: u32,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub crop_y: u32,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub crop_width: u32,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub crop_height: u32,
}
