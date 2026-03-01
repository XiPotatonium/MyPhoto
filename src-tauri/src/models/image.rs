use serde::{Deserialize, Serialize};

use crate::models::exif::ExifInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGroup {
    pub base_name: String,
    pub jpg_path: Option<String>,
    pub raw_path: Option<String>,
    pub file_count: u32,
    pub modified_time: Option<String>,
    pub file_created_time: Option<String>,
    pub exif_info: Option<ExifInfo>,
}
