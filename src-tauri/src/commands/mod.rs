pub mod exif_cmd;
pub mod filesystem;
pub mod hdr_cmd;
pub mod thumbnail;
pub mod trash_cmd;

use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use crate::error::AppError;

/// 后台批量任务的进度事件（事件名 `task-progress`）。
///
/// 字段使用 camelCase，前端按 `taskId` 过滤后更新左上角进度浮层。
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressEvent {
    pub task_id: String,
    /// `hdr` | `exif`
    pub kind: String,
    pub processed: usize,
    pub total: usize,
    /// 刚处理完的文件名，用于进度浮层的次要行文案。
    pub current: Option<String>,
    pub failed: usize,
}

/// 批量任务并发数：HDR 转换每个任务峰值内存较高，限制在 4 以内。
pub fn batch_concurrency(item_count: usize) -> usize {
    if item_count <= 1 {
        return 1;
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(4)
        .min(item_count)
}

/// 当前有效的文件夹加载会话。
/// 每次目录加载（`list_directory_contents` / `list_images`）都会注册新会话，
/// 上一轮遗留的缩略图 / EXIF 批量任务
/// 据此提前退出，避免快速切换文件夹时旧任务继续占用并发槽与磁盘 IO。
static ACTIVE_LOAD_SESSION: LazyLock<Mutex<Option<String>>> =
    LazyLock::new(|| Mutex::new(None));

/// 注册新的加载会话，使所有旧会话立即失效。
pub fn begin_load_session(session_id: &str) {
    if let Ok(mut active) = ACTIVE_LOAD_SESSION.lock() {
        *active = Some(session_id.to_string());
    }
}

/// 判断某个批量任务的会话是否仍然有效。
pub fn is_load_session_active(session_id: &str) -> bool {
    ACTIVE_LOAD_SESSION
        .lock()
        .map(|active| active.as_deref() == Some(session_id))
        .unwrap_or(true)
}

/// Windows 的 `std::fs::canonicalize` 返回 `\\?\E:\...` 形式的 NT 扩展路径。
/// 这个前缀会经由 `FolderItem.path` 一路传到前端，让路径面包屑显示成
/// `\\?\E:`，还会被当成 UNC 网络路径。统一转回普通 Win32 路径。
/// `\\?\UNC\host\share` 需要还原成 `\\host\share`，其余直接去掉前缀。
fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let text = path.to_string_lossy();
        if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{}", rest));
        }
        if let Some(rest) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path
}

/// Canonicalize a file path and verify it points to an existing file.
/// Returns the normalized path to prevent path injection.
pub fn validate_file_path(raw: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(raw);
    let canonical = std::fs::canonicalize(path).map_err(|_| {
        AppError::General(format!("Invalid or inaccessible path: {}", raw))
    })?;
    if !canonical.is_file() {
        return Err(AppError::General(format!("Path is not a file: {}", raw)));
    }
    Ok(strip_verbatim_prefix(canonical))
}

/// Canonicalize a directory path and verify it points to an existing directory.
/// Returns the normalized path to prevent path injection.
pub fn validate_dir_path(raw: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(raw);
    let canonical = std::fs::canonicalize(path).map_err(|_| {
        AppError::General(format!("Invalid or inaccessible path: {}", raw))
    })?;
    if !canonical.is_dir() {
        return Err(AppError::General(format!("Path is not a directory: {}", raw)));
    }
    Ok(strip_verbatim_prefix(canonical))
}
