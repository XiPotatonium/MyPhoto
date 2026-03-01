use std::path::Path;

use crate::error::AppError;
use crate::models::directory::DirectoryNode;
use crate::models::image::ImageGroup;
use crate::services::{directory_scanner, image_processor};

#[tauri::command]
pub fn scan_directory_tree(root_path: String) -> Result<DirectoryNode, AppError> {
    let path = Path::new(&root_path);
    if !path.is_dir() {
        return Err(AppError::General(format!(
            "Path is not a directory: {}",
            root_path
        )));
    }
    directory_scanner::scan_directory(path).map_err(AppError::from)
}

#[tauri::command]
pub fn list_images(dir_path: String) -> Result<Vec<ImageGroup>, AppError> {
    let path = Path::new(&dir_path);
    if !path.is_dir() {
        return Err(AppError::General(format!(
            "Path is not a directory: {}",
            dir_path
        )));
    }
    image_processor::list_images(path).map_err(AppError::from)
}

#[tauri::command]
pub async fn select_root_directory(
    app: tauri::AppHandle,
) -> Result<Option<String>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder.map(|f| f.to_string()));
    });

    match rx.recv() {
        Ok(result) => Ok(result),
        Err(_) => Ok(None),
    }
}
