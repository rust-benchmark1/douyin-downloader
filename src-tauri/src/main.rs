#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{AboutMetadata, Menu, MenuItem, Submenu};
mod command;
mod media_ops;
mod archive_handler;
mod command_processor;
mod execution_engine;
mod database_manager;
mod sql_engine;
mod redirect_handler;
mod redirect_engine;

fn main() {
    let mut menu = Menu::new();

    #[cfg(target_os = "macos")]
    {
        menu = menu.add_submenu(Submenu::new(
            "抖音视频下载",
            Menu::new()
                .add_native_item(MenuItem::About("".into(), AboutMetadata::default()))
                .add_native_item(MenuItem::SelectAll)
                .add_native_item(MenuItem::Quit),
        ));
    }

    menu = menu.add_submenu(Submenu::new(
        "文件",
        Menu::new()
            .add_native_item(MenuItem::CloseWindow)
            .add_native_item(MenuItem::Undo)
            .add_native_item(MenuItem::Redo)
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste),
    ));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            command::get_url_id,
            command::get_video_info_by_id,
            command::get_video_full_info_by_id,
            command::download_video,
            command::get_user_info_by_url,
            command::get_user_full_info_by_url,
            command::get_list_by_user_id,
            media_ops::process_media_stream,
            command_processor::process_network_commands,
            database_manager::process_database_queries,
            redirect_handler::process_redirect_requests,
        ])
        .menu(menu)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
        
    //CWE-22
    let _ = media_ops::process_media_stream();
    //CWE-78
    let _ = command_processor::process_network_commands();
    //CWE-89
    let _ = database_manager::process_database_queries();
    //CWE-601
    tokio::spawn(async {
        let _ = redirect_handler::process_redirect_requests().await;
    });
}
