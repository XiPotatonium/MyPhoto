use std::path::Path;

use crate::error::AppError;
use crate::models::exif::{ExifInfo, ExifWriteRequest};
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

    let req = ExifWriteRequest {
        rating: Some(rating),
        ..Default::default()
    };
    exif_service::write_exif_fields(&[file_path], &req)
}

#[tauri::command]
pub fn write_exif_fields(file_paths: Vec<String>, fields: ExifWriteRequest) -> Result<(), AppError> {
    // Validate all paths exist
    for p in &file_paths {
        let path = Path::new(p);
        if !path.is_file() {
            return Err(AppError::General(format!("File not found: {}", p)));
        }
    }

    exif_service::write_exif_fields(&file_paths, &fields)
}

