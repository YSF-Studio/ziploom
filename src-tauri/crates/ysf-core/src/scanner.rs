use crate::archive::{self, FileEntry};
use serde::Serialize;
use std::io::Read;

const RAW_SCAN_LIMIT: u64 = 1024 * 1024;
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct HeuristicThreat {
    pub file: String,
    pub threat: String,
    pub category: String,
    pub severity: String,
    pub detail: String,
}

#[derive(Debug, Clone, Default, Serialize, serde::Deserialize)]
pub struct ArchiveHeuristicSummary {
    pub threats: Vec<HeuristicThreat>,
    pub risk_score: f64,
}

fn push(threats: &mut Vec<HeuristicThreat>, file: String, threat: String, category: &str, severity: &str, detail: &str) {
    threats.push(HeuristicThreat {
        file,
        threat,
        category: category.to_string(),
        severity: severity.to_string(),
        detail: detail.to_string(),
    });
}

fn suspicious_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("setup") || lower.contains("update") || lower.contains("invoice") || lower.contains("doc") || lower.contains("report")
}

fn homoglyphish(name: &str) -> bool {
    name.chars().any(|c| matches!(c, 'а' | 'е' | 'о' | 'р' | 'с' | 'х' | 'і' | 'ј' | 'ԁ' | 'ԛ'))
}

fn typosquatish(name: &str) -> bool {
    let lower = name.to_lowercase();
    ["facebok", "googIe", "microsofr", "adobе", "acrobat", "winrar", "7zip", "chromeu", "firef0x"]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn has_lookalike_path(path: &str) -> bool {
    homoglyphish(path) || typosquatish(path)
}

fn read_entry_bytes(path: &str, entry_path: &str, password: Option<&str>) -> Option<Vec<u8>> {
    crate::forensic::read_archive_entry(path, entry_path, password, RAW_SCAN_LIMIT)
        .ok()
        .map(|entry| entry.data)
}

fn to_hex_prefix(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(data.len() * 2);
    for &b in data {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn pdf_markers(data: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(data).to_lowercase();
    let mut hits = Vec::new();
    for marker in [
        "/javascript",
        "/js",
        "/openaction",
        "/aa",
        "/launch",
        "/embeddedfile",
        "/richmedia",
        "/xfa",
        "/objstm",
        "/uri",
        "/flatedecode",
        "/ascii85decode",
        "/lzwdecode",
        "/runlengthdecode",
        "/crypt",
    ] {
        if text.contains(marker) {
            hits.push(marker.to_string());
        }
    }
    hits
}

fn ole_markers(data: &[u8]) -> Vec<String> {
    let mut hits = Vec::new();
    if data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
        hits.push("ole-header".to_string());
    }
    let text = String::from_utf8_lossy(data).to_lowercase();
    for marker in [
        "vbaproject",
        "vba",
        "thisdocument",
        "autoopen",
        "document_open",
        "workbook_open",
        "autoexec",
        "wscript.shell",
        "powershell",
        "createobject",
        "shell(",
        "urldownload",
        "oleobject",
    ] {
        if text.contains(marker) {
            hits.push(marker.to_string());
        }
    }
    hits
}

fn lnk_markers(data: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(data).to_lowercase();
    let mut hits = Vec::new();
    if data.len() >= 76 && data.starts_with(&[0x4C, 0x00, 0x00, 0x00]) {
        hits.push("lnk-header".to_string());
        let clsid_ok = data[4..20] == [
            0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
        ];
        if clsid_ok {
            hits.push("shell-link-clsid".to_string());
        }
        let link_flags = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        if link_flags & 0x0000_0001 != 0 {
            hits.push("has-link-target-id-list".to_string());
        }
        if link_flags & 0x0000_0002 != 0 {
            hits.push("has-link-info".to_string());
        }
        if link_flags & 0x0000_0020 != 0 {
            hits.push("has-expString".to_string());
        }
        if link_flags & 0x0000_0040 != 0 {
            hits.push("has-relative-path".to_string());
        }
        if link_flags & 0x0000_0080 != 0 {
            hits.push("has-working-dir".to_string());
        }
        if link_flags & 0x0000_0100 != 0 {
            hits.push("has-command-line-args".to_string());
        }
        if link_flags & 0x0000_0200 != 0 {
            hits.push("has-icon-location".to_string());
        }
    }
    for marker in [
        "cmd.exe",
        "powershell",
        "wscript",
        "mshta",
        "rundll32",
        "urlfileprotocol",
        "http://",
        "https://",
        "file://",
        "\\\\",
    ] {
        if text.contains(marker) {
            hits.push(marker.to_string());
        }
    }
    for marker in ["powershell.exe", "wscript.exe", "cscript.exe", "mshta.exe", "rundll32.exe", "regsvr32.exe"] {
        if text.contains(marker) {
            hits.push(marker.to_string());
        }
    }
    hits
}

fn office_markers(data: &[u8]) -> Vec<String> {
    let mut hits = Vec::new();
    let text = String::from_utf8_lossy(data).to_lowercase();
    if data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
        hits.push("ole-header".to_string());
    }
    for marker in [
        "worddocument",
        "workbook",
        "powerpoint document",
        "macros",
        "_vba_project",
        "dir",
        "vba/",
        "projectwm",
        "ole10native",
        "encryptedpackage",
    ] {
        if text.contains(marker) {
            hits.push(marker.to_string());
        }
    }
    hits
}

fn utf16le_contains(data: &[u8], needle: &str) -> bool {
    let wide: Vec<u8> = needle
        .encode_utf16()
        .flat_map(|u| u.to_le_bytes())
        .collect();
    data.windows(wide.len()).any(|w| w == wide.as_slice())
}

fn script_markers(ext: &str, text: &str) -> Vec<String> {
    let mut markers = Vec::new();
    let dangerous = [
        ("python", ["eval(", "exec(", "__import__", "base64.b64decode"].as_slice()),
        ("bash", ["curl ", "wget ", "nc ", "bash -c", "sh -c"].as_slice()),
        ("powershell", ["-encodedcommand", "downloadstring", "iex(", "invoke-expression"].as_slice()),
        ("javascript", ["eval(", "atob(", "fromcharcode(", "unescape("].as_slice()),
    ];
    for (family, pats) in dangerous {
        if matches!(family, "python") && !matches!(ext, "py" | "pyw") {
            continue;
        }
        if matches!(family, "bash") && !matches!(ext, "sh" | "bash" | "zsh") {
            continue;
        }
        for pat in pats {
            if text.contains(pat) {
                markers.push(format!("{family}:{pat}"));
            }
        }
    }
    markers
}

fn inflate_pdf_streams(data: &[u8]) -> Vec<u8> {
    let mut joined = Vec::new();
    let mut idx = 0usize;
    while let Some(start) = data[idx..].windows(6).position(|w| w == b"stream") {
        let stream_start = idx + start + 6;
        let after = &data[stream_start..];
        let mut content_start = 0usize;
        while content_start < after.len() && matches!(after[content_start], b'\r' | b'\n' | b' ' | b'\t') {
            content_start += 1;
        }
        let after = &after[content_start..];
        if let Some(end) = after.windows(9).position(|w| w == b"endstream") {
            let chunk = &after[..end];
            let mut decoder = flate2::read::ZlibDecoder::new(chunk);
            let mut out = Vec::new();
            if decoder.read_to_end(&mut out).is_ok() && !out.is_empty() {
                joined.extend(out);
                joined.push(b'\n');
            } else if chunk.len() > 4 {
                let raw_ascii = String::from_utf8_lossy(chunk);
                if raw_ascii.to_lowercase().contains("/javascript") || raw_ascii.to_lowercase().contains("stream") {
                    joined.extend_from_slice(chunk);
                    joined.push(b'\n');
                }
            }
            idx = stream_start + content_start + end + 9;
        } else {
            break;
        }
    }
    joined
}

pub fn scan_archive_metadata(entries: &[FileEntry]) -> ArchiveHeuristicSummary {
    let mut threats = Vec::new();
    let mut risk_score = 0.0f64;

    if entries.len() > 10_000 {
        push(&mut threats, "*".into(), format!("File flood — {} entries", entries.len()), "archive", "high", "Archive contains an unusually large number of files");
        risk_score += 0.2;
    }

    let mut total_uncompressed = 0u64;
    let mut nested_archives = 0usize;

    for e in entries {
        if e.is_dir {
            continue;
        }
        total_uncompressed = total_uncompressed.saturating_add(e.size);
        let path = e.path.replace('\\', "/");
        let lower = path.to_lowercase();
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        let has_ext = path.contains('.');

        if lower.contains("../") || lower.contains("..\\") {
            push(&mut threats, path.clone(), "Path traversal".into(), "path", "high", "Entry name contains parent-directory traversal");
            risk_score += 0.2;
        }
        if path.starts_with('/') || path.starts_with('\\') || (path.len() > 2 && path.as_bytes()[1] == b':' && path.as_bytes()[2] == b'\\') {
            push(&mut threats, path.clone(), "Absolute path".into(), "path", "high", "Entry starts with an absolute path prefix");
            risk_score += 0.2;
        }
        if e.compressed_size.is_some() {
            let comp = e.compressed_size.unwrap_or(0).max(1);
            if e.size > 0 && comp > 0 && e.size / comp > 1000 {
                push(&mut threats, path.clone(), "Zip bomb ratio".into(), "bomb", "critical", "Compression ratio exceeds 1000:1");
                risk_score += 0.35;
            }
        }
        if e.size == 0 && suspicious_name(&path) {
            push(&mut threats, path.clone(), "Zero-byte dropper".into(), "dropper", "medium", "Zero-byte file in a suspicious location/name");
            risk_score += 0.08;
        }
        if !has_ext && e.detected_type.as_deref() == Some("PE (EXE/DLL)") {
            push(&mut threats, path.clone(), "No extension + PE magic".into(), "spoofing", "high", "Binary starts with MZ but has no extension");
            risk_score += 0.2;
        }
        if e.detected_type.as_deref() == Some("ELF") {
            push(&mut threats, path.clone(), "ELF executable".into(), "executable", "medium", "ELF magic detected inside archive");
            risk_score += 0.08;
        }
        if e.detected_type.as_deref() == Some("Mach-O fat") || e.detected_type.as_deref() == Some("Mach-O 64") || e.detected_type.as_deref() == Some("Mach-O 32") {
            push(&mut threats, path.clone(), "Mach-O executable".into(), "executable", "medium", "Mach-O binary detected inside archive");
            risk_score += 0.08;
        }
        if e.detected_type.as_deref() == Some("JPEG") || e.detected_type.as_deref() == Some("PNG") || e.detected_type.as_deref() == Some("GIF") || e.detected_type.as_deref() == Some("TIFF") || e.detected_type.as_deref() == Some("WebM/MKV") || e.detected_type.as_deref() == Some("MP4/ISO BMFF") {
            if let Some(cs) = e.compressed_size {
                if e.size > cs.saturating_mul(200) {
                    push(&mut threats, path.clone(), "Image anomaly".into(), "stegano", "low", "File size is unexpectedly large relative to image/video container");
                    risk_score += 0.03;
                }
            }
        }
        if suspicious_name(&path) && (ext == "exe" || ext == "msi" || ext == "scr" || ext == "dll") {
            push(&mut threats, path.clone(), "Suspicious setup-like executable".into(), "name", "medium", "Filename looks like a disguised installer or update");
            risk_score += 0.05;
        }
        if typosquatish(&path) {
            push(&mut threats, path.clone(), "Typosquatting".into(), "name", "medium", "Filename resembles a common product with a typo");
            risk_score += 0.06;
        }
        if homoglyphish(&path) {
            push(&mut threats, path.clone(), "Homoglyph Unicode".into(), "name", "medium", "Filename uses lookalike Unicode characters");
            risk_score += 0.06;
        }
        if path.split('/').count() > 6 {
            push(&mut threats, path.clone(), "Deep nesting".into(), "archive", "low", "Entry is nested unusually deep");
            risk_score += 0.02;
        }
        if ext == "zip" || ext == "7z" || ext == "rar" || ext == "tar" || ext == "gz" || ext == "bz2" || ext == "xz" || ext == "zst" {
            nested_archives += 1;
        }
    }

    if total_uncompressed > 10 * 1024 * 1024 * 1024 {
        push(&mut threats, "*".into(), "Decompression bomb".into(), "bomb", "critical", "Total uncompressed size exceeds 10 GB");
        risk_score += 0.35;
    }
    if nested_archives > 3 {
        push(&mut threats, "*".into(), "Deep nesting".into(), "archive", "low", "Archive contains nested archive files");
        risk_score += 0.04;
    }

    ArchiveHeuristicSummary { threats, risk_score }
}

pub fn scan_archive_content(path: &str, password: Option<&str>, entries: &[FileEntry]) -> Vec<HeuristicThreat> {
    let mut threats = Vec::new();
    let fmt = archive::detect_format(path);
    if fmt.is_none() {
        return threats;
    }
    for e in entries {
        if e.is_dir {
            continue;
        }
        let path_lower = e.path.to_lowercase();
        let ext = path_lower.rsplit('.').next().unwrap_or("");
        let interesting = matches!(
            ext,
            "pdf"
                | "js"
                | "py"
                | "sh"
                | "bash"
                | "zsh"
                | "ps1"
                | "vbs"
                | "bat"
                | "cmd"
                | "html"
                | "htm"
                | "lnk"
                | "doc"
                | "xls"
                | "ppt"
                | "docm"
                | "xlsm"
                | "pptm"
                | "docx"
                | "xlsx"
                | "pptx"
        );
        if !interesting {
            continue;
        }

        let raw = read_entry_bytes(path, &e.path, password);
        let mut corpus = String::new();
        if let Some(bytes) = &raw {
            corpus.push_str(&String::from_utf8_lossy(bytes));
            corpus.push('\n');
            corpus.push_str(&to_hex_prefix(&bytes[..bytes.len().min(256)]));
            let inflated = inflate_pdf_streams(bytes);
            if !inflated.is_empty() {
                corpus.push('\n');
                corpus.push_str(&String::from_utf8_lossy(&inflated));
            }
        }
        let lc = corpus.to_lowercase();

        if ext == "pdf" {
            let markers = raw.as_deref().map(pdf_markers).unwrap_or_default();
            if !markers.is_empty() || lc.contains("/javascript") || lc.contains("/openaction") || lc.contains("/launch") || lc.contains("/jbig2decode") {
                push(
                    &mut threats,
                    e.path.clone(),
                    "PDF with JavaScript".into(),
                    "pdf",
                    "high",
                    &format!("PDF contains suspicious markers: {}", markers.join(", ")),
                );
            }
            if lc.contains("/embeddedfile") || lc.contains("/richmedia") || lc.contains("/xfa") || lc.contains("/uri") {
                push(&mut threats, e.path.clone(), "PDF embedded files".into(), "pdf", "medium", "PDF contains EmbeddedFile/RichMedia markers");
            }
        }

        if matches!(ext, "py" | "sh" | "bash" | "zsh" | "ps1" | "vbs" | "bat" | "cmd" | "js" | "html" | "htm") {
            let markers = script_markers(ext, &lc);
            if !markers.is_empty() {
                push(
                    &mut threats,
                    e.path.clone(),
                    "Dangerous script markers".into(),
                    "script",
                    "high",
                    &format!("Matched markers: {}", markers.join(", ")),
                );
            }
        }

        if ext == "lnk" {
            let markers = raw.as_deref().map(lnk_markers).unwrap_or_default();
            if !markers.is_empty() {
                push(
                    &mut threats,
                    e.path.clone(),
                    "Suspicious LNK shortcut".into(),
                    "shortcut",
                    "high",
                    &format!("Shortcut target markers: {}", markers.join(", ")),
                );
            }
            if let Some(bytes) = &raw {
                if utf16le_contains(bytes, "powershell.exe")
                    || utf16le_contains(bytes, "cmd.exe")
                    || utf16le_contains(bytes, "mshta.exe")
                    || utf16le_contains(bytes, "rundll32.exe")
                    || utf16le_contains(bytes, "wscript.exe")
                {
                    push(
                        &mut threats,
                        e.path.clone(),
                        "LNK hidden target".into(),
                        "shortcut",
                        "high",
                        "Unicode target string reveals a suspicious executable",
                    );
                }
            }
        }

        if ext == "doc" || ext == "xls" || ext == "ppt" || ext == "docm" || ext == "xlsm" || ext == "pptm" || ext == "docx" || ext == "xlsx" || ext == "pptx" {
            let markers = raw.as_deref().map(ole_markers).unwrap_or_default();
            let office_hint = String::from_utf8_lossy(raw.as_deref().unwrap_or(&[])).to_lowercase();
            let office_struct = raw.as_deref().map(office_markers).unwrap_or_default();
            if markers.iter().any(|m| m == "ole-header" || m.contains("vba") || m.contains("autoopen") || m.contains("document_open") || m.contains("workbook_open"))
                || office_hint.contains("vbaproject.bin")
                || office_hint.contains("macroenabled")
                || office_struct.iter().any(|m| m.contains("worddocument") || m.contains("workbook") || m.contains("_vba_project") || m.contains("macros"))
            {
                push(
                    &mut threats,
                    e.path.clone(),
                    "OLE/VBA macro markers".into(),
                    "macro",
                    "high",
                    &format!("Matched markers: {}", [markers, office_struct].concat().join(", ")),
                );
            }
        }

        if has_lookalike_path(&e.path) {
            push(&mut threats, e.path.clone(), "Lookalike filename".into(), "name", "medium", "Filename uses typo or homoglyph tricks");
        }
    }
    threats
}
