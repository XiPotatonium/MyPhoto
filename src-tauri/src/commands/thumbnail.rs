use std::path::PathBuf;

use crate::error::AppError;
use crate::services::thumbnail_service;
use tauri::{AppHandle, Emitter};
use super::{validate_file_path, is_load_session_active};

#[derive(serde::Serialize, Clone)]
struct ThumbnailResult {
    session_id: String,
    base_name: String,
    thumbnail: Option<String>,
    error: Option<String>,
}

#[tauri::command]
pub async fn generate_thumbnails_batch(
    app: AppHandle,
    file_paths: Vec<String>,
    session_id: String,
) -> Result<(), AppError> {
    use futures::stream::{self, StreamExt};
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    // 限制并发数，避免大量文件同时生成缩略图导致资源耗尽
    let semaphore = Arc::new(Semaphore::new(8));
    let session_id = Arc::new(session_id);
    let session_guard = Arc::clone(&session_id);

    stream::iter(file_paths)
        // 用户已切到别的文件夹时立即终止整条任务流，不再继续解码剩余文件。
        // 只靠前端过滤事件是不够的：旧任务仍会占满 8 个并发槽与磁盘 IO。
        .take_while(move |_| {
            let active = is_load_session_active(session_guard.as_str());
            async move { active }
        })
        .map(|file_path| {
            let app_handle = app.clone();
            let sem = semaphore.clone();
            let session_id = session_id.clone();
            async move {
                if !is_load_session_active(session_id.as_str()) {
                    return;
                }
                // 获取信号量许可，限制并发上限
                let _permit = sem.acquire().await.unwrap();
                // 等待许可期间可能已切换文件夹
                if !is_load_session_active(session_id.as_str()) {
                    return;
                }

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
                            session_id: (*session_id).clone(),
                            base_name: base_name.clone(),
                            thumbnail: Some(thumb),
                            error: None,
                        },
                        Err(e) => ThumbnailResult {
                            session_id: (*session_id).clone(),
                            base_name: base_name.clone(),
                            thumbnail: None,
                            error: Some(e.to_string()),
                        },
                    }
                } else {
                    ThumbnailResult {
                        session_id: (*session_id).clone(),
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
    // 交由 read_image_bytes 按格式分流：
    // - jpg/jpeg/png 返回原始文件字节（保留 Ultra HDR gain map 等附加数据）
    // - RAW(raf/dng/tif/tiff/bmp) 解码后重编码为 JPEG，供浏览器 <img> 渲染
    // read_image_bytes 内部已按需处理阻塞 I/O，此处直接 await
    let bytes = thumbnail_service::read_image_bytes(&path).await?;
    Ok(tauri::ipc::Response::new(bytes))
}
