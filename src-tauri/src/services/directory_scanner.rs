use std::path::Path;

use crate::models::directory::DirectoryNode;

pub fn scan_directory(root_path: &Path) -> Result<DirectoryNode, std::io::Error> {
    let name = root_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root_path.to_string_lossy().to_string());

    let mut children = Vec::new();

    if root_path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(root_path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    && !e
                        .file_name()
                        .to_string_lossy()
                        .starts_with('.')
            })
            .collect();

        entries.sort_by(|a, b| {
            a.file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase())
        });

        for entry in entries {
            match scan_directory(&entry.path()) {
                Ok(child) => children.push(child),
                Err(_) => continue, // skip directories we can't read
            }
        }
    }

    Ok(DirectoryNode {
        name,
        path: root_path.to_string_lossy().to_string(),
        children,
    })
}
