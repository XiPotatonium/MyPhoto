mod commands;
mod error;
mod models;
mod services;

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
            // 菜单文案统一为中文。预定义 .quit() 的文本由系统语言决定，
            // 英文系统上会显示 "Quit" 造成中英混排，改用自定义项。
            let quit_item = MenuItemBuilder::new("退出")
                .id("quit")
                .build(app)?;
            // macOS 会把第一个子菜单的标题替换为应用名
            let about_menu = SubmenuBuilder::new(app, "应用")
                .item(&quit_item)
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

            // 0.3.0：目录树移除后，「设置根目录」迁入原生文件菜单
            let open_folder = MenuItemBuilder::new("打开目录…")
                .id("open_folder")
                .build(app)?;
            let file_menu = SubmenuBuilder::new(app, "文件")
                .item(&open_folder)
                .build()?;

            let settings_menu = SubmenuBuilder::new(app, "设置")
                .item(&sort_menu)
                .item(&theme_menu)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&about_menu)
                .item(&file_menu)
                .item(&settings_menu)
                .build()?;

            app.set_menu(menu)?;

            app.on_menu_event(move |app_handle, event| {
                match event.id().0.as_str() {
                    "quit" => {
                        app_handle.exit(0);
                    }
                    "open_folder" => {
                        let _ = app_handle.emit_to("main", "menu-open-folder", "");
                    }
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
            commands::filesystem::list_directory_contents,
            commands::filesystem::list_images,
            commands::filesystem::select_root_directory,
            commands::thumbnail::generate_thumbnails_batch,
            commands::thumbnail::read_image_file,
            commands::exif_cmd::read_exif,
            commands::exif_cmd::read_exif_batch,
            commands::exif_cmd::write_rating,
            commands::exif_cmd::write_exif_fields,
            commands::trash_cmd::move_to_trash,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
