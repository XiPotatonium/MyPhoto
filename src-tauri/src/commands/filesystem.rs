use crate::error::AppError;
use crate::models::image::{DirectoryContents, ImageGroup};
use crate::services::image_processor;
use super::validate_dir_path;

/// 0.3.0：文件浏览器改为直接浏览目录，一次返回子文件夹与图片组。
/// 注册加载会话的逻辑与 list_images 一致，保证切换目录时旧批量任务立即失效。
#[tauri::command]
pub async fn list_directory_contents(
    dir_path: String,
    session_id: String,
) -> Result<DirectoryContents, AppError> {
    super::begin_load_session(&session_id);
    let path = validate_dir_path(&dir_path)?;
    tokio::task::spawn_blocking(move || image_processor::list_directory_contents(&path))
        .await
        .map_err(|e| AppError::General(format!("Task join error: {}", e)))?
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn list_images(
    dir_path: String,
    session_id: String,
) -> Result<Vec<ImageGroup>, AppError> {
    // 注册本次加载会话，让上一轮遗留的缩略图 / EXIF 批量任务尽快退出。
    // 放在最前面（校验之前，且放在 list_images 而非两个 batch 命令里）：
    // 任何一次点击都会取消旧任务，空文件夹也必定调用到本命令。
    super::begin_load_session(&session_id);
    let path = validate_dir_path(&dir_path)?;
    // 将阻塞的目录扫描与 HDR 头检测放入 spawn_blocking，避免阻塞 async runtime
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
