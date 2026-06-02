use tauri::Manager;

mod commands;
mod db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    db::init().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_title("AnalysisLoom — Forensic Analysis Workstation")?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_cases,
            commands::create_case,
            commands::get_case,
            commands::delete_case,
            commands::parse_mft,
            commands::start_carving,
            commands::get_carving_progress,
            commands::cancel_carving,
            commands::get_timeline,
            commands::keyword_search,
            commands::preview_file,
            commands::about_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AnalysisLoom");
}
