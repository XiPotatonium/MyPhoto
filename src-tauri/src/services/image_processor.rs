use std::collections::HashMap;
use std::path::Path;

use crate::models::image::{ImageGroup, SortField, SortOrder};

const IMAGE_EXTENSIONS_JPG: &[&str] = &["jpg", "jpeg", "png"];
const IMAGE_EXTENSIONS_RAW: &[&str] = &["dng", "raf"];

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

pub fn list_images(
    dir_path: &Path,
    sort_field: &SortField,
    sort_order: &SortOrder,
) -> Result<Vec<ImageGroup>, std::io::Error> {
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

        let modified_time = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
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
    }

    let mut result: Vec<ImageGroup> = groups.into_values().collect();

    match sort_field {
        SortField::Date => {
            result.sort_by(|a, b| {
                let cmp = a.modified_time.cmp(&b.modified_time);
                match sort_order {
                    SortOrder::Asc => cmp,
                    SortOrder::Desc => cmp.reverse(),
                }
            });
        }
        SortField::Rating => {
            // Rating sort requires EXIF reading which is expensive;
            // for now, fall back to name sort. Rating-based sort
            // would need to be handled with cached rating data.
            result.sort_by(|a, b| {
                let cmp = a.base_name.to_lowercase().cmp(&b.base_name.to_lowercase());
                match sort_order {
                    SortOrder::Asc => cmp,
                    SortOrder::Desc => cmp.reverse(),
                }
            });
        }
    }

    Ok(result)
}
