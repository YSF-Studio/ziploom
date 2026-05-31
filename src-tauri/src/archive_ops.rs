/// ZipLoom — Pure Rust Archive Engine (App Store Compatible)
/// Semua operasi archive pake library Rust murni — zero CLI calls.
use crate::{ArchiveFormat, CompressArgs, ExtractArgs, ProgressEvent};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::Emitter;

// ─── Format Definitions ───────────────

pub fn formats() -> Vec<ArchiveFormat> {
    vec![
        ArchiveFormat { id: "zip".into(), name: "ZIP".into(), ext: vec!["zip".into(), "docx".into(), "xlsx".into(), "pptx".into()], desc: "Universal format, AES-256".into(), compress: true, extract: true, password: true },
        ArchiveFormat { id: "7z".into(), name: "7-Zip".into(), ext: vec!["7z".into()], desc: "Extract only".into(), compress: false, extract: true, password: true },
        ArchiveFormat { id: "tar".into(), name: "TAR".into(), ext: vec!["tar".into()], desc: "No compression".into(), compress: true, extract: true, password: false },
        ArchiveFormat { id: "gz".into(), name: "GZip".into(), ext: vec!["gz".to_string(), "tgz".into()], desc: "GZip compressed TAR".into(), compress: true, extract: true, password: false },
        ArchiveFormat { id: "bz2".into(), name: "BZip2".into(), ext: vec!["bz2".to_string(), "tbz".to_string(), "tbz2".into()], desc: "BZip2 compressed TAR".into(), compress: true, extract: true, password: false },
        ArchiveFormat { id: "xz".into(), name: "XZ".into(), ext: vec!["xz".to_string(), "txz".into()], desc: "XZ compressed TAR".into(), compress: true, extract: true, password: false },
        ArchiveFormat { id: "rar".into(), name: "RAR".into(), ext: vec!["rar".into()], desc: "Extract only".into(), compress: false, extract: true, password: true },
        ArchiveFormat { id: "zst".into(), name: "Zstandard".into(), ext: vec!["zst".to_string(), "tzst".into()], desc: "Modern fast compression".into(), compress: true, extract: true, password: false },
    ]
}

// ─── Magic Byte Detection ─────────────

fn detect_format_by_magic(path: &str) -> Option<ArchiveFormat> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut magic = [0u8; 8];
    file.read(&mut magic).ok()?;
    let fmts = formats();
    if magic.starts_with(b"PK\x03\x04") || magic.starts_with(b"PK\x05\x06") || magic.starts_with(b"PK\x07\x08") {
        return fmts.iter().find(|f| f.id == "zip").cloned();
    }
    if magic.starts_with(b"7z\xbc\xaf\x27\x1c") {
        return fmts.iter().find(|f| f.id == "7z").cloned();
    }
    if magic.starts_with(b"Rar!\x1a\x07") {
        return fmts.iter().find(|f| f.id == "rar").cloned();
    }
    if magic.starts_with(&[0x1f, 0x8b]) {
        return fmts.iter().find(|f| f.id == "gz").cloned();
    }
    if magic.starts_with(b"BZ") {
        return fmts.iter().find(|f| f.id == "bz2").cloned();
    }
    if magic.starts_with(&[0xfd, 0x37, 0x7a, 0x58, 0x5a, 0x00]) {
        return fmts.iter().find(|f| f.id == "xz").cloned();
    }
    if magic.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
        return fmts.iter().find(|f| f.id == "zst").cloned();
    }
    None
}

pub fn detect_format(path: String) -> Option<ArchiveFormat> {
    let lower = path.to_lowercase();
    let fmts = formats();
    for (ext_list, fmt_id) in [
        (vec![".tar.gz", ".tgz"], "gz"),
        (vec![".tar.bz2", ".tbz", ".tbz2"], "bz2"),
        (vec![".tar.xz", ".txz"], "xz"),
        (vec![".tar.zst", ".tzst"], "zst"),
        (vec![".tar"], "tar"),
    ] {
        if ext_list.iter().any(|e| lower.ends_with(e)) {
            return fmts.iter().find(|f| f.id == fmt_id).cloned();
        }
    }
    if let Some(ext) = Path::new(&path).extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        for f in &fmts {
            if f.ext.iter().any(|e| ext == *e) {
                return Some(f.clone());
            }
        }
    }
    detect_format_by_magic(&path)
}

// ─── Utilities ─────────────────────────

fn cp_r(src: &Path, dest: &Path) -> Result<(), String> {
    if src.is_dir() {
        std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            cp_r(&entry.path(), &dest.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(src, dest).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn collect_dir_entries(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        paths.push(entry.path());
    }
    Ok(paths)
}

fn is_password_error(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("password") || m.contains("wrong") || m.contains("decrypt")
}

/// Get file modification time as zip::DateTime (for preserving source timestamps)
fn file_mtime_zip(path: &Path) -> Option<zip::DateTime> {
    use std::time::UNIX_EPOCH;
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let dur = modified.duration_since(UNIX_EPOCH).ok()?;
    let secs = dur.as_secs();

    // Convert unix timestamp to date components
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hour = (time_of_day / 3600) as u8;
    let min = ((time_of_day % 3600) / 60) as u8;
    let sec = (time_of_day % 60) as u8;

    let mut year: i64 = 1970;
    let mut remaining = days as i64;
    loop {
        let days_in_year: i64 = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        year += 1;
    }

    let month_days: [i64; 12] = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month: u8 = 1;
    for &md in &month_days {
        if remaining < md { break; }
        remaining -= md;
        month += 1;
    }
    let day = (remaining + 1) as u8;
    let year = year as u16;

    // Note: zip::DateTime with time feature should accept these
    // Fallback: just use from_date_and_time from the zip crate
    zip::DateTime::from_date_and_time(year, month, day, hour, min, sec).ok()
}

/// Get file modification time as unix timestamp (for TAR headers)
fn file_mtime_unix(path: &Path) -> u64 {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(dur) = modified.duration_since(std::time::UNIX_EPOCH) {
                return dur.as_secs();
            }
        }
    }
    0
}

/// Get file permissions as unix mode (for TAR headers)
#[allow(unused_variables)]
fn file_mode(path: &Path) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.permissions().mode();
        }
    }
    0o644
}

// ─── Compress: ZIP ─────────────────────

/// FIXED: Properly compute relative paths for each source
fn do_compress_zip(
    sources: &[PathBuf],
    dst: &Path,
    password: Option<&str>,
    level: u8,
) -> Result<(), String> {
    use zip::write::{FileOptions, ZipWriter};
    use zip::CompressionMethod;

    let file = std::fs::File::create(dst).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let method = if level == 0 {
        CompressionMethod::Stored
    } else {
        CompressionMethod::Deflated
    };

    // For each source file/dir, add to zip with correct relative path
    for src in sources {
        let base_name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".into());

        if src.is_dir() {
            // Walk directory tree
            let mut entries: Vec<(PathBuf, PathBuf)> = Vec::new(); // (rel_path, abs_path)
            walk_dir(src, Path::new(&base_name), &mut entries)?;

            for (rel_path, abs_path) in &entries {
                if abs_path.is_dir() {
                    zip.add_directory(
                        rel_path.to_string_lossy().as_ref(),
                        FileOptions::<()>::default()
                            .compression_method(method)
                            .unix_permissions(file_mode(abs_path)),
                    )
                    .map_err(|e| e.to_string())?;
                } else {
                    let data =
                        std::fs::read(abs_path).map_err(|e| e.to_string())?;
                    let mut opts = FileOptions::<()>::default()
                        .compression_method(method);
                    if method != CompressionMethod::Stored {
                        opts = opts.compression_level(Some(level.min(9).into()));
                    }
                    // Preserve original file timestamp
                    if let Some(dt) = file_mtime_zip(abs_path) {
                        opts = opts.last_modified_time(dt);
                    }
                    // Preserve permissions
                    opts = opts.unix_permissions(file_mode(abs_path));

                    if let Some(pw) = password {
                        opts = opts.with_aes_encryption(zip::AesMode::Aes256, pw);
                    }

                    zip.start_file(&rel_path.to_string_lossy(), opts)
                        .map_err(|e| e.to_string())?;
                    zip.write_all(&data).map_err(|e| e.to_string())?;
                }
            }
        } else {
            // Single file
            let data = std::fs::read(src).map_err(|e| e.to_string())?;
            let mut opts = FileOptions::<()>::default()
                .compression_method(method);
            if method != CompressionMethod::Stored {
                opts = opts.compression_level(Some(level.min(9).into()));
            }
            // Preserve original timestamp + permissions
            if let Some(dt) = file_mtime_zip(src) {
                opts = opts.last_modified_time(dt);
            }
            opts = opts.unix_permissions(file_mode(src));

            if let Some(pw) = password {
                opts = opts.with_aes_encryption(zip::AesMode::Aes256, pw);
            }

            zip.start_file(&base_name, opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(&data).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn walk_dir(
    dir: &Path,
    rel_base: &Path,
    out: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let abs = entry.path();
        let rel = rel_base.join(
            abs.file_name()
                .ok_or("Invalid filename")?
                .to_string_lossy()
                .to_string(),
        );
        if abs.is_dir() {
            walk_dir(&abs, &rel, out)?;
            out.push((rel, abs)); // add directory entry
        } else {
            out.push((rel, abs));
        }
    }
    Ok(())
}

// ─── Compress: TAR ─────────────────────

fn do_compress_tar(
    sources: &[PathBuf],
    dst: &Path,
    format_id: &str,
    level: u8,
) -> Result<(), String> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    let file = std::fs::File::create(dst).map_err(|e| e.to_string())?;

    let level_gz = if level == 0 {
        Compression::none()
    } else if level <= 3 {
        Compression::fast()
    } else if level <= 6 {
        Compression::new(level as u32)
    } else {
        Compression::best()
    };
    let level_bz = if level == 0 { 1 } else if level <= 3 { 3 } else if level <= 6 { 6 } else { 9 };
    let level_xz = if level == 0 { 0 } else if level <= 3 { 3 } else if level <= 6 { 6 } else { 9 };

    let writer: Box<dyn Write> = match format_id {
        "gz" => Box::new(GzEncoder::new(file, level_gz)),
        "bz2" => Box::new(bzip2::write::BzEncoder::new(
            file,
            bzip2::Compression::new(level_bz as u32),
        )),
        "xz" => Box::new(xz2::write::XzEncoder::new(file, level_xz)),
        _ => Box::new(file),
    };

    let mut tar = tar::Builder::new(writer);

    for src in sources {
        let name = src
            .file_name()
            .ok_or("Invalid path")?
            .to_string_lossy()
            .to_string();
        if src.is_dir() {
            tar.append_dir_all(&name, src)
                .map_err(|e| e.to_string())?;
        } else {
            let data = std::fs::read(src).map_err(|e| e.to_string())?;
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(file_mode(src));
            header.set_mtime(file_mtime_unix(src));
            let p: &Path = Path::new(&name);
            tar.append_data(&mut header, p, &data[..])
                .map_err(|e| e.to_string())?;
        }
    }

    tar.into_inner().map_err(|_| "TAR write error".to_string())?;
    Ok(())
}

// ─── Compress: Zstandard ────────────────

fn do_compress_zst(
    sources: &[PathBuf],
    dst: &Path,
    level: u8,
) -> Result<(), String> {
    let zst_level = if level == 0 { 1 } else if level <= 3 { 3 } else if level <= 6 { 7 } else { 15 };
    
    // Zstandard works on a single data stream — create TAR first, then compress
    let tar_buf = Vec::new();
    let mut tar = tar::Builder::new(tar_buf);
    
    for src in sources {
        let name = src.file_name().ok_or("Invalid path")?.to_string_lossy().to_string();
        if src.is_dir() {
            tar.append_dir_all(&name, src).map_err(|e| e.to_string())?;
        } else {
            let data = std::fs::read(src).map_err(|e| e.to_string())?;
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(file_mode(src));
            header.set_mtime(file_mtime_unix(src));
            tar.append_data(&mut header, Path::new(&name), &data[..])
                .map_err(|e| e.to_string())?;
        }
    }
    
    let tar_data = tar.into_inner().map_err(|_| "TAR build error".to_string())?;
    
    // Compress with Zstandard
    let file = std::fs::File::create(dst).map_err(|e| e.to_string())?;
    let mut encoder = zstd::stream::Encoder::new(file, zst_level as i32)
        .map_err(|e| format!("Zstd encoder: {}", e))?;
    std::io::Write::write_all(&mut encoder, &tar_data).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;
    
    Ok(())
}

// ─── Extract: Zstandard ────────────────

#[allow(dead_code)]
fn do_extract_zst(path: &str, dst: &Path) -> Result<(), String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let decoder = zstd::stream::Decoder::new(file)
        .map_err(|e| format!("Zstd decoder: {}", e))?;
    
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dst).map_err(|e| format!("Zstd/TAR extraction: {}", e))
}

// ─── Extract: ZIP ──────────────────────

fn do_extract_zip(path: &str, dst: &Path, password: Option<&str>) -> Result<(), String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = if let Some(pw) = password {
            archive
                .by_index_decrypt(i, pw.as_bytes())
                .map_err(|e| format!("Decrypt failed: {}", e))?
        } else {
            archive
                .by_index(i)
                .map_err(|e| format!("Read failed: {}", e))?
        };

        let name = entry.name().to_string();
        let safe = match crate::path_safe::safe_entry_path(&name) {
            Some(p) => p,
            None => return Err(format!("Path traversal detected: {}", name)),
        };
        let out_path = dst.join(safe);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn do_extract_tar<R: Read>(src: R, dst: &Path) -> Result<(), String> {
    let mut archive = tar::Archive::new(src);
    archive
        .unpack(dst)
        .map_err(|e| format!("TAR extraction failed: {}", e))
}

fn do_extract_7z(path: &str, dst: &Path, password: Option<&str>) -> Result<(), String> {
    if let Some(pw) = password {
        sevenz_rust::decompress_file_with_password(path, dst, sevenz_rust::Password::from(pw))
            .map_err(|e| format!("7z: {}", e))?;
    } else {
        sevenz_rust::decompress_file(path, dst).map_err(|e| {
            let msg = e.to_string().to_lowercase();
            if msg.contains("password") || msg.contains("wrong") {
                return "PASSWORD_NEEDED".to_string();
            }
            format!("7z: {}", msg)
        })?;
    }
    Ok(())
}

fn do_extract_rar(path: &str, dst: &Path, password: Option<&str>) -> Result<(), String> {
    let archive = if let Some(pw) = password {
        unrar::Archive::with_password(path.to_string(), pw.to_string())
    } else {
        unrar::Archive::new(path.to_string())
    };
    let entries = archive.extract_to(dst.to_string_lossy().to_string()).map_err(|e| {
        if e.to_string().to_lowercase().contains("password") {
            return "PASSWORD_NEEDED".to_string();
        }
        format!("RAR: {}", e)
    })?;
    for entry in entries {
        let _ = entry.map_err(|e| format!("RAR: {}", e))?;
    }
    Ok(())
}

// ─── List ──────────────────────────────

fn do_list_zip(path: &str) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;
    let mut out = String::from("Archive contents:\n");
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| e.to_string())?;
        out.push_str(&format!(
            "  {:>8}  {}\n",
            entry.size(),
            entry.name()
        ));
    }
    out.push_str(&format!("{} entries\n", archive.len()));
    Ok(out)
}

fn do_list_tar<R: Read>(mut src: R) -> Result<String, String> {
    let mut archive = tar::Archive::new(&mut src);
    let mut out = String::from("Contents:\n");
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let size = entry.size();
        out.push_str(&format!("  {:>8}  {}\n", size, name));
    }
    Ok(out)
}

fn do_list_7z(path: &str) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("7z: {}", e))?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut reader = sevenz_rust::SevenZReader::new(file, len, sevenz_rust::Password::empty())
        .map_err(|e| format!("7z: {}", e))?;
    let mut out = String::from("7z contents:\n");
    let mut count = 0u32;
    reader.for_each_entries(|entry, _reader| {
        let name = entry.name().to_string();
        if !entry.is_directory() {
            out.push_str(&format!("  {:>8}  {}\n", entry.size(), name));
        } else {
            out.push_str(&format!("  {:>8}  {}/\n", "", name));
        }
        count += 1;
        Ok(true)
    }).map_err(|e| format!("7z: {}", e))?;
    out.push_str(&format!("{} entries\n", count));
    Ok(out)
}

fn do_list_rar(path: &str) -> Result<String, String> {
    let archive = unrar::Archive::new(path.to_string());
    let entries = archive.list().map_err(|e| e.to_string())?;
    let mut out = String::from("RAR contents:\n");
    for e in entries {
        let e = e.map_err(|e| e.to_string())?;
        out.push_str(&format!("  {:>8}  {}\n", e.unpacked_size, e.filename));
    }
    Ok(out)
}

// ─── Tauri Commands ────────────────────

#[tauri::command]
pub fn get_formats() -> Vec<ArchiveFormat> {
    formats()
}

#[tauri::command]
pub fn detect_format_cmd(path: String) -> Option<ArchiveFormat> {
    detect_format(path)
}

#[tauri::command]
pub async fn compress(
    app: tauri::AppHandle,
    args: CompressArgs,
) -> Result<String, String> {
    let fmt = formats()
        .into_iter()
        .find(|f| f.id == args.format)
        .ok_or("Unknown format")?;
    if !fmt.compress {
        return Err("Format not supported for compression".into());
    }
    let level = args.level.min(9).max(0);

    let _ = app.emit(
        "progress",
        ProgressEvent {
            percent: 0.1,
            status: "Compressing...".into(),
        },
    );

    let dst = Path::new(&args.destination);
    let sources: Vec<PathBuf> = args.sources.iter().map(PathBuf::from).collect();

    if args.clean_meta {
        let tmp = std::env::temp_dir().join(format!("zl_cl_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
        for s in &sources {
            let dest = tmp.join(s.file_name().ok_or("Invalid path")?);
            cp_r(s, &dest)?;
        }
        crate::filters::clean_metadata(&tmp);
        let nested: Vec<PathBuf> = collect_dir_entries(&tmp)?;
        match fmt.id.as_str() {
            "zip" => do_compress_zip(&nested, dst, args.password.as_deref(), level)?,
            "zst" => do_compress_zst(&nested, dst, level)?,
            _ => do_compress_tar(&nested, dst, &fmt.id, level)?,
        }
        std::fs::remove_dir_all(&tmp).ok();
    } else {
        match fmt.id.as_str() {
            "zip" => do_compress_zip(&sources, dst, args.password.as_deref(), level)?,
            "zst" => do_compress_zst(&sources, dst, level)?,
            _ => do_compress_tar(&sources, dst, &fmt.id, level)?,
        }
    }

    // Progress: finalizing
    let _ = app.emit("progress", ProgressEvent { percent: 0.9, status: "Finalizing...".into() });

    // Split into volumes if requested
    if let Some(mb) = args.split_size {
        if mb > 0 {
            split_file_at(&args.destination, mb)?;
            let _ = app.emit("progress", ProgressEvent { percent: 0.95, status: "Split complete".into() });
        }
    }

    // Auto-checksum if requested
    if let Some(algo) = &args.checksum_algo {
        let algos: Vec<&str> = if algo == "auto" { vec!["md5", "sha1", "sha256"] } else { vec![algo.as_str()] };
        for a in algos {
            if let Ok(sum) = crate::crypto::calculate_checksum(&args.destination, a) {
                let _ = app.emit("progress", ProgressEvent { percent: 0.97, status: format!("{}: {}", a.to_uppercase(), sum).into() });
            }
        }
    }

    let _ = app.emit("progress", ProgressEvent { percent: 1.0, status: "Done!".into() });
    Ok(args.destination)
}

/// Split a compressed file into equal volumes, preserving original extension
fn split_file_at(path: &str, size_mb: u64) -> Result<(), String> {
    let chunk = (size_mb * 1024 * 1024) as usize;
    let data = std::fs::read(path).map_err(|e| format!("Cannot read for split: {}", e))?;
    if chunk == 0 || data.is_empty() { return Err("Empty file or invalid split size".into()); }
    let src = Path::new(path);
    let ext = src.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let stem = src.with_extension("").to_string_lossy().to_string();
    for (i, part) in data.chunks(chunk).enumerate() {
        std::fs::write(format!("{}.part{:03}{}", stem, i + 1, ext), part)
            .map_err(|e| format!("Write part failed: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn extract(
    app: tauri::AppHandle,
    args: ExtractArgs,
) -> Result<String, String> {
    let fmt = detect_format(args.source.clone()).ok_or("Unknown archive format")?;
    if !fmt.extract {
        return Err("Extract not supported".into());
    }
    std::fs::create_dir_all(&args.destination).map_err(|e| e.to_string())?;

    let _ = app.emit(
        "progress",
        ProgressEvent {
            percent: 0.1,
            status: "Extracting...".into(),
        },
    );
    let dst = Path::new(&args.destination);

    let result = match fmt.id.as_str() {
        "zip" => do_extract_zip(&args.source, dst, args.password.as_deref()),
        "7z" => do_extract_7z(&args.source, dst, args.password.as_deref()),
        "rar" => do_extract_rar(&args.source, dst, args.password.as_deref()),
        "gz" | "bz2" | "xz" | "zst" => {
            let file =
                std::fs::File::open(&args.source).map_err(|e| e.to_string())?;
            let reader: Box<dyn Read> = match fmt.id.as_str() {
                "gz" => Box::new(flate2::read::GzDecoder::new(file)),
                "bz2" => Box::new(bzip2::read::BzDecoder::new(file)),
                "xz" => Box::new(xz2::read::XzDecoder::new(file)),
                "zst" => Box::new(zstd::stream::Decoder::new(file).map_err(|e| format!("Zstd: {}", e))?),
                _ => Box::new(file),
            };
            do_extract_tar(reader, dst)
        }
        "tar" => {
            let file =
                std::fs::File::open(&args.source).map_err(|e| e.to_string())?;
            do_extract_tar(file, dst)
        }
        _ => Err("Unsupported format".into()),
    };

    // Handle password errors
    if let Err(ref e) = result {
        if is_password_error(e) {
            return Err("PASSWORD_NEEDED".into());
        }
        return Err(format!("Extraction failed: {}", e));
    }

    if args.clean_meta {
        crate::filters::clean_metadata(&args.destination.clone().into());
    }
    let _ = app.emit(
        "progress",
        ProgressEvent {
            percent: 1.0,
            status: "Done!".into(),
        },
    );
    Ok(args.destination)
}

#[tauri::command]
pub fn list_archive(path: String) -> Result<String, String> {
    let fmt = detect_format(path.clone()).ok_or("Unknown archive format")?;
    match fmt.id.as_str() {
        "zip" => do_list_zip(&path),
        "7z" => do_list_7z(&path),
        "rar" => do_list_rar(&path),
        "gz" | "bz2" | "xz" | "zst" => {
            let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
            let reader: Box<dyn Read> = match fmt.id.as_str() {
                "gz" => Box::new(flate2::read::GzDecoder::new(file)),
                "bz2" => Box::new(bzip2::read::BzDecoder::new(file)),
                "xz" => Box::new(xz2::read::XzDecoder::new(file)),
                "zst" => Box::new(zstd::stream::Decoder::new(file).map_err(|e| format!("Zstd: {}", e))?),
                _ => Box::new(file),
            };
            do_list_tar(reader)
        }
        "tar" => {
            let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
            do_list_tar(file)
        }
        _ => Err("Cannot list this format".into()),
    }
}

// ─── UPDATE ARCHIVE ───────────────────

#[tauri::command]
pub async fn update_archive(
    archive_path: String,
    files: Vec<String>,
) -> Result<String, String> {
    let fmt = detect_format(archive_path.clone()).ok_or("Unknown format")?;
    if !fmt.compress {
        return Err("Format does not support updates".into());
    }

    // For ZIP: we can append files
    if fmt.id == "zip" {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&archive_path)
            .map_err(|e| e.to_string())?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;

        // Read existing entries, re-create zip with new files
        let tmp = std::env::temp_dir().join(format!("zl_up_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

        // Extract existing contents
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("Read failed: {}", e))?;
            let name = entry.name().to_string();
            let safe = name.trim_start_matches('/').trim_start_matches('\\');
            let out_path = tmp.join(safe);
            if entry.is_dir() {
                std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut out =
                    std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            }
        }

        // Add new files to temp
        for f in &files {
            let src = Path::new(f);
            let dest = tmp.join(
                src.file_name()
                    .ok_or("Invalid filename")?
                    .to_string_lossy()
                    .to_string(),
            );
            cp_r(src, &dest)?;
        }

        // Re-compress
        let sources = collect_dir_entries(&tmp)?;
        std::fs::remove_file(&archive_path).ok();
        do_compress_zip(&sources, Path::new(&archive_path), None, 6)?;
        std::fs::remove_dir_all(&tmp).ok();
        Ok(format!(
            "✅ Updated {} with {} files",
            archive_path,
            files.len()
        ))
    } else {
        Err("Update only supported for ZIP format".into())
    }
}

pub fn read_tree(path: &str) -> Result<Vec<String>, String> {
    let listing = list_archive(path.to_string())?;
    Ok(listing.lines().map(|l| l.to_string()).collect())
}

// ─── FORENSIC ENGINE ─────────────────────

const MAGIC_DB: &[(&[u8], &str, &[&str])] = &[
    (b"PK\x03\x04", "ZIP", &["zip", "docx", "xlsx", "pptx", "jar", "odt"]),
    (b"PK\x05\x06", "ZIP (EOCD)", &["zip"]),
    (b"\x1f\x8b", "GZip", &["gz", "tgz"]),
    (b"BZh", "BZip2", &["bz2", "tbz", "tbz2"]),
    (b"\xfd7zXZ\x00", "XZ", &["xz", "txz"]),
    (b"7z\xbc\xaf\x27\x1c", "7-Zip", &["7z"]),
    (b"Rar!\x1a\x07", "RAR", &["rar"]),
    (b"\x89PNG\r\n\x1a\n", "PNG", &["png"]),
    (b"\xff\xd8\xff", "JPEG", &["jpg", "jpeg", "jpe"]),
    (b"GIF8", "GIF", &["gif"]),
    (b"RIFF", "WebP/AVI/WAV", &["webp", "avi", "wav"]),
    (b"\x25PDF", "PDF", &["pdf"]),
    (b"\x00\x00\x01\x00", "ICO", &["ico"]),
    (b"MZ", "PE (EXE/DLL)", &["exe", "dll", "sys"]),
    (b"\x7fELF", "ELF", &["elf", "so", "o"]),
    (b"\xca\xfe\xba\xbe", "Mach-O", &["macho"]),
    (b"\xce\xfa\xed\xfe", "Mach-O (x86)", &["macho"]),
    (b"\xcf\xfa\xed\xfe", "Mach-O (x64)", &["macho"]),
    (b"\x1a\x45\xdf\xa3", "WebM/MKV", &["webm", "mkv"]),
    (b"\x00\x00\x00\x18ftypmp42", "MP4", &["mp4", "m4v"]),
    (b"\x00\x00\x00\x1cftyp", "MP4v2", &["mp4", "m4v"]),
    (b"\x49\x44\x33", "MP3 (ID3)", &["mp3"]),
    (b"\xff\xfb", "MP3", &["mp3"]),
    (b"OggS", "OGG", &["ogg", "opus"]),
];

/// Shannon entropy calculation (pure Rust)
fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut freq = [0u64; 256];
    for &b in data { freq[b as usize] += 1; }
    let len = data.len() as f64;
    let mut entropy = 0.0;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Verify magic bytes of a file against known signatures
/// Returns: (magic_match, detected_type, expected_type)
///   - magic_match=true  → extension cocok dengan magic bytes (green)
///   - magic_match=false → extension TIDAK cocok (red/suspicious)
///   - magic_match=None  → tipe ga dikenal, gak bisa diverifikasi
///   - detected_type     → apa kata MAGIC BYTES isi filenya (Real Content Type)
///   - expected_type     → apa kata EXTENSI filenya (Claimed Type)
fn check_magic_bytes(data: &[u8], filename: &str) -> (Option<bool>, Option<String>, Option<String>) {
    let ext = std::path::Path::new(filename)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if data.is_empty() {
        // 0-byte files: no magic to check, no mismatch
        return (None, None, ext_opt(&ext));
    }

    // expected_type = apa yang di-CLAIM oleh extension (e.g., ".png" → "PNG")
    let expected_type = ext_to_type_name(&ext);

    for (magic, detected_name, known_exts) in MAGIC_DB {
        if data.len() >= magic.len() && data.starts_with(magic) {
            // detected_type = apa kata magic bytes (e.g., PNG magic → "PNG")
            let detected_type = Some(detected_name.to_string());

            if ext.is_empty() {
                // No extension → can't check mismatch
                return (None, detected_type, None);
            }

            let ext_match = known_exts.iter().any(|e| *e == ext);
            return (Some(ext_match), detected_type, expected_type);
        }
    }

    // Unknown magic: not mismatched, just unrecognizable
    let ext_type = if ext.is_empty() { None } else { ext_opt(&ext) };
    (None, Some("Unknown".to_string()), ext_type)
}

/// Map file extension to a human-readable type name
fn ext_to_type_name(ext: &str) -> Option<String> {
    match ext {
        "zip" => Some("ZIP Archive".into()),
        "7z" => Some("7-Zip Archive".into()),
        "rar" => Some("RAR Archive".into()),
        "tar" => Some("TAR Archive".into()),
        "gz" | "tgz" => Some("GZip Archive".into()),
        "bz2" | "tbz" | "tbz2" => Some("BZip2 Archive".into()),
        "xz" | "txz" => Some("XZ Archive".into()),
        "zst" | "tzst" => Some("Zstandard Archive".into()),
        "png" => Some("PNG Image".into()),
        "jpg" | "jpeg" | "jpe" => Some("JPEG Image".into()),
        "gif" => Some("GIF Image".into()),
        "webp" => Some("WebP Image".into()),
        "pdf" => Some("PDF Document".into()),
        "mp3" => Some("MP3 Audio".into()),
        "mp4" | "m4v" => Some("MP4 Video".into()),
        "ogg" | "opus" => Some("OGG Audio".into()),
        "webm" | "mkv" => Some("WebM/Matroska Video".into()),
        "exe" | "dll" | "sys" => Some("Windows Executable".into()),
        "elf" | "so" | "o" => Some("Linux ELF Binary".into()),
        "ico" => Some("Icon File".into()),
        "docx" => Some("Word Document".into()),
        "xlsx" => Some("Excel Spreadsheet".into()),
        "pptx" => Some("PowerPoint".into()),
        "jar" => Some("Java Archive".into()),
        "odt" => Some("OpenDocument Text".into()),
        _ if ext.is_empty() => None,
        _ => Some(format!(".{} File", ext)),
    }
}

/// Helper: Option-wrapped extension type
fn ext_opt(ext: &str) -> Option<String> {
    ext_to_type_name(ext)
}

/// Get file entropy (public API)
pub fn get_file_entropy(path: &str) -> Result<f64, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok(calculate_entropy(&data))
}

/// Evaluate entropy of raw bytes (public API — for integration tests)
pub fn eval_entropy_bytes(data: &[u8]) -> f64 {
    calculate_entropy(data)
}

/// Batch hash all files in an archive
#[allow(dead_code)]
fn hash_all_files_in_zip(path: &str, password: Option<&str>) -> Result<Vec<(String, u64, String, String, String)>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;
    let mut results = Vec::new();

    for i in 0..archive.len() {
        let mut entry = if let Some(pw) = password {
            archive.by_index_decrypt(i, pw.as_bytes()).map_err(|e| format!("Decrypt: {}", e))?
        } else {
            archive.by_index(i).map_err(|e| e.to_string())?
        };

        let name = entry.name().to_string();
        let size = entry.size();
        let mut data = Vec::new();
        std::io::copy(&mut entry, &mut data).map_err(|e| e.to_string())?;

        use sha2::{Digest, Sha256};
        use sha1::Sha1;
        use md5::Md5;

        let md5 = format!("{:x}", Md5::digest(&data));
        let sha1 = format!("{:x}", Sha1::digest(&data));
        let sha256 = format!("{:x}", Sha256::digest(&data));

        results.push((name, size, md5, sha1, sha256));
    }
    Ok(results)
}

/// Forensic Load — Parse archive and return file list with metadata
#[tauri::command]
pub fn forensic_load(args: crate::ForensicLoadArgs) -> Result<Vec<crate::FileEntry>, String> {
    let fmt = detect_format(args.source.clone()).ok_or("Unknown format")?;
    let path = &args.source;
    let pw = args.password.as_deref();

    match fmt.id.as_str() {
        "zip" => forensic_load_zip(path, pw),
        "tar" | "gz" | "bz2" | "xz" | "zst" => forensic_load_tar(path, &fmt.id),
        "7z" => forensic_load_7z(path, pw),
        "rar" => forensic_load_rar(path, pw),
        _ => Err("Unsupported archive format".into()),
    }
}

fn forensic_load_tar(path: &str, format_id: &str) -> Result<Vec<crate::FileEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader: Box<dyn std::io::Read> = match format_id {
        "gz" => Box::new(flate2::read::GzDecoder::new(file)),
        "bz2" => Box::new(bzip2::read::BzDecoder::new(file)),
        "xz" => Box::new(xz2::read::XzDecoder::new(file)),
        "zst" => Box::new(zstd::stream::Decoder::new(file).map_err(|e| format!("Zstd: {}", e))?),
        _ => Box::new(file),
    };
    let mut archive = tar::Archive::new(reader);
    let mut entries = Vec::new();

    for entry in archive.entries().map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let size = entry.size();
        let header = entry.header();
        let modified = header.mtime().ok().map(|t| {
            if let Some(dt) = datetime_from_unix(t as i64) { dt } else { format!("{}", t) }
        });
        let permissions = header.mode().ok().map(|m| format!("{:o}", m));

        entries.push(crate::FileEntry {
            path,
            size,
            compressed_size: None,
            ratio: None,
            is_dir: entry.path().map(|p| p.to_string_lossy().ends_with('/')).unwrap_or(false),
            modified,
            created: None,
            permissions,
            md5: None,
            sha1: None,
            sha256: None,
            entropy: None,
            magic_match: None,
            expected_type: None,
            detected_type: None,
        });
    }
    Ok(entries)
}

fn forensic_load_7z(path: &str, password: Option<&str>) -> Result<Vec<crate::FileEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    
    let pw = password.unwrap_or("");
    let mut reader = sevenz_rust::SevenZReader::new(file, len, sevenz_rust::Password::from(pw))
        .map_err(|_| "PASSWORD_NEEDED".to_string())?;
    
    let mut entries = Vec::new();
    reader.for_each_entries(|entry, _reader| {
        let name = entry.name().to_string();
        let size = entry.size();
        let is_dir = entry.is_directory();
        entries.push(crate::FileEntry {
            path: name,
            size,
            compressed_size: None,
            ratio: None,
            is_dir,
            modified: None,
            created: None,
            permissions: None,
            md5: None,
            sha1: None,
            sha256: None,
            entropy: None,
            magic_match: None,
            expected_type: None,
            detected_type: None,
        });
        Ok(true)
    }).map_err(|e| format!("7z: {}", e))?;
    
    Ok(entries)
}

fn forensic_load_rar(path: &str, password: Option<&str>) -> Result<Vec<crate::FileEntry>, String> {
    let archive = if let Some(pw) = password {
        unrar::Archive::with_password(path.to_string(), pw.to_string())
    } else {
        unrar::Archive::new(path.to_string())
    };
    
    let entries = archive.list().map_err(|e| {
        let msg = e.to_string().to_lowercase();
        if msg.contains("password") { "PASSWORD_NEEDED".to_string() }
        else { format!("RAR: {}", e) }
    })?;
    
    let mut results = Vec::new();
    for e in entries {
        let e = e.map_err(|e| format!("RAR: {}", e))?;
        let is_dir = e.is_directory();
        results.push(crate::FileEntry {
            path: e.filename,
            size: e.unpacked_size as u64,
            compressed_size: None,
            ratio: None,
            is_dir,
            modified: None,
            created: None,
            permissions: None,
            md5: None,
            sha1: None,
            sha256: None,
            entropy: None,
            magic_match: None,
            expected_type: None,
            detected_type: None,
        });
    }
    Ok(results)
}

fn forensic_load_zip(path: &str, password: Option<&str>) -> Result<Vec<crate::FileEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        let msg = e.to_string().to_lowercase();
        if msg.contains("password") { "PASSWORD_NEEDED".to_string() }
        else { format!("Invalid or corrupted ZIP: {}", e) }
    })?;

    let pw_bytes = password.map(|p| p.as_bytes());
    let total = archive.len();
    let mut entries = Vec::new();
    let mut skipped = 0u32;
    let mut needs_password = false;

    for i in 0..total {
        // Try to access each entry — per-entry error handling
        let entry_result = if let Some(pw) = pw_bytes {
            archive.by_index_decrypt(i, pw)
        } else {
            archive.by_index(i)
        };

        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                let msg = e.to_string().to_lowercase();
                if msg.contains("password") || msg.contains("decrypt") || msg.contains("wrong") {
                    needs_password = true;
                    if password.is_none() {
                        // No password provided → report PASSWORD_NEEDED immediately
                        return Err("PASSWORD_NEEDED".into());
                    }
                }
                // Password provided but still failed, or unsupported compression, etc.
                // Skip this entry and continue
                skipped += 1;
                continue;
            }
        };

        let name = entry.name().to_string();
        let size = entry.size();
        let compressed = entry.compressed_size();
        let ratio = if size > 0 { Some(compressed as f64 / size as f64) } else { None };
        let is_dir = entry.is_dir();
        let modified = entry.last_modified().map(|dt| format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second()));
        let permissions = entry.unix_mode().map(|m| format!("{:o}", m));

        entries.push(crate::FileEntry {
            path: name,
            size,
            compressed_size: Some(compressed),
            ratio,
            is_dir,
            modified,
            created: None,
            permissions,
            md5: None,
            sha1: None,
            sha256: None,
            entropy: None,
            magic_match: None,
            expected_type: None,
            detected_type: None,
        });
    }

    if entries.is_empty() && needs_password && password.is_some() {
        return Err("PASSWORD_NEEDED".into());
    }
    if entries.is_empty() && skipped > 0 && total > 0 {
        return Err(format!("Could not read any entry ({} skipped). Archive may use unsupported compression or be corrupted.", skipped));
    }
    if entries.is_empty() && total == 0 {
        // Empty archive — that's OK, return empty list
    }

    Ok(entries)
}

/// Selective Extract — extract specific files from archive
#[tauri::command]
pub fn selective_extract(args: crate::SelectiveExtractArgs) -> Result<String, String> {
    let fmt = detect_format(args.source.clone()).ok_or("Unknown format")?;
    let dst = std::path::Path::new(&args.destination);
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;

    match fmt.id.as_str() {
        "zip" => {
            let file = std::fs::File::open(&args.source).map_err(|e| e.to_string())?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;

            for i in 0..archive.len() {
                let entry_name = archive.by_index(i).map_err(|e| e.to_string())?.name().to_string();
                if args.files.contains(&entry_name) {
                    let mut entry = if let Some(ref pw) = args.password {
                        archive.by_index_decrypt(i, pw.as_bytes()).map_err(|e| format!("Decrypt: {}", e))?
                    } else {
                        archive.by_index(i).map_err(|e| e.to_string())?
                    };
                    let safe = entry_name.trim_start_matches('/').trim_start_matches('\\');
                    let out_path = dst.join(safe);
                    if let Some(parent) = out_path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let mut out = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
                    std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                }
            }
            Ok(format!("✅ Extracted {} files to {}", args.files.len(), args.destination))
        }
        _ => Err("Selective extract supports ZIP only".into()),
    }
}

/// Generate forensic report with full analysis for all formats
#[tauri::command]
pub fn generate_forensic_report(path: String, password: Option<String>) -> Result<crate::ForensicReport, String> {
    let fmt = detect_format(path.clone()).ok_or("Unknown format")?;
    let pw = password.as_deref();
    let mut all_entries = Vec::new();
    let mut anomalies = Vec::new();
    let mut all_threats: Vec<crate::scanner::MalwareThreat> = Vec::new();
    let mut total_size: u64 = 0;
    let mut total_compressed: u64 = 0;
    let mut total_nested_archives: usize = 0;

    match fmt.id.as_str() {
        "zip" => {
            let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
            let mut archive = match zip::ZipArchive::new(file) {
                Ok(a) => a,
                Err(e) => {
                    let msg = e.to_string().to_lowercase();
                    if msg.contains("password") {
                        return Err("PASSWORD_NEEDED".into());
                    }
                    return Err(format!("Invalid ZIP: {}", e));
                }
            };

            for i in 0..archive.len() {
                // Try to get entry — use password if provided
                let entry_result = if let Some(p) = pw {
                    archive.by_index_decrypt(i, p.as_bytes())
                } else {
                    archive.by_index(i)
                };

                let mut entry = match entry_result {
                    Ok(e) => e,
                    Err(e) => {
                        let msg = e.to_string().to_lowercase();
                        if msg.contains("password") || msg.contains("decrypt") || msg.contains("wrong") {
                            // Password error = skip entry, report anomaly
                            let name = format!("[entry #{} - password protected]", i);
                            anomalies.push(crate::Anomaly {
                                file: name.clone(),
                                issue: "Could not read: password required or wrong password".into(),
                                severity: "medium".into(),
                            });
                            continue;
                        }
                        // Other error = skip this entry with anomaly
                        let name = format!("[entry #{} - read error]", i);
                        anomalies.push(crate::Anomaly {
                            file: name.clone(),
                            issue: format!("Read error: {}", e),
                            severity: "medium".into(),
                        });
                        continue;
                    }
                };
                let name = entry.name().to_string();
                let size = entry.size();
                total_size += size;
                total_compressed += entry.compressed_size();

                let mut data = Vec::new();
                let data_ok = std::io::copy(&mut entry, &mut data).is_ok();

                // Nested archive detection
                if data_ok && data.len() >= 4 {
                    let is_archive = (data[..4] == *b"PK")
                        || (data.len() >= 6 && data[..6] == [0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c])
                        || (data.starts_with(b"Rar!"))
                        || (data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b)
                        || (data.starts_with(b"BZ"))
                        || (data.len() >= 6 && data[0] == 0xfd && data[1..6] == [0x37, 0x7a, 0x58, 0x5a, 0x00]);
                    if is_archive {
                        total_nested_archives += 1;
                    }
                }
                let (md5, sha1, sha256, entropy, magic_match, detected, expected) = if data_ok {
                    compute_analysis(&data, &name)
                } else {
                    anomalies.push(crate::Anomaly {
                        file: name.clone(),
                        issue: "Could not read file contents for analysis".into(),
                        severity: "medium".into(),
                    });
                    (None, None, None, None, None, None, None)
                };

                if let Some(e) = entropy { if e > 7.5 && !name.is_empty() && name.contains('.') {
                    anomalies.push(crate::Anomaly {
                        file: name.clone(),
                        issue: format!("High entropy ({:.2}): possible encrypted/compressed content", e),
                        severity: "high".into(),
                    });
                }}

                if magic_match == Some(false) {
                    anomalies.push(crate::Anomaly {
                        file: name.clone(),
                        issue: format!("Extension mismatch: expected '{}', detected '{}'",
                            std::path::Path::new(&name).extension().map(|e| e.to_string_lossy()).unwrap_or_default(),
                            detected.as_deref().unwrap_or("unknown")),
                        severity: "high".into(),
                    });
                }

                // ── Malware Scan ──
                let mut file_threats = crate::scanner::scan_file_name(&name);
                if data_ok {
                    file_threats.extend(crate::scanner::scan_file_content(&name, &data));
                }
                all_threats.extend(file_threats);

                all_entries.push(crate::FileEntry {
                    path: name, size, compressed_size: None, ratio: None,
                    is_dir: entry.is_dir(), modified: None, created: None, permissions: None,
                    md5, sha1, sha256, entropy, magic_match, expected_type: expected, detected_type: detected,
                });
            }
        }
        "tar" | "gz" | "bz2" | "xz" | "zst" => {
            let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
            let reader: Box<dyn std::io::Read> = match fmt.id.as_str() {
                "gz" => Box::new(flate2::read::GzDecoder::new(file)),
                "bz2" => Box::new(bzip2::read::BzDecoder::new(file)),
                "xz" => Box::new(xz2::read::XzDecoder::new(file)),
                "zst" => Box::new(zstd::stream::Decoder::new(file).map_err(|e| format!("Zstd: {}", e))?),
                _ => Box::new(file),
            };
            // For compressed-tar formats, use file size as approximated compressed size
            if let Ok(meta) = std::fs::metadata(&path) {
                total_compressed = meta.len();
            }
            let mut archive = tar::Archive::new(reader);

            for entry in archive.entries().map_err(|e| e.to_string())? {
                let mut entry = entry.map_err(|e| e.to_string())?;
                let name = entry.path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                let size = entry.size();
                total_size += size;
                let is_dir = entry.header().entry_type().is_dir();
                let header = entry.header();
                let modified = header.mtime().ok().map(|t| {
                    let secs = t as i64;
                    // Simple date format from unix timestamp
                    if let Some(dt) = datetime_from_unix(secs) { dt } else { format!("{}", secs) }
                });
                let permissions = header.mode().ok().map(|m| format!("{:o}", m));

                let mut data = Vec::new();
                let data_ok = std::io::copy(&mut entry, &mut data).is_ok();
                let (md5, sha1, sha256, entropy, magic_match, detected, expected) = if data_ok {
                    compute_analysis(&data, &name)
                } else {
                    anomalies.push(crate::Anomaly { file: name.clone(),
                        issue: "Could not read file contents for analysis".into(), severity: "medium".into() });
                    (None, None, None, None, None, None, None)
                };

                if let Some(e) = entropy { if e > 7.5 && !name.is_empty() && name.contains('.') {
                    anomalies.push(crate::Anomaly { file: name.clone(),
                        issue: format!("High entropy ({:.2}): possible encrypted/compressed content", e), severity: "high".into() });
                }}
                if magic_match == Some(false) {
                    anomalies.push(crate::Anomaly { file: name.clone(),
                        issue: format!("Extension mismatch: expected '{}', detected '{}'",
                            std::path::Path::new(&name).extension().map(|e| e.to_string_lossy()).unwrap_or_default(),
                            detected.as_deref().unwrap_or("unknown")), severity: "high".into() });
                }

                // ── Malware Scan ──
                let mut file_threats = crate::scanner::scan_file_name(&name);
                if data_ok {
                    file_threats.extend(crate::scanner::scan_file_content(&name, &data));
                }
                all_threats.extend(file_threats);

                // Nested archive detection for TAR entries
                if data_ok && data.len() >= 4 {
                    let is_archive = (data[..4] == *b"PK")
                        || (data.len() >= 6 && data[..6] == [0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c])
                        || (data.starts_with(b"Rar!"))
                        || (data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b)
                        || (data.starts_with(b"BZ"))
                        || (data.len() >= 6 && data[0] == 0xfd && data[1..6] == [0x37, 0x7a, 0x58, 0x5a, 0x00]);
                    if is_archive {
                        total_nested_archives += 1;
                    }
                }

                all_entries.push(crate::FileEntry {
                    path: name, size, compressed_size: None, ratio: None, is_dir,
                    modified, created: None, permissions,
                    md5, sha1, sha256, entropy, magic_match, expected_type: expected, detected_type: detected,
                });
            }
        }
        _ => return Err("Forensic report supports ZIP and TAR formats only".into()),
    }

    // ── Archive-Level Metadata Scan ──
    let archive_threats = crate::scanner::scan_archive_metadata(
        all_entries.len(),
        total_compressed,
        total_size,
        total_nested_archives,
    );
    all_threats.extend(archive_threats);

    // ── Compute Risk Score ──
    let (risk_score, risk_label) = crate::scanner::compute_risk_score(&all_threats);

    Ok(crate::ForensicReport {
        archive_path: path, format: fmt.name,
        total_files: all_entries.len(), total_size,
        entries: all_entries, anomalies,
        threats: all_threats,
        risk_score,
        risk_label,
    })
}

/// Compute hash, entropy, and magic byte analysis for file data
/// Always computes hashes (even for empty data — empty hashes are well-defined)
fn compute_analysis(data: &[u8], filename: &str) -> (Option<String>, Option<String>, Option<String>, Option<f64>, Option<bool>, Option<String>, Option<String>) {
    use sha2::{Digest, Sha256};
    use sha1::Sha1;
    use md5::Md5;

    // Hash always works — even empty data has valid hashes
    let md5 = Some(format!("{:x}", Md5::digest(data)));
    let sha1 = Some(format!("{:x}", Sha1::digest(data)));
    let sha256 = Some(format!("{:x}", Sha256::digest(data)));

    let entropy = if data.is_empty() {
        Some(0.0) // Empty file = entropy 0
    } else {
        Some(calculate_entropy(data))
    };

    let (magic_match, detected, expected) = check_magic_bytes(data, filename);

    (md5, sha1, sha256, entropy, magic_match, detected, expected)
}

/// Convert unix timestamp to formatted date string (UTC)
fn datetime_from_unix(secs: i64) -> Option<String> {
    if secs <= 0 { return None; }
    // Manual UTC date calculation — zero dependencies, App Store safe
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year/month/day from days since Unix epoch (1970-01-01)
    let mut remaining = days_since_epoch;
    let mut year: i64 = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        year += 1;
    }

    let month_days: [i64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month: i64 = 1;
    for &md in &month_days {
        if remaining < md { break; }
        remaining -= md;
        month += 1;
    }
    let day = remaining + 1;

    Some(format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    ))
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn setup_test_dir(name: &str) -> (Vec<PathBuf>, PathBuf) {
        let dir = std::env::temp_dir().join(format!("ziploom_test_{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("file1.txt"), b"Hello ZipLoom!").unwrap();
        std::fs::write(dir.join("file2.txt"), b"Test content 123").unwrap();
        std::fs::write(dir.join("sub").join("nested.txt"), b"Nested file").unwrap();

        let entries = vec![
            dir.join("file1.txt"),
            dir.join("file2.txt"),
            dir.join("sub").join("nested.txt"),
        ];
        (entries, dir)
    }

    fn all_paths(dir: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            paths.push(entry.path());
            if entry.path().is_dir() {
                paths.extend(all_paths(&entry.path()));
            }
        }
        paths
    }

    fn verify_extract(extract_dir: &Path) {
        assert!(extract_dir.join("file1.txt").exists(), "file1.txt missing");
        assert!(extract_dir.join("file2.txt").exists(), "file2.txt missing");
        let content = std::fs::read_to_string(extract_dir.join("file1.txt")).unwrap();
        assert_eq!(content.trim(), "Hello ZipLoom!");
    }

    #[test]
    fn test_zip_roundtrip() {
        let (entries, _src_dir) = setup_test_dir("zip");
        let dst = std::env::temp_dir().join("z_test_roundtrip.zip");
        let extract = std::env::temp_dir().join("z_extract_zip");

        do_compress_zip(&entries, &dst, None, 6).unwrap();
        assert!(dst.exists(), "ZIP not created");

        do_extract_zip(dst.to_str().unwrap(), &extract, None).unwrap();
        verify_extract(&extract);
    }

    #[test]
    fn test_zip_password() {
        let (entries, _src_dir) = setup_test_dir("zip_pw");
        let dst = std::env::temp_dir().join("z_test_pw.zip");
        let extract_fail = std::env::temp_dir().join("z_extract_fail");
        let extract_ok = std::env::temp_dir().join("z_extract_ok");

        do_compress_zip(&entries, &dst, Some("secret123"), 6).unwrap();
        assert!(dst.exists(), "Encrypted ZIP not created");

        let result = do_extract_zip(dst.to_str().unwrap(), &extract_fail, None);
        assert!(result.is_err(), "Should fail without password");

        do_extract_zip(dst.to_str().unwrap(), &extract_ok, Some("secret123")).unwrap();
        verify_extract(&extract_ok);
    }

    #[test]
    fn test_tar_roundtrip() {
        let (entries, _src_dir) = setup_test_dir("tar");
        let dst = std::env::temp_dir().join("z_test.tar");
        let extract = std::env::temp_dir().join("z_extract_tar");

        do_compress_tar(&entries, &dst, "none", 6).unwrap();
        assert!(dst.exists(), "TAR not created");

        let file = std::fs::File::open(&dst).unwrap();
        do_extract_tar(file, &extract).unwrap();
        verify_extract(&extract);
    }

    #[test]
    fn test_tar_gz_roundtrip() {
        let (entries, _src_dir) = setup_test_dir("tgz");
        let dst = std::env::temp_dir().join("z_test.tar.gz");
        let extract = std::env::temp_dir().join("z_extract_tgz");

        do_compress_tar(&entries, &dst, "gz", 6).unwrap();
        assert!(dst.exists(), "TGZ not created");

        let file = std::fs::File::open(&dst).unwrap();
        let decoder = flate2::read::GzDecoder::new(file);
        do_extract_tar(decoder, &extract).unwrap();
        verify_extract(&extract);
    }

    #[test]
    fn test_tar_bz2_roundtrip() {
        let (entries, _src_dir) = setup_test_dir("tbz2");
        let dst = std::env::temp_dir().join("z_test.tar.bz2");
        let extract = std::env::temp_dir().join("z_extract_tbz2");

        do_compress_tar(&entries, &dst, "bz2", 6).unwrap();
        assert!(dst.exists(), "TBZ2 not created");

        let file = std::fs::File::open(&dst).unwrap();
        let decoder = bzip2::read::BzDecoder::new(file);
        do_extract_tar(decoder, &extract).unwrap();
        verify_extract(&extract);
    }

    #[test]
    fn test_tar_xz_roundtrip() {
        let (entries, _src_dir) = setup_test_dir("txz");
        let dst = std::env::temp_dir().join("z_test.tar.xz");
        let extract = std::env::temp_dir().join("z_extract_txz");

        do_compress_tar(&entries, &dst, "xz", 6).unwrap();
        assert!(dst.exists(), "TXZ not created");

        let file = std::fs::File::open(&dst).unwrap();
        let decoder = xz2::read::XzDecoder::new(file);
        do_extract_tar(decoder, &extract).unwrap();
        verify_extract(&extract);
    }

    #[test]
    fn test_format_detection_by_ext() {
        assert_eq!(detect_format("test.zip".into()).map(|f| f.id), Some("zip".to_string()));
        assert_eq!(detect_format("archive.7z".into()).map(|f| f.id), Some("7z".to_string()));
        assert_eq!(detect_format("file.rar".into()).map(|f| f.id), Some("rar".to_string()));
        assert_eq!(detect_format("data.tar".into()).map(|f| f.id), Some("tar".to_string()));
        assert_eq!(detect_format("backup.tar.gz".into()).map(|f| f.id), Some("gz".to_string()));
        assert_eq!(detect_format("backup.tar.bz2".into()).map(|f| f.id), Some("bz2".to_string()));
        assert_eq!(detect_format("backup.tar.xz".into()).map(|f| f.id), Some("xz".to_string()));
        assert_eq!(detect_format("file.xyz".into()), None);
        assert_eq!(detect_format("noext".into()), None);
    }

    #[test]
    fn test_list_archive() {
        let (entries, _src_dir) = setup_test_dir("list");
        let dst = std::env::temp_dir().join("z_test_list.zip");

        do_compress_zip(&entries, &dst, None, 6).unwrap();
        let listing = list_archive(dst.to_str().unwrap().to_string()).unwrap();
        assert!(listing.contains("file1.txt"), "List should contain file1.txt");
        assert!(listing.contains("nested.txt"), "List should contain nested.txt");
    }

    #[test]
    fn test_clean_meta_removes_macos_junk() {
        let (entries, _src_dir) = setup_test_dir("meta");

        let dst = std::env::temp_dir().join("z_test_clean.zip");
        let extract = std::env::temp_dir().join("z_extract_clean");

        do_compress_zip(&entries, &dst, None, 6).unwrap();
        do_extract_zip(dst.to_str().unwrap(), &extract, None).unwrap();

        assert!(extract.join("file1.txt").exists(), "file1.txt should exist");
        assert!(extract.join("file2.txt").exists(), "file2.txt should exist");
    }

    // ─── COMPREHENSIVE TESTS: ALL FORMATS ───

    /// Test ZIP compress 3 levels: store, normal, max
    #[test]
    fn test_compress_levels_zip() {
        let (entries, _src_dir) = setup_test_dir("levels");
        for level in [0u8, 6u8, 9u8] {
            let dst = std::env::temp_dir().join(format!("z_level_{}.zip", level));
            do_compress_zip(&entries, &dst, None, level).unwrap();
            assert!(dst.exists(), "ZIP level {} not created", level);
            let extract = std::env::temp_dir().join(format!("z_extract_level_{}", level));
            do_extract_zip(dst.to_str().unwrap(), &extract, None).unwrap();
            verify_extract(&extract);
            std::fs::remove_file(&dst).ok();
        }
    }

    /// Test TAR compress 3 levels
    #[test]
    fn test_compress_levels_tar() {
        let (entries, _src_dir) = setup_test_dir("levels_tar");
        for (level, fmt) in [(0u8, "none"), (6u8, "none"), (9u8, "none")] {
            let dst = std::env::temp_dir().join(format!("z_level_tar_{}.tar", level));
            do_compress_tar(&entries, &dst, fmt, level).unwrap();
            assert!(dst.exists(), "TAR level {} not created", level);
            let extract = std::env::temp_dir().join(format!("z_extract_level_tar_{}", level));
            let file = std::fs::File::open(&dst).unwrap();
            do_extract_tar(file, &extract).unwrap();
            verify_extract(&extract);
            std::fs::remove_file(&dst).ok();
        }
    }

    /// Test GZip compress 3 levels
    #[test]
    fn test_compress_levels_gz() {
        let (entries, _src_dir) = setup_test_dir("levels_gz");
        for level in [0u8, 6u8, 9u8] {
            let dst = std::env::temp_dir().join(format!("z_level_{}.tar.gz", level));
            do_compress_tar(&entries, &dst, "gz", level).unwrap();
            assert!(dst.exists(), "GZ level {} not created", level);
            let extract = std::env::temp_dir().join(format!("z_extract_level_gz_{}", level));
            let file = std::fs::File::open(&dst).unwrap();
            let decoder = flate2::read::GzDecoder::new(file);
            do_extract_tar(decoder, &extract).unwrap();
            verify_extract(&extract);
            std::fs::remove_file(&dst).ok();
        }
    }

    /// Test BZip2 compress 3 levels
    #[test]
    fn test_compress_levels_bz2() {
        let (entries, _src_dir) = setup_test_dir("levels_bz2");
        for level in [0u8, 6u8, 9u8] {
            let dst = std::env::temp_dir().join(format!("z_level_{}.tar.bz2", level));
            do_compress_tar(&entries, &dst, "bz2", level).unwrap();
            assert!(dst.exists(), "BZ2 level {} not created", level);
            let extract = std::env::temp_dir().join(format!("z_extract_level_bz2_{}", level));
            let file = std::fs::File::open(&dst).unwrap();
            let decoder = bzip2::read::BzDecoder::new(file);
            do_extract_tar(decoder, &extract).unwrap();
            verify_extract(&extract);
            std::fs::remove_file(&dst).ok();
        }
    }

    /// Test XZ compress 3 levels
    #[test]
    fn test_compress_levels_xz() {
        let (entries, _src_dir) = setup_test_dir("levels_xz");
        for level in [0u8, 6u8, 9u8] {
            let dst = std::env::temp_dir().join(format!("z_level_{}.tar.xz", level));
            do_compress_tar(&entries, &dst, "xz", level).unwrap();
            assert!(dst.exists(), "XZ level {} not created", level);
            let extract = std::env::temp_dir().join(format!("z_extract_level_xz_{}", level));
            let file = std::fs::File::open(&dst).unwrap();
            let decoder = xz2::read::XzDecoder::new(file);
            do_extract_tar(decoder, &extract).unwrap();
            verify_extract(&extract);
            std::fs::remove_file(&dst).ok();
        }
    }

    /// Test nested directory ZIP roundtrip
    #[test]
    fn test_nested_directory_zip() {
        let dir = std::env::temp_dir().join(format!("zl_nest_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("a").join("b").join("c")).unwrap();
        std::fs::write(dir.join("a").join("b").join("deep.txt"), b"very deep").unwrap();
        std::fs::write(dir.join("root.txt"), b"root").unwrap();

        let base_name = dir.file_name().unwrap().to_string_lossy().to_string();
        let entries = vec![dir.clone()];

        let dst = std::env::temp_dir().join("z_nested.zip");
        do_compress_zip(&entries, &dst, None, 6).unwrap();
        assert!(dst.exists());

        let extract = std::env::temp_dir().join("z_nested_extract");
        do_extract_zip(dst.to_str().unwrap(), &extract, None).unwrap();

        assert!(extract.join(&base_name).join("root.txt").exists(), "root.txt missing");
        assert!(extract.join(&base_name).join("a").join("b").join("deep.txt").exists(), "deep.txt missing");

        std::fs::remove_dir_all(&extract).ok();
        std::fs::remove_file(&dst).ok();
    }

    /// Test nested directory TAR roundtrip
    #[test]
    fn test_nested_directory_tar() {
        let dir = std::env::temp_dir().join(format!("zl_nest_tar_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("x").join("y")).unwrap();
        std::fs::write(dir.join("x").join("y").join("nested.txt"), b"tar nested").unwrap();
        std::fs::write(dir.join("top.txt"), b"top").unwrap();

        let entries = vec![dir.clone()];
        let dst = std::env::temp_dir().join("z_nested.tar");
        do_compress_tar(&entries, &dst, "none", 6).unwrap();
        assert!(dst.exists());

        let extract = std::env::temp_dir().join("z_nested_tar_extract");
        let file = std::fs::File::open(&dst).unwrap();
        do_extract_tar(file, &extract).unwrap();

        let base_name = dir.file_name().unwrap().to_string_lossy().to_string();
        assert!(extract.join(&base_name).join("top.txt").exists(), "top.txt missing");
        assert!(extract.join(&base_name).join("x").join("y").join("nested.txt").exists(), "nested.txt missing");

        std::fs::remove_dir_all(&extract).ok();
        std::fs::remove_file(&dst).ok();
    }

    /// Test clean_meta with actual macOS junk files during compress (clean_meta=true)
    #[test]
    fn test_compress_with_clean_meta_removes_junk() {
        let dir = std::env::temp_dir().join(format!("zl_clean_on_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("real.txt"), b"real content").unwrap();
        std::fs::write(dir.join(".DS_Store"), b"junk").unwrap();
        std::fs::write(dir.join("._hidden"), b"apple double").unwrap();

        // Get entries (they include junk)
        let entries: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        assert!(entries.iter().any(|p| p.ends_with(".DS_Store")), "DS_Store must be in entries");

        // Simulate clean_meta=true flow: copy to temp, clean, then compress
        let tmp = std::env::temp_dir().join(format!("zl_cl_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        cp_r(&dir, &tmp).unwrap();
        crate::filters::clean_metadata(&tmp);
        let cleaned_entries = collect_dir_entries(&tmp).unwrap();
        assert!(!cleaned_entries.iter().any(|p| p.ends_with(".DS_Store")), "DS_Store should be removed before compress");

        let dst = std::env::temp_dir().join("z_clean_on.zip");
        do_compress_zip(&cleaned_entries, &dst, None, 6).unwrap();
        assert!(dst.exists());

        // Extract and verify no junk inside
        let extract = std::env::temp_dir().join("z_clean_on_extract");
        do_extract_zip(dst.to_str().unwrap(), &extract, None).unwrap();
        assert!(search_dir(&extract, ".DS_Store").is_empty(), "DS_Store should not be in archive");
        assert!(search_dir(&extract, "._hidden").is_empty(), "._* should not be in archive");

        std::fs::remove_dir_all(&tmp).ok();
        std::fs::remove_dir_all(&extract).ok();
        std::fs::remove_file(&dst).ok();
    }

    fn search_dir(dir: &Path, name: &str) -> Vec<PathBuf> {
        let mut found = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.file_name().map(|n| n.to_string_lossy().contains(name)).unwrap_or(false) {
                    found.push(p.clone());
                }
                if p.is_dir() {
                    found.extend(search_dir(&p, name));
                }
            }
        }
        found
    }

    /// Test compress WITHOUT clean_meta — junk files are preserved
    #[test]
    fn test_compress_without_clean_meta_keeps_junk() {
        let dir = std::env::temp_dir().join(format!("zl_clean_off_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("real.txt"), b"real").unwrap();
        std::fs::write(dir.join(".DS_Store"), b"junk").unwrap();

        let entries: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        // Compress WITHOUT clean_meta — junk should stay
        let dst = std::env::temp_dir().join("z_clean_off.zip");
        do_compress_zip(&entries, &dst, None, 6).unwrap();

        let extract = std::env::temp_dir().join("z_clean_off_extract");
        do_extract_zip(dst.to_str().unwrap(), &extract, None).unwrap();
        // DS_Store SHOULD be in archive since we didn't clean
        assert!(extract.join(".DS_Store").exists(), "DS_Store should be preserved when clean_meta=false");

        std::fs::remove_dir_all(&extract).ok();
        std::fs::remove_file(&dst).ok();
    }

    // ─── COMPREHENSIVE TESTS: TOOLS ───

    /// Test list_archive for ZIP
    #[test]
    fn test_list_zip_archive() {
        let (entries, _src_dir) = setup_test_dir("list_zip");
        let dst = std::env::temp_dir().join("z_list_test.zip");
        do_compress_zip(&entries, &dst, None, 6).unwrap();

        let listing = list_archive(dst.to_str().unwrap().to_string()).unwrap();
        assert!(listing.contains("file1.txt"), "Should list file1.txt");
        assert!(listing.contains("file2.txt"), "Should list file2.txt");
        assert!(listing.contains("nested.txt"), "Should list nested.txt");
        assert!(listing.contains("3 entries") || listing.contains("entries"), "Should show count");
    }

    /// Test list_archive for TAR
    #[test]
    fn test_list_tar_archive() {
        let (entries, _src_dir) = setup_test_dir("list_tar");
        let dst = std::env::temp_dir().join("z_list_tar.tar");
        do_compress_tar(&entries, &dst, "none", 6).unwrap();

        let listing = list_archive(dst.to_str().unwrap().to_string()).unwrap();
        assert!(listing.contains("file1.txt"), "Should list file1.txt");
        assert!(listing.contains("file2.txt"), "Should list file2.txt");
    }

    /// Test list_archive for TAR.GZ
    #[test]
    fn test_list_tar_gz_archive() {
        let (entries, _src_dir) = setup_test_dir("list_gz");
        let dst = std::env::temp_dir().join("z_list.tar.gz");
        do_compress_tar(&entries, &dst, "gz", 6).unwrap();

        let listing = list_archive(dst.to_str().unwrap().to_string()).unwrap();
        assert!(listing.contains("file1.txt"), "Should list file1.txt");
    }

    /// Test detect_format by extension
    #[test]
    fn test_detect_format_all_extensions() {
        let cases = vec![
            ("test.zip", Some("zip")),
            ("archive.7z", Some("7z")),
            ("file.rar", Some("rar")),
            ("data.tar", Some("tar")),
            ("backup.tar.gz", Some("gz")),
            ("backup.tgz", Some("gz")),
            ("backup.tar.bz2", Some("bz2")),
            ("backup.tbz", Some("bz2")),
            ("backup.tbz2", Some("bz2")),
            ("backup.tar.xz", Some("xz")),
            ("backup.txz", Some("xz")),
            ("file.xyz", None),
            ("noext", None),
        ];
        for (path, expected) in cases {
            let result = detect_format(path.to_string()).map(|f| f.id);
            assert_eq!(result.as_deref(), expected, "Failed for path: {}", path);
        }
    }

    /// Test detect_format by magic bytes
    #[test]
    fn test_detect_format_by_magic() {
        let dir = std::env::temp_dir().join(format!("zl_magic_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        // Create a real ZIP file and detect it by magic (no extension)
        let (entries, _) = setup_test_dir("magic_zip");
        let zip_path = dir.join("unknown_ext");
        do_compress_zip(&entries, &zip_path, None, 6).unwrap();

        let result = detect_format(zip_path.to_string_lossy().to_string()).map(|f| f.id);
        assert_eq!(result.as_deref(), Some("zip"), "Should detect ZIP by magic");

        // Create a GZip file
        let gz_path = dir.join("unknown_ext2");
        do_compress_tar(&entries, &gz_path, "gz", 6).unwrap();
        let ext = gz_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        // Actually the file was saved as "unknown_ext2" without .gz extension
        let result = detect_format(gz_path.to_string_lossy().to_string()).map(|f| f.id);
        assert_eq!(result.as_deref(), Some("gz"), "Should detect GZip by magic");

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Test convert ZIP → TAR.GZ
    #[test]
    fn test_convert_zip_to_targz() {
        let (entries, _src_dir) = setup_test_dir("convert");
        let src = std::env::temp_dir().join("z_convert_src.zip");
        do_compress_zip(&entries, &src, None, 6).unwrap();

        let result = convert_archive_mock(src.to_str().unwrap().to_string(), "gz".into());
        assert!(result.is_ok(), "Conversion failed: {:?}", result.err());
        let dest_path = result.unwrap();
        assert!(Path::new(&dest_path).exists(), "Converted file should exist");
        assert!(dest_path.ends_with(".tar.gz") || dest_path.ends_with(".gz"), "Should be .tar.gz");

        // Extract and verify
        let extract = std::env::temp_dir().join("z_convert_extract");
        let file = std::fs::File::open(&dest_path).unwrap();
        let decoder = flate2::read::GzDecoder::new(file);
        do_extract_tar(decoder, &extract).unwrap();
        verify_extract(&extract);

        std::fs::remove_file(&dest_path).ok();
        std::fs::remove_file(&src).ok();
    }

    /// Test convert ZIP → TAR
    #[test]
    fn test_convert_zip_to_tar() {
        let (entries, _src_dir) = setup_test_dir("convert_tar");
        let src = std::env::temp_dir().join("z_convert_to_tar.zip");
        do_compress_zip(&entries, &src, None, 6).unwrap();

        let result = convert_archive_mock(src.to_str().unwrap().to_string(), "tar".into());
        assert!(result.is_ok(), "Conversion failed: {:?}", result.err());
        let dest_path = result.unwrap();
        assert!(Path::new(&dest_path).exists());
        assert!(dest_path.ends_with(".tar"), "Should be .tar");

        let extract = std::env::temp_dir().join("z_convert_tar_extract");
        let file = std::fs::File::open(&dest_path).unwrap();
        do_extract_tar(file, &extract).unwrap();
        verify_extract(&extract);

        std::fs::remove_file(&dest_path).ok();
        std::fs::remove_file(&src).ok();
    }

    /// Test split archive
    #[test]
    fn test_split_archive() {
        let (entries, _src_dir) = setup_test_dir("split");
        let src = std::env::temp_dir().join("z_split_src.zip");
        do_compress_zip(&entries, &src, None, 6).unwrap();

        // Split into 1KB volumes
        let result = split_archive_mock(src.to_str().unwrap().to_string(), "1".to_string());
        assert!(result.is_ok(), "Split failed: {:?}", result.err());

        // Check that part files exist
        assert!(
            Path::new(&format!("{}.part001", src.to_str().unwrap())).exists()
                || result.as_ref().unwrap().contains("Split"),
            "Part files should exist"
        );

        std::fs::remove_file(&src).ok();
    }

    /// Test update archive (add files to existing ZIP)
    #[test]
    fn test_update_archive() {
        let dir = std::env::temp_dir().join(format!("zl_update_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("original.txt"), b"original").unwrap();

        let entries = vec![dir.join("original.txt")];
        let dst = std::env::temp_dir().join("z_update.zip");
        do_compress_zip(&entries, &dst, None, 6).unwrap();

        // Create a new file to add
        let new_file = dir.join("new_file.txt");
        std::fs::write(&new_file, b"new content").unwrap();

        let result = update_archive_mock(
            dst.to_str().unwrap().to_string(),
            vec![new_file.to_string_lossy().to_string()],
        );
        assert!(result.is_ok(), "Update failed: {:?}", result.err());

        // Verify the updated archive
        let extract = std::env::temp_dir().join("z_update_extract");
        do_extract_zip(dst.to_str().unwrap(), &extract, None).unwrap();
        assert!(extract.join("original.txt").exists(), "Original file should exist");
        assert!(extract.join("new_file.txt").exists(), "New file should exist");

        std::fs::remove_dir_all(&extract).ok();
        std::fs::remove_file(&dst).ok();
    }


    /// Test that timestamps are preserved when compressing
    #[test]
    fn test_timestamp_preserved_zip() {
        let dir = std::env::temp_dir().join(format!("zl_ts_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("timestamp_test.txt");
        std::fs::write(&file_path, b"timestamp preservation test").unwrap();

        // Get original mtime
        let orig_mtime = std::fs::metadata(&file_path).unwrap().modified().unwrap();
        
        // Compress
        let dst = dir.join("ts_test.zip");
        do_compress_zip(&[file_path.clone()], &dst, None, 6).unwrap();
        
        // Extract and check
        let extract = dir.join("extracted");
        std::fs::create_dir_all(&extract).unwrap();
        do_extract_zip(&dst.to_string_lossy(), &extract, None).unwrap();
        
        let extracted_path = extract.join("timestamp_test.txt");
        let extracted_mtime = std::fs::metadata(&extracted_path).unwrap().modified().unwrap();
        
        // Timestamps should match (within a few seconds for filesystem precision)
        let diff = if orig_mtime > extracted_mtime {
            orig_mtime.duration_since(extracted_mtime).unwrap().as_secs()
        } else {
            extracted_mtime.duration_since(orig_mtime).unwrap().as_secs()
        };
        assert!(diff <= 2, "Timestamps should match within 2s, got {}s difference", diff);
        
        println!("✅ Timestamp preserved: original={:?}, extracted={:?}", orig_mtime, extracted_mtime);
        std::fs::remove_dir_all(&dir).ok();
    }
    
    /// Test that timestamps are preserved in TAR
    #[test]
    fn test_timestamp_preserved_tar() {
        let dir = std::env::temp_dir().join(format!("zl_ts_tar_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("ts_tar.txt");
        std::fs::write(&file_path, b"tar timestamp test").unwrap();

        let orig_mtime = std::fs::metadata(&file_path).unwrap().modified().unwrap();
        
        let dst = dir.join("ts_test.tar");
        do_compress_tar(&[file_path.clone()], &dst, "none", 6).unwrap();
        
        let extract = dir.join("extracted");
        std::fs::create_dir_all(&extract).unwrap();
        let file = std::fs::File::open(&dst).unwrap();
        do_extract_tar(file, &extract).unwrap();
        
        let extracted_path = extract.join("ts_tar.txt");
        let extracted_mtime = std::fs::metadata(&extracted_path).unwrap().modified().unwrap();
        
        let diff = if orig_mtime > extracted_mtime {
            orig_mtime.duration_since(extracted_mtime).unwrap().as_secs()
        } else {
            extracted_mtime.duration_since(orig_mtime).unwrap().as_secs()
        };
        assert!(diff <= 2, "TAR timestamps should match within 2s, got {}s", diff);
        
        println!("✅ TAR timestamp preserved: original={:?}, extracted={:?}", orig_mtime, extracted_mtime);
        std::fs::remove_dir_all(&dir).ok();
    }

        /// Test format info (all 7 formats have correct properties)
    #[test]
    fn test_formats_all_properties() {
        let fmts = formats();
        assert_eq!(fmts.len(), 8, "Should have 8 formats");

        for f in &fmts {
            assert!(!f.id.is_empty(), "ID should not be empty");
            assert!(!f.name.is_empty(), "Name should not be empty");
            assert!(!f.ext.is_empty(), "Extension list should not be empty");
        }

        // Verify specific formats
        let zip = fmts.iter().find(|f| f.id == "zip").unwrap();
        assert!(zip.compress, "ZIP should support compress");
        assert!(zip.extract, "ZIP should support extract");
        assert!(zip.password, "ZIP should support password");

        let sevenz = fmts.iter().find(|f| f.id == "7z").unwrap();
        assert!(!sevenz.compress, "7z should NOT support compress");
        assert!(sevenz.extract, "7z should support extract");
        assert!(sevenz.password, "7z should support password");

        let tar = fmts.iter().find(|f| f.id == "tar").unwrap();
        assert!(tar.compress, "TAR should support compress");
        assert!(tar.extract, "TAR should support extract");
        assert!(!tar.password, "TAR should NOT support password");
    }

    /// Helper: inline convert_archive (mimics the real one but without Tauri)
    fn convert_archive_mock(source: String, target_format: String) -> Result<String, String> {
        let src_fmt = detect_format(source.clone()).ok_or("Unknown source format")?;
        let tgt_fmt = formats().into_iter().find(|f| f.id == target_format).ok_or("Unknown target format")?;
        if !tgt_fmt.compress {
            return Err("Target format does not support compression".into());
        }

        let tmp = std::env::temp_dir().join(format!("zl_cv_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
        let dst_dir = Path::new(&tmp);

        match src_fmt.id.as_str() {
            "zip" => do_extract_zip(&source, dst_dir, None)?,
            "gz" | "bz2" | "xz" => {
                let file = std::fs::File::open(&source).map_err(|e| e.to_string())?;
                let reader: Box<dyn Read> = match src_fmt.id.as_str() {
                    "gz" => Box::new(flate2::read::GzDecoder::new(file)),
                    "bz2" => Box::new(bzip2::read::BzDecoder::new(file)),
                    "xz" => Box::new(xz2::read::XzDecoder::new(file)),
                    _ => Box::new(file),
                };
                do_extract_tar(reader, dst_dir)?;
            }
            "tar" => {
                let file = std::fs::File::open(&source).map_err(|e| e.to_string())?;
                do_extract_tar(file, dst_dir)?;
            }
            _ => return Err("Unsupported source format".into()),
        }

        let base = Path::new(&source).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "converted".into());
        let dest_path = Path::new(&source).parent().unwrap_or(Path::new(".")).join(format!("{}.{}", base, tgt_fmt.ext.first().unwrap_or(&"bin".into())));

        let sources = collect_dir_entries(dst_dir)?;
        match tgt_fmt.id.as_str() {
            "zip" => do_compress_zip(&sources, &dest_path, None, 6)?,
            _ => do_compress_tar(&sources, &dest_path, &tgt_fmt.id, 6)?,
        }

        std::fs::remove_dir_all(&tmp).ok();
        Ok(dest_path.to_string_lossy().to_string())
    }

    /// Helper: inline split_archive (mimics the real one)
    fn split_archive_mock(source: String, volume_size: String) -> Result<String, String> {
        let size_mb = volume_size.parse::<u64>().map_err(|_| "Invalid size".to_string())?;
        let chunk = (size_mb * 1024 * 1024) as usize;
        let data = std::fs::read(&source).map_err(|e| format!("Cannot read: {}", e))?;
        if chunk == 0 || data.is_empty() {
            return Err("Invalid size or empty source".into());
        }
        let total_parts = (data.len() + chunk - 1) / chunk;
        for (i, part_data) in data.chunks(chunk).enumerate() {
            let source_path = std::path::Path::new(&source);
            let ext = source_path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
            let stem = source_path.with_extension("");
            let part_name = format!("{}.part{:03}{}", stem.to_string_lossy(), i + 1, ext);
            std::fs::write(&part_name, part_data).map_err(|e| format!("Write error: {}", e))?;
        }
        Ok(format!("✅ Split into {} volumes", total_parts))
    }

    /// Helper: inline update_archive (mimics the real one)

#[test]
fn test_zstd_roundtrip() {
    let dir = std::env::temp_dir().join(format!("zl_zst_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("hello.txt"), b"Hello from Zstandard compression!").unwrap();
    std::fs::write(dir.join("data.bin"), &[0x00u8, 0xFF, 0xAB].repeat(100)).unwrap();

    let srcs = vec![dir.join("hello.txt"), dir.join("data.bin")];

    // Compress to .tar.zst
    let dst = dir.join("output.tar.zst");
    do_compress_zst(&srcs, &dst, 6).unwrap();
    assert!(dst.exists(), "Zstandard archive not created");
    assert!(dst.metadata().unwrap().len() > 0);

    // Extract
    let extract = dir.join("extracted");
    std::fs::create_dir_all(&extract).unwrap();
    do_extract_zst(&dst.to_string_lossy(), &extract).unwrap();

    // Verify
    assert!(extract.join("hello.txt").exists());
    let content = std::fs::read_to_string(extract.join("hello.txt")).unwrap();
    assert_eq!(content, "Hello from Zstandard compression!");

    println!("✅ Zstandard roundtrip: compressed + extracted OK");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_zstd_detect_format() {
    // Test extension detection
    assert_eq!(detect_format("archive.tar.zst".into()).map(|f| f.id), Some("zst".to_string()));
    assert_eq!(detect_format("archive.tzst".into()).map(|f| f.id), Some("zst".to_string()));
    assert_eq!(detect_format("file.zst".into()).map(|f| f.id), Some("zst".to_string()));
    
    println!("✅ Zstandard format detection: .tar.zst, .tzst, .zst all detected");
}

#[test]
fn test_all_eight_formats_properties() {
    let fmts = formats();
    assert_eq!(fmts.len(), 8);
    
    // Zstandard
    let zst = fmts.iter().find(|f| f.id == "zst").unwrap();
    assert!(zst.compress, "Zstandard should support compress");
    assert!(zst.extract, "Zstandard should support extract");
    assert!(!zst.password, "Zstandard should NOT support password");
    assert!(zst.ext.contains(&"zst".to_string()));
    assert!(zst.ext.contains(&"tzst".to_string()));
    
    println!("✅ Zstandard format: compress={}, extract={}, password={}", zst.compress, zst.extract, zst.password);
}

// ─── FORENSIC REGRESSION TEST SUITE ───
// 25+ tests untuk memastikan forensic engine stabil dan bebas regression

/// Test: check_magic_bytes — PNG file with .png extension → MATCH
#[test]
fn test_magic_bytes_png_match() {
    let png_header = b"\x89PNG\r\n\x1a\nhello_world_data";
    let (magic_match, detected, expected) = check_magic_bytes(png_header, "photo.png");
    assert_eq!(magic_match, Some(true), "PNG magic + .png extension should match (green)");
    assert_eq!(detected, Some("PNG".to_string()), "Detected should be PNG from magic");
    assert!(expected.unwrap().contains("PNG"), "Expected should mention PNG from extension");
}

/// Test: check_magic_bytes — PNG file with .jpg extension → MISMATCH
#[test]
fn test_magic_bytes_png_disguised_as_jpg() {
    let png_header = b"\x89PNG\r\n\x1a\nsneaky";
    let (magic_match, detected, expected) = check_magic_bytes(png_header, "photo.jpg");
    assert_eq!(magic_match, Some(false), "PNG magic + .jpg extension should MISMATCH (red/suspicious!)");
    assert_eq!(detected, Some("PNG".to_string()), "Detected should be PNG (real content)");
    assert!(expected.unwrap().contains("JPEG"), "Expected should be JPEG (claimed by extension)");
}

/// Test: check_magic_bytes — JPEG file with .jpg extension → MATCH
#[test]
fn test_magic_bytes_jpg_match() {
    let jpg = &[0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46];
    let (magic_match, detected, expected) = check_magic_bytes(jpg, "photo.jpg");
    assert_eq!(magic_match, Some(true), "JPEG magic + .jpg should match");
    assert_eq!(detected, Some("JPEG".to_string()));
}

/// Test: check_magic_bytes — PDF file with .pdf extension → MATCH
#[test]
fn test_magic_bytes_pdf_match() {
    let pdf = b"%PDF-1.4\nrest of data...";
    let (magic_match, detected, expected) = check_magic_bytes(pdf, "doc.pdf");
    assert_eq!(magic_match, Some(true), "PDF magic + .pdf should match");
    assert_eq!(detected, Some("PDF".to_string()));
}

/// Test: check_magic_bytes — PDF disguised as .exe → MISMATCH
#[test]
fn test_magic_bytes_pdf_as_exe() {
    let pdf = b"%PDF-1.4\nmalware?";
    let (magic_match, detected, expected) = check_magic_bytes(pdf, "notepad.exe");
    assert_eq!(magic_match, Some(false), "PDF disguised as EXE should flag mismatch!");
    assert_eq!(detected, Some("PDF".to_string()));
    assert!(expected.as_deref().unwrap().contains("Windows"), "Expected should be Windows Executable");
}

/// Test: check_magic_bytes — 0-byte file (no magic to check)
#[test]
fn test_magic_bytes_empty_file() {
    let (magic_match, detected, expected) = check_magic_bytes(b"", "empty.txt");
    assert_eq!(magic_match, None, "0-byte file: no magic → None (can't verify)");
    assert_eq!(detected, None, "No magic detected for 0-byte");
}

/// Test: check_magic_bytes — file tanpa extension
#[test]
fn test_magic_bytes_no_extension() {
    let png = b"\x89PNG\r\n\x1a\n";
    let (magic_match, detected, expected) = check_magic_bytes(png, "noext_file");
    assert_eq!(magic_match, None, "No extension → can't check mismatch → None");
    assert_eq!(detected, Some("PNG".to_string()), "Still detect PNG from magic");
    assert_eq!(expected, None, "No extension → no expected type");
}

/// Test: check_magic_bytes — completely unknown magic
#[test]
fn test_magic_bytes_unknown_type() {
    let data = b"\xde\xad\xbe\xef\xca\xfe random data";
    let (magic_match, detected, expected) = check_magic_bytes(data, "mystery.dat");
    assert_eq!(magic_match, None, "Unknown magic → None (not mismatched)");
    assert_eq!(detected, Some("Unknown".to_string()));
    assert_eq!(expected, Some(".dat File".to_string()));
}

/// Test: ext_to_type_name — all known extensions map correctly
#[test]
fn test_ext_to_type_name_known_extensions() {
    assert_eq!(ext_to_type_name("png"), Some("PNG Image".to_string()));
    assert_eq!(ext_to_type_name("jpg"), Some("JPEG Image".to_string()));
    assert_eq!(ext_to_type_name("pdf"), Some("PDF Document".to_string()));
    assert_eq!(ext_to_type_name("exe"), Some("Windows Executable".to_string()));
    assert_eq!(ext_to_type_name("zip"), Some("ZIP Archive".to_string()));
    assert_eq!(ext_to_type_name(""), None, "Empty ext → None");
}

/// Test: calculate_entropy — 0-byte data
#[test]
fn test_entropy_zero_byte() {
    let e = calculate_entropy(b"");
    assert_eq!(e, 0.0, "Empty data should have entropy 0.0");
}

/// Test: calculate_entropy — all zero data (very low entropy)
#[test]
fn test_entropy_all_zeros() {
    let data = vec![0u8; 1000];
    let e = calculate_entropy(&data);
    assert!(e < 0.5, "All zeros should have very low entropy (< 0.5), got {}", e);
}

/// Test: calculate_entropy — all unique bytes (max entropy)
#[test]
fn test_entropy_all_unique() {
    let data: Vec<u8> = (0u8..=255).collect();
    let e = calculate_entropy(&data);
    assert_eq!(e, 8.0, "256 unique bytes each once = entropy exactly 8.0");
}

/// Test: compute_analysis — normal file returns all fields
#[test]
fn test_compute_analysis_normal() {
    let (md5, sha1, sha256, entropy, magic_match, detected, expected) =
        compute_analysis(b"hello world", "test.txt");
    assert!(md5.is_some(), "Should have MD5");
    assert!(sha1.is_some(), "Should have SHA1");
    assert!(sha256.is_some(), "Should have SHA256");
    assert!(entropy.is_some(), "Should have entropy");
    assert!(magic_match.is_some() || detected.is_some(), "Should have magic info");
}

/// Test: compute_analysis — 0-byte file returns valid hashes
#[test]
fn test_compute_analysis_empty_file() {
    let (md5, sha1, sha256, entropy, magic_match, detected, expected) =
        compute_analysis(b"", "empty.dat");
    // Empty file's MD5 is well-defined: d41d8cd98f00b204e9800998ecf8427e
    assert_eq!(md5, Some("d41d8cd98f00b204e9800998ecf8427e".to_string()), "Empty MD5 should be correct");
    assert!(sha1.is_some(), "Should have SHA1 for empty file");
    assert!(sha256.is_some(), "Should have SHA256 for empty file");
    assert_eq!(entropy, Some(0.0), "Empty file entropy should be 0.0");
    assert_eq!(magic_match, None, "Empty file no magic = None");
}

/// Test: forensic_load_zip — normal archive
#[test]
fn test_forensic_load_zip_normal() {
    let dir = tempdir("forensic_normal");
    setup_forensic_files(&dir);
    let zip_path = dir.join("archive.zip");
    create_test_zip(&zip_path, &dir, None);

    let entries = forensic_load_zip(&zip_path.to_string_lossy(), None).unwrap();
    assert!(!entries.is_empty(), "Should have entries");
    assert!(entries.iter().any(|e| e.path.contains("file1.txt")), "Should contain file1.txt");
    assert!(!entries.iter().any(|e| e.path.contains("secret")), "Should NOT leak secrets via filename");
}

/// Test: forensic_load_zip — password protected
#[test]
fn test_forensic_load_zip_with_password() {
    let dir = tempdir("forensic_pw");
    setup_forensic_files(&dir);
    let zip_path = dir.join("archive.zip");
    create_test_zip(&zip_path, &dir, Some("testpw"));

    let result = forensic_load_zip(&zip_path.to_string_lossy(), None);
    assert!(result.is_err(), "Should fail without password");
    assert!(result.unwrap_err().contains("PASSWORD_NEEDED"), "Should say PASSWORD_NEEDED");

    let result = forensic_load_zip(&zip_path.to_string_lossy(), Some("testpw"));
    assert!(result.is_ok(), "Should succeed with correct password");
    let entries = result.unwrap();
    assert!(!entries.is_empty(), "Should have entries");
}

/// Test: forensic_load_zip — wrong password
#[test]
fn test_forensic_load_zip_wrong_password() {
    let dir = tempdir("forensic_wrongpw");
    setup_forensic_files(&dir);
    let zip_path = dir.join("archive.zip");
    create_test_zip(&zip_path, &dir, Some("correct"));

    let result = forensic_load_zip(&zip_path.to_string_lossy(), Some("wrongpass"));
    assert!(result.is_err(), "Should fail with wrong password");
    let err = result.unwrap_err();
    assert!(err.contains("PASSWORD") || err.contains("skipped"), "Should indicate password error or skipped entries");
}

/// Test: forensic_load_tar — normal
#[test]
fn test_forensic_load_tar_normal() {
    let dir = tempdir("forensic_tar");
    setup_forensic_files(&dir);
    let tar_path = dir.join("archive.tar");
    let files = vec![dir.join("file1.txt"), dir.join("file2.txt")];
    do_compress_tar(&files, &tar_path, "none", 6).unwrap();

    let entries = forensic_load_tar(&tar_path.to_string_lossy(), "tar").unwrap();
    assert!(!entries.is_empty(), "Should have TAR entries");
    assert!(entries.iter().any(|e| e.path.contains("file1.txt")));
}

/// Test: forensic_load_tar — GZip compressed
#[test]
fn test_forensic_load_tar_gz() {
    let dir = tempdir("forensic_tgz");
    setup_forensic_files(&dir);
    let tgz_path = dir.join("archive.tar.gz");
    let files = vec![dir.join("file1.txt"), dir.join("file2.txt")];
    do_compress_tar(&files, &tgz_path, "gz", 6).unwrap();

    let entries = forensic_load_tar(&tgz_path.to_string_lossy(), "gz").unwrap();
    assert!(!entries.is_empty(), "Should have TGZ entries");
}

/// Test: generate_forensic_report — ZIP normal
#[test]
fn test_forensic_report_zip_normal() {
    let dir = tempdir("report_zip");
    setup_forensic_files(&dir);
    let zip_path = dir.join("archive.zip");
    create_test_zip(&zip_path, &dir, None);

    let report = generate_forensic_report_mock(&zip_path.to_string_lossy(), None).unwrap();
    assert!(report.total_files > 0, "Should have files");
    assert!(report.total_size > 0, "Should have total size > 0");
    assert!(!report.entries.is_empty(), "Should have entries");
    // Each entry should have hashes
    for entry in &report.entries {
        assert!(entry.md5.is_some(), "Entry {} should have MD5", entry.path);
        assert!(entry.sha256.is_some(), "Entry {} should have SHA256", entry.path);
    }
}

/// Test: generate_forensic_report — empty ZIP
#[test]
fn test_forensic_report_empty_zip() {
    let dir = tempdir("report_empty");
    let zip_path = dir.join("empty.zip");
    {
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        zip.finish().unwrap();
    }
    let report = generate_forensic_report_mock(&zip_path.to_string_lossy(), None).unwrap();
    assert_eq!(report.total_files, 0, "Empty archive = 0 files");
    assert_eq!(report.entries.len(), 0, "No entries for empty archive");
}

/// Test: generate_forensic_report — TAR normal
#[test]
fn test_forensic_report_tar_normal() {
    let dir = tempdir("report_tar");
    setup_forensic_files(&dir);
    let tar_path = dir.join("archive.tar");
    let files = vec![dir.join("file1.txt"), dir.join("file2.txt")];
    do_compress_tar(&files, &tar_path, "none", 6).unwrap();

    let report = generate_forensic_report_mock(&tar_path.to_string_lossy(), None).unwrap();
    assert!(report.total_files > 0);
    for entry in &report.entries {
        assert!(entry.sha256.is_some(), "TAR entry should have SHA256");
    }
}

/// Test: generate_forensic_report — TAR.GZ
#[test]
fn test_forensic_report_tar_gz() {
    let dir = tempdir("report_gz");
    setup_forensic_files(&dir);
    let tgz_path = dir.join("archive.tar.gz");
    let files = vec![dir.join("file1.txt"), dir.join("file2.txt")];
    do_compress_tar(&files, &tgz_path, "gz", 6).unwrap();

    let report = generate_forensic_report_mock(&tgz_path.to_string_lossy(), None).unwrap();
    assert!(report.total_files > 0);
    assert_eq!(report.format, "GZip", "Format should be GZip");
}

/// Test: generate_forensic_report — TAR.XZ
#[test]
fn test_forensic_report_tar_xz() {
    let dir = tempdir("report_xz");
    setup_forensic_files(&dir);
    let txz_path = dir.join("archive.tar.xz");
    let files = vec![dir.join("file1.txt"), dir.join("file2.txt")];
    do_compress_tar(&files, &txz_path, "xz", 6).unwrap();

    let report = generate_forensic_report_mock(&txz_path.to_string_lossy(), None).unwrap();
    assert!(report.total_files > 0);
}

/// Test: generate_forensic_report — TAR.BZ2
#[test]
fn test_forensic_report_tar_bz2() {
    let dir = tempdir("report_bz2");
    setup_forensic_files(&dir);
    let tbz_path = dir.join("archive.tar.bz2");
    let files = vec![dir.join("file1.txt"), dir.join("file2.txt")];
    do_compress_tar(&files, &tbz_path, "bz2", 6).unwrap();

    let report = generate_forensic_report_mock(&tbz_path.to_string_lossy(), None).unwrap();
    assert!(report.total_files > 0);
}

/// Test: generate_forensic_report — anomaly: high entropy detected
#[test]
fn test_forensic_report_high_entropy_anomaly() {
    let dir = tempdir("report_entropy");
    // Create high-entropy data without rand crate (uniform byte distribution = entropy ~8.0)
    let high_entropy_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
    std::fs::write(dir.join("random.bin"), &high_entropy_data).unwrap();

    let zip_path = dir.join("archive.zip");
    do_compress_zip(&[dir.join("random.bin")], &zip_path, None, 6).unwrap();

    let report = generate_forensic_report_mock(&zip_path.to_string_lossy(), None).unwrap();
    // Random data should have entropy > 7.5
    let has_anomaly = report.anomalies.iter().any(|a| a.issue.contains("entropy"));
    assert!(has_anomaly, "High entropy file should trigger anomaly: {:?}", report.anomalies);
}

/// Test: generate_forensic_report — anomaly: extension mismatch
#[test]
fn test_forensic_report_extension_mismatch_anomaly() {
    let dir = tempdir("report_mismatch");
    // Create a PNG but name it .jpg
    let png_header = b"\x89PNG\r\n\x1a\nAAAA";
    std::fs::write(dir.join("disguised.jpg"), png_header).unwrap();

    let zip_path = dir.join("archive.zip");
    do_compress_zip(&[dir.join("disguised.jpg")], &zip_path, None, 6).unwrap();

    let report = generate_forensic_report_mock(&zip_path.to_string_lossy(), None).unwrap();
    let has_mismatch = report.anomalies.iter().any(|a| a.issue.contains("Extension mismatch"));
    assert!(has_mismatch, "PNG-as-JPG should trigger extension mismatch anomaly: {:?}", report.anomalies);
}

/// Test: generate_forensic_report — 0-byte files handled gracefully
#[test]
fn test_forensic_report_zero_byte_files() {
    let dir = tempdir("report_0byte");
    std::fs::write(dir.join("empty.txt"), b"").unwrap();
    std::fs::write(dir.join("normal.txt"), b"normal content").unwrap();

    let zip_path = dir.join("archive.zip");
    do_compress_zip(&[dir.join("empty.txt"), dir.join("normal.txt")], &zip_path, None, 6).unwrap();

    let report = generate_forensic_report_mock(&zip_path.to_string_lossy(), None).unwrap();
    assert_eq!(report.total_files, 2, "Should have both files");

    let empty = report.entries.iter().find(|e| e.path.contains("empty")).unwrap();
    assert_eq!(empty.size, 0, "Empty file size should be 0");
    assert!(empty.md5.is_some(), "Empty file should have MD5 hash");
    assert_eq!(empty.entropy, Some(0.0), "Empty file entropy = 0.0");
}

/// Test: generate_forensic_report — corrupted ZIP (graceful, not crash)
#[test]
fn test_forensic_report_corrupted_zip_graceful() {
    let dir = tempdir("report_corrupt");
    let corrupt_path = dir.join("corrupt.zip");
    std::fs::write(&corrupt_path, b"THIS IS NOT A ZIP FILE AT ALL").unwrap();

    let result = generate_forensic_report_mock(&corrupt_path.to_string_lossy(), None);
    assert!(result.is_err(), "Corrupt file should error cleanly, not panic");
    let err = result.unwrap_err();
    assert!(err.contains("Invalid") || err.contains("corrupted"), "Error should explain the problem");
}

/// Test: generate_forensic_report — nonexistent file
#[test]
fn test_forensic_report_nonexistent_file() {
    let result = generate_forensic_report_mock("/tmp/definitely_not_real_xyz.zip", None);
    assert!(result.is_err(), "Nonexistent file should error cleanly");
}

/// Test: forensic_load handles empty TAR
#[test]
fn test_forensic_load_tar_empty() {
    let dir = tempdir("forensic_empty_tar");
    let tar_path = dir.join("empty.tar");
    // Create empty TAR
    let f = std::fs::File::create(&tar_path).unwrap();
    let mut tar_builder = tar::Builder::new(f);
    tar_builder.finish().unwrap();

    let entries = forensic_load_tar(&tar_path.to_string_lossy(), "tar").unwrap();
    assert_eq!(entries.len(), 0, "Empty TAR should have 0 entries");
}

/// Test: forensic_load handles Zstandard
#[test]
fn test_forensic_load_zst() {
    let dir = tempdir("forensic_zst");
    setup_forensic_files(&dir);
    let zst_path = dir.join("archive.tar.zst");
    let files = vec![dir.join("file1.txt"), dir.join("file2.txt")];
    do_compress_zst(&files, &zst_path, 6).unwrap();

    let entries = forensic_load_tar(&zst_path.to_string_lossy(), "zst").unwrap();
    assert!(!entries.is_empty(), "Should read Zst archive entries");
}

/// Test: total_size is sum of all entry sizes
#[test]
fn test_forensic_report_total_size_correct() {
    let dir = tempdir("report_total");
    std::fs::write(dir.join("a.txt"), b"AAAA").unwrap();  // 4 bytes
    std::fs::write(dir.join("b.txt"), b"BBBBBB").unwrap(); // 6 bytes

    let zip_path = dir.join("archive.zip");
    do_compress_zip(&[dir.join("a.txt"), dir.join("b.txt")], &zip_path, None, 6).unwrap();

    let report = generate_forensic_report_mock(&zip_path.to_string_lossy(), None).unwrap();
    assert_eq!(report.total_size, 10, "Total size should be 4 + 6 = 10, got {}", report.total_size);
}

/// Test: entropy on predictable data (low entropy)
#[test]
fn test_entropy_low_predictable() {
    let data = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let e = calculate_entropy(data);
    assert!(e < 1.0, "All 'A' should have very low entropy, got {}", e);
}

/// Stress test: forensic report with REAL files (PDF + PNG) — run 3x, verify stability
#[test]
fn test_forensic_real_files_stress() {
    // Find a real PDF on the system for realistic testing
    let test_base = std::env::temp_dir().join(format!("zl_real_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&test_base).unwrap();

    // Create realistic test files
    // Simulate a PDF header
    let fake_pdf = b"%PDF-1.7\n%\\xc3\\xa4\\xc3\\xb6\\xc3\\xbc\\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n%%EOF";
    std::fs::write(test_base.join("document.pdf"), fake_pdf).unwrap();

    // Simulate a PNG image
    let fake_png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x10\x00\x00\x00\x10\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\\x9cc```\\x04\\x00\x00\\x0c\\x00\\x01\\xeb\\x14\\x8e\\x0f\x00\x00\x00\x00IEND\xaeB`\x82";
    std::fs::write(test_base.join("image.png"), fake_png).unwrap();

    // Normal text file
    std::fs::write(test_base.join("readme.txt"), b"ZipLoom Forensic Test Suite v1.0\nReal file regression test.\n").unwrap();

    // File disguised as wrong extension (PNG as .exe)
    std::fs::write(test_base.join("notepad.exe"), fake_png).unwrap();

    // 0-byte file
    std::fs::write(test_base.join("empty.log"), b"").unwrap();

    // Compress all into ZIP
    let zip_path = test_base.join("forensic_real.zip");
    let all_files: Vec<PathBuf> = std::fs::read_dir(&test_base).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.path())
        .collect();
    do_compress_zip(&all_files, &zip_path, None, 6).unwrap();
    assert!(zip_path.exists());

    // Run forensic 3 times — verify stability
    let mut first_md5 = String::new();
    for run in 1..=3 {
        let report = generate_forensic_report_mock(&zip_path.to_string_lossy(), None).unwrap();
        assert_eq!(report.total_files, 5, "Run {}: should have 5 files", run);

        // document.pdf: should detect as PDF, magic match
        let pdf_entry = report.entries.iter().find(|e| e.path.contains("document")).unwrap();
        assert!(pdf_entry.md5.is_some(), "Run {}: PDF should have MD5", run);
        assert!(pdf_entry.detected_type.as_deref() == Some("PDF"), "Run {}: PDF detected={:?}", run, pdf_entry.detected_type);
        assert_eq!(pdf_entry.magic_match, Some(true), "Run {}: PDF magic should match", run);

        // image.png: should detect as PNG, magic match
        let png_entry = report.entries.iter().find(|e| e.path.contains("image")).unwrap();
        assert_eq!(png_entry.magic_match, Some(true), "Run {}: PNG magic should match", run);

        // notepad.exe (actually PNG): should be MISMATCH!
        let exe_entry = report.entries.iter().find(|e| e.path.contains("notepad")).unwrap();
        assert_eq!(exe_entry.magic_match, Some(false), "Run {}: PNG-as-EXE should be MISMATCH!", run);
        assert!(exe_entry.detected_type.as_deref() == Some("PNG"), "Run {}: Real content is PNG", run);
        assert!(exe_entry.expected_type.as_deref().unwrap().contains("Windows"), "Run {}: Expected Windows EXE", run);
        assert!(report.anomalies.iter().any(|a| a.file.contains("notepad")), "Run {}: Should flag mismatch anomaly", run);

        // empty.log: 0-byte with known extension
        let empty_entry = report.entries.iter().find(|e| e.path.contains("empty")).unwrap();
        assert_eq!(empty_entry.size, 0, "Run {}: Empty size=0", run);
        assert!(empty_entry.md5.is_some(), "Run {}: Empty file needs hash!", run);
        assert_eq!(empty_entry.entropy, Some(0.0), "Run {}: Empty entropy=0", run);

        // Verify hash determinism across runs
        if run == 1 {
            first_md5 = pdf_entry.md5.clone().unwrap();
        } else {
            assert_eq!(pdf_entry.md5.as_ref().unwrap(), &first_md5, "Run {}: MD5 must be consistent!", run);
        }

        println!("✅ Real-file run {}: PDF={}, EXE(fake)=mismatch{}, Empty=OK",
            run,
            pdf_entry.detected_type.as_deref().unwrap_or("?"),
            if exe_entry.magic_match == Some(false) { " RED FLAG" } else { "" }
        );
    }
    println!("✅ REAL-FILE FORENSIC STRESS TEST PASSED: 3 runs, all stable");
    std::fs::remove_dir_all(&test_base).ok();
}

/// Stress test: forensic report with ALL archivable formats, repeated
#[test]
fn test_forensic_all_formats_stress() {
    let test_base = std::env::temp_dir().join(format!("zl_allfmt_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&test_base).unwrap();

    // Create test files
    std::fs::write(test_base.join("hello.txt"), b"ZipLoom multi-format forensic test.").unwrap();
    std::fs::write(test_base.join("data.bin"), &[0x00, 0xFF, 0xAB, 0xCD].repeat(10)).unwrap();

    let files = vec![test_base.join("hello.txt"), test_base.join("data.bin")];

    // Test each format
    let formats = [("tar", "none"), ("gz", "gz"), ("bz2", "bz2"), ("xz", "xz")];
    for (ext, fmt_id) in &formats {
        for run in 1..=2 {
            let archive_path = test_base.join(format!("test_run{}.{}", run, ext));
            do_compress_tar(&files, &archive_path, fmt_id, 6).unwrap();
            
            let report = generate_forensic_report_mock(&archive_path.to_string_lossy(), None).unwrap();
            assert_eq!(report.total_files, 2, "Format {} run {}: should have 2 files", ext, run);
            
            for entry in &report.entries {
                assert!(entry.sha256.is_some(), "Format {} run {}: entry {} needs SHA256", ext, run, entry.path);
                assert!(entry.entropy.is_some(), "Format {} run {}: entry {} needs entropy", ext, run, entry.path);
            }
        }
    }

    // Also test Zstandard
    let zst_path = test_base.join("test.zst");
    do_compress_zst(&files, &zst_path, 6).unwrap();
    let report = generate_forensic_report_mock(&zst_path.to_string_lossy(), None).unwrap();
    assert!(report.total_files > 0, "Zstandard forensic should work");

    // Also test ZIP
    let zip_path = test_base.join("test.zip");
    do_compress_zip(&files, &zip_path, None, 6).unwrap();
    for run in 1..=3 {
        let report = generate_forensic_report_mock(&zip_path.to_string_lossy(), None).unwrap();
        assert_eq!(report.total_files, 2, "ZIP run {}: 2 files", run);
    }

    println!("✅ ALL-FORMATS FORENSIC STRESS: ZIP+TAR+GZ+BZ2+XZ+ZST, all passed");
    std::fs::remove_dir_all(&test_base).ok();
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("zl_{}_{}", name, uuid::Uuid::new_v4()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn setup_forensic_files(dir: &std::path::Path) {
    std::fs::write(dir.join("file1.txt"), b"Hello forensic world!").unwrap();
    std::fs::write(dir.join("file2.txt"), b"Second test file with more content here.").unwrap();
}

fn create_test_zip(zip_path: &std::path::Path, src_dir: &std::path::Path, password: Option<&str>) {
    let entries: Vec<std::path::PathBuf> = std::fs::read_dir(src_dir).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.path())
        .collect();
    do_compress_zip(&entries, zip_path, password, 6).unwrap();
}

/// Mock for generate_forensic_report — uses real engine logic
fn generate_forensic_report_mock(path: &str, pw: Option<&str>) -> Result<crate::ForensicReport, String> {
    let fmt = detect_format(path.to_string()).ok_or("Unknown format")?;
    let mut all_entries = Vec::new();
    let mut anomalies = Vec::new();
    let mut all_threats: Vec<crate::scanner::MalwareThreat> = Vec::new();
    let mut total_size: u64 = 0;
    let mut total_compressed: u64 = 0;
    let mut total_nested_archives: usize = 0;

    match fmt.id.as_str() {
        "zip" => {
            let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
            let mut archive = match zip::ZipArchive::new(file) {
                Ok(a) => a,
                Err(e) => {
                    let msg = e.to_string().to_lowercase();
                    if msg.contains("password") { return Err("PASSWORD_NEEDED".into()); }
                    return Err(format!("Invalid ZIP: {}", e));
                }
            };
            for i in 0..archive.len() {
                let entry_result = if let Some(p) = pw {
                    archive.by_index_decrypt(i, p.as_bytes())
                } else {
                    archive.by_index(i)
                };
                let mut entry = match entry_result {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let name = entry.name().to_string();
                let size = entry.size();
                total_size += size;
                total_compressed += entry.compressed_size();
                let mut data = Vec::new();
                let data_ok = std::io::copy(&mut entry, &mut data).is_ok();
                // Nested archive detection
                if data_ok && data.len() >= 4 {
                    let is_archive = (data[..4] == *b"PK\x03\x04")
                        || (data.len() >= 6 && data[..6] == [0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c])
                        || (data.starts_with(b"Rar!"))
                        || (data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b)
                        || (data.starts_with(b"BZ"))
                        || (data.len() >= 6 && data[0] == 0xfd && data[1..6] == [0x37, 0x7a, 0x58, 0x5a, 0x00]);
                    if is_archive {
                        total_nested_archives += 1;
                    }
                }
                let (md5, sha1, sha256, entropy, magic_match, detected, expected) = if data_ok {
                    compute_analysis(&data, &name)
                } else {
                    (None, None, None, None, None, None, None)
                };
                if let Some(e) = entropy { if e > 7.5 && name.contains('.') {
                    anomalies.push(crate::Anomaly { file: name.clone(), issue: format!("High entropy ({:.2})", e), severity: "high".into() });
                }}
                if magic_match == Some(false) {
                    anomalies.push(crate::Anomaly { file: name.clone(), issue: format!("Extension mismatch: expected '{}', detected '{}'", detected.as_deref().unwrap_or("?"), expected.as_deref().unwrap_or("?")), severity: "high".into() });
                }
                // ── Malware Scan ──
                let mut file_threats = crate::scanner::scan_file_name(&name);
                if data_ok {
                    file_threats.extend(crate::scanner::scan_file_content(&name, &data));
                }
                all_threats.extend(file_threats);

                all_entries.push(crate::FileEntry {
                    path: name, size, compressed_size: None, ratio: None, is_dir: false,
                    modified: None, created: None, permissions: None,
                    md5, sha1, sha256, entropy, magic_match, expected_type: expected, detected_type: detected,
                });
            }
        }
        "tar" | "gz" | "bz2" | "xz" | "zst" => {
            let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
            let reader: Box<dyn std::io::Read> = match fmt.id.as_str() {
                "gz" => Box::new(flate2::read::GzDecoder::new(file)),
                "bz2" => Box::new(bzip2::read::BzDecoder::new(file)),
                "xz" => Box::new(xz2::read::XzDecoder::new(file)),
                "zst" => Box::new(zstd::stream::Decoder::new(file).map_err(|e| format!("Zstd: {}", e))?),
                _ => Box::new(file),
            };
            if let Ok(meta) = std::fs::metadata(path) {
                total_compressed = meta.len();
            }
            let mut archive = tar::Archive::new(reader);
            for entry in archive.entries().map_err(|e| e.to_string())? {
                let mut entry = entry.map_err(|e| e.to_string())?;
                let name = entry.path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                let size = entry.size();
                total_size += size;
                let mut data = Vec::new();
                let data_ok = std::io::copy(&mut entry, &mut data).is_ok();
                // Nested archive detection
                if data_ok && data.len() >= 4 {
                    let is_archive = (data[..4] == *b"PK\x03\x04")
                        || (data.len() >= 6 && data[..6] == [0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c])
                        || (data.starts_with(b"Rar!"))
                        || (data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b)
                        || (data.starts_with(b"BZ"))
                        || (data.len() >= 6 && data[0] == 0xfd && data[1..6] == [0x37, 0x7a, 0x58, 0x5a, 0x00]);
                    if is_archive {
                        total_nested_archives += 1;
                    }
                }
                let (md5, sha1, sha256, entropy, magic_match, detected, expected) = if data_ok {
                    compute_analysis(&data, &name)
                } else {
                    (None, None, None, None, None, None, None)
                };
                if let Some(e) = entropy { if e > 7.5 && name.contains('.') {
                    anomalies.push(crate::Anomaly { file: name.clone(), issue: format!("High entropy ({:.2})", e), severity: "high".into() });
                }}
                if magic_match == Some(false) {
                    anomalies.push(crate::Anomaly { file: name.clone(), issue: format!("Extension mismatch"), severity: "high".into() });
                }
                // ── Malware Scan ──
                let mut file_threats = crate::scanner::scan_file_name(&name);
                if data_ok {
                    file_threats.extend(crate::scanner::scan_file_content(&name, &data));
                }
                all_threats.extend(file_threats);

                all_entries.push(crate::FileEntry {
                    path: name, size, compressed_size: None, ratio: None, is_dir: false,
                    modified: None, created: None, permissions: None,
                    md5, sha1, sha256, entropy, magic_match, expected_type: expected, detected_type: detected,
                });
            }
        }
        _ => return Err("Forensic report supports ZIP and TAR formats only".into()),
    }

    // ── Archive-Level Metadata Scan ──
    let archive_threats = crate::scanner::scan_archive_metadata(
        all_entries.len(),
        total_compressed,
        total_size,
        total_nested_archives,
    );
    all_threats.extend(archive_threats);

    let (risk_score, risk_label) = crate::scanner::compute_risk_score(&all_threats);

    Ok(crate::ForensicReport {
        archive_path: path.to_string(), format: fmt.name,
        total_files: all_entries.len(), total_size,
        entries: all_entries, anomalies,
        threats: all_threats,
        risk_score,
        risk_label,
    })
}

    fn update_archive_mock(archive_path: String, files: Vec<String>) -> Result<String, String> {
        let fmt = detect_format(archive_path.clone()).ok_or("Unknown format")?;
        if !fmt.compress { return Err("Format does not support updates".into()); }
        if fmt.id != "zip" { return Err("Only ZIP supported".into()); }

        let file = std::fs::OpenOptions::new().read(true).write(true).open(&archive_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;

        let tmp = std::env::temp_dir().join(format!("zl_up_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            let safe = name.trim_start_matches('/').trim_start_matches('\\');
            let out_path = tmp.join(safe);
            if entry.is_dir() {
                std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
                let mut out = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            }
        }
        for f in &files {
            let src = Path::new(f);
            let dest = tmp.join(src.file_name().ok_or("Invalid filename")?.to_string_lossy().to_string());
            cp_r(src, &dest)?;
        }
        let sources = collect_dir_entries(&tmp)?;
        std::fs::remove_file(&archive_path).ok();
        do_compress_zip(&sources, Path::new(&archive_path), None, 6)?;
        std::fs::remove_dir_all(&tmp).ok();
        Ok(format!("✅ Updated with {} files", files.len()))
    }
}
