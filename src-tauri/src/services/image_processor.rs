use std::collections::HashMap;
use std::path::Path;

use crate::models::image::ImageGroup;
use crate::services::exif_service;

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
    use std::io::Read;

    // 头部段（含 XMP）通常在前几十 KB，限制读取量避免大文件开销
    const HEADER_LIMIT: u64 = 512 * 1024;
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf = Vec::new();
    if file.take(HEADER_LIMIT).read_to_end(&mut buf).is_err() {
        return false;
    }

    if buf.len() < 4 || buf[0] != 0xFF || buf[1] != 0xD8 {
        return false;
    }

    let mut pos = 2;
    while pos + 4 <= buf.len() {
        if buf[pos] != 0xFF {
            break;
        }
        let marker = buf[pos + 1];
        // SOS/EOI 之后为压缩数据，头部段解析结束
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        let len = ((buf[pos + 2] as usize) << 8) | buf[pos + 3] as usize;
        if len < 2 {
            break;
        }
        let data_start = pos + 4;
        let data_end = (pos + 2 + len).min(buf.len());
        if data_start > data_end {
            break;
        }
        let data = &buf[data_start..data_end];
        let is_hdr = match marker {
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
        pos += 2 + len;
    }
    false
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

    // 读取每个图片组的 EXIF 信息，并检测 JPEG 是否为 gain map HDR
    for group in groups.values_mut() {
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

        let file_path = group.jpg_path.as_ref().or(group.raw_path.as_ref());
        if let Some(path_str) = file_path {
            let path = Path::new(path_str);
            // 容错处理：EXIF 读取失败时回退到空信息
            match exif_service::read_exif(path) {
                Ok(exif) => {
                    group.exif_info = Some(exif);
                }
                Err(e) => {
                    log::debug!("Failed to read EXIF for {}: {}", path_str, e);
                    group.exif_info = None;
                }
            }
        }
    }

    let result: Vec<ImageGroup> = groups.into_values().collect();

    // 排序逻辑已移至前端，后端返回原始列表
    Ok(result)
}
