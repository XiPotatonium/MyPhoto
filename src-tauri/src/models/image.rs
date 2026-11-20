use serde::{Deserialize, Serialize};

use crate::models::exif::ExifInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGroup {
    pub base_name: String,
    pub jpg_path: Option<String>,
    pub raw_path: Option<String>,
    pub file_count: u32,
    pub is_hdr: bool,
    pub modified_time: Option<String>,
    pub file_created_time: Option<String>,
    pub exif_info: Option<ExifInfo>,
}

/// 0.3.0：目录树移除后，文件浏览器需要直接展示子文件夹。
/// photo_count 是「直属图片组数」（同名 JPG+RAW 算一张），cover_path 为
/// 该文件夹内第一张图片的路径，均不递归到孙目录。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderItem {
    pub name: String,
    pub path: String,
    pub photo_count: usize,
    pub cover_path: Option<String>,
}

/// 一次 IPC 返回当前目录的全部展示内容，减少一次 read_dir 与往返。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryContents {
    pub folders: Vec<FolderItem>,
    pub images: Vec<ImageGroup>,
}
