mod common;
pub(crate) mod jpg;
pub(crate) mod raf;
pub(crate) mod tiff;
pub(crate) mod dng;
pub(crate) mod bmp;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::exif::{ExifInfo, ExifWriteRequest};

// ── read ──────────────────────────────────────────────────────────────────────

/// Read EXIF info from any supported image file.
/// Dispatches to the appropriate format-specific reader based on file extension.
pub fn read_exif(file_path: &Path) -> Result<ExifInfo, crate::error::AppError> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("raf") => raf::read_exif_raf(file_path),
        Some("dng") => dng::read_exif_dng(file_path),
        Some("tif") | Some("tiff") => tiff::read_exif_tiff(file_path),
        Some("bmp") => bmp::read_exif_bmp(file_path),
        _ => jpg::read_exif_jpg(file_path),
    }
}

// ── write EXIF fields ─────────────────────────────────────────────────────────

/// Write EXIF fields to one or more image files in parallel.
///
/// Only fields with `Some` value in `req` are written; the rest are preserved.
/// When multiple file paths are provided, they are processed concurrently using
/// a thread pool sized to the number of available CPU cores.
///
/// Supported formats: JPEG, RAF, DNG, TIFF.
/// BMP format does not support EXIF and will return an error.
pub fn write_exif_fields(
    file_paths: &[String],
    req: &ExifWriteRequest,
) -> Result<(), crate::error::AppError> {
    // Separate paths by format type
    let mut jpg_paths: Vec<&String> = Vec::new();
    let mut raf_paths: Vec<&String> = Vec::new();
    let mut tiff_paths: Vec<&String> = Vec::new();
    let mut dng_paths: Vec<&String> = Vec::new();
    let mut unsupported_write_paths: Vec<&String> = Vec::new();

    for p in file_paths {
        let ext = Path::new(p)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        match ext.as_deref() {
            Some("raf") => raf_paths.push(p),
            Some("tif") | Some("tiff") => tiff_paths.push(p),
            Some("dng") => dng_paths.push(p),
            Some("bmp") => unsupported_write_paths.push(p),
            _ => jpg_paths.push(p),
        }
    }

    // BMP does not support EXIF write
    if !unsupported_write_paths.is_empty() {
        return Err(crate::error::AppError::General(format!(
            "EXIF write not supported for BMP format: {}",
            unsupported_write_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        )));
    }

    // Process RAF files
    for raf_path in &raf_paths {
        let path = Path::new(raf_path.as_str());
        raf::write_exif_fields_raf(path, req)?;
    }

    // Process TIFF files
    for tiff_path in &tiff_paths {
        let path = Path::new(tiff_path.as_str());
        tiff::write_exif_fields_tiff(path, req)?;
    }

    // Process DNG files
    for dng_path in &dng_paths {
        let path = Path::new(dng_path.as_str());
        dng::write_exif_fields_dng(path, req)?;
    }

    if jpg_paths.is_empty() {
        return Ok(());
    }

    if jpg_paths.len() == 1 {
        // Single file — skip thread overhead
        let path = Path::new(jpg_paths[0].as_str());
        return jpg::write_exif_fields_jpg(path, req);
    }

    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(jpg_paths.len());

    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let chunk_size = (jpg_paths.len() + num_threads - 1) / num_threads;
    let mut handles = vec![];

    for chunk in jpg_paths.chunks(chunk_size) {
        let chunk_owned: Vec<String> = chunk.iter().map(|s| s.to_string()).collect();
        let errors_clone = Arc::clone(&errors);
        let req_clone = req.clone();

        let handle = thread::spawn(move || {
            for file_path_str in chunk_owned {
                let path = Path::new(&file_path_str);
                if let Err(e) = jpg::write_exif_fields_jpg(path, &req_clone) {
                    let mut errs = errors_clone.lock().unwrap();
                    errs.push(format!("{}: {}", file_path_str, e));
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().map_err(|_| {
            crate::error::AppError::General("Thread panicked during EXIF write".to_string())
        })?;
    }

    let errors = errors.lock().unwrap();
    if !errors.is_empty() {
        return Err(crate::error::AppError::General(format!(
            "Failed to write EXIF to {} file(s): {}",
            errors.len(),
            errors.join("; ")
        )));
    }

    Ok(())
}
