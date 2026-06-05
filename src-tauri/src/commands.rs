use serde::{Deserialize, Serialize};
use ysf_core::archive;

/// Represents an entry in an archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub path: String,
    pub size: u64,
    pub compressed_size: Option<u64>,
    pub is_dir: bool,
    pub modified: Option<String>,
}

/// Result of inspecting an archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInfo {
    pub format: String,
    pub entries: Vec<ArchiveEntry>,
    pub total_files: usize,
    pub total_size: u64,
    pub total_compressed: Option<u64>,
}

/// Result of compress/extract operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub success: bool,
    pub output_path: String,
    pub files_processed: usize,
    pub total_size: u64,
    pub message: String,
}

// ─── Supported Formats ───

#[tauri::command]
pub fn supported_formats() -> Vec<String> {
    archive::FORMATS_SUPPORTED.iter().map(|s| s.to_string()).collect()
}

// ─── Inspect Archive ───

#[tauri::command]
pub fn archive_needs_password(path: String) -> Result<bool, String> {
    archive::needs_password(&path)
}

#[tauri::command]
pub fn inspect_archive(path: String, password: Option<String>) -> Result<ArchiveInfo, String> {
    let pw = password.as_deref();
    let entries = archive::forensic_load(&path, pw).map_err(|e| {
        if e == "PASSWORD_NEEDED" || e == "WRONG_PASSWORD" {
            e
        } else {
            format!("Failed to read archive: {e}")
        }
    })?;

    let format = archive::detect_format(&path).unwrap_or("unknown");
    let format_str = format.to_string();

    let archive_entries: Vec<ArchiveEntry> = entries.iter().map(|e| ArchiveEntry {
        path: e.path.clone(),
        size: e.size,
        compressed_size: e.compressed_size,
        is_dir: e.is_dir,
        modified: e.modified.clone(),
    }).collect();

    let total_size: u64 = archive_entries.iter().map(|e| e.size).sum();
    let total_compressed: Option<u64> = if archive_entries.iter().any(|e| e.compressed_size.is_some()) {
        Some(archive_entries.iter().filter_map(|e| e.compressed_size).sum())
    } else {
        None
    };

    Ok(ArchiveInfo {
        format: format_str,
        total_files: archive_entries.len(),
        total_size,
        total_compressed,
        entries: archive_entries,
    })
}

// ─── Compress Files ───

#[tauri::command]
pub fn compress_files(
    sources: Vec<String>,
    output: String,
    format: String,
    password: Option<String>,
) -> Result<OperationResult, String> {
    if sources.is_empty() {
        return Err("No source files specified".into());
    }
    if password.is_some() && format != "zip" {
        return Err("Password-protected archives are only supported for ZIP format".into());
    }

    match format.as_str() {
        "zip" => compress_zip(&sources, &output, password.as_deref()),
        "tar" => compress_tar(&sources, &output, "plain"),
        "tar.gz" | "tgz" => compress_tar(&sources, &output, "gzip"),
        "tar.bz2" | "tbz2" => compress_tar(&sources, &output, "bzip2"),
        "tar.xz" | "txz" => compress_tar(&sources, &output, "xz"),
        "tar.zst" | "tzst" | "zstd" => compress_tar(&sources, &output, "zstd"),
        _ => Err(format!("Unsupported format: {format}")),
    }
}

fn add_sources_to_zip<'a, W: std::io::Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    sources: &[String],
    options: zip::write::FileOptions<'a, ()>,
) -> Result<(usize, u64), String> {
    use std::io::Write;

    let mut files_processed = 0usize;
    let mut total_size = 0u64;

    for source in sources {
        let path = std::path::Path::new(source);
        if path.is_dir() {
            let walker = walkdir::WalkDir::new(path);
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let rel_path = entry.path().strip_prefix(path.parent().unwrap_or(path))
                        .unwrap_or(entry.path());
                    let name = rel_path.to_string_lossy().replace('\\', "/");

                    zip.start_file(&*name, options)
                        .map_err(|e| format!("ZIP error: {}", e))?;

                    let data = std::fs::read(entry.path())
                        .map_err(|e| format!("Read error: {}", e))?;
                    total_size += data.len() as u64;
                    zip.write_all(&data)
                        .map_err(|e| format!("ZIP write error: {}", e))?;
                    files_processed += 1;
                }
            }
        } else if path.is_file() {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".into());

            zip.start_file(&name, options)
                .map_err(|e| format!("ZIP error: {}", e))?;

            let data = std::fs::read(path)
                .map_err(|e| format!("Read error: {}", e))?;
            total_size += data.len() as u64;
            zip.write_all(&data)
                .map_err(|e| format!("ZIP write error: {}", e))?;
            files_processed += 1;
        }
    }

    Ok((files_processed, total_size))
}

fn compress_zip(
    sources: &[String],
    output: &str,
    password: Option<&str>,
) -> Result<OperationResult, String> {
    use zip::write::SimpleFileOptions;
    use zip::AesMode;

    let file = std::fs::File::create(output)
        .map_err(|e| format!("Cannot create output file: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);

    let password_buf = password.map(str::to_string);
    let base = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let (files_processed, total_size) = if let Some(pw) = password_buf.as_deref() {
        if pw.is_empty() {
            return Err("Password cannot be empty".into());
        }
        let options = base.with_aes_encryption(AesMode::Aes256, pw);
        add_sources_to_zip(&mut zip, sources, options)?
    } else {
        add_sources_to_zip(&mut zip, sources, base)?
    };

    zip.finish().map_err(|e| format!("ZIP finalize error: {}", e))?;

    let msg = if password.is_some() {
        format!(
            "Compressed {} files ({} KB) to password-protected ZIP {}",
            files_processed,
            total_size / 1024,
            output
        )
    } else {
        format!(
            "Compressed {} files ({} KB) to {}",
            files_processed,
            total_size / 1024,
            output
        )
    };

    Ok(OperationResult {
        success: true,
        output_path: output.to_string(),
        files_processed,
        total_size,
        message: msg,
    })
}

fn append_tar_sources<W: std::io::Write>(
    tar: &mut tar::Builder<W>,
    sources: &[String],
) -> Result<(usize, u64), String> {
    let mut files_processed = 0usize;
    let mut total_size = 0u64;

    for source in sources {
        let path = std::path::Path::new(source);
        if path.is_dir() {
            tar.append_dir_all(path.file_name().unwrap_or_default(), path)
                .map_err(|e| format!("TAR dir error: {e}"))?;
            for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    files_processed += 1;
                    total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        } else if path.is_file() {
            tar.append_path_with_name(path, path.file_name().unwrap_or_default())
                .map_err(|e| format!("TAR error: {e}"))?;
            files_processed += 1;
            total_size += path.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }

    Ok((files_processed, total_size))
}

fn compress_tar(sources: &[String], output: &str, variant: &str) -> Result<OperationResult, String> {
    let file = std::fs::File::create(output)
        .map_err(|e| format!("Cannot create output: {e}"))?;

    let (files_processed, total_size) = match variant {
        "gzip" => {
            let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            let mut tar = tar::Builder::new(encoder);
            let counts = append_tar_sources(&mut tar, sources)?;
            let encoder = tar.into_inner().map_err(|e| format!("TAR finalize: {e}"))?;
            encoder.finish().map_err(|e| format!("GZ finalize: {e}"))?;
            counts
        }
        "bzip2" => {
            let encoder = bzip2::write::BzEncoder::new(file, bzip2::Compression::default());
            let mut tar = tar::Builder::new(encoder);
            let counts = append_tar_sources(&mut tar, sources)?;
            let encoder = tar.into_inner().map_err(|e| format!("TAR finalize: {e}"))?;
            encoder.finish().map_err(|e| format!("BZ2 finalize: {e}"))?;
            counts
        }
        "xz" => {
            let encoder = xz2::write::XzEncoder::new(file, 6);
            let mut tar = tar::Builder::new(encoder);
            let counts = append_tar_sources(&mut tar, sources)?;
            let encoder = tar.into_inner().map_err(|e| format!("TAR finalize: {e}"))?;
            encoder.finish().map_err(|e| format!("XZ finalize: {e}"))?;
            counts
        }
        "zstd" => {
            let mut encoder = zstd::stream::write::Encoder::new(file, 3)
                .map_err(|e| format!("ZST encoder: {e}"))?;
            let mut tar = tar::Builder::new(&mut encoder);
            let counts = append_tar_sources(&mut tar, sources)?;
            tar.into_inner().map_err(|e| format!("TAR finalize: {e}"))?;
            encoder.finish().map_err(|e| format!("ZST finalize: {e}"))?;
            counts
        }
        _ => {
            let mut tar = tar::Builder::new(file);
            let counts = append_tar_sources(&mut tar, sources)?;
            tar.finish().map_err(|e| format!("TAR finalize: {e}"))?;
            counts
        }
    };

    Ok(OperationResult {
        success: true,
        output_path: output.to_string(),
        files_processed,
        total_size,
        message: format!(
            "Compressed {files_processed} files ({} KB) to {output}",
            total_size / 1024
        ),
    })
}

// ─── Extract Archive (pure Rust: zip / tar / sevenz-rust / unrar crate) ───

#[tauri::command]
pub fn extract_archive(
    archive_path: String,
    output_dir: String,
    password: Option<String>,
) -> Result<OperationResult, String> {
    let pw = password.as_deref();
    let (files_processed, total_size) =
        ysf_core::forensic::extract_archive(&archive_path, &output_dir, pw, None)?;

    Ok(OperationResult {
        success: true,
        output_path: output_dir,
        files_processed,
        total_size,
        message: format!("Extracted {files_processed} files ({} KB)", total_size / 1024),
    })
}

// ─── AES-256 Encryption ───

#[tauri::command]
pub fn encrypt_file(path: String, password: String) -> Result<String, String> {
    let data = std::fs::read(&path)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    let encrypted = ysf_core::crypto::aes_encrypt(&data, &password)?;
    let out_path = format!("{}.aes256", path);
    std::fs::write(&out_path, &encrypted)
        .map_err(|e| format!("Cannot write encrypted file: {e}"))?;
    Ok(out_path)
}

#[tauri::command]
pub fn decrypt_file(path: String, password: String) -> Result<String, String> {
    let encrypted = std::fs::read(&path)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    let decrypted = ysf_core::crypto::aes_decrypt(&encrypted, &password)?;
    let out_path = if path.ends_with(".aes256") {
        path[..path.len() - 7].to_string()
    } else {
        format!("{}.decrypted", path)
    };
    std::fs::write(&out_path, &decrypted)
        .map_err(|e| format!("Cannot write decrypted file: {e}"))?;
    Ok(out_path)
}

// ─── Utilities ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStat {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[tauri::command]
pub fn stat_paths(paths: Vec<String>) -> Vec<SourceStat> {
    paths
        .into_iter()
        .map(|path| {
            let meta = std::fs::metadata(&path).ok();
            SourceStat {
                path: path.clone(),
                is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                size: meta.map(|m| m.len()).unwrap_or(0),
            }
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub available: bool,
}

#[tauri::command]
pub fn check_tools() -> Vec<ToolStatus> {
    ysf_core::forensic::rust_backends()
        .into_iter()
        .map(|(name, available)| ToolStatus {
            name: name.to_string(),
            available,
        })
        .collect()
}

#[tauri::command]
pub fn hash_file_sha256(path: String) -> Result<String, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(&path).map_err(|e| format!("Cannot open: {e}"))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| format!("Read error: {e}"))?;
    let hashes = ysf_core::hashing::multi_hash_buffer(&data);
    hashes.sha256.ok_or_else(|| "SHA-256 failed".into())
}

#[tauri::command]
pub fn hash_archive(path: String) -> Result<ysf_core::hashing::HashSet, String> {
    ysf_core::forensic::hash_archive_file(&path)
}

#[tauri::command]
pub fn get_progress() -> ysf_core::ProgressState {
    ysf_core::get_progress()
}

#[tauri::command]
pub fn preview_archive_entry(
    archive_path: String,
    entry_path: String,
    password: Option<String>,
) -> Result<ysf_core::preview::ArchiveEntryPreview, String> {
    ysf_core::preview::preview_archive_entry(
        &archive_path,
        &entry_path,
        password.as_deref(),
    )
}

#[tauri::command]
pub fn forensic_scan_archive(
    path: String,
    password: Option<String>,
) -> Result<archive::ForensicReport, String> {
    let cancel = std::sync::atomic::AtomicBool::new(false);
    archive::forensic_scan_archive(&path, password.as_deref(), &cancel)
        .map_err(|e| format!("Forensic scan failed: {e}"))
}

#[tauri::command]
pub fn test_archive_integrity(path: String, password: Option<String>) -> Result<bool, String> {
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let report = archive::forensic_scan_archive(&path, password.as_deref(), &cancel)
        .map_err(|e| e.to_string())?;
    Ok(matches!(report.risk_label.as_str(), "Clean" | "Low Risk"))
}

#[tauri::command]
pub fn extract_archive_entries(
    archive_path: String,
    output_dir: String,
    paths: Vec<String>,
    password: Option<String>,
) -> Result<OperationResult, String> {
    if paths.is_empty() {
        return Err("No files selected".into());
    }

    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Cannot create output dir: {e}"))?;

    let pw = password.as_deref();
    let (files_processed, total_size) =
        ysf_core::forensic::extract_archive(&archive_path, &output_dir, pw, Some(&paths))?;

    Ok(OperationResult {
        success: true,
        output_path: output_dir,
        files_processed,
        total_size,
        message: format!("Extracted {files_processed} selected files ({} KB)", total_size / 1024),
    })
}

// ─── About ───

#[tauri::command]
pub fn about_info() -> serde_json::Value {
    serde_json::json!({
        "appName": "ZipLoom",
        "version": "0.1.0",
        "developer": "YSF Studio — Yusuf Shalahuddin",
        "build": "Master Build — All Features Unlocked",
        "features": [
            "Drag & Drop Archive Compression & Extraction",
            "Multi-format Support: ZIP, TAR, GZ, BZ2, XZ, 7Z, RAR",
            "Archive Inspector — Preview contents without extracting",
            "AES-256 Encryption for Sensitive Archives",
            "Clean ZIP Output — No macOS Metadata Pollution",
            "100% Offline — Zero Data Collection"
        ],
        "formats": ysf_core::archive::FORMATS_SUPPORTED,
        "disclaimer": "This software is provided 'AS-IS'. Results should be independently verified before use in legal proceedings.",
        "offline": true,
        "privacy": "100% offline — zero data collection. No telemetry, no analytics, no external network calls."
    })
}
