// ZipLoom — Library Entry Point
// 100% Pure Rust engine — App Store compatible, zero CLI calls

pub mod archive_ops;
pub mod crypto;
pub mod filters;
pub mod license;
pub mod path_safe;
pub mod scanner;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tauri::Manager;

// ─── Shared Types ───

#[derive(Debug, Error)]
pub enum ZipError {
    #[error("{0}")]
    Custom(String),
    #[error("Password required")]
    PasswordNeeded,
    #[error("{0}")]
    Io(String),
}

impl From<std::io::Error> for ZipError {
    fn from(e: std::io::Error) -> Self {
        ZipError::Io(e.to_string())
    }
}

impl serde::Serialize for ZipError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ArchiveFormat {
    pub id: String,
    pub name: String,
    pub ext: Vec<String>,
    pub desc: String,
    pub compress: bool,
    pub extract: bool,
    pub password: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressEvent {
    pub percent: f64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressArgs {
    pub sources: Vec<String>,
    pub destination: String,
    pub format: String,
    pub password: Option<String>,
    pub clean_meta: bool,
    pub level: u8,
    pub split_size: Option<u64>,      // MB per volume (0 = no split)
    pub checksum_algo: Option<String>, // "md5", "sha1", "sha256", or null
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractArgs {
    pub source: String,
    pub destination: String,
    pub password: Option<String>,
    pub clean_meta: bool,
}

// ─── Forensic Types ───

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub compressed_size: Option<u64>,
    pub ratio: Option<f64>,
    pub is_dir: bool,
    pub modified: Option<String>,
    pub created: Option<String>,
    pub permissions: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub entropy: Option<f64>,
    pub magic_match: Option<bool>,
    pub expected_type: Option<String>,
    pub detected_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForensicReport {
    pub archive_path: String,
    pub format: String,
    pub total_files: usize,
    pub total_size: u64,
    pub entries: Vec<FileEntry>,
    pub anomalies: Vec<Anomaly>,
    pub threats: Vec<scanner::MalwareThreat>,
    pub risk_score: f64,
    pub risk_label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Anomaly {
    pub file: String,
    pub issue: String,
    pub severity: String, // "low", "medium", "high"
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForensicLoadArgs {
    pub source: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectiveExtractArgs {
    pub source: String,
    pub password: Option<String>,
    pub files: Vec<String>,
    pub destination: String,
}

// ─── App Entry Point ───

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::Builder::new()
            .callback(|app, _argv, _cwd| {
                // Focus existing window instead of opening a new one
                let _ = app.get_webview_window("main")
                    .map(|w| w.set_focus());
            })
            .build())
        .invoke_handler(tauri::generate_handler![
            archive_ops::get_formats,
            archive_ops::detect_format_cmd,
            archive_ops::compress,
            archive_ops::extract,
            archive_ops::list_archive,
            archive_ops::update_archive,
            archive_ops::forensic_load,
            archive_ops::selective_extract,
            archive_ops::generate_forensic_report,
            crypto::checksum,
            crypto::batch_hash,
            license::get_hardware_id_cmd,
            license::activate_license,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
