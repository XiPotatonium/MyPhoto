mod commands;
mod error;
mod models;
pub mod services;

use tauri::{Emitter, menu::{MenuItem, MenuItemBuilder, MenuBuilder, SubmenuBuilder}};

fn set_radio_text(items: &[MenuItem<tauri::Wry>], labels: &[&str], selected: usize) {
    for (i, item) in items.iter().enumerate() {
        let prefix = if i == selected { "✓ " } else { "  " };
        let _ = item.set_text(format!("{}{}", prefix, labels[i]));
    }
}

fn set_sort_field(items: &[MenuItem<tauri::Wry>; 3], field: &str) -> usize {
    let idx = match field {
        "name" => 0,
        "date" => 1,
        "rating" => 2,
        _ => 0,
    };
    set_radio_text(items, &["文件名", "拍摄时间", "星级评分"], idx);
    idx
}

fn set_sort_order(items: &[MenuItem<tauri::Wry>; 2], order: &str) -> usize {
    let idx = match order {
        "asc" => 0,
        "desc" => 1,
        _ => 0,
    };
    set_radio_text(items, &["升序", "降序"], idx);
    idx
}

fn set_theme(items: &[MenuItem<tauri::Wry>; 2], theme: &str) -> usize {
    let idx = match theme {
        "light" => 0,
        "dark" => 1,
        _ => 0,
    };
    set_radio_text(items, &["浅色", "深色"], idx);
    idx
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // macOS 要求第一个子菜单放在应用名下
            let about_menu = SubmenuBuilder::new(app, "MyPhoto")
                .quit()
                .build()?;

            let sort_name = MenuItemBuilder::new("✓ 文件名")
                .id("sort_name")
                .build(app)?;
            let sort_date = MenuItemBuilder::new("  拍摄时间")
                .id("sort_date")
                .build(app)?;
            let sort_rating = MenuItemBuilder::new("  星级评分")
                .id("sort_rating")
                .build(app)?;
            let sort_asc = MenuItemBuilder::new("✓ 升序")
                .id("sort_asc")
                .build(app)?;
            let sort_desc = MenuItemBuilder::new("  降序")
                .id("sort_desc")
                .build(app)?;

            let sort_field_items = [sort_name.clone(), sort_date.clone(), sort_rating.clone()];
            let sort_order_items = [sort_asc.clone(), sort_desc.clone()];

            let sort_menu = SubmenuBuilder::new(app, "排序")
                .item(&sort_name)
                .item(&sort_date)
                .item(&sort_rating)
                .separator()
                .item(&sort_asc)
                .item(&sort_desc)
                .build()?;

            let theme_light = MenuItemBuilder::new("✓ 浅色")
                .id("theme_light")
                .build(app)?;
            let theme_dark = MenuItemBuilder::new("  深色")
                .id("theme_dark")
                .build(app)?;

            let theme_items = [theme_light.clone(), theme_dark.clone()];

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

            // Clone items into the closure for direct updates
            let sort_field_items = sort_field_items;
            let sort_order_items = sort_order_items;
            let theme_items = theme_items;

            app.on_menu_event(move |app_handle, event| {
                match event.id().0.as_str() {
                    "sort_name" => {
                        set_sort_field(&sort_field_items, "name");
                        let _ = app_handle.emit_to("main", "menu-sort-field", "name");
                    }
                    "sort_date" => {
                        set_sort_field(&sort_field_items, "date");
                        let _ = app_handle.emit_to("main", "menu-sort-field", "date");
                    }
                    "sort_rating" => {
                        set_sort_field(&sort_field_items, "rating");
                        let _ = app_handle.emit_to("main", "menu-sort-field", "rating");
                    }
                    "sort_asc" => {
                        set_sort_order(&sort_order_items, "asc");
                        let _ = app_handle.emit_to("main", "menu-sort-order", "asc");
                    }
                    "sort_desc" => {
                        set_sort_order(&sort_order_items, "desc");
                        let _ = app_handle.emit_to("main", "menu-sort-order", "desc");
                    }
                    "theme_light" => {
                        set_theme(&theme_items, "light");
                        let _ = app_handle.emit_to("main", "menu-theme", "light");
                    }
                    "theme_dark" => {
                        set_theme(&theme_items, "dark");
                        let _ = app_handle.emit_to("main", "menu-theme", "dark");
                    }
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
