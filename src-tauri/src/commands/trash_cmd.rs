use crate::error::AppError;

#[tauri::command]
pub fn move_to_trash(paths: Vec<String>) -> Result<(), AppError> {
    for path_str in &paths {
        let path = std::path::Path::new(path_str);
        if !path.exists() {
            return Err(AppError::General(format!(
                "File not found: {}",
                path_str
            )));
        }
        trash::delete(path).map_err(|e| AppError::Trash(e.to_string()))?;
    }
    Ok(())
}
