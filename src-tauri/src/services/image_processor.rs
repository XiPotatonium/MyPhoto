use std::collections::HashMap;
use std::path::Path;

use crate::models::image::ImageGroup;
use crate::services::exif_service;

const IMAGE_EXTENSIONS_JPG: &[&str] = &["jpg", "jpeg", "png"];
const IMAGE_EXTENSIONS_RAW: &[&str] = &["raf"];

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

    // 读取每个图片组的 EXIF 信息
    for group in groups.values_mut() {
        let file_path = group.jpg_path.as_ref().or(group.raw_path.as_ref());
        if let Some(path_str) = file_path {
            let path = Path::new(path_str);
            // 容错处理：EXIF 读取失败时回退到空信息
            match exif_service::read_exif(path) {
                Ok(exif) => {
                    group.exif_info = Some(exif);
                }
                Err(e) => {
                    eprintln!("Failed to read EXIF for {}: {}", path_str, e);
                    group.exif_info = None;
                }
            }
        }
    }

    let result: Vec<ImageGroup> = groups.into_values().collect();

    // 排序逻辑已移至前端，后端返回原始列表
    Ok(result)
}
