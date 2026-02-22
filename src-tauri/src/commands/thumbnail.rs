use std::path::Path;

use crate::error::AppError;
use crate::services::thumbnail_service;

#[tauri::command]
pub fn generate_thumbnail(file_path: String) -> Result<String, AppError> {
    let path = Path::new(&file_path);
    if !path.is_file() {
        return Err(AppError::General(format!(
            "File not found: {}",
            file_path
        )));
    }
    thumbnail_service::generate_thumbnail(path)
}

#[tauri::command]
pub fn read_image_file(file_path: String) -> Result<String, AppError> {
    let path = Path::new(&file_path);
    if !path.is_file() {
        return Err(AppError::General(format!(
            "File not found: {}",
            file_path
        )));
    }
    thumbnail_service::read_full_image(path)
}
