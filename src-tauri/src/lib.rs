use tauri::Manager;

pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_title("ZipLoom — Archive Utility")?;

            // ─── GUI Screenshot Mode ───
            let screenshot_tab = std::env::var("ZIPLOOM_TAB").ok();
            if std::env::var("ZIPLOOM_SCREENSHOT").is_ok() {
                let w = window.clone();
                let tab = screenshot_tab.clone();
                std::thread::spawn(move || {
                    use std::time::Duration;
                    // Wait for WebView to fully render
                    std::thread::sleep(Duration::from_secs(4));

                    // ── Setup: light mode ──
                    let _ = w.eval(
                        "var s=document.createElement('style');\
                         s.textContent=':root{--bg:#f5f5f7!important;--card:#ffffff!important;--border:#d2d2d7!important;--text:#1d1d1f!important;--text-secondary:#6e6e73!important;--text-muted:#aeaeb2!important;--primary:#2563eb!important;--primary-bg:rgba(37,99,235,0.10)!important;--success:#16a34a!important;--success-bg:rgba(22,163,74,0.10)!important;--card-hover:#fafafa!important;--border-light:#c0c0c5!important;}';\
                         document.head.appendChild(s);"
                    );
                    std::thread::sleep(Duration::from_secs(1));

                    // ── Apply sample data for all tabs ──
                    let _ = w.eval(
                        "window.__zipLoom?.setSources([\
                          '/Users/yusuf/samples/report_q1_2026.pdf',\
                          '/Users/yusuf/samples/evidence_manifest.zip',\
                          '/Users/yusuf/samples/disk_image.dd',\
                          '/Users/yusuf/samples/case_notes.txt'\
                        ]);"
                    );
                    std::thread::sleep(Duration::from_millis(500));
                    let _ = w.eval(
                        "window.__zipLoom?.setExtractResult('/Users/yusuf/samples/evidence_manifest.zip');"
                    );
                    std::thread::sleep(Duration::from_millis(500));
                    let _ = w.eval(
                        "window.__zipLoom?.setInspectArchive('/Users/yusuf/samples/evidence_manifest.zip');"
                    );
                    std::thread::sleep(Duration::from_millis(500));

                    // ── Switch to requested tab ──
                    let tab_idx: u32 = tab.as_deref().unwrap_or("0").parse().unwrap_or(0);
                    let _ = w.eval(&format!("window.__zipLoom?.setTab({});", tab_idx));
                    std::thread::sleep(Duration::from_secs(2));

                    eprintln!("[SCREENSHOT] Tab {} ready — light mode + sample data", tab_idx);
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::supported_formats,
            commands::archive_needs_password,
            commands::inspect_archive,
            commands::compress_files,
            commands::extract_archive,
            commands::encrypt_file,
            commands::decrypt_file,
            commands::stat_paths,
            commands::check_tools,
            commands::hash_file_sha256,
            commands::hash_archive,
            commands::get_progress,
            commands::preview_archive_entry,
            commands::forensic_scan_archive,
            commands::extract_archive_entries,
            commands::test_archive_integrity,
            commands::about_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZipLoom");
}
