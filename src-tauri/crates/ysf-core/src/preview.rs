//! File Preview — View files directly inside AnalysisLoom
//!
//! Provides text preview, image preview, hex dump, and metadata for
//! any file on the filesystem. Backend-only (pure Rust), sends raw
//! data to the Tauri frontend for rendering.
//!
//! ISO 27042: Reproducible analysis requires inline file inspection.

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::hashing::compute_entropy;

const TEXT_PREVIEW_LIMIT: usize = 50 * 1024;
const HEX_PREVIEW_LIMIT: usize = 4 * 1024;
pub const ARCHIVE_PREVIEW_MAX: u64 = 2 * 1024 * 1024;
pub const ARCHIVE_IMAGE_MAX: u64 = 1 * 1024 * 1024;

const BLOCKED_PREVIEW_EXT: &[&str] = &[
    "exe", "dll", "sys", "scr", "com", "pif", "msi", "bat", "cmd", "ps1", "vbs", "hta",
];
const IMAGE_PREVIEW_EXT: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileKind {
    Text, Image, Archive, Pdf, Office, Binary, Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    pub path: String,
    pub filename: String,
    pub size: u64,
    pub kind: FileKind,
    pub mime_type: String,
    pub extension: String,
    pub preview: PreviewContent,
    pub metadata: FileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewContent {
    Text(String),
    Image { data_base64: String, width: u32, height: u32 },
    HexDump(String),
    ArchiveList(Vec<String>),
    Unsupported(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntryPreview {
    pub path: String,
    pub size: u64,
    pub truncated: bool,
    /// text | hex | image | blocked
    pub preview_type: String,
    pub text: Option<String>,
    pub hex: Option<String>,
    pub image_base64: Option<String>,
    pub mime_type: String,
    pub warning: Option<String>,
    pub safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: String,
    pub created: String,
    pub permissions: String,
    pub is_dir: bool,
    pub magic_match: Option<String>,
    pub entropy: Option<f64>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
}

/// Preview a single file inside an archive — read-only, size-capped, no execution.
pub fn preview_archive_entry(
    archive_path: &str,
    entry_path: &str,
    password: Option<&str>,
) -> Result<ArchiveEntryPreview, String> {
    let ext = Path::new(entry_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let max = if IMAGE_PREVIEW_EXT.contains(&ext.as_str()) {
        ARCHIVE_IMAGE_MAX
    } else {
        ARCHIVE_PREVIEW_MAX
    };

    let entry = crate::forensic::read_archive_entry(archive_path, entry_path, password, max)?;
    let mut warning: Option<String> = None;
    let mut safe = true;

    if BLOCKED_PREVIEW_EXT.contains(&ext.as_str()) {
        warning = Some(
            "Executable/script file — showing hex dump only. Do not extract or run unless you trust the source."
                .into(),
        );
        safe = false;
        let hex = preview_hex(&entry.data)?;
        return Ok(ArchiveEntryPreview {
            path: entry_path.to_string(),
            size: entry.total_size,
            truncated: entry.truncated,
            preview_type: "hex".into(),
            text: None,
            hex: match hex {
                PreviewContent::HexDump(h) => Some(h),
                _ => None,
            },
            image_base64: None,
            mime_type: mime_for(&ext),
            warning,
            safe,
        });
    }

    if ext == "svg" {
        warning = Some("SVG can contain scripts — rendered as plain text, not as image.".into());
        safe = false;
    }

    if ext == "html" || ext == "htm" {
        warning = Some("HTML is shown as plain text only — never rendered in the browser.".into());
    }

    if IMAGE_PREVIEW_EXT.contains(&ext.as_str()) && ext != "svg" {
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&entry.data);
        return Ok(ArchiveEntryPreview {
            path: entry_path.to_string(),
            size: entry.total_size,
            truncated: entry.truncated,
            preview_type: "image".into(),
            text: None,
            hex: None,
            image_base64: Some(b64),
            mime_type: mime_for(&ext),
            warning,
            safe,
        });
    }

    let kind = detect_kind(&ext, entry_path);
    match kind {
        FileKind::Text => {
            let text = preview_text(&entry.data)?;
            Ok(ArchiveEntryPreview {
                path: entry_path.to_string(),
                size: entry.total_size,
                truncated: entry.truncated,
                preview_type: "text".into(),
                text: match text {
                    PreviewContent::Text(t) => Some(t),
                    _ => None,
                },
                hex: None,
                image_base64: None,
                mime_type: mime_for(&ext),
                warning,
                safe,
            })
        }
        _ => {
            let hex = preview_hex(&entry.data)?;
            Ok(ArchiveEntryPreview {
                path: entry_path.to_string(),
                size: entry.total_size,
                truncated: entry.truncated,
                preview_type: "hex".into(),
                text: None,
                hex: match hex {
                    PreviewContent::HexDump(h) => Some(h),
                    _ => None,
                },
                image_base64: None,
                mime_type: mime_for(&ext),
                warning,
                safe,
            })
        }
    }
}

pub fn preview_file(path: &str) -> Result<PreviewResult, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("File not found: {}", path));
    }
    let meta = std::fs::metadata(path).map_err(|e| format!("Cannot read metadata: {}", e))?;
    let filename = p.file_name().map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let extension = p.extension().map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let modified = meta.modified().ok().map(|t| {
        let dt: chrono::DateTime<chrono::Utc> = t.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }).unwrap_or_else(|| "unknown".to_string());
    let created = meta.created().ok().map(|t| {
        let dt: chrono::DateTime<chrono::Utc> = t.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }).unwrap_or_else(|| "unknown".to_string());
    let perms = {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            format!("{:o}", meta.permissions().mode() & 0o777)
        }
        #[cfg(not(unix))]
        {
            let _ = &meta;
            "000".to_string()
        }
    };
    
    let kind = detect_kind(&extension, path);
    let data = std::fs::read(path).unwrap_or_default();
    let preview = generate_preview(path, &kind, &data)?;
    let hashes = crate::hashing::multi_hash_buffer(&data);
    let entropy = Some(compute_entropy(&data));
    let (_, _, magic_name) = crate::hashing::check_magic_bytes(&data, &filename);
    
    Ok(PreviewResult {
        path: path.to_string(), filename, size: meta.len(), kind,
        mime_type: mime_for(&extension), extension, preview,
        metadata: FileMetadata {
            size: meta.len(), modified, created, permissions: perms,
            is_dir: meta.is_dir(), magic_match: magic_name, entropy,
            md5: hashes.md5.clone(),
            sha1: hashes.sha1.clone(),
            sha256: hashes.sha256.clone(),
        },
    })
}

fn detect_kind(ext: &str, _path: &str) -> FileKind {
    match ext {
        "txt"|"md"|"csv"|"tsv"|"log"|"json"|"xml"|"yaml"|"yml"
        |"html"|"htm"|"css"|"js"|"ts"|"rs"|"py"|"rb"|"go"
        |"c"|"h"|"cpp"|"hpp"|"java"|"kt"|"swift"|"sh"|"bash"
        |"zsh"|"conf"|"cfg"|"ini"|"toml"|"env"|"sql"|"r"
        |"php"|"pl"|"lua"|"dart"|"scala"|"hs" => FileKind::Text,
        "png"|"jpg"|"jpeg"|"gif"|"webp"|"bmp"|"tiff"|"tif"|"ico"|"svg" => FileKind::Image,
        "zip"|"rar"|"7z"|"tar"|"gz"|"bz2"|"xz"|"zst" => FileKind::Archive,
        "pdf" => FileKind::Pdf,
        "docx"|"xlsx"|"pptx"|"doc"|"xls"|"ppt"|"odt"|"ods" => FileKind::Office,
        _ => FileKind::Unknown,
    }
}

fn generate_preview(path: &str, kind: &FileKind, data: &[u8]) -> Result<PreviewContent, String> {
    match kind {
        FileKind::Text => preview_text(data),
        FileKind::Image => preview_image(path, data),
        FileKind::Archive => preview_archive(path),
        FileKind::Pdf => Ok(PreviewContent::Unsupported(
            "PDF preview not yet supported. Metadata available below.".into())),
        FileKind::Office => Ok(PreviewContent::Unsupported(
            "Office document preview not yet supported. Metadata available below.".into())),
        FileKind::Binary | FileKind::Unknown => preview_hex(data),
    }
}

fn preview_text(data: &[u8]) -> Result<PreviewContent, String> {
    let text = String::from_utf8_lossy(data).to_string();
    let truncated = if text.len() > TEXT_PREVIEW_LIMIT {
        let mut t = text[..TEXT_PREVIEW_LIMIT].to_string();
        t.push_str(&format!("\n\n... [truncated at {} KB, file is {} KB total]",
            TEXT_PREVIEW_LIMIT / 1024, data.len() / 1024));
        t
    } else { text };
    Ok(PreviewContent::Text(truncated))
}

fn preview_image(path: &str, data: &[u8]) -> Result<PreviewContent, String> {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(data);
    let (width, height) = match image::ImageReader::open(path) {
        Ok(r) => r.into_dimensions().unwrap_or((0, 0)),
        Err(_) => (0, 0),
    };
    Ok(PreviewContent::Image { data_base64: b64, width, height })
}

fn preview_hex(data: &[u8]) -> Result<PreviewContent, String> {
    let limit = data.len().min(HEX_PREVIEW_LIMIT);
    let mut hex = String::from("\nOffset    Hex                                           ASCII\n");
    hex.push_str(&"-".repeat(80));
    for (i, chunk) in data[..limit].chunks(16).enumerate() {
        let offset = i * 16;
        let hex_part: String = chunk.iter().map(|b| format!("{:02X} ", b)).collect();
        let ascii_part: String = chunk.iter()
            .map(|b| if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' }).collect();
        hex.push_str(&format!("{:08X}  {:-48}  {}\n", offset, hex_part, ascii_part));
    }
    if data.len() > HEX_PREVIEW_LIMIT {
        hex.push_str(&format!("\n... [showing first {} KB of {} KB total]",
            HEX_PREVIEW_LIMIT / 1024, data.len() / 1024));
    }
    Ok(PreviewContent::HexDump(hex))
}

fn preview_archive(path: &str) -> Result<PreviewContent, String> {
    match crate::archive::forensic_load(path, None) {
        Ok(entries) => {
            let listing: Vec<String> = entries.iter()
                .map(|e| format!("{} ({} bytes)", e.path, e.size)).collect();
            Ok(PreviewContent::ArchiveList(listing))
        },
        Err(e) => Ok(PreviewContent::Unsupported(format!("Archive error: {}", e)))
    }
}

fn mime_for(ext: &str) -> String {
    match ext {
        "txt"|"md" => "text/plain", "html"|"htm" => "text/html", "css" => "text/css",
        "js" => "text/javascript", "json" => "application/json", "xml" => "application/xml",
        "csv" => "text/csv", "png" => "image/png", "jpg"|"jpeg" => "image/jpeg",
        "gif" => "image/gif", "webp" => "image/webp", "bmp" => "image/bmp",
        "svg" => "image/svg+xml", "pdf" => "application/pdf",
        "zip" => "application/zip", "rar" => "application/vnd.rar",
        "tar" => "application/x-tar",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn unique_temp(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{}-{}", label, uuid::Uuid::new_v4()))
    }

    #[test]
    fn test_preview_text_file() {
        let dir = unique_temp("pv_txt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("s.txt"), b"Hello forensic world\n").unwrap();
        let r = preview_file(dir.join("s.txt").to_str().unwrap()).unwrap();
        assert_eq!(r.kind, FileKind::Text);
        assert!(matches!(r.preview, PreviewContent::Text(_)));
        assert!(r.metadata.md5.is_some());
        assert!(r.metadata.entropy.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_preview_image_file() {
        let dir = unique_temp("pv_img");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Create minimal PNG
        let png = make_png();
        std::fs::write(dir.join("t.png"), &png).unwrap();
        let r = preview_file(dir.join("t.png").to_str().unwrap()).unwrap();
        assert_eq!(r.kind, FileKind::Image);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_preview_hex() {
        let dir = unique_temp("pv_hex");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let data: Vec<u8> = (0..64u8).collect();
        std::fs::write(dir.join("b.bin"), &data).unwrap();
        let r = preview_file(dir.join("b.bin").to_str().unwrap()).unwrap();
        assert!(matches!(r.preview, PreviewContent::HexDump(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_preview_metadata() {
        let dir = unique_temp("pv_meta");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("m.txt"), b"test").unwrap();
        let r = preview_file(dir.join("m.txt").to_str().unwrap()).unwrap();
        assert!(r.metadata.size > 0);
        assert!(r.metadata.entropy.unwrap_or(0.0) > 0.0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_preview_not_found() {
        assert!(preview_file("/nonexistent/x.txt").is_err());
    }

    #[test]
    fn test_preview_archive() {
        let dir = unique_temp("pv_arc");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), b"data").unwrap();
        let zp = dir.join("t.zip");
        let f = std::fs::File::create(&zp).unwrap();
        let mut w = zip::ZipWriter::new(f);
        w.start_file("a.txt", zip::write::SimpleFileOptions::default()).unwrap();
        w.write_all(b"data").unwrap();
        w.finish().unwrap();
        let r = preview_file(zp.to_str().unwrap()).unwrap();
        assert!(matches!(r.preview, PreviewContent::ArchiveList(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn make_png() -> Vec<u8> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        let mut d = vec![0x89,0x50,0x4E,0x47,0x0D,0x0A,0x1A,0x0A];
        // IHDR: 1x1 RGB
        let ihdr = [0u8,0,0,1, 0,0,0,1, 8,2,0,0,0];
        let mut c = b"IHDR".to_vec(); c.extend_from_slice(&ihdr);
        let crc = crc32fast::hash(&c);
        d.extend_from_slice(&(13u32.to_be_bytes())); d.extend_from_slice(&c); d.extend_from_slice(&crc.to_be_bytes());
        // IDAT
        let raw = [0u8, 255, 0, 0];
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::fast());
        enc.write_all(&raw).unwrap();
        let comp = enc.finish().unwrap();
        c = b"IDAT".to_vec(); c.extend_from_slice(&comp);
        let crc = crc32fast::hash(&c);
        d.extend_from_slice(&(comp.len() as u32).to_be_bytes()); d.extend_from_slice(&c); d.extend_from_slice(&crc.to_be_bytes());
        // IEND
        c = b"IEND".to_vec(); let crc = crc32fast::hash(&c);
        d.extend_from_slice(&[0,0,0,0]); d.extend_from_slice(b"IEND"); d.extend_from_slice(&crc.to_be_bytes());
        d
    }
}
