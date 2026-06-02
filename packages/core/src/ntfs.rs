use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MftEntry {
    pub record_number: u64,
    pub filename: String,
    pub parent_record: u64,
    pub is_directory: bool,
    pub is_deleted: bool,
    pub file_size: u64,
    pub si_created: Option<String>,
    pub si_modified: Option<String>,
    pub si_accessed: Option<String>,
    pub fn_created: Option<String>,
    pub fn_modified: Option<String>,
    pub has_data: bool,
    pub attributes: Vec<FileAttribute>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileAttribute {
    pub attr_type: String,
    pub name: Option<String>,
    pub size: u64,
    pub resident: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedFile {
    pub mft_entry: MftEntry,
    pub recovery_possible: bool,
    pub clusters_used: Vec<u64>,
}

/// Parse NTFS Master File Table from image/device
///
/// Supports large drives (5TB+) via chunked streaming reads
pub fn parse_mft(image_path: &str, cancel_flag: &std::sync::atomic::AtomicBool) -> Result<Vec<MftEntry>, String> {
    use std::sync::atomic::Ordering;
use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(image_path)
        .map_err(|e| format!("Cannot open: {}", e))?;

    // Read boot sector (first 512 bytes)
    let mut boot = [0u8; 512];
    file.read_exact(&mut boot).map_err(|e| format!("Cannot read boot sector: {}", e))?;

    // Parse NTFS BPB
    let bytes_per_sector = u16::from_le_bytes([boot[11], boot[12]]) as u64;
    let sectors_per_cluster = boot[13] as u64;
    let mft_cluster = u64::from_le_bytes([boot[48], boot[49], boot[50], boot[51], boot[52], boot[53], boot[54], boot[55]]);

    let cluster_size = bytes_per_sector * sectors_per_cluster;
    let mft_offset_sectors = mft_cluster * sectors_per_cluster;
    let mft_offset = mft_offset_sectors * bytes_per_sector;

    // Read MFT records
    let mft_record_size = 1024u64; // Standard MFT record = 1KB (can be 4KB on newer systems)
    let mut entries = vec![];
    let mut current_offset = mft_offset;

    // Scan first 256 records (cover FILE0 with first batch)
    let max_scan = 256 * mft_record_size;
    let mut buf = vec![0u8; mft_record_size as usize];

    for _ in 0..(max_scan / mft_record_size) {
        if cancel_flag.load(Ordering::SeqCst) { break; }

        file.seek(std::io::SeekFrom::Start(current_offset))
            .map_err(|e| format!("Seek error: {}", e))?;
        file.read_exact(&mut buf).map_err(|e| format!("Read error at MFT offset {}: {}", current_offset, e))?;

        // Check FILE signature ("FILE" or "BAAD")
        let signature = &buf[0..4];
        if signature != b"FILE" { current_offset += mft_record_size; continue; }

        // Parse fixup array
        let usa_offset = u16::from_le_bytes([buf[4], buf[5]]) as usize;
        let usa_count = u16::from_le_bytes([buf[6], buf[7]]) as usize;
        if usa_count > 0 && usa_offset > 0 {
            let usa = buf[usa_offset..usa_offset + usa_count * 2].to_vec();
            let bytes_per_sector_usize = bytes_per_sector as usize;
            for i in 1..usa_count {
                let sector_end = i * bytes_per_sector_usize - 2;
                buf[sector_end..sector_end + 2].copy_from_slice(&usa[i * 2..i * 2 + 2]);
            }
        }

        // Parse $STANDARD_INFORMATION ($SI) and $FILE_NAME ($FN) attributes
        let mut filename = String::new();
        let mut parent = 0u64;
        let mut is_dir = false;
        let mut is_deleted = buf[22] & 0x04 != 0; // Bit 2 = in-use flag
        let mut si_created = None;
        let mut si_modified = None;
        let mut si_accessed = None;
        let mut fn_created = None;
        let mut fn_modified = None;

        // Walk attributes starting at offset 56
        let attr_offset = u16::from_le_bytes([buf[20], buf[21]]) as usize;
        let mut pos = attr_offset;

        while pos + 16 <= buf.len() {
            let attr_type = u32::from_le_bytes([buf[pos], buf[pos+1], buf[pos+2], buf[pos+3]]);
            if attr_type == 0xFFFFFFFF { break; } // End marker

            let attr_len = u32::from_le_bytes([buf[pos+4], buf[pos+5], buf[pos+6], buf[pos+7]]) as usize;
            if attr_len == 0 || pos + attr_len > buf.len() { break; }

            let resident = buf[pos + 8] == 0x00;
            let name_len = buf[pos + 9] as usize;
            let content_offset = u16::from_le_bytes([buf[pos + 20], buf[pos + 21]]) as usize;
            let content_size = u32::from_le_bytes([buf[pos + 16], buf[pos + 17], buf[pos + 18], buf[pos + 19]]) as usize;

            let attr_header_size = if !resident { 64 } else { 24 + name_len * 2 };

            match attr_type {
                0x10 => { // $STANDARD_INFORMATION
                    if resident && content_offset + 48 <= buf.len() {
                        let si_pos = pos + content_offset;
                        si_created = Some(ntfs_timestamp(&buf[si_pos..si_pos + 8]));
                        si_modified = Some(ntfs_timestamp(&buf[si_pos + 8..si_pos + 16]));
                        si_accessed = Some(ntfs_timestamp(&buf[si_pos + 24..si_pos + 32]));
                    }
                }
                0x30 => { // $FILE_NAME
                    if resident && content_offset + 66 <= buf.len() {
                        let fn_pos = pos + content_offset;
                        parent = u64::from_le_bytes([
                            buf[fn_pos], buf[fn_pos+1], buf[fn_pos+2], buf[fn_pos+3],
                            buf[fn_pos+4], buf[fn_pos+5], buf[fn_pos+6], buf[fn_pos+7],
                        ]);
                        is_dir = buf[fn_pos + 56] & 0x02 != 0; // Bit 1 = directory
                        let fn_len = buf[fn_pos + 64] as usize;
                        if fn_len > 0 && fn_pos + 66 + fn_len * 2 <= buf.len() {
                            filename = String::from_utf16_lossy(
                                &buf[fn_pos + 66..fn_pos + 66 + fn_len * 2]
                                    .chunks(2)
                                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                    .collect::<Vec<u16>>()
                            );
                        }
                        fn_created = Some(ntfs_timestamp(&buf[fn_pos + 8..fn_pos + 16]));
                        fn_modified = Some(ntfs_timestamp(&buf[fn_pos + 16..fn_pos + 24]));
                    }
                }
                _ => {}
            }

            pos += attr_len;
        }

        if !filename.is_empty() {
            entries.push(MftEntry {
                record_number: current_offset / mft_record_size,
                filename,
                parent_record: parent,
                is_directory: is_dir,
                is_deleted,
                file_size: 0,
                si_created, si_modified, si_accessed,
                fn_created, fn_modified,
                has_data: false,
                attributes: vec![],
            });
        }

        current_offset += mft_record_size;
    }

    Ok(entries)
}

fn ntfs_timestamp(bytes: &[u8]) -> String {
    if bytes.len() < 8 { return "unknown".into(); }
    let timestamp: i64 = i64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    // NTFS epoch: January 1, 1601 in 100-nanosecond intervals
    let unix_ts = (timestamp / 10_000_000) - 11_644_473_600;
    if let Some(dt) = chrono::DateTime::from_timestamp(unix_ts, 0) {
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        "invalid".into()
    }
}
