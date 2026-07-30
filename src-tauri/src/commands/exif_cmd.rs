use crate::error::AppError;
use crate::models::exif::{ExifInfo, ExifWriteRequest};
use crate::services::exif_service;
use super::validate_file_path;

#[tauri::command]
pub fn read_exif(file_path: String) -> Result<ExifInfo, AppError> {
    let path = validate_file_path(&file_path)?;

    exif_service::read_exif(&path)
}

#[tauri::command]
pub fn write_rating(file_path: String, rating: u8) -> Result<(), AppError> {
    if rating > 5 {
        return Err(AppError::General("Rating must be 0-5".to_string()));
    }
    let path = validate_file_path(&file_path)?;

    let req = ExifWriteRequest {
        rating: Some(rating),
        ..Default::default()
    };
    exif_service::write_exif_fields(&[path.to_string_lossy().to_string()], &req)
}

#[tauri::command]
pub fn write_exif_fields(file_paths: Vec<String>, fields: ExifWriteRequest) -> Result<(), AppError> {
    // Validate and canonicalize all paths
    let validated: Result<Vec<String>, AppError> = file_paths
        .into_iter()
        .map(|p| {
            let canonical = validate_file_path(&p)?;
            Ok(canonical.to_string_lossy().to_string())
        })
        .collect();
    let validated_paths = validated?;

    exif_service::write_exif_fields(&validated_paths, &fields)
}

