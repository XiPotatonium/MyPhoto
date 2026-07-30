pub mod exif_cmd;
pub mod filesystem;
pub mod thumbnail;
pub mod trash_cmd;

use std::path::{Path, PathBuf};
use crate::error::AppError;

/// Canonicalize a file path and verify it points to an existing file.
/// Returns the normalized path to prevent path injection.
pub fn validate_file_path(raw: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(raw);
    let canonical = std::fs::canonicalize(path).map_err(|_| {
        AppError::General(format!("Invalid or inaccessible path: {}", raw))
    })?;
    if !canonical.is_file() {
        return Err(AppError::General(format!("Path is not a file: {}", raw)));
    }
    Ok(canonical)
}

/// Canonicalize a directory path and verify it points to an existing directory.
/// Returns the normalized path to prevent path injection.
pub fn validate_dir_path(raw: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(raw);
    let canonical = std::fs::canonicalize(path).map_err(|_| {
        AppError::General(format!("Invalid or inaccessible path: {}", raw))
    })?;
    if !canonical.is_dir() {
        return Err(AppError::General(format!("Path is not a directory: {}", raw)));
    }
    Ok(canonical)
}
