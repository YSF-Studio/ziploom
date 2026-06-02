use tauri::Manager;

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_title("CollectionLoom — Portable Forensic Acquisition")?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_disks,
            commands::start_disk_imaging,
            commands::get_imaging_progress,
            commands::cancel_imaging,
            commands::list_ram_tools,
            commands::get_ram_size,
            commands::capture_ram,
            commands::enable_write_blocker,
            commands::disable_write_blocker,
            commands::check_write_blocker,
            commands::list_android_devices,
            commands::adb_backup,
            commands::list_ios_devices,
            commands::ios_backup,
            commands::list_interfaces,
            commands::start_network_capture,
            commands::cancel_network_capture,
            commands::scan_encryption,
            commands::create_chain_of_custody,
            commands::generate_coc_report,
            commands::about_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CollectionLoom");
}
