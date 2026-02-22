use std::path::Path;

use crate::error::AppError;
use crate::models::exif::ExifInfo;
use crate::services::exif_service;

#[tauri::command]
pub fn read_exif(file_path: String) -> Result<ExifInfo, AppError> {
    let path = Path::new(&file_path);
    if !path.is_file() {
        return Err(AppError::General(format!(
            "File not found: {}",
            file_path
        )));
    }
    exif_service::read_exif(path)
}

#[tauri::command]
pub fn write_rating(file_path: String, rating: u8) -> Result<(), AppError> {
    if rating > 5 {
        return Err(AppError::General("Rating must be 0-5".to_string()));
    }
    let path = Path::new(&file_path);
    if !path.is_file() {
        return Err(AppError::General(format!(
            "File not found: {}",
            file_path
        )));
    }
    exif_service::write_rating(path, rating)
}

#[tauri::command]
pub fn batch_write_gps(
    paths: Vec<String>,
    latitude: f64,
    longitude: f64,
) -> Result<(), AppError> {
    exif_service::batch_write_gps(&paths, latitude, longitude)
}
