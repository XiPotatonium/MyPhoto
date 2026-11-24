//! HDR 转换相关命令。
//!
//! - `plan_hdr_conversion`：右键菜单打开弹窗时调用，统计可转换 / 不支持 /
//!   会被覆盖的数量，供前端展示确认信息。
//! - `convert_hdr_batch`：确认后调用，后台并发转换并通过 `task-progress`
//!   事件推送进度。

use std::path::Path;

use tauri::{AppHandle, Emitter};

use super::{batch_concurrency, validate_file_path, TaskProgressEvent};
use crate::error::AppError;
use crate::services::hdr_service;

/// HDR 转换计划，字段 camelCase 以匹配前端。
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HdrPlan {
    /// 选中的照片总数（与前端选中数量一致）。
    pub total: usize,
    /// 支持转换的 TIF/BMP 数量。
    pub convertible: usize,
    /// 不支持转换的文件名（JPG / PNG / RAW 等）。
    pub unsupported: Vec<String>,
    /// 已存在同名 JPG、转换时会被覆盖的文件名（输出名）。
    pub overwrite: Vec<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HdrConvertResult {
    pub input_path: String,
    pub output_path: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

fn file_name_of(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}

/// 统计选中的图片里有多少张能转 HDR、哪些不支持、哪些会覆盖已有 JPG。
///
/// 每张照片传入一个路径（优先 TIF/BMP，找不到时传 JPG/RAW），
/// 因此 `total` 就是选中的照片数量。
#[tauri::command]
pub async fn plan_hdr_conversion(file_paths: Vec<String>) -> Result<HdrPlan, AppError> {
    let plan = tokio::task::spawn_blocking(move || {
        let mut plan = HdrPlan {
            total: file_paths.len(),
            convertible: 0,
            unsupported: Vec::new(),
            overwrite: Vec::new(),
        };

        for raw_path in file_paths {
            let path = match validate_file_path(&raw_path) {
                Ok(path) => path,
                Err(_) => {
                    plan.unsupported.push(file_name_of(&raw_path));
                    continue;
                }
            };

            if !hdr_service::is_convertible_file(&path) {
                plan.unsupported.push(file_name_of(&raw_path));
                continue;
            }

            plan.convertible += 1;
            if let Some(output) = hdr_service::output_path_for(&path) {
                if output.exists() {
                    plan.overwrite.push(file_name_of(&output.to_string_lossy()));
                }
            }
        }

        plan
    })
    .await
    .map_err(|e| AppError::General(format!("plan_hdr_conversion task failed: {}", e)))?;

    Ok(plan)
}

/// 批量把 TIF/BMP 转换为同目录同名 HDR JPG（覆盖已有 .jpg）。
///
/// 每完成一张推送一次 `task-progress` 事件；单张失败不会中断整批。
#[tauri::command]
pub async fn convert_hdr_batch(
    app: AppHandle,
    task_id: String,
    file_paths: Vec<String>,
    quality: Option<u8>,
) -> Result<Vec<HdrConvertResult>, AppError> {
    use futures::stream::{self, StreamExt};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    let total = file_paths.len();
    if total == 0 {
        return Ok(Vec::new());
    }

    let processed = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));
    let collected: Arc<Mutex<Vec<(usize, HdrConvertResult)>>> =
        Arc::new(Mutex::new(Vec::with_capacity(total)));
    let results_store = Arc::clone(&collected);
    let concurrency = batch_concurrency(total);

    stream::iter(file_paths.into_iter().enumerate())
        .for_each_concurrent(concurrency, move |(index, raw_path)| {
            let app = app.clone();
            let task_id = task_id.clone();
            let processed = Arc::clone(&processed);
            let failed = Arc::clone(&failed);
            let collected = Arc::clone(&collected);

            async move {
                let display = file_name_of(&raw_path);
                let input = raw_path.clone();
                let outcome = tokio::task::spawn_blocking(move || -> Result<String, AppError> {
                    let path = validate_file_path(&input)?;
                    let outcome = hdr_service::convert_file_to_hdr(&path, quality)?;
                    Ok(outcome.output_path.to_string_lossy().to_string())
                })
                .await;

                let result = match outcome {
                    Ok(Ok(output_path)) => HdrConvertResult {
                        input_path: raw_path,
                        output_path: Some(output_path),
                        success: true,
                        error: None,
                    },
                    Ok(Err(error)) => HdrConvertResult {
                        input_path: raw_path,
                        output_path: None,
                        success: false,
                        error: Some(error.to_string()),
                    },
                    Err(error) => HdrConvertResult {
                        input_path: raw_path,
                        output_path: None,
                        success: false,
                        error: Some(format!("转换任务异常: {}", error)),
                    },
                };

                if !result.success {
                    failed.fetch_add(1, Ordering::Relaxed);
                }

                let done = processed.fetch_add(1, Ordering::SeqCst) + 1;
                let _ = app.emit(
                    "task-progress",
                    TaskProgressEvent {
                        task_id,
                        kind: "hdr".to_string(),
                        processed: done,
                        total,
                        current: Some(display),
                        failed: failed.load(Ordering::Relaxed),
                    },
                );

                collected.lock().unwrap().push((index, result));
            }
        })
        .await;

    let mut results = results_store.lock().unwrap().clone();
    results.sort_by_key(|(index, _)| *index);
    Ok(results.into_iter().map(|(_, result)| result).collect())
}
