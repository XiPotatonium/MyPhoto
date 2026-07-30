use std::path::PathBuf;

use crate::error::AppError;
use crate::services::thumbnail_service;
use tauri::{AppHandle, Emitter};
use super::validate_file_path;

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
    use futures::stream::{self, StreamExt};
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    // 限制并发数，避免大量文件同时生成缩略图导致资源耗尽
    let semaphore = Arc::new(Semaphore::new(8));

    stream::iter(file_paths)
        .map(|file_path| {
            let app_handle = app.clone();
            let sem = semaphore.clone();
            async move {
                // 获取信号量许可，限制并发上限
                let _permit = sem.acquire().await.unwrap();

                let path = PathBuf::from(&file_path);
                let base_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                // 轻量检查：路径存在且为文件（不做 canonicalize）
                let result = if path.exists() && path.is_file() {
                    match thumbnail_service::generate_thumbnail(&path).await {
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
            }
        })
        .buffer_unordered(8)
        .collect::<()>()
        .await;

    Ok(())
}

#[tauri::command]
pub async fn read_image_file(file_path: String) -> Result<tauri::ipc::Response, AppError> {
    let path = validate_file_path(&file_path)?;
    // 使用 spawn_blocking 包裹同步文件 I/O，避免阻塞 tokio async runtime
    let bytes = tokio::task::spawn_blocking(move || {
        std::fs::read(&path).map_err(|e| AppError::Io(e))
    })
    .await
    .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))??;
    Ok(tauri::ipc::Response::new(bytes))
}
