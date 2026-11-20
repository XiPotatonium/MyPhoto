use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::models::image::{DirectoryContents, FolderItem, ImageGroup};

const IMAGE_EXTENSIONS_JPG: &[&str] = &["jpg", "jpeg", "png"];
const IMAGE_EXTENSIONS_RAW: &[&str] = &["raf", "tif", "tiff", "bmp", "dng"];

fn is_image_file(ext: &str) -> bool {
    let lower = ext.to_lowercase();
    IMAGE_EXTENSIONS_JPG.contains(&lower.as_str()) || IMAGE_EXTENSIONS_RAW.contains(&lower.as_str())
}

fn is_jpg_ext(ext: &str) -> bool {
    let lower = ext.to_lowercase();
    IMAGE_EXTENSIONS_JPG.contains(&lower.as_str())
}

fn is_raw_ext(ext: &str) -> bool {
    let lower = ext.to_lowercase();
    IMAGE_EXTENSIONS_RAW.contains(&lower.as_str())
}

/// 检测 JPEG 是否为 Gain Map HDR（Ultra HDR / ISO 21496-1）
/// 仅解析文件头部的 JPEG 段（到 SOS 为止），命中以下任一标志即为 HDR：
/// - APP2 段内容以 "urn:iso:std:iso:ts:21496" 开头（ISO 21496-1 gain map）
/// - APP1 XMP 段中包含 "hdrgm:Version"（Adobe gain map）
/// 解析失败一律视为非 HDR
fn is_hdr_jpeg(path: &Path) -> bool {
    use std::io::{BufReader, Read};

    // 逐段流式读取头部，遇到 SOS/EOI 立即停止。
    // 旧实现用 take(512KB).read_to_end() 会把整个上限读满才解析，
    // 而实际头部段通常只有几十 KB（实测样例 76.5 KB），大文件夹中会放大成数百 MB 无效 IO。
    const HEADER_LIMIT: u64 = 512 * 1024;
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut reader = BufReader::new(file.take(HEADER_LIMIT));

    let mut soi = [0u8; 2];
    if reader.read_exact(&mut soi).is_err() || soi != [0xFF, 0xD8] {
        return false;
    }

    loop {
        let mut marker = [0u8; 2];
        if reader.read_exact(&mut marker).is_err() || marker[0] != 0xFF {
            return false;
        }

        // 跳过 JPEG 允许的 0xFF 填充字节
        let mut code = marker[1];
        while code == 0xFF {
            let mut next = [0u8; 1];
            if reader.read_exact(&mut next).is_err() {
                return false;
            }
            code = next[0];
        }

        // SOS/EOI 之后为压缩数据，头部段解析结束
        if code == 0xDA || code == 0xD9 {
            return false;
        }

        let mut len_buf = [0u8; 2];
        if reader.read_exact(&mut len_buf).is_err() {
            return false;
        }
        let len = ((len_buf[0] as usize) << 8) | len_buf[1] as usize;
        if len < 2 {
            return false;
        }

        let mut data = vec![0u8; len - 2];
        if reader.read_exact(&mut data).is_err() {
            return false;
        }

        let is_hdr = match code {
            0xE2 => data.starts_with(b"urn:iso:std:iso:ts:21496"),
            0xE1 => {
                data.starts_with(b"http://ns.adobe.com/xap/1.0/")
                    && data
                        .windows(b"hdrgm:Version".len())
                        .any(|w| w == b"hdrgm:Version")
            }
            _ => false,
        };
        if is_hdr {
            return true;
        }
    }
}

pub fn list_images(dir_path: &Path) -> Result<Vec<ImageGroup>, std::io::Error> {
    let mut groups: HashMap<String, ImageGroup> = HashMap::new();

    let entries = std::fs::read_dir(dir_path)?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => ext.to_string(),
            None => continue,
        };

        if !is_image_file(&ext) {
            continue;
        }

        let base_name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let path_str = path.to_string_lossy().to_string();

        let metadata = entry.metadata().ok();

        let modified_time = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            });

        let file_created_time = metadata
            .as_ref()
            .and_then(|m| m.created().ok())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            });

        let group = groups.entry(base_name.clone()).or_insert_with(|| ImageGroup {
            base_name: base_name.clone(),
            jpg_path: None,
            raw_path: None,
            file_count: 0,
            is_hdr: false,
            modified_time: None,
            file_created_time: None,
            exif_info: None,
        });

        group.file_count += 1;

        if is_jpg_ext(&ext) {
            group.jpg_path = Some(path_str);
        } else if is_raw_ext(&ext) {
            group.raw_path = Some(path_str);
        }

        // Use the most recent modified time
        if modified_time.is_some()
            && (group.modified_time.is_none()
                || modified_time.as_ref() > group.modified_time.as_ref())
        {
            group.modified_time = modified_time;
        }

        // Use the earliest file creation time
        if file_created_time.is_some()
            && (group.file_created_time.is_none()
                || file_created_time.as_ref() < group.file_created_time.as_ref())
        {
            group.file_created_time = file_created_time;
        }
    }

    // 并行检测每个图片组的 JPEG 是否为 gain map HDR。
    // EXIF 不在此处读取：改为前端拿到列表后通过 read_exif_batch 异步补齐，
    // 使 list_images 快速返回。
    // 使用 std::thread::scope 做数据并行（无需额外依赖，与本项目 EXIF 写入并行方式一致）。
    let mut group_refs: Vec<&mut ImageGroup> = groups.values_mut().collect();
    detect_hdr_parallel(&mut group_refs);

    let result: Vec<ImageGroup> = groups.into_values().collect();

    // 排序逻辑已移至前端，后端返回原始列表
    Ok(result)
}

/// 0.3.0：目录树移除后，一次 IPC 取回「子文件夹 + 图片组」。
/// 前端拿到即可立刻渲染混合网格，不用再发一次目录扫描命令。
pub fn list_directory_contents(dir_path: &Path) -> Result<DirectoryContents, std::io::Error> {
    Ok(DirectoryContents {
        folders: list_folders(dir_path)?,
        images: list_images(dir_path)?,
    })
}

/// 列出直属子文件夹，并为每个文件夹挑一张封面照片。
/// 严格只看一层：不递归扫描孙目录，避免大图库上出现不可控的 IO。
fn list_folders(dir_path: &Path) -> Result<Vec<FolderItem>, std::io::Error> {
    fn smaller(current: &Option<String>, candidate: &str) -> bool {
        match current {
            Some(c) => candidate < c.as_str(),
            None => true,
        }
    }

    let mut folders = Vec::new();
    for entry in std::fs::read_dir(dir_path)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        // 隐藏目录不展示，与资源管理器习惯一致
        if name.starts_with('.') {
            continue;
        }

        let mut stems: HashSet<String> = HashSet::new();
        let mut cover_compressed: Option<String> = None;
        let mut cover_any: Option<String> = None;

        if let Ok(children) = std::fs::read_dir(&path) {
            for child in children.filter_map(|e| e.ok()) {
                let child_path = child.path();
                if !child_path.is_file() {
                    continue;
                }
                let ext = match child_path.extension().and_then(|e| e.to_str()) {
                    Some(e) => e.to_string(),
                    None => continue,
                };
                if !is_image_file(&ext) {
                    continue;
                }
                // 同名 JPG + RAW 属于同一张照片
                if let Some(stem) = child_path.file_stem().and_then(|s| s.to_str()) {
                    stems.insert(stem.to_string());
                }
                let path_str = child_path.to_string_lossy().to_string();
                // 取字典序最小者作为稳定封面；优先压缩格式（解码更快）
                if is_jpg_ext(&ext) && smaller(&cover_compressed, &path_str) {
                    cover_compressed = Some(path_str.clone());
                }
                if smaller(&cover_any, &path_str) {
                    cover_any = Some(path_str);
                }
            }
        }

        folders.push(FolderItem {
            name,
            path: path.to_string_lossy().to_string(),
            photo_count: stems.len(),
            cover_path: cover_compressed.or(cover_any),
        });
    }

    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(folders)
}

/// 对图片组并行检测 gain map HDR（仅解析 JPEG 头部段）。
fn detect_hdr_parallel(groups: &mut [&mut ImageGroup]) {
    if groups.is_empty() {
        return;
    }

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(groups.len());

    if num_threads <= 1 {
        for group in groups.iter_mut() {
            detect_hdr_one(group);
        }
        return;
    }

    let chunk_size = (groups.len() + num_threads - 1) / num_threads;
    std::thread::scope(|s| {
        for chunk in groups.chunks_mut(chunk_size) {
            s.spawn(move || {
                for group in chunk {
                    detect_hdr_one(group);
                }
            });
        }
    });
}

/// 检测单个图片组的 JPEG 是否为 gain map HDR。
fn detect_hdr_one(group: &mut ImageGroup) {
    if let Some(jpg_path) = group.jpg_path.as_ref() {
        let path = Path::new(jpg_path);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext == "jpg" || ext == "jpeg" {
            group.is_hdr = is_hdr_jpeg(path);
        }
    }
}
