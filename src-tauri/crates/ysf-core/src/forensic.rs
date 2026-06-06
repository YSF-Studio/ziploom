//! Per-entry forensic analysis and archive extraction — pure Rust (zip/tar/7z/rar crates).

use crate::archive::{detect_format, map_zip_err, zip_password_required, FileEntry};
use crate::hashing::{self, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};

const ENTROPY_SAMPLE: usize = 256 * 1024;
const MAGIC_SAMPLE: usize = 512;
/// Files larger than this are spooled to disk for hashing (RAR) instead of loading into RAM.
#[cfg(not(target_os = "windows"))]
const MAX_IN_MEMORY: u64 = 8 * 1024 * 1024;

fn file_entry_count(entries: &[FileEntry]) -> usize {
    entries.iter().filter(|e| !e.is_dir && e.size > 0).count()
}

fn scan_file_progress(done: usize, total: usize, name: &str) {
    if total == 0 {
        return;
    }
    let pct = 5.0 + (done as f64 / total as f64) * 85.0;
    crate::progress::update_progress(
        pct,
        &format!("Analyzing {name} ({done}/{total})"),
        done as u64,
        total as u64,
    );
}

fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}

/// Validate and join an archive entry name with the output directory.
/// Rejects absolute paths and parent-directory traversal (zip-slip / CWE-22).
fn safe_extract_path(output_dir: &str, entry_name: &str) -> Result<PathBuf, String> {
    let name = normalize_path(entry_name);

    if name.is_empty() {
        return Err("Empty archive entry path".into());
    }

    if name.starts_with('/') {
        return Err(format!("Rejected absolute archive path: {entry_name}"));
    }

    #[cfg(windows)]
    if name.contains(':') {
        return Err(format!("Rejected absolute archive path: {entry_name}"));
    }

    let path = Path::new(&name);
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(format!("Rejected path traversal in archive entry: {entry_name}"));
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(format!("Rejected absolute archive path: {entry_name}"));
            }
            _ => {}
        }
    }

    Ok(PathBuf::from(output_dir).join(path))
}

fn find_entry<'a>(entries: &'a mut [FileEntry], path: &str) -> Option<&'a mut FileEntry> {
    let norm = normalize_path(path);
    entries
        .iter_mut()
        .find(|e| normalize_path(&e.path) == norm)
}

#[cfg(not(target_os = "windows"))]
fn map_rar_err(e: unrar::error::UnrarError) -> String {
    let msg = format!("{e}");
    if msg.to_lowercase().contains("password") {
        "PASSWORD_NEEDED".to_string()
    } else {
        format!("RAR error: {e}")
    }
}

/// Apply entropy, magic bytes, and per-file MD5/SHA1/SHA256 from raw bytes.
pub fn enrich_entry_from_bytes(entry: &mut FileEntry, data: &[u8]) {
    if entry.is_dir || data.is_empty() {
        return;
    }

    let entropy_src = if data.len() > ENTROPY_SAMPLE {
        &data[..ENTROPY_SAMPLE]
    } else {
        data
    };
    entry.entropy = Some(hashing::compute_entropy(entropy_src));

    let magic_src = if data.len() > MAGIC_SAMPLE {
        &data[..MAGIC_SAMPLE]
    } else {
        data
    };
    let (magic_match, detected, _) = hashing::check_magic_bytes(magic_src, &entry.path);
    entry.magic_match = magic_match;
    entry.detected_type = detected;
    if magic_match == Some(false) {
        entry.expected_type = Path::new(&entry.path)
            .extension()
            .map(|e| e.to_string_lossy().to_string());
    }

    let h = hashing::multi_hash_buffer(data);
    entry.md5 = h.md5;
    entry.sha1 = h.sha1;
    entry.sha256 = h.sha256;
}

/// Enrich from a spooled file on disk (streaming hash — any file size).
#[cfg(not(target_os = "windows"))]
fn enrich_entry_from_path(entry: &mut FileEntry, path: &Path) -> Result<(), String> {
    if entry.is_dir {
        return Ok(());
    }

    let size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(entry.size);
    if size == 0 {
        return Ok(());
    }

    let prefix_len = MAGIC_SAMPLE
        .max(ENTROPY_SAMPLE)
        .min(size as usize);
    let mut prefix = vec![0u8; prefix_len];
    if prefix_len > 0 {
        let mut file = std::fs::File::open(path).map_err(|e| format!("Open {}: {e}", path.display()))?;
        file.read_exact(&mut prefix)
            .map_err(|e| format!("Read {}: {e}", entry.path))?;
    }

    let entropy_src = if prefix.len() > ENTROPY_SAMPLE {
        &prefix[..ENTROPY_SAMPLE]
    } else {
        &prefix
    };
    entry.entropy = Some(hashing::compute_entropy(entropy_src));

    let magic_src = if prefix.len() > MAGIC_SAMPLE {
        &prefix[..MAGIC_SAMPLE]
    } else {
        &prefix
    };
    let (magic_match, detected, _) = hashing::check_magic_bytes(magic_src, &entry.path);
    entry.magic_match = magic_match;
    entry.detected_type = detected;
    if magic_match == Some(false) {
        entry.expected_type = Path::new(&entry.path)
            .extension()
            .map(|e| e.to_string_lossy().to_string());
    }

    let cancel = std::sync::atomic::AtomicBool::new(false);
    let h = hashing::multi_hash(path, &cancel)?;
    entry.md5 = h.md5;
    entry.sha1 = h.sha1;
    entry.sha256 = h.sha256;
    Ok(())
}

/// Stream-read content: per-file hash always computed (no size cap).
fn enrich_from_reader(entry: &mut FileEntry, reader: &mut dyn Read, size: u64) -> Result<(), String> {
    if entry.is_dir || size == 0 {
        return Ok(());
    }

    let prefix_len = MAGIC_SAMPLE
        .max(ENTROPY_SAMPLE)
        .min(size as usize);
    let mut prefix = vec![0u8; prefix_len];
    if prefix_len > 0 {
        reader
            .read_exact(&mut prefix)
            .map_err(|e| format!("Read {}: {e}", entry.path))?;
    }

    let entropy_src = if prefix.len() > ENTROPY_SAMPLE {
        &prefix[..ENTROPY_SAMPLE]
    } else {
        &prefix
    };
    entry.entropy = Some(hashing::compute_entropy(entropy_src));

    let magic_src = if prefix.len() > MAGIC_SAMPLE {
        &prefix[..MAGIC_SAMPLE]
    } else {
        &prefix
    };
    let (magic_match, detected, _) = hashing::check_magic_bytes(magic_src, &entry.path);
    entry.magic_match = magic_match;
    entry.detected_type = detected;
    if magic_match == Some(false) {
        entry.expected_type = Path::new(&entry.path)
            .extension()
            .map(|e| e.to_string_lossy().to_string());
    }

    let cancel = std::sync::atomic::AtomicBool::new(false);
    let h = if size as usize <= prefix.len() {
        hashing::multi_hash_buffer(&prefix)
    } else {
        let remainder = size.saturating_sub(prefix.len() as u64);
        hashing::multi_hash_reader(reader, &prefix, &cancel, Some(remainder))
            .map_err(|e| format!("Hash {}: {e}", entry.path))?
    };
    entry.md5 = h.md5;
    entry.sha1 = h.sha1;
    entry.sha256 = h.sha256;
    Ok(())
}

/// Read each archive member and compute entropy / magic / per-file hashes.
pub fn enrich_entries_content(
    archive_path: &str,
    password: Option<&str>,
    entries: &mut [FileEntry],
) -> Result<(), String> {
    let fmt = detect_format(archive_path).ok_or_else(|| format!("Unsupported: {archive_path}"))?;
    match fmt {
        "zip" => enrich_zip(archive_path, password, entries),
        "tar" | "tar.gz" | "tar.bz2" | "tar.xz" | "tar.zst" => {
            enrich_tar(archive_path, entries)
        }
        "7z" => enrich_7z(archive_path, password, entries),
        "rar" => enrich_rar(archive_path, password, entries),
        _ => Err(format!("Forensic scan not supported for format: {fmt}")),
    }
}

fn enrich_zip(path: &str, password: Option<&str>, entries: &mut [FileEntry]) -> Result<(), String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("ZIP error: {e}"))?;
    let total = file_entry_count(entries);
    let mut done = 0usize;

    for i in 0..archive.len() {
        let mut entry = if let Some(pw) = password {
            archive
                .by_index_decrypt(i, pw.as_bytes())
                .map_err(map_zip_err)?
        } else {
            archive.by_index(i).map_err(|e| {
                if zip_password_required(&e) {
                    "PASSWORD_NEEDED".to_string()
                } else {
                    format!("ZIP read error: {e}")
                }
            })?
        };

        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        let size = entry.size();
        if let Some(fe) = find_entry(entries, &name) {
            enrich_from_reader(fe, &mut entry, size)?;
        } else {
            let mut sink = std::io::sink();
            std::io::copy(&mut entry, &mut sink).ok();
        }
        done += 1;
        scan_file_progress(done, total, &name);
    }
    Ok(())
}

fn open_tar_reader(path: &str) -> Result<Box<dyn Read>, String> {
    let lower = path.to_lowercase();
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open: {e}"))?;
    Ok(if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Box::new(flate2::read::GzDecoder::new(file))
    } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
        Box::new(bzip2::read::BzDecoder::new(file))
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        Box::new(xz2::read::XzDecoder::new(file))
    } else if lower.ends_with(".tar.zst") || lower.ends_with(".tzst") {
        Box::new(
            zstd::stream::read::Decoder::new(file).map_err(|e| format!("ZST error: {e}"))?,
        )
    } else {
        Box::new(file)
    })
}

fn enrich_tar(path: &str, entries: &mut [FileEntry]) -> Result<(), String> {
    let reader = open_tar_reader(path)?;
    let mut archive = tar::Archive::new(reader);
    let total = file_entry_count(entries);
    let mut done = 0usize;

    for entry_result in archive.entries().map_err(|e| format!("TAR error: {e}"))? {
        let mut entry = entry_result.map_err(|e| format!("TAR entry: {e}"))?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let name = entry
            .path()
            .map_err(|e| format!("TAR path: {e}"))?
            .to_string_lossy()
            .to_string();
        let size = entry.header().size().unwrap_or(0);
        if let Some(fe) = find_entry(entries, &name) {
            enrich_from_reader(fe, &mut entry, size)?;
        } else {
            let mut sink = std::io::sink();
            std::io::copy(&mut entry, &mut sink).ok();
        }
        done += 1;
        scan_file_progress(done, total, &name);
    }
    Ok(())
}

fn enrich_7z(path: &str, password: Option<&str>, entries: &mut [FileEntry]) -> Result<(), String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("Cannot open: {e}"))?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let pwd: sevenz_rust::Password = password.unwrap_or("").into();
    let mut reader = sevenz_rust::SevenZReader::new(&mut file, len, pwd).map_err(|e| {
        let msg = format!("{e}");
        if msg.to_lowercase().contains("password") {
            "PASSWORD_NEEDED".to_string()
        } else {
            format!("7z error: {msg}")
        }
    })?;

    let total = file_entry_count(entries);
    let done = std::cell::RefCell::new(0usize);

    reader
        .for_each_entries(|entry, reader| {
            if entry.is_directory || entry.size == 0 {
                return Ok(true);
            }
            let name = entry.name.clone();
            if let Some(fe) = find_entry(entries, &name) {
                let size = entry.size;
                if let Err(e) = enrich_from_reader(fe, reader, size) {
                    log::warn!("7z forensic {name}: {e}");
                }
            } else {
                let mut sink = std::io::sink();
                std::io::copy(reader, &mut sink).ok();
            }
            let mut d = done.borrow_mut();
            *d += 1;
            scan_file_progress(*d, total, &name);
            Ok(true)
        })
        .map_err(|e| format!("7z scan error: {e}"))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn enrich_rar(_path: &str, _password: Option<&str>, _entries: &mut [FileEntry]) -> Result<(), String> {
    Err("RAR format is not supported on Windows".into())
}

#[cfg(not(target_os = "windows"))]
fn enrich_rar(path: &str, password: Option<&str>, entries: &mut [FileEntry]) -> Result<(), String> {
    let archive = match password {
        Some(pw) => unrar::Archive::with_password(path, pw.as_bytes()),
        None => unrar::Archive::new(path),
    };
    let mut open = archive.open_for_processing().map_err(map_rar_err)?;
    let total = file_entry_count(entries);
    let mut done = 0usize;
    let spool = tempfile::tempdir().map_err(|e| format!("Temp dir: {e}"))?;

    while let Some(file) = open.read_header().map_err(map_rar_err)? {
        let name = file.entry().filename.to_string_lossy().to_string();
        if file.entry().is_directory() {
            open = file.skip().map_err(map_rar_err)?;
            continue;
        }
        let size = file.entry().unpacked_size as u64;
        if let Some(fe) = find_entry(entries, &name) {
            if size > MAX_IN_MEMORY {
                let spool_path = spool.path().join(format!("_{done}"));
                open = file.extract_to(&spool_path).map_err(map_rar_err)?;
                enrich_entry_from_path(fe, &spool_path)?;
                let _ = std::fs::remove_file(&spool_path);
            } else {
                let (data, next) = file.read().map_err(map_rar_err)?;
                enrich_entry_from_bytes(fe, &data);
                open = next;
            }
        } else if size > MAX_IN_MEMORY {
            open = file.skip().map_err(map_rar_err)?;
        } else {
            open = file.skip().map_err(map_rar_err)?;
        }
        done += 1;
        scan_file_progress(done, total, &name);
    }
    Ok(())
}

/// Hash archive container file (MD5, SHA1, SHA256, …).
pub fn hash_archive_file(path: &str) -> Result<HashSet, String> {
    crate::progress::reset_progress("Computing archive hashes…");
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let result = hashing::multi_hash(Path::new(path), &cancel);
    match &result {
        Ok(_) => crate::progress::finish_progress(Ok("Hashes complete".into())),
        Err(e) => crate::progress::finish_progress(Err(e.clone())),
    }
    result
}

/// Check if internal path matches any selected path (exact or suffix).
pub fn path_selected(internal: &str, selected: &[String]) -> bool {
    let norm = normalize_path(internal);
    selected.iter().any(|s| {
        let sel = normalize_path(s);
        norm == sel || norm.ends_with(&format!("/{sel}"))
    })
}

fn should_extract(name: &str, selected: Option<&[String]>) -> bool {
    match selected {
        None => true,
        Some(paths) if paths.is_empty() => true,
        Some(paths) => path_selected(name, paths),
    }
}

/// Extract archive — pure Rust. `selected` = None extracts all entries.
pub fn extract_archive(
    archive_path: &str,
    output_dir: &str,
    password: Option<&str>,
    selected: Option<&[String]>,
) -> Result<(usize, u64), String> {
    crate::progress::reset_progress("Extracting archive…");
    std::fs::create_dir_all(output_dir).map_err(|e| format!("Cannot create output dir: {e}"))?;
    let fmt = detect_format(archive_path).unwrap_or("unknown");
    let result = match fmt {
        "zip" => extract_zip(archive_path, output_dir, selected, password),
        f if f.contains("tar") => extract_tar(archive_path, output_dir, selected),
        "7z" => extract_7z(archive_path, output_dir, selected, password),
        "rar" => extract_rar(archive_path, output_dir, selected, password),
        _ => Err(format!("Unsupported extraction format: {fmt}")),
    };
    match &result {
        Ok((n, _)) => crate::progress::finish_progress(Ok(format!("Extracted {n} files"))),
        Err(e) => crate::progress::finish_progress(Err(e.clone())),
    }
    result
}

pub fn extract_zip_filtered(
    archive_path: &str,
    output_dir: &str,
    selected: &[String],
    password: Option<&str>,
) -> Result<(usize, u64), String> {
    extract_zip(archive_path, output_dir, Some(selected), password)
}

pub fn extract_tar_filtered(
    archive_path: &str,
    output_dir: &str,
    selected: &[String],
) -> Result<(usize, u64), String> {
    extract_tar(archive_path, output_dir, Some(selected))
}

pub fn extract_7z_filtered(
    archive_path: &str,
    output_dir: &str,
    selected: &[String],
    password: Option<&str>,
) -> Result<(usize, u64), String> {
    extract_7z(archive_path, output_dir, Some(selected), password)
}

pub fn extract_rar_filtered(
    archive_path: &str,
    output_dir: &str,
    selected: &[String],
    password: Option<&str>,
) -> Result<(usize, u64), String> {
    extract_rar(archive_path, output_dir, Some(selected), password)
}

fn extract_zip(
    archive_path: &str,
    output_dir: &str,
    selected: Option<&[String]>,
    password: Option<&str>,
) -> Result<(usize, u64), String> {
    let file = std::fs::File::open(archive_path).map_err(|e| format!("Cannot open: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("ZIP error: {e}"))?;
    let mut count = 0usize;
    let mut total = 0u64;

    for i in 0..archive.len() {
        let mut entry = if let Some(pw) = password {
            archive
                .by_index_decrypt(i, pw.as_bytes())
                .map_err(map_zip_err)?
        } else {
            archive.by_index(i).map_err(|e| {
                if zip_password_required(&e) {
                    "PASSWORD_NEEDED".to_string()
                } else {
                    format!("ZIP entry error: {e}")
                }
            })?
        };

        let name = entry.name().to_string();
        if entry.is_dir() {
            if should_extract(&name, selected) {
                let dir_path = safe_extract_path(output_dir, &name)?;
                std::fs::create_dir_all(&dir_path).map_err(|e| format!("Create dir: {e}"))?;
            }
            continue;
        }
        if !should_extract(&name, selected) {
            let mut sink = std::io::sink();
            std::io::copy(&mut entry, &mut sink).ok();
            continue;
        }

        let out_path = safe_extract_path(output_dir, &name)?;
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {e}"))?;
        }
        let mut outfile =
            std::fs::File::create(&out_path).map_err(|e| format!("Create file: {e}"))?;
        let bytes = std::io::copy(&mut entry, &mut outfile)
            .map_err(|e| format!("Extract error: {e}"))?;
        count += 1;
        total += bytes;
    }
    Ok((count, total))
}

fn extract_tar(
    archive_path: &str,
    output_dir: &str,
    selected: Option<&[String]>,
) -> Result<(usize, u64), String> {
    let reader = open_tar_reader(archive_path)?;
    let mut archive = tar::Archive::new(reader);
    let mut count = 0usize;
    let mut total = 0u64;

    for entry_result in archive.entries().map_err(|e| format!("TAR error: {e}"))? {
        let mut entry = entry_result.map_err(|e| format!("TAR entry: {e}"))?;
        let path = entry
            .path()
            .map_err(|e| format!("TAR path: {e}"))?
            .to_string_lossy()
            .to_string();

        if entry.header().entry_type().is_dir() {
            if should_extract(&path, selected) {
                let dir_path = safe_extract_path(output_dir, &path)?;
                std::fs::create_dir_all(&dir_path).map_err(|e| format!("Create dir: {e}"))?;
            }
            continue;
        }
        if !should_extract(&path, selected) {
            let mut sink = std::io::sink();
            std::io::copy(&mut entry, &mut sink).ok();
            continue;
        }

        let out_path = safe_extract_path(output_dir, &path)?;
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {e}"))?;
        }
        let mut outfile =
            std::fs::File::create(&out_path).map_err(|e| format!("Create file: {e}"))?;
        let bytes = std::io::copy(&mut entry, &mut outfile)
            .map_err(|e| format!("Extract: {e}"))?;
        count += 1;
        total += bytes;
    }
    Ok((count, total))
}

fn extract_7z(
    archive_path: &str,
    output_dir: &str,
    selected: Option<&[String]>,
    password: Option<&str>,
) -> Result<(usize, u64), String> {
    let mut file = std::fs::File::open(archive_path).map_err(|e| format!("Cannot open: {e}"))?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let pwd: sevenz_rust::Password = password.unwrap_or("").into();
    let mut reader = sevenz_rust::SevenZReader::new(&mut file, len, pwd).map_err(|e| {
        let msg = format!("{e}");
        if msg.to_lowercase().contains("password") {
            "PASSWORD_NEEDED".to_string()
        } else {
            format!("7z error: {msg}")
        }
    })?;

    let counts = std::cell::RefCell::new((0usize, 0u64));
    let extract_err = std::cell::RefCell::new(None::<String>);

    reader
        .for_each_entries(|entry, reader| {
            let name = entry.name().to_string();
            let dest = match safe_extract_path(output_dir, &name) {
                Ok(p) => p,
                Err(e) => {
                    *extract_err.borrow_mut() = Some(e);
                    return Ok(false);
                }
            };

            if entry.is_directory() {
                if should_extract(&name, selected) {
                    std::fs::create_dir_all(&dest).map_err(sevenz_rust::Error::io)?;
                }
                return Ok(true);
            }

            if !should_extract(&name, selected) {
                std::io::copy(reader, &mut std::io::sink()).map_err(sevenz_rust::Error::io)?;
                return Ok(true);
            }

            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(sevenz_rust::Error::io)?;
            }
            let mut outfile = std::fs::File::create(&dest).map_err(sevenz_rust::Error::io)?;
            let bytes = std::io::copy(reader, &mut outfile).map_err(sevenz_rust::Error::io)?;
            let mut c = counts.borrow_mut();
            c.0 += 1;
            c.1 += bytes;
            Ok(true)
        })
        .map_err(|e| format!("7z extract error: {e}"))?;

    if let Some(e) = extract_err.into_inner() {
        return Err(e);
    }

    Ok(counts.into_inner())
}

#[cfg(target_os = "windows")]
fn extract_rar(
    _archive_path: &str,
    _output_dir: &str,
    _selected: Option<&[String]>,
    _password: Option<&str>,
) -> Result<(usize, u64), String> {
    Err("RAR format is not supported on Windows".into())
}

#[cfg(not(target_os = "windows"))]
fn extract_rar(
    archive_path: &str,
    output_dir: &str,
    selected: Option<&[String]>,
    password: Option<&str>,
) -> Result<(usize, u64), String> {
    let archive = match password {
        Some(pw) => unrar::Archive::with_password(archive_path, pw.as_bytes()),
        None => unrar::Archive::new(archive_path),
    };
    let mut open = archive.open_for_processing().map_err(map_rar_err)?;
    let mut count = 0usize;
    let mut total = 0u64;

    while let Some(file) = open.read_header().map_err(map_rar_err)? {
        let name = file.entry().filename.to_string_lossy().to_string();
        if file.entry().is_directory() {
            if should_extract(&name, selected) {
                let dir_path = safe_extract_path(output_dir, &name)?;
                std::fs::create_dir_all(&dir_path).map_err(|e| format!("Create dir: {e}"))?;
            }
            open = file.skip().map_err(map_rar_err)?;
            continue;
        }
        if !should_extract(&name, selected) {
            open = file.skip().map_err(map_rar_err)?;
            continue;
        }
        let size_before = file.entry().unpacked_size;
        let out_path = safe_extract_path(output_dir, &name)?;
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {e}"))?;
        }
        open = file.extract_to(&out_path).map_err(map_rar_err)?;
        count += 1;
        total += size_before as u64;
    }
    Ok((count, total))
}

/// Bytes read from a single archive member (capped for safe preview).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryBytes {
    pub data: Vec<u8>,
    pub total_size: u64,
    pub truncated: bool,
}

/// Read up to `max_bytes` from one archive entry — never loads the full file if larger.
pub fn read_archive_entry(
    archive_path: &str,
    entry_path: &str,
    password: Option<&str>,
    max_bytes: u64,
) -> Result<EntryBytes, String> {
    let fmt = detect_format(archive_path).ok_or_else(|| format!("Unsupported: {archive_path}"))?;
    match fmt {
        "zip" => read_zip_entry(archive_path, entry_path, password, max_bytes),
        "tar" | "tar.gz" | "tar.bz2" | "tar.xz" | "tar.zst" => {
            read_tar_entry(archive_path, entry_path, max_bytes)
        }
        "7z" => read_7z_entry(archive_path, entry_path, password, max_bytes),
        "rar" => read_rar_entry(archive_path, entry_path, password, max_bytes),
        _ => Err(format!("Preview not supported for format: {fmt}")),
    }
}

fn read_limited(reader: &mut dyn Read, max_bytes: u64) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let mut limited = reader.take(max_bytes);
    limited
        .read_to_end(&mut buf)
        .map_err(|e| format!("Read error: {e}"))?;
    Ok(buf)
}

fn read_zip_entry(
    archive_path: &str,
    entry_path: &str,
    password: Option<&str>,
    max_bytes: u64,
) -> Result<EntryBytes, String> {
    let file = std::fs::File::open(archive_path).map_err(|e| format!("Cannot open: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("ZIP error: {e}"))?;
    let target = normalize_path(entry_path);

    for i in 0..archive.len() {
        let mut entry = if let Some(pw) = password {
            archive
                .by_index_decrypt(i, pw.as_bytes())
                .map_err(map_zip_err)?
        } else {
            archive.by_index(i).map_err(|e| {
                if zip_password_required(&e) {
                    "PASSWORD_NEEDED".to_string()
                } else {
                    format!("ZIP entry error: {e}")
                }
            })?
        };

        let name = normalize_path(entry.name());
        if entry.is_dir() || name != target {
            let mut sink = std::io::sink();
            std::io::copy(&mut entry, &mut sink).ok();
            continue;
        }

        let total = entry.size();
        let data = read_limited(&mut entry, max_bytes)?;
        return Ok(EntryBytes {
            truncated: total > max_bytes,
            total_size: total,
            data,
        });
    }
    Err(format!("Entry not found: {entry_path}"))
}

fn read_tar_entry(archive_path: &str, entry_path: &str, max_bytes: u64) -> Result<EntryBytes, String> {
    let reader = open_tar_reader(archive_path)?;
    let mut archive = tar::Archive::new(reader);
    let target = normalize_path(entry_path);

    for entry_result in archive.entries().map_err(|e| format!("TAR error: {e}"))? {
        let mut entry = entry_result.map_err(|e| format!("TAR entry: {e}"))?;
        let path = normalize_path(
            &entry
                .path()
                .map_err(|e| format!("TAR path: {e}"))?
                .to_string_lossy(),
        );

        if entry.header().entry_type().is_dir() || path != target {
            let mut sink = std::io::sink();
            std::io::copy(&mut entry, &mut sink).ok();
            continue;
        }

        let total = entry.header().size().unwrap_or(0);
        let data = read_limited(&mut entry, max_bytes)?;
        return Ok(EntryBytes {
            truncated: total > max_bytes,
            total_size: total,
            data,
        });
    }
    Err(format!("Entry not found: {entry_path}"))
}

fn read_7z_entry(
    archive_path: &str,
    entry_path: &str,
    password: Option<&str>,
    max_bytes: u64,
) -> Result<EntryBytes, String> {
    let mut file = std::fs::File::open(archive_path).map_err(|e| format!("Cannot open: {e}"))?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let pwd: sevenz_rust::Password = password.unwrap_or("").into();
    let mut reader = sevenz_rust::SevenZReader::new(&mut file, len, pwd).map_err(|e| {
        let msg = format!("{e}");
        if msg.to_lowercase().contains("password") {
            "PASSWORD_NEEDED".to_string()
        } else {
            format!("7z error: {msg}")
        }
    })?;

    let target = normalize_path(entry_path);
    let found = std::cell::RefCell::new(None::<EntryBytes>);
    let read_err = std::cell::RefCell::new(None::<String>);

    reader
        .for_each_entries(|entry, reader| {
            if entry.is_directory {
                return Ok(true);
            }
            let name = normalize_path(&entry.name);
            if name != target {
                let mut sink = std::io::sink();
                std::io::copy(reader, &mut sink).ok();
                return Ok(true);
            }
            let total = entry.size;
            match read_limited(reader, max_bytes) {
                Ok(data) => {
                    *found.borrow_mut() = Some(EntryBytes {
                        truncated: total > max_bytes,
                        total_size: total,
                        data,
                    });
                }
                Err(e) => {
                    *read_err.borrow_mut() = Some(e);
                }
            }
            Ok(false)
        })
        .map_err(|e| format!("7z read error: {e}"))?;

    if let Some(e) = read_err.into_inner() {
        return Err(e);
    }
    found
        .into_inner()
        .ok_or_else(|| format!("Entry not found: {entry_path}"))
}

#[cfg(target_os = "windows")]
fn read_rar_entry(
    _archive_path: &str,
    _entry_path: &str,
    _password: Option<&str>,
    _max_bytes: u64,
) -> Result<EntryBytes, String> {
    Err("RAR format is not supported on Windows".into())
}

#[cfg(not(target_os = "windows"))]
fn read_rar_entry(
    archive_path: &str,
    entry_path: &str,
    password: Option<&str>,
    max_bytes: u64,
) -> Result<EntryBytes, String> {
    let archive = match password {
        Some(pw) => unrar::Archive::with_password(archive_path, pw.as_bytes()),
        None => unrar::Archive::new(archive_path),
    };
    let mut open = archive.open_for_processing().map_err(map_rar_err)?;
    let target = normalize_path(entry_path);
    let spool = tempfile::tempdir().map_err(|e| format!("Temp dir: {e}"))?;

    while let Some(file) = open.read_header().map_err(map_rar_err)? {
        let name = normalize_path(&file.entry().filename.to_string_lossy());
        if file.entry().is_directory() || name != target {
            open = file.skip().map_err(map_rar_err)?;
            continue;
        }

        let total = file.entry().unpacked_size as u64;
        if total > max_bytes {
            let spool_path = spool.path().join("preview");
            file.extract_to(&spool_path).map_err(map_rar_err)?;
            let mut f = std::fs::File::open(&spool_path).map_err(|e| format!("Open spool: {e}"))?;
            let data = read_limited(&mut f, max_bytes)?;
            let _ = std::fs::remove_file(&spool_path);
            return Ok(EntryBytes {
                truncated: true,
                total_size: total,
                data,
            });
        }

        let (data, _) = file.read().map_err(map_rar_err)?;
        return Ok(EntryBytes {
            truncated: false,
            total_size: total,
            data: data,
        });
    }
    Err(format!("Entry not found: {entry_path}"))
}

/// Built-in Rust archive backends (no external CLI).
pub fn rust_backends() -> Vec<(&'static str, bool)> {
    vec![
        ("zip", true),
        ("tar", true),
        ("7z", true),
        ("rar", !cfg!(target_os = "windows")),
    ]
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use std::io::Write;

    fn tempdir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ziploom_sec_{label}_{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn rejects_zip_slip_paths() {
        let out = "/tmp/safe_output";
        assert!(safe_extract_path(out, "../etc/passwd").is_err());
        assert!(safe_extract_path(out, "foo/../../secret.txt").is_err());
        assert!(safe_extract_path(out, "/absolute/path.txt").is_err());
        assert!(safe_extract_path(out, "..\\windows\\system32").is_err());
    }

    #[test]
    fn allows_safe_nested_paths() {
        let out = "/tmp/safe_output";
        let path = safe_extract_path(out, "nested/deep/file.txt").unwrap();
        assert!(path.ends_with("nested/deep/file.txt"));
    }

    #[test]
    fn zip_slip_malicious_archive_blocked() {
        let dir = tempdir("zip_slip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let zip_path = dir.join("evil.zip");
        let out_dir = dir.join("extract");
        std::fs::create_dir_all(&out_dir).unwrap();

        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("../escape.txt", options).unwrap();
        zip.write_all(b"pwned").unwrap();
        zip.finish().unwrap();

        let result = extract_zip(
            zip_path.to_str().unwrap(),
            out_dir.to_str().unwrap(),
            None,
            None,
        );
        assert!(result.is_err(), "zip-slip entry must be rejected");
        assert!(!dir.parent().unwrap().join("escape.txt").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
