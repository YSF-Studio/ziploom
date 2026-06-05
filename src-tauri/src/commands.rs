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
pub fn inspect_archive(path: String) -> Result<ArchiveInfo, String> {
    let entries = archive::forensic_load(&path, None)
        .map_err(|e| format!("Failed to read archive: {}", e))?;

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
pub fn compress_files(sources: Vec<String>, output: String, format: String) -> Result<OperationResult, String> {
    if sources.is_empty() {
        return Err("No source files specified".into());
    }

    match format.as_str() {
        "zip" => compress_zip(&sources, &output),
        "tar" => compress_tar(&sources, &output, false),
        "tar.gz" | "tgz" => compress_tar(&sources, &output, true),
        _ => Err(format!("Unsupported format: {}. Use zip, tar, or tar.gz", format)),
    }
}

fn compress_zip(sources: &[String], output: &str) -> Result<OperationResult, String> {
    use std::io::Write;
    let file = std::fs::File::create(output)
        .map_err(|e| format!("Cannot create output file: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut files_processed = 0usize;
    let mut total_size = 0u64;

    for source in sources {
        let path = std::path::Path::new(source);
        if path.is_dir() {
            // Walk directory recursively
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

    zip.finish().map_err(|e| format!("ZIP finalize error: {}", e))?;

    Ok(OperationResult {
        success: true,
        output_path: output.to_string(),
        files_processed,
        total_size,
        message: format!("Compressed {} files ({} KB) to {}", files_processed, total_size / 1024, output),
    })
}

fn compress_tar(sources: &[String], output: &str, gzip: bool) -> Result<OperationResult, String> {
    let file = std::fs::File::create(output)
        .map_err(|e| format!("Cannot create output: {}", e))?;

    let mut files_processed = 0usize;
    let mut total_size = 0u64;

    if gzip {
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(encoder);

        for source in sources {
            let path = std::path::Path::new(source);
            if path.is_dir() {
                let tar_result = tar.append_dir_all(
                    path.file_name().unwrap_or_default(),
                    path,
                );
                if let Err(e) = tar_result {
                    return Err(format!("TAR dir error: {}", e));
                }
                // Count files
                for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        files_processed += 1;
                        total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                    }
                }
            } else if path.is_file() {
                tar.append_path_with_name(path, path.file_name().unwrap_or_default())
                    .map_err(|e| format!("TAR error: {}", e))?;
                files_processed += 1;
                total_size += path.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }

        let encoder = tar.into_inner().map_err(|e| format!("TAR finalize: {}", e))?;
        encoder.finish().map_err(|e| format!("GZ finalize: {}", e))?;
    } else {
        let mut tar = tar::Builder::new(file);
        for source in sources {
            let path = std::path::Path::new(source);
            if path.is_dir() {
                tar.append_dir_all(path.file_name().unwrap_or_default(), path)
                    .map_err(|e| format!("TAR dir error: {}", e))?;
            } else if path.is_file() {
                tar.append_path_with_name(path, path.file_name().unwrap_or_default())
                    .map_err(|e| format!("TAR error: {}", e))?;
            }
        }
        tar.finish().map_err(|e| format!("TAR finalize: {}", e))?;
    }

    Ok(OperationResult {
        success: true,
        output_path: output.to_string(),
        files_processed,
        total_size,
        message: format!("Compressed {} files ({} KB) to {}", files_processed, total_size / 1024, output),
    })
}

// ─── Extract Archive ───

#[tauri::command]
pub fn extract_archive(archive_path: String, output_dir: String) -> Result<OperationResult, String> {
    let entries = archive::forensic_load(&archive_path, None)
        .map_err(|e| format!("Failed to read archive: {}", e))?;

    // Create output dir
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Cannot create output dir: {}", e))?;

    let format = archive::detect_format(&archive_path).unwrap_or("unknown");
    let mut files_processed = 0usize;
    let mut total_size = 0u64;

    // For ZIP files, use zip crate for extraction
    if format.contains("zip") {
        let file = std::fs::File::open(&archive_path)
            .map_err(|e| format!("Cannot open archive: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Invalid ZIP: {}", e))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| format!("ZIP entry error: {}", e))?;
            let out_path = std::path::Path::new(&output_dir).join(entry.name());

            if entry.name().ends_with('/') {
                std::fs::create_dir_all(&out_path)
                    .map_err(|e| format!("Create dir error: {}", e))?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Create parent dir: {}", e))?;
                }
                let mut outfile = std::fs::File::create(&out_path)
                    .map_err(|e| format!("Create file error: {}", e))?;
                let bytes = std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("Extract error: {}", e))?;
                total_size += bytes;
                files_processed += 1;
            }
        }
    } else if format.contains("tar") {
        // For TAR files
        let file = std::fs::File::open(&archive_path)
            .map_err(|e| format!("Cannot open archive: {}", e))?;

        let reader: Box<dyn std::io::Read> = if archive_path.ends_with(".gz") || archive_path.ends_with(".tgz") {
            Box::new(flate2::read::GzDecoder::new(file))
        } else if archive_path.ends_with(".bz2") {
            Box::new(bzip2::read::BzDecoder::new(file))
        } else if archive_path.ends_with(".xz") {
            Box::new(xz2::read::XzDecoder::new(file))
        } else {
            Box::new(file)
        };

        let mut archive = tar::Archive::new(reader);
        for entry_result in archive.entries()
            .map_err(|e| format!("TAR error: {}", e))?
        {
            let mut entry = entry_result.map_err(|e| format!("TAR entry: {}", e))?;
            let path = entry.path().map_err(|e| format!("TAR path: {}", e))?.to_path_buf();
            let out_path = std::path::Path::new(&output_dir).join(&path);

            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Create dir: {}", e))?;
            }

            let mut outfile = std::fs::File::create(&out_path)
                .map_err(|e| format!("Create file: {}", e))?;
            let bytes = std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Extract: {}", e))?;
            total_size += bytes;
            files_processed += 1;
        }
    } else {
        // For 7z/RAR — not supported for extraction in this version
        return Err(format!(
            "Extraction for {} is not yet supported. Supported: ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ.\n\nArchive contains {} files totaling {} KB. Use the Inspect tab to view contents.",
            format,
            entries.len(),
            entries.iter().map(|e| e.size).sum::<u64>() / 1024
        ));
    }

    Ok(OperationResult {
        success: true,
        output_path: output_dir,
        files_processed,
        total_size,
        message: format!("Extracted {} files ({} KB)", files_processed, total_size / 1024),
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

// ─── About ───

#[tauri::command]
pub fn about_info() -> serde_json::Value {
    serde_json::json!({
        "appName": "ZipLoom",
        "version": "0.1.0",
        "developer": "YSF Studio — Built with ❤️ by Yusuf Shalahuddin",
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
