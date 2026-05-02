mod read_exif_service;
mod write_exif_service;

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
        Some("raf") => read_exif_service::read_exif_raf(file_path),
        _ => read_exif_service::read_exif_jpg(file_path),
    }
}

// ── write EXIF fields ─────────────────────────────────────────────────────────

/// Write EXIF fields to one or more image files in parallel.
///
/// Only fields with `Some` value in `req` are written; the rest are preserved.
/// When multiple file paths are provided, they are processed concurrently using
/// a thread pool sized to the number of available CPU cores.
///
/// Both JPEG and RAF (Fujifilm RAW) files are supported for writing.
pub fn write_exif_fields(
    file_paths: &[String],
    req: &ExifWriteRequest,
) -> Result<(), crate::error::AppError> {
    // Separate RAF and non-RAF paths
    let mut jpg_paths: Vec<&String> = Vec::new();
    let mut raf_paths: Vec<&String> = Vec::new();

    for p in file_paths {
        let ext = Path::new(p)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        if ext.as_deref() == Some("raf") {
            raf_paths.push(p);
        } else {
            jpg_paths.push(p);
        }
    }

    // Process RAF files (currently returns NotImplemented error)
    for raf_path in &raf_paths {
        let path = Path::new(raf_path.as_str());
        write_exif_service::write_exif_fields_raf(path, req)?;
    }

    if jpg_paths.is_empty() {
        return Ok(());
    }

    if jpg_paths.len() == 1 {
        // Single file — skip thread overhead
        let path = Path::new(jpg_paths[0].as_str());
        return write_exif_service::write_exif_fields_jpg(path, req);
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
                if let Err(e) = write_exif_service::write_exif_fields_jpg(path, &req_clone) {
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
