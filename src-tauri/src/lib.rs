mod commands;
mod error;
mod models;
mod services;

// 0.3.3：原生菜单栏（应用 / 文件 / 设置）已整体移除。
// 打开目录、排序、主题全部由前端顶栏工具栏承担，窗口控制由自定义标题栏承担，
// 因此这里不再构建 Menu，也不再向前端发射 menu-* 事件。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::filesystem::list_directory_contents,
            commands::filesystem::list_images,
            commands::filesystem::select_root_directory,
            commands::thumbnail::generate_thumbnails_batch,
            commands::thumbnail::read_image_file,
            commands::exif_cmd::read_exif,
            commands::exif_cmd::read_exif_batch,
            commands::exif_cmd::write_rating,
            commands::exif_cmd::write_exif_fields,
            commands::exif_cmd::write_exif_fields_batch,
            commands::hdr_cmd::plan_hdr_conversion,
            commands::hdr_cmd::convert_hdr_batch,
            commands::trash_cmd::move_to_trash,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
