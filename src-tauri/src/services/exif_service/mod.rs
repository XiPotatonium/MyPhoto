mod read_exif_service;
mod write_gps_service;
mod write_rating_service;

use std::path::Path;

use crate::models::exif::ExifInfo;

// ── read ──────────────────────────────────────────────────────────────────────

/// Read EXIF info from any supported image file.
/// Dispatches to the appropriate format-specific reader based on file extension.
pub fn read_exif(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("raf") => read_exif_service::read_exif_raf(file_path),
        _ => read_exif_service::read_exif_jpg(file_path),
    }
}

// ── write GPS ─────────────────────────────────────────────────────────────────

/// Write GPS coordinates to a batch of files in parallel.
///
/// Currently only JPEG files are supported.
pub fn batch_write_gps(
    file_paths: &[String],
    latitude: f64,
    longitude: f64,
) -> Result<(), crate::error::AppError> {
    write_gps_service::batch_write_gps(file_paths, latitude, longitude)
}

// ── write rating ──────────────────────────────────────────────────────────────

/// Write a star rating (0–5) to an image file.
///
/// Currently only JPEG files are supported; RAF support is not yet implemented.
pub fn write_rating(file_path: &Path, rating: u8) -> Result<(), crate::error::AppError> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("raf") => Err(crate::error::AppError::General(
            "Writing rating to RAF files is not yet implemented".to_string(),
        )),
        _ => write_rating_service::write_rating_jpg(file_path, rating),
    }
}
