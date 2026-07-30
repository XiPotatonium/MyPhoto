use crate::error::AppError;
use super::validate_file_path;

#[tauri::command]
pub fn move_to_trash(paths: Vec<String>) -> Result<(), AppError> {
    for path_str in &paths {
        let canonical = validate_file_path(path_str)?;
        trash::delete(&canonical).map_err(|e| AppError::Trash(e.to_string()))?;
    }
    Ok(())
}
