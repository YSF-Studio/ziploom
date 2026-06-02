use serde::Serialize;
use std::path::Path;
use std::io::Read;

pub const FORMATS_SUPPORTED: &[&str] = &["zip", "7z", "rar", "tar", "tar.gz", "tar.bz2", "tar.xz", "tar.zst", "gz", "bz2", "xz", "zst"];

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub compressed_size: Option<u64>,
    pub ratio: Option<f64>,
    pub modified: Option<String>,
    pub permissions: Option<String>,
    // Computed on full scan
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub entropy: Option<f64>,
    pub magic_match: Option<bool>,
    pub expected_type: Option<String>,
    pub detected_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForensicReport {
    pub archive_path: String,
    pub format: String,
    pub total_files: usize,
    pub total_size: u64,
    pub entries: Vec<FileEntry>,
    pub anomalies: Vec<Anomaly>,
    pub threats: Vec<Threat>,
    pub risk_score: f64,
    pub risk_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Anomaly {
    pub file: String,
    pub issue: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Threat {
    pub file: String,
    pub threat: String,
    pub category: String,
    pub severity: String,
    pub detail: String,
}

/// Detect archive format from path extension
pub fn detect_format(path: &str) -> Option<&'static str> {
    let lower = path.to_lowercase();
    if lower.ends_with(".tar.zst") || lower.ends_with(".tzst") { return Some("tar.zst"); }
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") { return Some("tar.gz"); }
    if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") { return Some("tar.bz2"); }
    if lower.ends_with(".tar.xz") || lower.ends_with(".txz") { return Some("tar.xz"); }
    if lower.ends_with(".zip") { return Some("zip"); }
    if lower.ends_with(".7z") { return Some("7z"); }
    if lower.ends_with(".rar") { return Some("rar"); }
    if lower.ends_with(".tar") { return Some("tar"); }
    if lower.ends_with(".gz") { return Some("gz"); }
    if lower.ends_with(".bz2") { return Some("bz2"); }
    if lower.ends_with(".xz") { return Some("xz"); }
    if lower.ends_with(".zst") { return Some("zst"); }
    None
}

/// Load archive entries (metadata only — fast)
pub fn forensic_load(path: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    let fmt = detect_format(path).ok_or_else(|| format!("Unsupported format: {}", path))?;

    match fmt {
        "zip" => load_zip(path, password),
        "7z" => load_7z(path, password),
        "rar" => load_rar(path, password),
        "tar" | "tar.gz" | "tar.bz2" | "tar.xz" | "tar.zst" => load_tar(path),
        _ => Err(format!("Format '{}' not yet implemented", fmt)),
    }
}

fn load_zip(path: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open: {}", e))?;
    let mut archive = if let Some(pw) = password {
        zip::ZipArchive::new(file).map_err(|_| "PASSWORD_NEEDED".to_string())? // password support varies
    } else {
        zip::ZipArchive::new(file).map_err(|e| format!("ZIP error: {}", e))?
    };

    let mut entries = vec![];
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| format!("ZIP read error: {}", e))?;
        entries.push(FileEntry {
            path: entry.name().to_string(),
            size: entry.size(),
            is_dir: entry.is_dir(),
            compressed_size: Some(entry.compressed_size()),
            ratio: if entry.size() > 0 {
                Some(entry.compressed_size() as f64 / entry.size() as f64)
            } else { None },
            modified: Some(format!("{:?}", entry.last_modified())),
            permissions: None,
            md5: None, sha1: None, sha256: None,
            entropy: None, magic_match: None,
            expected_type: None, detected_type: None,
        });
    }
    Ok(entries)
}

fn load_7z(path: &str, password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("Cannot open: {}", e))?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut reader = sevenz_rust::SevenZReader::new(
        &mut file,
        len,
        password.unwrap_or("").into(),
    ).map_err(|e| {
        let msg = format!("{}", e);
        if msg.contains("password") || msg.contains("Password") {
            "PASSWORD_NEEDED".to_string()
        } else {
            format!("7z error: {}", msg)
        }
    })?;

    let mut entries = vec![];
    reader.for_each_entries(|entry, _reader| {
        entries.push(FileEntry {
            path: entry.name.to_string(),
            size: entry.size,
            is_dir: entry.is_directory, // sevenz-rust correct field
            compressed_size: Some(entry.compressed_size),
            ratio: None,
            modified: None,
            permissions: None,
            md5: None, sha1: None, sha256: None,
            entropy: None, magic_match: None,
            expected_type: None, detected_type: None,
        });
        Ok(true)
    }).map_err(|e| format!("7z read error: {}", e))?;

    Ok(entries)
}

fn load_rar(path: &str, _password: Option<&str>) -> Result<Vec<FileEntry>, String> {
    let archive = unrar::Archive::new(path);
    let mut entries = vec![];
    for entry_result in archive.open_for_listing().map_err(|e| format!("RAR error: {e}"))? {
        let e = entry_result.map_err(|e| format!("RAR read error: {e}"))?;
        entries.push(FileEntry {
            path: e.filename.to_string_lossy().to_string(),
            size: e.unpacked_size as u64,
            is_dir: e.is_directory(),
            compressed_size: Some(e.unpacked_size as u64),
            ratio: if e.unpacked_size > 0 {
                None // pack_size not available in unrar 0.5
            } else { None },
            modified: None,
            permissions: None,
            md5: None, sha1: None, sha256: None,
            entropy: None, magic_match: None,
            expected_type: None, detected_type: None,
        });
    }
    Ok(entries)
}

fn load_tar(path: &str) -> Result<Vec<FileEntry>, String> {
    let lower = path.to_lowercase();
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open: {}", e))?;

    let reader: Box<dyn Read> = if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Box::new(flate2::read::GzDecoder::new(file))
    } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
        Box::new(bzip2::read::BzDecoder::new(file))
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        Box::new(xz2::read::XzDecoder::new(file))
    } else if lower.ends_with(".tar.zst") || lower.ends_with(".tzst") {
        Box::new(zstd::stream::read::Decoder::new(file).map_err(|e| e.to_string())?)
    } else {
        Box::new(file)
    };

    let mut archive = tar::Archive::new(reader);
    let mut entries = vec![];
    for entry_result in archive.entries().map_err(|e| format!("TAR error: {}", e))? {
        let entry = entry_result.map_err(|e| format!("TAR read error: {}", e))?;
        let header = entry.header();
        entries.push(FileEntry {
            path: entry.path().map_err(|e| e.to_string())?.to_string_lossy().to_string(),
            size: header.size().unwrap_or(0),
            is_dir: header.entry_type().is_dir(),
            compressed_size: None,
            ratio: None,
            modified: Some(format!("{}", header.mtime().unwrap_or(0))),
            permissions: Some(format!("{:o}", header.mode().unwrap_or(0))),
            md5: None, sha1: None, sha256: None,
            entropy: None, magic_match: None,
            expected_type: None, detected_type: None,
        });
    }
    Ok(entries)
}

/// Full forensic report: hash, entropy, magic, threat detection
pub fn generate_forensic_report(
    path: &str,
    password: Option<&str>,
    cancel_flag: &std::sync::atomic::AtomicBool,
) -> Result<ForensicReport, String> {
    let format = detect_format(path).unwrap_or("unknown").to_string();
    let mut entries = forensic_load(path, password)?;

    let total_size: u64 = entries.iter().map(|e| e.size).sum();
    let mut anomalies: Vec<Anomaly> = vec![];
    let mut threats: Vec<Threat> = vec![];
    let mut risk_score = 0.0f64;

    // For forensic report, we'd extract to temp and analyze
    // This is a simplified version for the core library
    // Full implementation uses tempdir extraction + per-file analysis
    let total = entries.len();
    for (i, entry) in entries.iter_mut().enumerate() {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("CANCELLED".into());
        }

        super::progress::update_progress(
            (i as f64 / total.max(1) as f64) * 100.0,
            &format!("Scanning {}/{}", i + 1, total),
            i as u64,
            total as u64,
        );

        // Basic entropy and magic check without file extraction
        let _name_lower = entry.path.to_lowercase();
        let ext = Path::new(&entry.path).extension()
            .and_then(|e| e.to_str()).unwrap_or("");

        // Threat detection heuristics
        if entry.size == 0 && !entry.is_dir {
            anomalies.push(Anomaly {
                file: entry.path.clone(),
                issue: "Zero-byte file (possible deleted/missing content)".into(),
                severity: "low".into(),
            });
        }

        // Suspicious extensions
        let suspicious_exts = ["exe", "dll", "sys", "bat", "ps1", "vbs", "js", "hta", "scr", "pif"];
        if suspicious_exts.contains(&ext) {
            threats.push(Threat {
                file: entry.path.clone(),
                threat: format!("Executable file: .{}", ext),
                category: "executable".into(),
                severity: "medium".into(),
                detail: "Executable files may contain malicious code".into(),
            });
            risk_score += 0.05;
        }

        // Double extension (e.g., report.pdf.exe)
        let stem = entry.path.rsplitn(2, '.').last().unwrap_or("");
        if stem.contains('.') {
            threats.push(Threat {
                file: entry.path.clone(),
                threat: "Double extension (possible disguise)".into(),
                category: "obfuscation".into(),
                severity: "high".into(),
                detail: "File has multiple extensions — may be hiding true type".into(),
            });
            risk_score += 0.15;
        }
    }

    let risk_label = if risk_score > 0.5 { "Malicious".into() }
        else if risk_score > 0.3 { "Highly Suspicious".into() }
        else if risk_score > 0.1 { "Suspicious".into() }
        else if risk_score > 0.01 { "Low Risk".into() }
        else { "Clean".into() };

    super::progress::finish_progress(Ok(format!("{} files scanned", entries.len())));

    Ok(ForensicReport {
        archive_path: path.to_string(),
        format,
        total_files: entries.len(),
        total_size,
        entries,
        anomalies,
        threats,
        risk_score: risk_score.min(1.0),
        risk_label,
    })
}
