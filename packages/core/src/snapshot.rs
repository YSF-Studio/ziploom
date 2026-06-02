//! System Snapshot — Capture system state, diff changes, detect anomalies
//!
//! Used by CollectionLoom to monitor a system before & after acquisition,
//! detecting changes to files, processes, and network state.
//!
//! ISO 27037: Chain of custody includes system state baselines.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Unique snapshot identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SnapshotId(pub String);

impl SnapshotId {
    pub fn new(label: &str) -> Self {
        let ts = Utc::now().format("%Y%m%d_%H%M%S");
        Self(format!("SNAP-{}-{}", ts, label))
    }
}

/// Metadata for a single file in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub modified: String,         // ISO 8601
    pub permissions: String,
    pub is_dir: bool,
    pub is_symlink: bool,
}

/// A running process entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub state: String,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
}

/// A network connection entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntry {
    pub protocol: String,     // TCP, UDP
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,        // LISTEN, ESTABLISHED, etc.
    pub pid: u32,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub kernel: String,
    pub uptime_secs: u64,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
}

/// Complete system state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub id: SnapshotId,
    pub timestamp: String,
    pub label: String,
    pub info: SystemInfo,
    pub files: Vec<FileEntry>,
    pub processes: Vec<ProcessEntry>,
    pub network: Vec<NetworkEntry>,
}

/// Diff between two snapshots — what changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub snapshot_a: SnapshotId,
    pub snapshot_b: SnapshotId,
    pub timestamp: String,
    
    // File changes
    pub files_added: Vec<DiffFile>,
    pub files_removed: Vec<DiffFile>,
    pub files_modified: Vec<DiffFile>,
    
    // Process changes
    pub processes_started: Vec<DiffProcess>,
    pub processes_stopped: Vec<DiffProcess>,
    
    // Network changes
    pub connections_new: Vec<DiffNetwork>,
    pub connections_closed: Vec<DiffNetwork>,
    
    // Summary
    pub summary: DiffSummary,
}

/// A file with both-before-and-after info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: String,
    pub before: Option<FileEntry>,
    pub after: Option<FileEntry>,
}

/// Process diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffProcess {
    pub pid: u32,
    pub name: String,
    pub state_before: Option<String>,
    pub state_after: Option<String>,
}

/// Network diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffNetwork {
    pub local_addr: String,
    pub remote_addr: String,
    pub protocol: String,
    pub state_before: Option<String>,
    pub state_after: Option<String>,
}

/// Summary statistics for a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_files_before: usize,
    pub total_files_after: usize,
    pub files_added: usize,
    pub files_removed: usize,
    pub files_modified: usize,
    pub processes_before: usize,
    pub processes_after: usize,
    pub processes_changed: usize,
    pub connections_before: usize,
    pub connections_after: usize,
    pub connections_changed: usize,
    pub risk_level: String,  // LOW, MEDIUM, HIGH
}

// ═══════════════════════════════════════
// SNAPSHOT CAPTURE
// ═══════════════════════════════════════

/// Take a full system snapshot — files, processes, network, system info
pub fn take_snapshot(label: &str, scan_root: Option<&str>) -> Result<SystemSnapshot, String> {
    let root = scan_root.unwrap_or("/home");
    
    let info = capture_system_info()?;
    let files = scan_filesystem(root)?;
    let processes = capture_processes()?;
    let network = capture_network()?;
    
    Ok(SystemSnapshot {
        id: SnapshotId::new(label),
        timestamp: Utc::now().to_rfc3339(),
        label: label.to_string(),
        info,
        files,
        processes,
        network,
    })
}

/// Compare two snapshots and produce a diff
pub fn compare_snapshots(a: &SystemSnapshot, b: &SystemSnapshot) -> SnapshotDiff {
    let file_keys_before: HashSet<&str> = a.files.iter().map(|f| f.path.as_str()).collect();
    let file_keys_after: HashSet<&str> = b.files.iter().map(|f| f.path.as_str()).collect();
    
    let files_added: Vec<DiffFile> = b.files.iter()
        .filter(|f| !file_keys_before.contains(f.path.as_str()))
        .map(|f| DiffFile { path: f.path.clone(), before: None, after: Some(f.clone()) })
        .collect();
    
    let files_removed: Vec<DiffFile> = a.files.iter()
        .filter(|f| !file_keys_after.contains(f.path.as_str()))
        .map(|f| DiffFile { path: f.path.clone(), before: Some(f.clone()), after: None })
        .collect();
    
    // Find modified files (same path, different size or mtime)
    let files_before: HashMap<&str, &FileEntry> = a.files.iter()
        .map(|f| (f.path.as_str(), f)).collect();
    
    let files_modified: Vec<DiffFile> = b.files.iter()
        .filter_map(|fb| {
            files_before.get(fb.path.as_str()).and_then(|fa| {
                if fa.size != fb.size || fa.modified != fb.modified {
                    Some(DiffFile {
                        path: fb.path.clone(),
                        before: Some((*fa).clone()),
                        after: Some(fb.clone()),
                    })
                } else {
                    None
                }
            })
        })
        .collect();
    
    // Process diff
    let procs_before: HashMap<u32, &ProcessEntry> = a.processes.iter()
        .map(|p| (p.pid, p)).collect();
    let procs_after: HashSet<u32> = b.processes.iter().map(|p| p.pid).collect();
    
    let processes_started: Vec<DiffProcess> = b.processes.iter()
        .filter(|p| !procs_before.contains_key(&p.pid))
        .map(|p| DiffProcess { pid: p.pid, name: p.name.clone(), state_before: None, state_after: Some(p.state.clone()) })
        .collect();
    
    let processes_stopped: Vec<DiffProcess> = a.processes.iter()
        .filter(|p| !procs_after.contains(&p.pid))
        .map(|p| DiffProcess { pid: p.pid, name: p.name.clone(), state_before: Some(p.state.clone()), state_after: None })
        .collect();
    
    // Network diff
    let net_keys_before: HashSet<(String, String, String)> = a.network.iter()
        .map(|n| (n.local_addr.clone(), n.remote_addr.clone(), n.protocol.clone())).collect();
    let net_keys_after: HashSet<(String, String, String)> = b.network.iter()
        .map(|n| (n.local_addr.clone(), n.remote_addr.clone(), n.protocol.clone())).collect();
    
    let connections_new: Vec<DiffNetwork> = b.network.iter()
        .filter(|n| !net_keys_before.contains(&(n.local_addr.clone(), n.remote_addr.clone(), n.protocol.clone())))
        .map(|n| DiffNetwork { local_addr: n.local_addr.clone(), remote_addr: n.remote_addr.clone(), protocol: n.protocol.clone(), state_before: None, state_after: Some(n.state.clone()) })
        .collect();
    
    let connections_closed: Vec<DiffNetwork> = a.network.iter()
        .filter(|n| !net_keys_after.contains(&(n.local_addr.clone(), n.remote_addr.clone(), n.protocol.clone())))
        .map(|n| DiffNetwork { local_addr: n.local_addr.clone(), remote_addr: n.remote_addr.clone(), protocol: n.protocol.clone(), state_before: Some(n.state.clone()), state_after: None })
        .collect();
    
    // Risk assessment
    let total_changes = files_added.len() + files_removed.len() + files_modified.len()
        + processes_started.len() + processes_stopped.len()
        + connections_new.len() + connections_closed.len();
    
    let risk_level = if total_changes > 100 {
        "HIGH"
    } else if total_changes > 20 {
        "MEDIUM"
    } else {
        "LOW"
    };
    
    let summary = DiffSummary {
        total_files_before: a.files.len(),
        total_files_after: b.files.len(),
        files_added: files_added.len(),
        files_removed: files_removed.len(),
        files_modified: files_modified.len(),
        processes_before: a.processes.len(),
        processes_after: b.processes.len(),
        processes_changed: processes_started.len() + processes_stopped.len(),
        connections_before: a.network.len(),
        connections_after: b.network.len(),
        connections_changed: connections_new.len() + connections_closed.len(),
        risk_level: risk_level.to_string(),
    };
    
    SnapshotDiff {
        snapshot_a: a.id.clone(),
        snapshot_b: b.id.clone(),
        timestamp: Utc::now().to_rfc3339(),
        files_added,
        files_removed,
        files_modified,
        processes_started,
        processes_stopped,
        connections_new,
        connections_closed,
        summary,
    }
}

/// Generate a human-readable diff report
pub fn generate_diff_report(diff: &SnapshotDiff) -> String {
    let mut report = String::new();
    report.push_str(&format!("═══════════════════════════════════════════\n"));
    report.push_str(&format!("  SYSTEM SNAPSHOT DIFF REPORT\n"));
    report.push_str(&format!("  {}  vs  {}\n", diff.snapshot_a.0, diff.snapshot_b.0));
    report.push_str(&format!("  Generated: {}\n", diff.timestamp));
    report.push_str(&format!("═══════════════════════════════════════════\n\n"));
    
    report.push_str(&format!("📊 SUMMARY — Risk Level: {}\n", diff.summary.risk_level));
    report.push_str(&format!("  Files:     {} before → {} after ({} added, {} removed, {} modified)\n",
        diff.summary.total_files_before, diff.summary.total_files_after,
        diff.files_added.len(), diff.files_removed.len(), diff.files_modified.len()));
    report.push_str(&format!("  Processes: {} before → {} after ({} started, {} stopped)\n",
        diff.summary.processes_before, diff.summary.processes_after,
        diff.processes_started.len(), diff.processes_stopped.len()));
    report.push_str(&format!("  Networks:  {} before → {} after ({} new, {} closed)\n",
        diff.summary.connections_before, diff.summary.connections_after,
        diff.connections_new.len(), diff.connections_closed.len()));
    
    if !diff.files_added.is_empty() {
        report.push_str(&format!("\n📁 FILES ADDED:\n"));
        for f in diff.files_added.iter().take(20) {
            report.push_str(&format!("  + {}\n", f.path));
        }
        if diff.files_added.len() > 20 {
            report.push_str(&format!("  ... and {} more\n", diff.files_added.len() - 20));
        }
    }
    
    if !diff.files_removed.is_empty() {
        report.push_str(&format!("\n🗑️ FILES REMOVED:\n"));
        for f in diff.files_removed.iter().take(20) {
            report.push_str(&format!("  - {}\n", f.path));
        }
        if diff.files_removed.len() > 20 {
            report.push_str(&format!("  ... and {} more\n", diff.files_removed.len() - 20));
        }
    }
    
    if !diff.files_modified.is_empty() {
        report.push_str(&format!("\n✏️ FILES MODIFIED:\n"));
        for f in diff.files_modified.iter().take(10) {
            let before_size = f.before.as_ref().map(|b| b.size).unwrap_or(0);
            let after_size = f.after.as_ref().map(|a| a.size).unwrap_or(0);
            report.push_str(&format!("  ~ {} ({}B → {}B)\n", f.path, before_size, after_size));
        }
        if diff.files_modified.len() > 10 {
            report.push_str(&format!("  ... and {} more\n", diff.files_modified.len() - 10));
        }
    }
    
    if !diff.processes_started.is_empty() {
        report.push_str(&format!("\n⚙️ PROCESSES STARTED:\n"));
        for p in diff.processes_started.iter().take(10) {
            report.push_str(&format!("  + PID {}: {}\n", p.pid, p.name));
        }
    }
    
    if !diff.processes_stopped.is_empty() {
        report.push_str(&format!("\n⚙️ PROCESSES STOPPED:\n"));
        for p in diff.processes_stopped.iter().take(10) {
            report.push_str(&format!("  - PID {}: {}\n", p.pid, p.name));
        }
    }
    
    if !diff.connections_new.is_empty() {
        report.push_str(&format!("\n🌐 NEW CONNECTIONS:\n"));
        for n in diff.connections_new.iter().take(10) {
            report.push_str(&format!("  + {} {} → {} ({})\n", n.protocol, n.local_addr, n.remote_addr, n.state_after.as_deref().unwrap_or("?")));
        }
    }
    
    report
}

// ═══════════════════════════════════════
// INTERNAL HELPERS
// ═══════════════════════════════════════

fn capture_system_info() -> Result<SystemInfo, String> {
    let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    let kernel = std::fs::read_to_string("/proc/version")
        .unwrap_or_else(|_| "unknown".to_string())
        .split_whitespace()
        .nth(2)
        .unwrap_or("unknown")
        .to_string();
    
    let uptime = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;
    
    let (total_mb, avail_mb) = read_memory_info();
    
    Ok(SystemInfo {
        hostname,
        kernel,
        uptime_secs: uptime,
        total_memory_mb: total_mb,
        available_memory_mb: avail_mb,
    })
}

fn read_memory_info() -> (u64, u64) {
    let content = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let total = content.lines()
        .find(|l| l.starts_with("MemTotal:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse::<u64>().ok())
        .map(|k| k / 1024)
        .unwrap_or(0);
    let available = content.lines()
        .find(|l| l.starts_with("MemAvailable:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse::<u64>().ok())
        .map(|k| k / 1024)
        .unwrap_or(0);
    (total, available)
}

fn scan_filesystem(root: &str) -> Result<Vec<FileEntry>, String> {
    let root_path = Path::new(root);
    if !root_path.exists() {
        return Err(format!("Scan root does not exist: {}", root));
    }
    
    let mut entries = Vec::new();
    scan_dir(root_path, &mut entries, 0, 5)?; // max depth 5
    Ok(entries)
}

fn scan_dir(dir: &Path, entries: &mut Vec<FileEntry>, depth: usize, max_depth: usize) -> Result<(), String> {
    if depth > max_depth { return Ok(()); }
    
    let read_dir = std::fs::read_dir(dir).map_err(|e| format!("Cannot read dir {:?}: {}", dir, e))?;
    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
        let path = entry.path();
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        
        let modified = metadata.modified()
            .ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|| "unknown".to_string());
        
        let perms = format!("{:o}", metadata.permissions().mode() & 0o777);
        
        entries.push(FileEntry {
            path: path.to_string_lossy().to_string(),
            size: metadata.len(),
            modified,
            permissions: perms.clone(),
            is_dir: metadata.is_dir(),
            is_symlink: metadata.file_type().is_symlink(),
        });
        
        if metadata.is_dir() {
            match scan_dir(&path, entries, depth + 1, max_depth) {
                Ok(()) => {},
                Err(e) => {
                    // Skip directories we can't read
                    entries.push(FileEntry {
                        path: path.to_string_lossy().to_string(),
                        size: 0,
                        modified: "unknown".to_string(),
                        permissions: perms.clone(),
                        is_dir: true,
                        is_symlink: false,
                    });
                }
            }
        }
    }
    Ok(())
}

fn capture_processes() -> Result<Vec<ProcessEntry>, String> {
    let mut processes = Vec::new();
    let proc_dir = std::fs::read_dir("/proc").map_err(|e| format!("Cannot read /proc: {}", e))?;
    
    for entry in proc_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let pid: u32 = match name.to_string_lossy().parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        
        let stat_path = entry.path().join("stat");
        let stat_content = match std::fs::read_to_string(&stat_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        // Parse /proc/[pid]/stat: PID (comm) state ...
        let after_paren = match stat_content.find(") ") {
            Some(i) => &stat_content[i+2..],
            None => continue,
        };
        let state = after_paren.chars().next().unwrap_or('?').to_string();
        
        // Get process name from between parentheses
        let pname_start = stat_content.find('(').unwrap_or(0) + 1;
        let pname_end = stat_content.rfind(')').unwrap_or(0);
        let pname = if pname_end > pname_start {
            stat_content[pname_start..pname_end].to_string()
        } else {
            name.to_string_lossy().to_string()
        };
        
        // Memory from status
        let status_path = entry.path().join("status");
        let mem_bytes = std::fs::read_to_string(&status_path)
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("VmRSS:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(|kb| kb * 1024)
            })
            .unwrap_or(0);
        
        processes.push(ProcessEntry {
            pid,
            name: pname,
            state,
            cpu_percent: 0.0, // would need multiple samples
            memory_bytes: mem_bytes,
        });
    }
    
    Ok(processes)
}

fn capture_network() -> Result<Vec<NetworkEntry>, String> {
    let mut entries = Vec::new();
    entries.extend(parse_proc_net_tcp("/proc/net/tcp", "TCP")?);
    entries.extend(parse_proc_net_tcp("/proc/net/tcp6", "TCP6")?);
    Ok(entries)
}

fn parse_proc_net_tcp(path: &str, protocol: &str) -> Result<Vec<NetworkEntry>, String> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut entries = Vec::new();
    
    for line in content.lines().skip(1) { // skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 { continue; }
        
        let local = parse_socket_addr(parts[1]);
        let remote = parse_socket_addr(parts[2]);
        let state_code = parts[3];
        let state = tcp_state_name(state_code);
        
        let pid = parts.last()
            .and_then(|s| s.split('/').next())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        entries.push(NetworkEntry {
            protocol: protocol.to_string(),
            local_addr: local,
            remote_addr: remote,
            state,
            pid,
        });
    }
    
    Ok(entries)
}

fn parse_socket_addr(hex: &str) -> String {
    // Format: 0100007F:1F90
    let parts: Vec<&str> = hex.split(':').collect();
    if parts.len() != 2 { return hex.to_string(); }
    
    let ip_hex = parts[0];
    let port_hex = parts[1];
    
    // Parse hex IP (little-endian)
    let ip_bytes: Vec<u8> = (0..ip_hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&ip_hex[i..i+2], 16).ok())
        .collect();
    
    let ip = ip_bytes.iter()
        .map(|b| format!("{}", b))
        .collect::<Vec<_>>()
        .join(".");
    
    let port = u16::from_str_radix(port_hex, 16).unwrap_or(0);
    format!("{}:{}", ip, port)
}

fn tcp_state_name(code: &str) -> String {
    match code {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        _ => code,
    }.to_string()
}

// ═══════════════════════════════════════
// TESTS
// ═══════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snapshot_id_generation() {
        let id = SnapshotId::new("test");
        assert!(id.0.starts_with("SNAP-"));
        assert!(id.0.ends_with("-test"));
    }
    
    #[test]
    fn test_take_snapshot_basic() {
        // Take a snapshot of /tmp (small, controlled directory)
        let snap = take_snapshot("test_basic", Some("/tmp"))
            .expect("Failed to take snapshot");
        
        assert!(snap.id.0.contains("test_basic"));
        assert!(!snap.info.hostname.is_empty());
        assert!(snap.info.total_memory_mb > 0);
        // Should find at least /tmp contents
        assert!(snap.files.len() >= 1, "Should have at least 1 file in /tmp");
    }
    
    #[test]
    fn test_compare_snapshots_no_changes() {
        let a = take_snapshot("compare_a", Some("/tmp")).unwrap();
        let b = take_snapshot("compare_b", Some("/tmp")).unwrap();
        let diff = compare_snapshots(&a, &b);
        
        // Summary should be populated
        assert_eq!(diff.snapshot_a.0, a.id.0);
        assert_eq!(diff.snapshot_b.0, b.id.0);
        assert!(!diff.summary.risk_level.is_empty());
    }
    
    #[test]
    fn test_generate_diff_report() {
        let a = take_snapshot("report_a", Some("/tmp")).unwrap();
        let b = take_snapshot("report_b", Some("/tmp")).unwrap();
        let diff = compare_snapshots(&a, &b);
        let report = generate_diff_report(&diff);
        
        assert!(report.contains("SYSTEM SNAPSHOT DIFF REPORT"));
        assert!(report.contains(&diff.summary.risk_level));
        assert!(report.contains("Files:"));
        assert!(report.contains("Processes:"));
    }
    
    #[test]
    fn test_snapshot_detects_file_creation() {
        use std::io::Write;
        
        let temp = std::env::temp_dir().join("snap_test_create");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(temp.join("before.txt"), b"before").unwrap();
        
        let a = take_snapshot("create_a", Some(temp.to_str().unwrap())).unwrap();
        
        // Create a new file
        std::fs::write(temp.join("after.txt"), b"after").unwrap();
        
        let b = take_snapshot("create_b", Some(temp.to_str().unwrap())).unwrap();
        let diff = compare_snapshots(&a, &b);
        
        assert!(diff.files_added.iter().any(|f| f.path.contains("after.txt")),
            "Should detect new file: {:?}", diff.files_added);
        
        let _ = std::fs::remove_dir_all(&temp);
    }
    
    #[test]
    fn test_snapshot_detects_file_deletion() {
        let temp = std::env::temp_dir().join("snap_test_delete");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(temp.join("todelete.txt"), b"delete me").unwrap();
        
        let a = take_snapshot("delete_a", Some(temp.to_str().unwrap())).unwrap();
        
        // Remove the file
        std::fs::remove_file(temp.join("todelete.txt")).unwrap();
        
        let b = take_snapshot("delete_b", Some(temp.to_str().unwrap())).unwrap();
        let diff = compare_snapshots(&a, &b);
        
        assert!(diff.files_removed.iter().any(|f| f.path.contains("todelete.txt")),
            "Should detect deleted file");
        
        let _ = std::fs::remove_dir_all(&temp);
    }
    
    #[test]
    fn test_snapshot_detects_file_modification() {
        let temp = std::env::temp_dir().join("snap_test_modify");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(temp.join("modify.txt"), b"original content").unwrap();
        
        // Small delay to ensure different mtime
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let a = take_snapshot("modify_a", Some(temp.to_str().unwrap())).unwrap();
        
        // Modify the file with different content
        std::fs::write(temp.join("modify.txt"), b"modified content!").unwrap();
        
        let b = take_snapshot("modify_b", Some(temp.to_str().unwrap())).unwrap();
        let diff = compare_snapshots(&a, &b);
        
        assert!(diff.files_modified.iter().any(|f| f.path.contains("modify.txt")),
            "Should detect modified file: {:?}", diff.files_modified);
        
        let _ = std::fs::remove_dir_all(&temp);
    }
    
    #[test]
    fn test_risk_level_assessment() {
        let temp = std::env::temp_dir().join("snap_test_risk");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        
        let a = take_snapshot("risk_a", Some(temp.to_str().unwrap())).unwrap();
        
        // Create many files to trigger MEDIUM risk
        for i in 0..25 {
            std::fs::write(temp.join(format!("file_{}.txt", i)), b"data").unwrap();
        }
        
        let b = take_snapshot("risk_b", Some(temp.to_str().unwrap())).unwrap();
        let diff = compare_snapshots(&a, &b);
        
        assert_eq!(diff.summary.risk_level, "MEDIUM",
            "25 file changes should be MEDIUM risk, got: {}", diff.summary.risk_level);
        
        let _ = std::fs::remove_dir_all(&temp);
    }
    
    #[test]
    fn test_capture_processes_works() {
        let procs = capture_processes().expect("Should capture processes");
        assert!(procs.len() > 5, "Should have at least 5 processes, got {}", procs.len());
        // Should at least find PID 1 (init/systemd)
        assert!(procs.iter().any(|p| p.pid == 1), "Should find PID 1");
    }
    
    #[test]
    fn test_system_info_works() {
        let info = capture_system_info().expect("Should capture system info");
        assert!(!info.hostname.is_empty());
        assert!(info.total_memory_mb > 0);
        assert!(info.uptime_secs > 0);
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let snap = take_snapshot("serde_test", Some("/tmp")).unwrap();
        let json = serde_json::to_string(&snap).expect("Serialize");
        let deserialized: SystemSnapshot = serde_json::from_str(&json).expect("Deserialize");
        assert_eq!(snap.id.0, deserialized.id.0);
        assert_eq!(snap.files.len(), deserialized.files.len());
    }
}
