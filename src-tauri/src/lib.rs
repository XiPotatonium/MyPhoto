mod commands;
mod error;
mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::filesystem::scan_directory_tree,
            commands::filesystem::list_images,
            commands::filesystem::select_root_directory,
            commands::thumbnail::generate_thumbnail,
            commands::thumbnail::generate_thumbnails_batch,
            commands::thumbnail::read_image_file,
            commands::exif_cmd::read_exif,
            commands::exif_cmd::write_rating,
            commands::exif_cmd::batch_write_gps,
            commands::trash_cmd::move_to_trash,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
