use crate::error::AppError;
use crate::models::directory::DirectoryNode;
use crate::models::image::ImageGroup;
use crate::services::{directory_scanner, image_processor};
use super::{validate_dir_path};

#[tauri::command]
pub fn scan_directory_tree(root_path: String) -> Result<DirectoryNode, AppError> {
    let path = validate_dir_path(&root_path)?;
    directory_scanner::scan_directory(&path).map_err(AppError::from)
}

#[tauri::command]
pub async fn list_images(dir_path: String) -> Result<Vec<ImageGroup>, AppError> {
    let path = validate_dir_path(&dir_path)?;
    // 将阻塞的目录扫描和 EXIF 解析放入 spawn_blocking，避免阻塞 async runtime
    tokio::task::spawn_blocking(move || image_processor::list_images(&path))
        .await
        .map_err(|e| AppError::General(format!("Task join error: {}", e)))?
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn select_root_directory(
    app: tauri::AppHandle,
) -> Result<Option<String>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder.map(|f| f.to_string()));
    });

    match rx.await {
        Ok(result) => Ok(result),
        Err(_) => Ok(None),
    }
}
