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

            // ─── Inject error handler for debugging ───
            let _ = window.eval(
                "window.onerror=function(m,u,l){console.error('FATAL:',m,u,l);};\
                 window.addEventListener('unhandledrejection',function(e){console.error('UNHANDLED:',e.reason);});"
            );

            // ─── GUI Screenshot Mode ───
            if std::env::var("ANALYSISLOOM_SCREENSHOT").is_ok() {
                let w = window.clone();
                std::thread::spawn(move || {
                    use std::time::Duration;
                    std::thread::sleep(Duration::from_secs(5));

                    // Cycle through sidebar sections by text label
                    let sections = [
                        "Case Manager", "Timeline", "Carved Files",
                        "Search", "Report", "About", "File Browser"
                    ];
                    for section in &sections {
                        let js = format!(
                            "Array.from(document.querySelectorAll('.sidebar-item')).find(b=>b.textContent.includes('{}'))?.click();",
                            section
                        );
                        let _ = w.eval(&js);
                        std::thread::sleep(Duration::from_secs(4));
                    }

                    // Final: back to File Browser
                    let _ = w.eval("Array.from(document.querySelectorAll('.sidebar-item')).find(b=>b.textContent.includes('File Browser'))?.click();");
                    std::thread::sleep(Duration::from_secs(3));
                    eprintln!("[SCREENSHOT] All sections navigated");
                });
            }

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
            commands::generate_case_report,
            commands::log_action,
            commands::get_audit_log,
            commands::add_bookmark,
            commands::list_bookmarks,
            commands::delete_bookmark,
            commands::about_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AnalysisLoom");
}
