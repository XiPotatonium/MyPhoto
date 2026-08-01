use crate::error::AppError;
use crate::models::exif::{ExifInfo, ExifWriteRequest};
use crate::services::exif_service;
use tauri::{AppHandle, Emitter};
use super::validate_file_path;

#[tauri::command]
pub fn read_exif(file_path: String) -> Result<ExifInfo, AppError> {
    let path = validate_file_path(&file_path)?;

    exif_service::read_exif(&path)
}

/// 批量 EXIF 读取的单项输入。
#[derive(serde::Deserialize)]
pub struct ExifBatchItem {
    pub base_name: String,
    pub file_path: String,
}

/// 批量 EXIF 读取的单项结果（字段 snake_case，与 thumbnail-ready 事件保持一致）。
#[derive(serde::Serialize, Clone)]
pub struct ExifBatchResult {
    session_id: String,
    base_name: String,
    exif_info: Option<ExifInfo>,
    error: Option<String>,
}

/// 读取单项 EXIF 并立即推送 `exif-ready` 事件，返回结果以供汇总。
fn read_one_exif(app: &AppHandle, session_id: &str, item: &ExifBatchItem) -> ExifBatchResult {
    let path = std::path::PathBuf::from(&item.file_path);
    let result = match exif_service::read_exif(&path) {
        Ok(info) => ExifBatchResult {
            session_id: session_id.to_string(),
            base_name: item.base_name.clone(),
            exif_info: Some(info),
            error: None,
        },
        Err(e) => ExifBatchResult {
            session_id: session_id.to_string(),
            base_name: item.base_name.clone(),
            exif_info: None,
            error: Some(e.to_string()),
        },
    };
    // 每完成一项立即推送事件到前端
    let _ = app.emit("exif-ready", &result);
    result
}

/// 两段式加载的第二阶段：并行读取一批图片的 EXIF。
///
/// 每完成一项即通过 `exif-ready` 事件推送（前端按 session_id 过滤后就地填充评分角标）；
/// 命令最终返回完整结果 Vec，前端据此对仍缺失的项做补漏回填。
/// 使用 std::thread::scope 做数据并行（无需额外依赖）。
#[tauri::command]
pub async fn read_exif_batch(
    app: AppHandle,
    items: Vec<ExifBatchItem>,
    session_id: String,
) -> Result<Vec<ExifBatchResult>, AppError> {
    let results = tokio::task::spawn_blocking(move || {
        if items.is_empty() {
            return Vec::new();
        }

        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(items.len());

        if num_threads <= 1 {
            return items
                .iter()
                .map(|item| read_one_exif(&app, &session_id, item))
                .collect::<Vec<_>>();
        }

        let chunk_size = (items.len() + num_threads - 1) / num_threads;
        std::thread::scope(|s| {
            let app = &app;
            let session_id = session_id.as_str();
            let handles: Vec<_> = items
                .chunks(chunk_size)
                .map(|chunk| {
                    s.spawn(move || {
                        chunk
                            .iter()
                            .map(|item| read_one_exif(app, session_id, item))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();

            let mut out = Vec::with_capacity(items.len());
            for h in handles {
                if let Ok(chunk_results) = h.join() {
                    out.extend(chunk_results);
                }
            }
            out
        })
    })
    .await
    .map_err(|e| AppError::General(format!("read_exif_batch task failed: {}", e)))?;

    Ok(results)
}

#[tauri::command]
pub fn write_rating(file_path: String, rating: u8) -> Result<(), AppError> {
    if rating > 5 {
        return Err(AppError::General("Rating must be 0-5".to_string()));
    }
    let path = validate_file_path(&file_path)?;

    let req = ExifWriteRequest {
        rating: Some(rating),
        ..Default::default()
    };
    exif_service::write_exif_fields(&[path.to_string_lossy().to_string()], &req)
}

#[tauri::command]
pub fn write_exif_fields(file_paths: Vec<String>, fields: ExifWriteRequest) -> Result<(), AppError> {
    // Validate and canonicalize all paths
    let validated: Result<Vec<String>, AppError> = file_paths
        .into_iter()
        .map(|p| {
            let canonical = validate_file_path(&p)?;
            Ok(canonical.to_string_lossy().to_string())
        })
        .collect();
    let validated_paths = validated?;

    exif_service::write_exif_fields(&validated_paths, &fields)
}
