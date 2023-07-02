use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FlipImage {
    Vertical,
    Horizontal,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CropRect {
    pub crop_x: u32,
    pub crop_y: u32,
    pub crop_width: u32,
    pub crop_height: u32,
}
