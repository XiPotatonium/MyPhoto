use serde::{Deserialize, Serialize};

/// Request payload for writing EXIF fields to an image file.
/// All fields are optional; only non-None fields will be written.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifWriteRequest {
    pub datetime: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<f32>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<f32>,
    pub iso: Option<u32>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub rating: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifInfo {
    pub datetime: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<f32>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<f32>,
    pub iso: Option<u32>,
    pub rating: Option<u8>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
}

impl Default for ExifInfo {
    fn default() -> Self {
        Self {
            datetime: None,
            gps_latitude: None,
            gps_longitude: None,
            camera_make: None,
            camera_model: None,
            lens_model: None,
            focal_length: None,
            shutter_speed: None,
            aperture: None,
            iso: None,
            rating: None,
            image_width: None,
            image_height: None,
        }
    }
}
