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

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("raf") => Err(AppError::General(
            "Writing rating to RAF files is not yet implemented".to_string(),
        )),
        _ => {
            let req = ExifWriteRequest {
                rating: Some(rating),
                ..Default::default()
            };
            exif_service::write_exif_fields(&[file_path], &req)
        }
    }
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

    // Separate RAF and non-RAF paths
    let (jpg_paths, raf_paths): (Vec<String>, Vec<String>) = file_paths
        .into_iter()
        .partition(|p| {
            let ext = Path::new(p)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            ext.as_deref() != Some("raf")
        });

    if !raf_paths.is_empty() {
        return Err(AppError::General(format!(
            "Writing EXIF fields to RAF files is not yet implemented ({} file(s) skipped)",
            raf_paths.len()
        )));
    }

    exif_service::write_exif_fields(&jpg_paths, &fields)
}

