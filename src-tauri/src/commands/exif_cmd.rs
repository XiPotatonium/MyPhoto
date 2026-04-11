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
        _ => exif_service::write_rating(path, rating),
    }
}

#[tauri::command]
pub fn write_exif_fields(file_path: String, fields: ExifWriteRequest) -> Result<(), AppError> {
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
            "Writing EXIF fields to RAF files is not yet implemented".to_string(),
        )),
        _ => exif_service::write_exif_fields(path, &fields),
    }
}

#[tauri::command]
pub fn batch_write_gps(
    paths: Vec<String>,
    latitude: f64,
    longitude: f64,
) -> Result<(), AppError> {
    // Separate paths by format
    let mut jpg_paths: Vec<String> = Vec::new();
    let mut raf_paths: Vec<String> = Vec::new();

    for p in &paths {
        let ext = Path::new(p)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        if ext.as_deref() == Some("raf") {
            raf_paths.push(p.clone());
        } else {
            jpg_paths.push(p.clone());
        }
    }

    // Process JPG files
    if !jpg_paths.is_empty() {
        exif_service::batch_write_gps(&jpg_paths, latitude, longitude)?;
    }

    // RAF files: not yet implemented
    if !raf_paths.is_empty() {
        return Err(AppError::General(format!(
            "Writing GPS to RAF files is not yet implemented ({} file(s) skipped)",
            raf_paths.len()
        )));
    }

    Ok(())
}
