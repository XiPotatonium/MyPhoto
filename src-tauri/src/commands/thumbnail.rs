use std::path::Path;

use crate::error::AppError;
use crate::services::thumbnail_service;
use tauri::{AppHandle, Emitter};

#[derive(serde::Serialize, Clone)]
struct ThumbnailResult {
    base_name: String,
    thumbnail: Option<String>,
    error: Option<String>,
}

#[tauri::command]
pub async fn generate_thumbnails_batch(
    app: AppHandle,
    file_paths: Vec<String>,
) -> Result<(), AppError> {
    use tokio::task::spawn;
    use futures::stream::{FuturesUnordered, StreamExt};

    let mut tasks = FuturesUnordered::new();

    for file_path in file_paths {
        let app_handle = app.clone();
        tasks.push(spawn(async move {
            let path = Path::new(&file_path);
            let base_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            let result = if path.is_file() {
                match thumbnail_service::generate_thumbnail(path) {
                    Ok(thumb) => ThumbnailResult {
                        base_name: base_name.clone(),
                        thumbnail: Some(thumb),
                        error: None,
                    },
                    Err(e) => ThumbnailResult {
                        base_name: base_name.clone(),
                        thumbnail: None,
                        error: Some(e.to_string()),
                    },
                }
            } else {
                ThumbnailResult {
                    base_name: base_name.clone(),
                    thumbnail: None,
                    error: Some("File not found".to_string()),
                }
            };

            // 每完成一个立即发送事件到前端
            let _ = app_handle.emit("thumbnail-ready", &result);
        }));
    }

    // 并发执行所有任务
    while tasks.next().await.is_some() {}

    Ok(())
}

#[tauri::command]
pub async fn read_image_file(file_path: String) -> Result<String, AppError> {
    let path = Path::new(&file_path).to_path_buf();
    if !path.is_file() {
        return Err(AppError::General(format!(
            "File not found: {}",
            file_path
        )));
    }
    // 在阻塞线程池中执行图片读取操作，避免阻塞主线程
    tokio::task::spawn_blocking(move || thumbnail_service::read_full_image(&path))
        .await
        .map_err(|e| AppError::General(format!("Task join error: {}", e)))?
}
