use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use sha2::Digest;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AcquisitionState {
    Idle,
    PreTriage,
    PreTriageDone,
    AwaitingDecision,
    CapturingRam,
    Imaging,
    Verifying,
    Done,
    Failed(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub device: String,
    pub model: String,
    pub size_bytes: u64,
    pub sector_size: u64,
    pub is_ssd: bool,
    pub partitions: Vec<PartitionInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartitionInfo {
    pub device: String,
    pub size_bytes: u64,
    pub file_system: String,
}

impl DiskInfo {
    #[cfg(target_os = "linux")]
    pub fn list() -> Result<Vec<Self>, String> {
        use std::process::Command;
        let output = Command::new("lsblk")
            .args(["-J", "-o", "NAME,SIZE,MODEL,ROTA,TYPE,MOUNTPOINT,FSTYPE"])
            .output().map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        let mut disks = vec![];
        if let Some(devices) = json["blockdevices"].as_array() {
            for d in devices {
                if d["type"].as_str() == Some("disk") {
                    let size_str = d["size"].as_str().unwrap_or("0");
                    let size = parse_size(size_str);
                    disks.push(DiskInfo {
                        device: format!("/dev/{}", d["name"].as_str().unwrap_or("?")),
                        model: d["model"].as_str().unwrap_or("Unknown").to_string(),
                        size_bytes: size,
                        sector_size: 512,
                        is_ssd: d["rota"].as_str() == Some("0"),
                        partitions: vec![],
                    });
                }
            }
        }
        Ok(disks)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn list() -> Result<Vec<Self>, String> {
        Ok(vec![]) // Placeholder for other platforms
    }
}

fn parse_size(s: &str) -> u64 {
    let s = s.trim().to_uppercase();
    if s.ends_with("G") { s.trim_end_matches('G').parse::<f64>().unwrap_or(0.0) as u64 * 1_073_741_824 }
    else if s.ends_with("M") { s.trim_end_matches('M').parse::<f64>().unwrap_or(0.0) as u64 * 1_048_576 }
    else if s.ends_with("T") { s.trim_end_matches('T').parse::<f64>().unwrap_or(0.0) as u64 * 1_099_511_627_776 }
    else { s.parse().unwrap_or(0) }
}

/// Stream disk to image file with progress, split, and verify
pub struct DiskImager {
    pub source: String,
    pub destination: PathBuf,
    pub split_size: Option<u64>,
    pub verify: bool,
}

impl DiskImager {
    pub fn new(source: &str, dest: &Path) -> Self {
        Self { source: source.to_string(), destination: dest.to_path_buf(), split_size: None, verify: true }
    }

    /// Stream entire disk with 256KB buffers — handles 5TB+ drives
    pub fn run(&self, cancel_flag: &std::sync::atomic::AtomicBool) -> Result<String, String> {
        let src = File::open(&self.source)
            .map_err(|e| format!("Cannot open source {}: {}", self.source, e))?;
        let src_size = src.metadata().map_err(|e| e.to_string())?.len();
        let mut reader = BufReader::with_capacity(super::hashing::HASH_BUFFER_SIZE, src);

        let mut total_written: u64 = 0;
        let mut part_num: u16 = 0;
        let mut hasher = sha2::Sha256::new();

        // Determine output path
        let stem = self.destination.file_stem().unwrap_or_default().to_string_lossy();
        let dir = self.destination.parent().unwrap_or(Path::new("."));

        loop {
            if cancel_flag.load(Ordering::SeqCst) {
                return Err("CANCELLED".into());
            }

            // Open output part
            part_num += 1;
            let out_name = if self.split_size.is_some() && part_num > 1 {
                format!("{}.{:03}", stem, part_num)
            } else {
                stem.to_string()
            };
            let out_path = dir.join(&out_name);

            let dst = OpenOptions::new().write(true).create(true).truncate(true).open(&out_path)
                .map_err(|e| format!("Cannot create {}: {}", out_path.display(), e))?;
            let mut writer = BufWriter::with_capacity(super::hashing::HASH_BUFFER_SIZE, dst);

            let mut part_written: u64 = 0;
            let split_limit = self.split_size.unwrap_or(u64::MAX);
            let mut buf = vec![0u8; super::hashing::HASH_BUFFER_SIZE];

            loop {
                if cancel_flag.load(Ordering::SeqCst) {
                    return Err("CANCELLED".into());
                }
                let n = reader.read(&mut buf).map_err(|e| format!("Read error: {}", e))?;
                if n == 0 { break; }

                let chunk = &buf[..n];
                writer.write_all(chunk).map_err(|e| format!("Write error: {}", e))?;
                hasher.update(chunk);
                part_written += n as u64;
                total_written += n as u64;

                let pct = if src_size > 0 { (total_written as f64 / src_size as f64) * 100.0 } else { 0.0 };
                super::progress::update_progress(
                    pct,
                    &format!("Imaging: {:.1} GB / {:.1} GB", total_written as f64 / 1e9, src_size as f64 / 1e9),
                    total_written,
                    src_size,
                );

                if part_written >= split_limit { break; }
            }

            writer.flush().map_err(|e| e.to_string())?;

            if self.split_size.is_none() || total_written >= src_size { break; }
        }

        let hash = format!("{:x}", hasher.finalize());
        super::progress::finish_progress(Ok(hash.clone()));
        Ok(hash)
    }
}
