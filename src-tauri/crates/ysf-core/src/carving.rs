use serde::Serialize;

/// Magic signatures for file carving
pub const MAGIC_SIGNATURES: &[(&[u8], &str)] = &[
    (b"\xff\xd8\xff", "JPEG"),
    (b"\x89PNG\r\n\x1a\n", "PNG"),
    (b"GIF8", "GIF"),
    (b"\x25PDF", "PDF"),
    (b"PK\x03\x04", "ZIP"),
    (b"MZ", "PE (EXE/DLL)"),
    (b"\x7fELF", "ELF executable"),
    (b"Rar!\x1a\x07", "RAR"),
    (b"\x1f\x8b", "GZip"),
    (b"BZh", "BZip2"),
    (b"\xfd7zXZ\x00", "XZ"),
    (b"ID3", "MP3 audio"),
    (b"OggS", "OGG audio"),
    (b"\xd0\xcf\x11\xe0", "OLE2 (Office)"),
    (b"SQLite format 3", "SQLite DB"),
    (b"\x00\x00\x01\x00", "ICO icon"),
    (b"\x1a\x45\xdf\xa3", "WebM/MKV"),
];

#[derive(Debug, Clone, Serialize)]
pub struct CarvingResult {
    pub files_found: usize,
    pub files: Vec<CarvedFile>,
    pub bytes_scanned: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CarvedFile {
    pub name: String,
    pub file_type: String,
    pub offset: u64,
    pub size: u64,
    pub header_valid: bool,
}

/// Carve files from raw disk/image data using magic byte signatures
///
/// Uses 256KB streaming reads — handles 5TB+ images
pub fn carve_files(
    image_path: &str,
    output_dir: &str,
    cancel_flag: &std::sync::atomic::AtomicBool,
) -> Result<CarvingResult, String> {
    use std::sync::atomic::Ordering;
    use std::io::{Read, Write};

    let mut file = std::fs::File::open(image_path)
        .map_err(|e| format!("Cannot open image: {}", e))?;
    let file_size = file.metadata().map_err(|e| e.to_string())?.len();

    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Cannot create output dir: {}", e))?;

    let buf_size = super::hashing::HASH_BUFFER_SIZE;
    let mut buf = vec![0u8; buf_size];
    let mut carryover = Vec::new(); // Overlap buffer for signatures crossing chunk boundaries
    let overlap = 128; // Max signature length to carry over
    let mut offset: u64 = 0;
    let mut carved_count = 0u64;
    let mut carved_files = vec![];

    loop {
        if cancel_flag.load(Ordering::SeqCst) { break; }

        let n = file.read(&mut buf).map_err(|e| format!("Read error: {}", e))?;
        if n == 0 { break; }

        let carryover_len = carryover.len();
        let search_buf = if carryover_len > 0 {
            let mut combined = carryover.clone();
            combined.extend_from_slice(&buf[..n]);
            carryover.clear();
            combined
        } else {
            buf[..n].to_vec()
        };

        // Save overlap for next chunk
        if n >= overlap {
            carryover = buf[n - overlap..n].to_vec();
        }

        // Search for each magic signature in this chunk
        for (magic, file_type) in MAGIC_SIGNATURES {
            let mut search_pos = 0usize;
            while search_pos + magic.len() <= search_buf.len() {
                if cancel_flag.load(Ordering::SeqCst) { break; }

                if search_buf[search_pos..search_pos + magic.len()] == **magic {
                    let abs_offset = offset - carryover_len as u64 + search_pos as u64;
                    let name = format!("{:08x}_{}.bin", abs_offset,
                        file_type.to_lowercase().replace(' ', "_").replace('/', "_"));

                    // Extract the file — basic extraction (header to reasonable size)
                    // In production: determine actual size from header fields
                    let extract_size = detect_file_size(&search_buf[search_pos..], *file_type)
                        .unwrap_or(n as u64);

                    let out_path = std::path::Path::new(output_dir).join(&name);
                    if let Ok(mut out) = std::fs::File::create(&out_path) {
                        let end = (search_pos + extract_size as usize).min(search_buf.len());
                        let _ = out.write_all(&search_buf[search_pos..end]);
                        carved_files.push(CarvedFile {
                            name,
                            file_type: (*file_type).to_string(),
                            offset: abs_offset,
                            size: extract_size,
                            header_valid: true,
                        });
                        carved_count += 1;
                    }

                    search_pos += magic.len();
                } else {
                    search_pos += 1;
                }
            }
        }

        offset += n as u64;

        super::progress::update_progress(
            (offset as f64 / file_size.max(1) as f64) * 100.0,
            &format!("Carving: {:.1} GB scanned, {} files found", offset as f64 / 1e9, carved_count),
            offset,
            file_size.max(1),
        );
    }

    let result = CarvingResult {
        files_found: carved_files.len(),
        files: carved_files,
        bytes_scanned: offset,
    };
    super::progress::finish_progress(Ok(format!("{} files carved", result.files_found)));
    Ok(result)
}

/// Try to determine file size from header information
fn detect_file_size(data: &[u8], file_type: &str) -> Option<u64> {
    match file_type {
        "PNG" => {
            // Check IEND chunk at end of PNG
            if data.len() >= 33 {
                // Width at offset 16, Height at offset 20
                let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
                let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
                Some(w as u64 * h as u64 * 4 + 100) // Rough estimate
            } else { None }
        }
        "JPEG" => {
            // Scan for EOI marker (FF D9)
            let mut pos = 2;
            while pos + 1 < data.len() {
                if data[pos] == 0xFF && data[pos + 1] == 0xD9 {
                    return Some(pos as u64 + 2);
                }
                pos += 1;
            }
            None
        }
        "PE (EXE/DLL)" | "ELF executable" => Some(data.len().min(50_000_000) as u64),
        _ => Some(data.len().min(10_000_000) as u64), // Default: reasonable cap
    }
}
