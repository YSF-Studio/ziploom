use tauri::Emitter;
use ysf_core::*;
use ysf_core::progress::finish_progress;
use std::sync::atomic::Ordering;

// ─── Disk Imaging ───

#[tauri::command]
pub fn list_disks() -> Result<Vec<imaging::DiskInfo>, String> {
    imaging::DiskInfo::list()
}

#[tauri::command]
pub async fn start_disk_imaging(
    source: String,
    destination: String,
    split_size_mb: u64,
    verify: bool,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Reset global state
    CANCEL_FLAG.store(false, Ordering::SeqCst);
    *PROGRESS_STATE.lock().unwrap() = ProgressState::default();
    *OPERATION_RESULT.lock().unwrap() = None;

    let cancel = CANCEL_FLAG.clone();

    tokio::task::spawn_blocking(move || {
        let mut imager = imaging::DiskImager::new(&source, std::path::Path::new(&destination));
        imager.split_size = if split_size_mb > 0 { Some(split_size_mb * 1_048_576) } else { None };
        imager.verify = verify;

        match imager.run(&cancel) {
            Ok(hash) => {
                let _ = app.emit("imaging_complete", &hash);
            }
            Err(e) => {
                finish_progress(Err(e.clone()));
                let _ = app.emit("imaging_error", &e);
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_imaging_progress() -> Result<ProgressState, String> {
    PROGRESS_STATE.lock().map(|s| s.clone()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_imaging() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
}

// ─── RAM Capture ───

#[tauri::command]
pub fn list_ram_tools() -> Result<Vec<String>, String> {
    Ok(ram::detect_tools().iter().map(|t| format!("{:?}", t)).collect())
}

#[tauri::command]
pub fn get_ram_size() -> Result<u64, String> {
    ram::get_ram_size()
}

#[tauri::command]
pub fn capture_ram(tool: String, output: String, compress: bool) -> Result<String, String> {
    match tool.as_str() {
        "Avml" => ram::capture_avml(&output, compress),
        "WinPmem" => ram::capture_winpmem(&output),
        "MRS" => ram::capture_mrs(&output),
        "LiME" => {
            use std::process::Command;
            // LiME requires sudo + insmod — use avml as fallback
            ram::capture_avml(&output, compress)
        }
        _ => Err(format!("Unknown tool: {}", tool)),
    }
}

// ─── Write Blocker ───

#[tauri::command]
pub fn enable_write_blocker(device: String) -> Result<(), String> {
    write_blocker::enable_write_blocker(&device)
}

#[tauri::command]
pub fn disable_write_blocker(device: String) -> Result<(), String> {
    write_blocker::disable_write_blocker(&device)
}

#[tauri::command]
pub fn check_write_blocker(device: String) -> Result<bool, String> {
    Ok(write_blocker::check_write_blocker(&device))
}

// ─── Mobile ───

#[tauri::command]
pub fn list_android_devices() -> Result<Vec<mobile::MobileDevice>, String> {
    mobile::list_android_devices()
}

#[tauri::command]
pub fn adb_backup(device_id: String, output: String) -> Result<String, String> {
    mobile::adb_backup(&device_id, &output)
}

#[tauri::command]
pub fn list_ios_devices() -> Result<Vec<mobile::MobileDevice>, String> {
    mobile::list_ios_devices()
}

#[tauri::command]
pub fn ios_backup(device_id: String, output: String) -> Result<String, String> {
    mobile::ios_backup(&device_id, &output)
}

// ─── Network ───

#[tauri::command]
pub fn list_interfaces() -> Result<Vec<String>, String> {
    network::list_interfaces()
}

#[tauri::command]
pub async fn start_network_capture(
    interface: String,
    bpf_filter: Option<String>,
    output_file: String,
) -> Result<String, String> {
    let config = network::NetworkCaptureConfig {
        interface,
        bpf_filter,
        output_file,
        ring_buffer_size: 256 * 1024 * 1024, // 256 MB
        max_duration_secs: 0, // until stopped
    };
    let cancel = CANCEL_FLAG.clone();
    CANCEL_FLAG.store(false, Ordering::SeqCst);
    tokio::task::spawn_blocking(move || network::start_capture(config, cancel))
        .await.map_err(|e| format!("Internal: {}", e))?
}

#[tauri::command]
pub fn cancel_network_capture() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
}

// ─── Encryption Detection ───

#[tauri::command]
pub fn scan_encryption() -> Result<EncryptionReport, String> {
    Ok(encryption_detect::scan_encryption())
}

// ─── Chain of Custody ───

#[tauri::command]
pub fn create_chain_of_custody(
    case_name: String,
    operator: String,
    source_device: String,
) -> Result<String, String> {
    let coc = evidence::ChainOfCustody::new(&case_name, &operator, &source_device, 0);
    Ok(coc.evidence_id)
}

#[tauri::command]
pub fn generate_coc_report(evidence_id: String) -> Result<String, String> {
    let report_path = format!("/tmp/{}_coc_report.pdf", evidence_id);
    let coc = evidence::ChainOfCustody::new("case", "operator", "device", 0);
    let pdf = report::generate_pdf_report(&report::PdfReport {
        title: format!("Chain of Custody — {}", evidence_id),
        evidence_id: evidence_id.clone(),
        operator: "Yusuf Shalahuddin".into(),
        case_name: "Forensic Case".into(),
        device: "Source Device".into(),
        date: chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        sections: vec![
            report::ReportSection { heading: "Evidence".into(), content: format!("ID: {}", evidence_id) },
        ],
    })?;
    std::fs::write(&report_path, pdf).map_err(|e| e.to_string())?;
    Ok(report_path)
}
