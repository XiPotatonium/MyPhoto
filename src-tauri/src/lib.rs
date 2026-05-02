mod commands;
mod error;
mod models;
pub mod services;

use tauri::{Emitter, menu::{CheckMenuItemBuilder, MenuBuilder, SubmenuBuilder}};

fn update_menu_checked(app_handle: &tauri::AppHandle, id: &str, checked: bool) {
    if let Some(menu) = app_handle.menu() {
        if let Some(item) = menu.get(id) {
            if let Some(check_item) = item.as_check_menuitem() {
                let _ = check_item.set_checked(checked);
            }
        }
    }
}

fn set_sort_field(app_handle: &tauri::AppHandle, field: &str) {
    update_menu_checked(app_handle, "sort_name", field == "name");
    update_menu_checked(app_handle, "sort_date", field == "date");
    update_menu_checked(app_handle, "sort_rating", field == "rating");
    let _ = app_handle.emit_to("main", "menu-sort-field", field);
}

fn set_sort_order(app_handle: &tauri::AppHandle, order: &str) {
    update_menu_checked(app_handle, "sort_asc", order == "asc");
    update_menu_checked(app_handle, "sort_desc", order == "desc");
    let _ = app_handle.emit_to("main", "menu-sort-order", order);
}

fn set_theme(app_handle: &tauri::AppHandle, theme: &str) {
    update_menu_checked(app_handle, "theme_light", theme == "light");
    update_menu_checked(app_handle, "theme_dark", theme == "dark");
    let _ = app_handle.emit_to("main", "menu-theme", theme);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // macOS 要求第一个子菜单放在应用名下
            let about_menu = SubmenuBuilder::new(app, "MyPhoto")
                .quit()
                .build()?;

            let sort_name = CheckMenuItemBuilder::new("文件名")
                .id("sort_name")
                .checked(true)
                .build(app)?;
            let sort_date = CheckMenuItemBuilder::new("拍摄时间")
                .id("sort_date")
                .checked(false)
                .build(app)?;
            let sort_rating = CheckMenuItemBuilder::new("星级评分")
                .id("sort_rating")
                .checked(false)
                .build(app)?;
            let sort_asc = CheckMenuItemBuilder::new("升序")
                .id("sort_asc")
                .checked(true)
                .build(app)?;
            let sort_desc = CheckMenuItemBuilder::new("降序")
                .id("sort_desc")
                .checked(false)
                .build(app)?;

            let sort_menu = SubmenuBuilder::new(app, "排序")
                .item(&sort_name)
                .item(&sort_date)
                .item(&sort_rating)
                .separator()
                .item(&sort_asc)
                .item(&sort_desc)
                .build()?;

            let theme_light = CheckMenuItemBuilder::new("浅色")
                .id("theme_light")
                .checked(true)
                .build(app)?;
            let theme_dark = CheckMenuItemBuilder::new("深色")
                .id("theme_dark")
                .checked(false)
                .build(app)?;

            let theme_menu = SubmenuBuilder::new(app, "主题")
                .item(&theme_light)
                .item(&theme_dark)
                .build()?;

            let settings_menu = SubmenuBuilder::new(app, "Settings")
                .item(&sort_menu)
                .item(&theme_menu)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&about_menu)
                .item(&settings_menu)
                .build()?;

            app.set_menu(menu)?;

            app.on_menu_event(move |app_handle, event| {
                match event.id().0.as_str() {
                    "sort_name" => set_sort_field(app_handle, "name"),
                    "sort_date" => set_sort_field(app_handle, "date"),
                    "sort_rating" => set_sort_field(app_handle, "rating"),
                    "sort_asc" => set_sort_order(app_handle, "asc"),
                    "sort_desc" => set_sort_order(app_handle, "desc"),
                    "theme_light" => set_theme(app_handle, "light"),
                    "theme_dark" => set_theme(app_handle, "dark"),
                    _ => {}
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::filesystem::scan_directory_tree,
            commands::filesystem::list_images,
            commands::filesystem::select_root_directory,
            commands::thumbnail::generate_thumbnails_batch,
            commands::thumbnail::read_image_file,
            commands::exif_cmd::read_exif,
            commands::exif_cmd::write_rating,
            commands::exif_cmd::write_exif_fields,
            commands::trash_cmd::move_to_trash,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
