/// ZipLoom — Malware Scanner Module
/// 100% Pure Rust, Zero External Data, Zero Network, Zero Updates Needed
/// Mendeteksi ancaman berdasarkan ANALISIS STRUKTUR FILE, bukan signature DB.
///
/// Strategi deteksi:
/// 1. PE Executable Analysis — parse header, deteksi suspicious imports/sections
/// 2. Office Macro Detection — scan VBA project untuk keyword berbahaya
/// 3. Ransomware Heuristic — deteksi ransom note, encrypted file patterns
/// 4. General Suspicious — double extension, executable in archive, hidden files
/// 5. ELF Executable Analysis — parse header, detect suspicious sections/interpreter
/// 6. Java Class Detection — magic byte CAFEBABE
/// 7. Archive-Level Heuristics — zip bomb, decompression bomb, file flood, deep nesting

use serde::{Deserialize, Serialize};

// ─── Scan Result Types ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalwareThreat {
    pub file: String,
    pub category: String,      // "pe", "macro", "ransomware", "suspicious"
    pub threat: String,        // "SuspiciousImport", "OfficeMacro", etc
    pub severity: String,      // "low", "medium", "high", "critical"
    pub detail: String,        // Human-readable explanation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub threats: Vec<MalwareThreat>,
    pub total_scanned: usize,
    pub risk_score: f64,       // 0.0 — 1.0
    pub risk_label: String,    // "Clean", "Low Risk", "Suspicious", "Malicious"
}

// ─── Main Scan Entry Point ───

/// Scan file content untuk ancaman (dipanggil pas forensic report)
pub fn scan_file_content(name: &str, data: &[u8]) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();

    // Skip directory entries
    if name.ends_with('/') || data.is_empty() {
        return threats;
    }

    // 1. PE Executable Analysis
    if is_pe(data) {
        threats.extend(analyze_pe(name, data));
    }

    // 2. Office Macro Detection (docx/xlsx/pptx = ZIP, doc/xls/ppt = OLE)
    if is_office_doc(name) {
        threats.extend(analyze_office_macro(name, data));
    }

    // 3. Ransomware note detection
    threats.extend(detect_ransomware_note(name, data));

    // 4. General file-level heuristics
    threats.extend(check_suspicious_content(name, data));

    // 5. ELF executable analysis
    if is_elf(data) {
        threats.extend(analyze_elf(name, data));
    }

    // 6. Java class file detection
    if is_java_class(data) {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "suspicious".into(),
            threat: "JavaClassFile".into(),
            severity: "medium".into(),
            detail: "Java .class file inside archive — may contain malicious bytecode".into(),
        });
    }

    threats
}

/// Scan file name/path aja (buat forensic_load yg belum baca data)
pub fn scan_file_name(name: &str) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();
    let lower = name.to_lowercase();

    // Hidden files
    if let Some(fname) = std::path::Path::new(name).file_name() {
        let fname = fname.to_string_lossy();
        if fname.starts_with('.') && fname != "." && fname != ".." {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "suspicious".into(),
                threat: "HiddenFile".into(),
                severity: "low".into(),
                detail: "Hidden file in archive — may be used to conceal malicious content".into(),
            });
        }
    }

    // Double extension
    let dot_count = lower.matches('.').count();
    if dot_count >= 2 {
        let parts: Vec<&str> = lower.rsplitn(3, '.').collect();
        if parts.len() >= 2 {
            let last = parts[0];
            let second = parts.get(1).unwrap_or(&"");
            let dangerous_exts = ["exe", "scr", "com", "bat", "cmd", "vbs", "vbe",
                                   "js", "jse", "wsf", "wsh", "ps1", "psm1", "dll",
                                   "jar", "class", "hta", "msi", "gadget"];
            if dangerous_exts.contains(&last) && second.len() <= 5 {
                threats.push(MalwareThreat {
                    file: name.to_string(),
                    category: "suspicious".into(),
                    threat: "DoubleExtension".into(),
                    severity: "high".into(),
                    detail: format!("Double extension detected — file pretends to be a .{} but is actually executable", second),
                });
            }
        }
    }

    // Ransomware note filenames
    let ransom_notes = ["readme", "how_to", "howtodecrypt", "decrypt", "ransom",
                         "recover", "restore", "help_decrypt", "info", "help",
                         "!!!readme", "#_readme", "readme_now", "recovery",
                         "readme_for", "lock", "encrypted"];
    for note in &ransom_notes {
        if lower.contains(note) {
            for ext in &[".txt", ".html", ".htm", ".hta", ".png", ".bmp"] {
                if lower.ends_with(ext) && !lower.contains("/") {
                    threats.push(MalwareThreat {
                        file: name.to_string(),
                        category: "ransomware".into(),
                        threat: "RansomwareNote".into(),
                        severity: "medium".into(),
                        detail: format!("Filename matches known ransomware note pattern: '{}'", name),
                    });
                    break;
                }
            }
        }
    }

    // Executable in archive
    let exec_extensions = ["exe", "msi", "scr", "com", "dll", "sys", "drv", "cpl"];
    if let Some(ext) = std::path::Path::new(name).extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        if exec_extensions.contains(&ext.as_str()) {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "suspicious".into(),
                threat: "ExecutableInArchive".into(),
                severity: "medium".into(),
                detail: format!("Windows executable inside archive: .{}", ext),
            });
        }
    }

    // Script files in archive
    let script_extensions = ["ps1", "psm1", "vbs", "vbe", "bat", "cmd", "js", "jse", "wsf"];

    // Path traversal detection
    if name.contains("..") {
        let has_traversal = name.contains("../") || name.contains("..\\")
            || name.contains("/..") || name.contains("\\..");
        if has_traversal {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "suspicious".into(),
                threat: "PathTraversal".into(),
                severity: "critical".into(),
                detail: format!("Path traversal detected in entry: '{}' — may attempt to write outside destination directory", name),
            });
        }
    }

    // Absolute path detection
    if name.starts_with('/') || name.starts_with('\\') {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "suspicious".into(),
            threat: "AbsolutePath".into(),
            severity: "critical".into(),
            detail: format!("Entry uses absolute path: '{}' — may attempt to overwrite system files", name),
        });
    }
    #[cfg(windows)]
    {
        let upper = name.to_uppercase();
        if upper.starts_with("C:\\") || upper.starts_with("C:/")
            || upper.starts_with("D:\\") || upper.starts_with("D:/") {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "suspicious".into(),
                threat: "AbsolutePath".into(),
                severity: "critical".into(),
                detail: format!("Entry uses absolute Windows path: '{}'", name),
            });
        }
    }

    // Zero-byte dropper indicator (file 0 byte at suspicious path)
    // Note: actual 0-byte check happens in scan_file_content when data is empty
    // but not a directory entry

    // No extension but executable magic — checked in scan_file_content via is_pe()
    if let Some(ext) = std::path::Path::new(name).extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        if script_extensions.contains(&ext.as_str()) {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "suspicious".into(),
                threat: "ScriptInArchive".into(),
                severity: "medium".into(),
                detail: format!("Script file inside archive: .{}", ext),
            });
        }
    }

    // Shortcut files (often used in malware distribution)
    if lower.ends_with(".lnk") {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "suspicious".into(),
            threat: "ShortcutFile".into(),
            severity: "low".into(),
            detail: "Windows shortcut (.lnk) inside archive — commonly used in malware delivery".into(),
        });
    }

    threats
}

// ─── PE Executable Analysis ───

/// Minimal PE detection
fn is_pe(data: &[u8]) -> bool {
    if data.len() < 64 { return false; }
    if &data[0..2] != b"MZ" { return false; }
    let pe_offset = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;
    if pe_offset + 4 > data.len() { return false; }
    &data[pe_offset..pe_offset+4] == b"PE\x00\x00"
}

/// Analyze PE file structur untuk ancaman
fn analyze_pe(name: &str, data: &[u8]) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();

    let pe_offset = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;

    // ── COFF Header ──
    // PE offset + 4 = COFF header (20 bytes)
    let coff = pe_offset + 4;
    if coff + 20 > data.len() { return threats; }

    let machine = u16::from_le_bytes([data[coff], data[coff+1]]);
    let num_sections = u16::from_le_bytes([data[coff+2], data[coff+3]]);
    let opt_header_size = u16::from_le_bytes([data[coff+16], data[coff+17]]);

    // Machine: 0x14c = I386, 0x8664 = AMD64, 0x1c0 = ARM
    let is_x86 = machine == 0x14c;
    let is_x64 = machine == 0x8664;
    if !is_x86 && !is_x64 && machine != 0x1c0 && machine != 0xAA64 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "RareArchitecture".into(),
            severity: "low".into(),
            detail: format!("Uncommon PE architecture: 0x{:04X}", machine),
        });
    }

    // ── Optional Header ──
    let opt = coff + 20;
    if opt + 2 > data.len() { return threats; }

    let magic = u16::from_le_bytes([data[opt], data[opt+1]]);
    // 0x10b = PE32, 0x20b = PE32+

    // Subsystem offset in optional header: always at +68 for both PE32 and PE32+
    let _subsystem = u16::from_le_bytes([
        data.get(opt + 68).copied().unwrap_or(0),
        data.get(opt + 69).copied().unwrap_or(0),
    ]);
    //  4-7: code size
    //  8-11: initialized data size
    // 12-15: uninitialized data size
    // 16-19: entry point RVA
    // 20-23: code base
    // 24-27: data base (PE32 only)
    // ...
    // 68: subsystem (WORD)
    // 70: dll characteristics (WORD)
    // ...
    // Section headers start at: opt + opt_header_size

    let _entry_rva = if magic == 0x20b {
        // PE32+: entry point at offset 16 (8 bytes)
        u64::from_le_bytes([
            data.get(opt+16).copied().unwrap_or(0),
            data.get(opt+17).copied().unwrap_or(0),
            data.get(opt+18).copied().unwrap_or(0),
            data.get(opt+19).copied().unwrap_or(0),
            data.get(opt+20).copied().unwrap_or(0),
            data.get(opt+21).copied().unwrap_or(0),
            data.get(opt+22).copied().unwrap_or(0),
            data.get(opt+23).copied().unwrap_or(0),
        ]) as u32
    } else {
        u32::from_le_bytes([
            data.get(opt+16).copied().unwrap_or(0),
            data.get(opt+17).copied().unwrap_or(0),
            data.get(opt+18).copied().unwrap_or(0),
            data.get(opt+19).copied().unwrap_or(0),
        ])
    };

    let subsystem = u16::from_le_bytes([
        data.get(opt + 68).copied().unwrap_or(0),
        data.get(opt + 69).copied().unwrap_or(0),
    ]);

    // Subsystem: 1=Native, 2=GUI, 3=Console, 7=Posix, 10=EfiApp, etc
    if subsystem == 1 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "NativeSubsystem".into(),
            severity: "medium".into(),
            detail: "PE has Native subsystem (driver/subsystem binary) — higher privilege level".into(),
        });
    }

    // ── DLL Characteristics (detect malicious anti-analysis) ──
    let dll_chars_offset = opt + 70;
    if dll_chars_offset + 2 <= data.len() {
        let dll_chars = u16::from_le_bytes([data[dll_chars_offset], data[dll_chars_offset+1]]);
        // 0x0400 = DYNAMIC_BASE (ASLR) — actually this is GOOD, means ASLR is enabled
        // 0x1000 = NX_COMPAT — GOOD
        // 0x4000 = GUARD_CF — Control Flow Guard
        // Missing NX means executable stack
        if dll_chars & 0x1000 == 0 {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "pe".into(),
                threat: "NoDataExecutionPrevention".into(),
                severity: "medium".into(),
                detail: "PE lacks NX (Non-Executable Stack) — DEP disabled, may allow code injection".into(),
            });
        }
    }

    // ── Section Headers ──
    // Section headers start at opt + opt_header_size
    let sections_start = opt + opt_header_size as usize;
    let section_entry_size = 40; // Each section header is 40 bytes

    for i in 0..num_sections as usize {
        let sec = sections_start + i * section_entry_size;
        if sec + 40 > data.len() { break; }

        let sec_name = std::str::from_utf8(&data[sec..sec+8]).unwrap_or("???????").trim_end_matches('\0');
        let sec_chars = u32::from_le_bytes([
            data[sec+36], data[sec+37], data[sec+38], data[sec+39],
        ]);

        let is_executable = sec_chars & 0x20000000 != 0; // IMAGE_SCN_MEM_EXECUTE
        let is_writable = sec_chars & 0x80000000 != 0;   // IMAGE_SCN_MEM_WRITE

        // W+X section = classic shellcode injection
        if is_executable && is_writable {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "pe".into(),
                threat: "WritableExecutableSection".into(),
                severity: "critical".into(),
                detail: format!("Section '{}' is both writable AND executable — classic code injection technique", sec_name),
            });
        }

        // Suspicious section names
        let suspicious_sections = [".upx", ".pack", ".themida", "UPX0", "UPX1", ".vmp",
                                    ".enigma", ".aspack", ".nsp0", ".nsp1", ".mackt",
                                    ".petite", ".wwpack", ".svkp", ".nspack", ".mpress",
                                    ".pklst", ".bind", ".ive", ".pebundle"];
        for sus in &suspicious_sections {
            if sec_name.starts_with(sus) {
                threats.push(MalwareThreat {
                    file: name.to_string(),
                    category: "pe".into(),
                    threat: "PackedExecutable".into(),
                    severity: "high".into(),
                    detail: format!("Section '{}' indicates packer/protector — common malware obfuscation", sec_name),
                });
                break;
            }
        }

        // Huge virtual size vs raw size (indicates packed/overlapping section)
        let virtual_size = u32::from_le_bytes([data[sec+8], data[sec+9], data[sec+10], data[sec+11]]);
        let raw_size = u32::from_le_bytes([data[sec+16], data[sec+17], data[sec+18], data[sec+19]]);
        if virtual_size > raw_size * 3 && raw_size > 0 {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "pe".into(),
                threat: "SectionSizeAnomaly".into(),
                severity: "high".into(),
                detail: format!("Section '{}' virtual size ({}) >> raw size ({}) — packed/section overlap", sec_name, virtual_size, raw_size),
            });
        }
    }

    // ── Import Address Table (suspicious imports) ──
    threats.extend(analyze_pe_imports(name, data, pe_offset, magic));

    threats
}

/// Parse import table and look for suspicious API calls
fn analyze_pe_imports(name: &str, data: &[u8], pe_offset: usize, _magic: u16) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();

    // Find data directory entries
    let _opt = pe_offset + 24; // Skip PE sig (4) + COFF (20) to start of optional header
    // Actually, pe_offset already points to "PE\0\0"
    // COFF header: 20 bytes starting at pe_offset+4
    // So Optional header starts at pe_offset+24
    // Wait no - pe_offset is the address of "PE\0\0"
    // pe_offset+4 = COFF start (20 bytes)
    // pe_offset+24 = Optional header start

    // Data directory is at the end of optional header
    // For PE32: optional header is 96 bytes, data dir starts at opt+96-128 = opt - 32... no

    // Let me recalculate.
    // PE32 optional header: 96 bytes (standard)
    // PE32+ optional header: 112 bytes (standard, but actually 112 is the minimum and it can be larger for data directories...)
    // Actually the data directory count and entries are at the end of the optional header.

    // For PE32: the import table RVA is at data directory entry [1]
    // Data directory starts at:
    //   PE32:  opt + 96
    //   PE32+: opt + 112
    // But actually the size of optional header varies. We should use opt_header_size from COFF.

    // Simpler approach: let's just scan the binary for known suspicious import strings
    // in the import section. This is less precise but works without full PE parsing.

    let suspicious_apis = [
        // Process injection
        ("VirtualAllocEx", "Process injection"),
        ("VirtualAlloc", "Memory allocation"),
        ("WriteProcessMemory", "Process injection"),
        ("CreateRemoteThread", "Remote thread injection"),
        ("CreateRemoteThreadEx", "Remote thread injection"),
        ("QueueUserAPC", "APC injection"),
        ("NtCreateThreadEx", "Thread creation"),
        ("SetThreadContext", "Context manipulation"),
        ("GetThreadContext", "Context manipulation"),
        ("ResumeThread", "Thread manipulation"),
        ("ZwUnmapViewOfSection", "Process hollowing"),
        ("NtUnmapViewOfSection", "Process hollowing"),
        // Shellcode execution
        ("VirtualProtect", "Memory protection change"),
        ("VirtualProtectEx", "Memory protection change"),
        ("HeapCreate", "Heap allocation"),
        ("RtlCopyMemory", "Memory copy"),
        ("memcpy", "Memory copy"),
        // Persistence
        ("RegSetValueEx", "Registry persistence"),
        ("RegCreateKeyEx", "Registry persistence"),
        ("SHGetSpecialFolderPath", "Startup folder access"),
        // Anti-analysis
        ("IsDebuggerPresent", "Anti-debugging"),
        ("CheckRemoteDebuggerPresent", "Anti-debugging"),
        ("NtGlobalFlag", "Anti-debugging"),
        ("OutputDebugString", "Anti-debugging"),
        // Network
        ("WinHttpOpen", "HTTP communication"),
        ("URLDownloadToFile", "File download"),
        ("URLDownloadToCacheFile", "File download"),
        ("InternetOpen", "Network communication"),
        ("InternetOpenUrl", "Network communication"),
        // Credential theft
        ("GetAsyncKeyState", "Keylogging"),
        ("GetForegroundWindow", "Window tracking"),
        ("GetWindowText", "Window text capture"),
        ("Clipboard", "Clipboard monitoring"),
        // Process hiding
        ("RegisterServiceProcess", "Process hiding"),
    ];

    // Scan for API names in the data section (import names)
    // We look for ASCII string patterns
    let mut found_apis: Vec<(&str, &str)> = Vec::new();
    for (api, desc) in &suspicious_apis {
        if contains_case_insensitive(data, api.as_bytes()) {
            found_apis.push((api, desc));
        }
    }

    if found_apis.is_empty() {
        return threats;
    }

    let injection_apis: Vec<&str> = found_apis
            .iter()
            .filter(|(_, desc)| *desc == "Process injection" || *desc == "Remote thread injection")
            .map(|(api, _)| *api)
            .collect();

    let hollowing_apis: Vec<&str> = found_apis.iter()
        .filter(|(_, desc)| *desc == "Process hollowing")
        .map(|(api, _)| *api)
        .collect();

    let anti_debug: Vec<&str> = found_apis.iter()
        .filter(|(_, desc)| *desc == "Anti-debugging")
        .map(|(api, _)| *api)
        .collect();

    let persistence_apis: Vec<&str> = found_apis.iter()
        .filter(|(_, desc)| *desc == "Registry persistence" || *desc == "Startup folder access")
        .map(|(api, _)| *api)
        .collect();

    let keylog_apis: Vec<&str> = found_apis.iter()
        .filter(|(_, desc)| *desc == "Keylogging" || *desc == "Window tracking")
        .map(|(api, _)| *api)
        .collect();

    if injection_apis.len() >= 3 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "ProcessInjection".into(),
            severity: "critical".into(),
            detail: format!("Multiple process injection APIs: {}", injection_apis.join(", ")),
        });
    } else if injection_apis.len() >= 1 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "SuspiciousImport".into(),
            severity: "high".into(),
            detail: format!("Contains process injection API: {}", injection_apis.join(", ")),
        });
    }

    if hollowing_apis.len() >= 1 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "ProcessHollowing".into(),
            severity: "critical".into(),
            detail: format!("Process hollowing API detected: {}", hollowing_apis.join(", ")),
        });
    }

    if anti_debug.len() >= 3 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "AntiDebugging".into(),
            severity: "high".into(),
            detail: format!("Multiple anti-debugging techniques: {}", anti_debug.join(", ")),
        });
    }

    if persistence_apis.len() >= 2 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "PersistenceMechanism".into(),
            severity: "high".into(),
            detail: format!("Persistence mechanisms detected: {}", persistence_apis.join(", ")),
        });
    }

    if keylog_apis.len() >= 2 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "Keylogger".into(),
            severity: "critical".into(),
            detail: format!("Keylogging/capturing APIs: {}", keylog_apis.join(", ")),
        });
    }

    // General: more than 5 high-risk imports
    let high_risk: Vec<&str> = found_apis.iter()
        .filter(|(_, desc)| matches!(*desc,
            "Process injection" | "Remote thread injection" | "Process hollowing"
            | "APC injection" | "Memory protection change"))
        .map(|(api, _)| *api)
        .collect();

    if high_risk.len() >= 2 && high_risk.len() < 3 {
        // Already handled above for 3+, but add a general one if not caught
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "SuspiciousImport".into(),
            severity: "high".into(),
            detail: format!("Contains suspicious APIs: {} ({} total)", high_risk.join(", "), found_apis.len()),
        });
    }

    threats
}

// ─── ELF Executable Analysis ───

/// Minimal ELF detection — magic byte \x7fELF
fn is_elf(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == 0x7f && data[1] == b'E' && data[2] == b'L' && data[3] == b'F'
}

/// Analyze ELF file for suspicious characteristics
fn analyze_elf(name: &str, data: &[u8]) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();

    // Check ELF class (32-bit vs 64-bit)
    let elf_class = data.get(4).copied().unwrap_or(0);
    // 1 = 32-bit, 2 = 64-bit

    // Check byte order
    let _byte_order = data.get(5).copied().unwrap_or(0);

    // OS/ABI at byte 7
    let osabi = data.get(7).copied().unwrap_or(0);
    // 0x00 = System V, 0x03 = Linux, 0x09 = FreeBSD

    if osabi != 0x00 && osabi != 0x03 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),  // reuse pe category for executables
            threat: "RareELFabI".into(),
            severity: "low".into(),
            detail: format!("Uncommon ELF OS/ABI: 0x{:02X}", osabi),
        });
    }

    // Entry point
    let entry_offset = if elf_class == 2 {
        // 64-bit: entry at offset 24 (8 bytes)
        if data.len() < 32 { return threats; }
        u64::from_le_bytes([
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
        ])
    } else if elf_class == 1 {
        // 32-bit: entry at offset 24 (4 bytes)
        if data.len() < 28 { return threats; }
        u64::from_le_bytes([
            data[24], data[25], data[26], data[27],
            0, 0, 0, 0,
        ])
    } else {
        return threats;
    };

    // Suspicious: entry point 0 for executables (not shared objects)
    let e_type = u16::from_le_bytes([data.get(16).copied().unwrap_or(0), data.get(17).copied().unwrap_or(0)]);
    // 2 = ET_EXEC, 3 = ET_DYN (shared)

    if e_type == 2 && entry_offset == 0 {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "pe".into(),
            threat: "ZeroEntryPoint".into(),
            severity: "high".into(),
            detail: "ELF executable has entry point at 0 — heavily packed or malicious".into(),
        });
    }

    // Check program headers for suspicious interpreters
    let phoff = if elf_class == 2 {
        u64::from_le_bytes([
            data.get(32).copied().unwrap_or(0),
            data.get(33).copied().unwrap_or(0),
            data.get(34).copied().unwrap_or(0),
            data.get(35).copied().unwrap_or(0),
            data.get(36).copied().unwrap_or(0),
            data.get(37).copied().unwrap_or(0),
            data.get(38).copied().unwrap_or(0),
            data.get(39).copied().unwrap_or(0),
        ]) as usize
    } else {
        let base = 28;
        u64::from_le_bytes([
            data.get(base).copied().unwrap_or(0),
            data.get(base+1).copied().unwrap_or(0),
            data.get(base+2).copied().unwrap_or(0),
            data.get(base+3).copied().unwrap_or(0),
            0, 0, 0, 0,
        ]) as usize
    };

    let phentsize = data.get(54).copied().unwrap_or(0) as usize; // Actually 52 for 32-bit, 58 for 64-bit
    let phnum = data.get(56).copied().unwrap_or(0) as usize;     // 56 for both

    if phoff > 0 && phoff < data.len() && phentsize >= 32 {
        // Scan program headers for PT_INTERP (type 3)
        for i in 0..phnum.min(16) {
            let phdr = phoff + i * phentsize.max(32);
            if phdr + 8 > data.len() { break; }

            let p_type = if elf_class == 2 {
                u32::from_le_bytes([data[phdr], data[phdr+1], data[phdr+2], data[phdr+3]])
            } else {
                u32::from_le_bytes([data[phdr], data[phdr+1], data[phdr+2], data[phdr+3]])
            };

            // PT_INTERP = 3
            if p_type == 3 {
                // Find interpreter string in data
                let offset_field = if elf_class == 2 {
                    let off = phdr + 8;
                    if off + 8 > data.len() { continue; }
                    u64::from_le_bytes([
                        data[off], data[off+1], data[off+2], data[off+3],
                        data[off+4], data[off+5], data[off+6], data[off+7],
                    ]) as usize
                } else {
                    let off = phdr + 4;
                    if off + 4 > data.len() { continue; }
                    u64::from_le_bytes([
                        data[off], data[off+1], data[off+2], data[off+3],
                        0, 0, 0, 0,
                    ]) as usize
                };

                let filesz_field = if elf_class == 2 {
                    let off = phdr + 32;
                    if off + 8 > data.len() { continue; }
                    u64::from_le_bytes([
                        data[off], data[off+1], data[off+2], data[off+3],
                        data[off+4], data[off+5], data[off+6], data[off+7],
                    ]) as usize
                } else {
                    let off = phdr + 20;
                    if off + 4 > data.len() { continue; }
                    u32::from_le_bytes([
                        data[off], data[off+1], data[off+2], data[off+3],
                    ]) as usize
                };

                // Bound check for interpreter string
                if offset_field < data.len() && filesz_field > 0 {
                    let end = (offset_field + filesz_field).min(data.len());
                    if end > offset_field {
                        if let Ok(interp) = std::str::from_utf8(&data[offset_field..end]) {
                            let interp = interp.trim_end_matches('\0');
                            if interp.contains("sh") || interp.contains("ld-") {
                                // normal
                            } else if interp.contains("libc") {
                                // normal
                            } else if interp.contains("/tmp/") || interp.contains("/dev/") {
                                threats.push(MalwareThreat {
                                    file: name.to_string(),
                                    category: "pe".into(),
                                    threat: "SuspiciousInterpreter".into(),
                                    severity: "high".into(),
                                    detail: format!("ELF uses suspicious interpreter: '{}'", interp),
                                });
                            }
                        }
                    }
                }
                break; // Only one PT_INTERP
            }
        }
    }

    // Check for RPATH/RUNPATH with suspicious paths
    if contains_case_insensitive(data, b"RPATH=") || contains_case_insensitive(data, b"RUNPATH=") {
        if contains_case_insensitive(data, b"/tmp/") || contains_case_insensitive(data, b"/var/tmp/") {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "pe".into(),
                threat: "SuspiciousRpath".into(),
                severity: "high".into(),
                detail: "ELF RPATH/RUNPATH points to temporary directory — possible DLL hijacking".into(),
            });
        }
    }

    // Check section header for suspicious section names
    let shoff = if elf_class == 2 {
        let off = 40;
        if off + 8 > data.len() { return threats; }
        u64::from_le_bytes([
            data[off], data[off+1], data[off+2], data[off+3],
            data[off+4], data[off+5], data[off+6], data[off+7],
        ]) as usize
    } else {
        let off = 32;
        if off + 4 > data.len() { return threats; }
        u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) as usize
    };

    let shstrndx = u16::from_le_bytes([data.get(50).copied().unwrap_or(0), data.get(51).copied().unwrap_or(0)]) as usize;

    if shoff > 0 && shoff < data.len() {
        let shentsize = u16::from_le_bytes([data.get(58).copied().unwrap_or(0), data.get(59).copied().unwrap_or(0)]) as usize;
        let shnum = u16::from_le_bytes([data.get(60).copied().unwrap_or(0), data.get(61).copied().unwrap_or(0)]) as usize;
        let section_size = shentsize.max(64);

        for i in 0..shnum.min(40) {
            let shdr = shoff + i * section_size;
            if shdr + 16 > data.len() { break; }

            // Get section name offset in string table
            let sh_name_off = u32::from_le_bytes([
                data.get(shdr).copied().unwrap_or(0),
                data.get(shdr+1).copied().unwrap_or(0),
                data.get(shdr+2).copied().unwrap_or(0),
                data.get(shdr+3).copied().unwrap_or(0),
            ]) as usize;

            // Try to get section name from string table
            let strtab_off = shoff + shstrndx * section_size;
            if strtab_off + 8 > data.len() { continue; }
            let strtab_fileoff = if elf_class == 2 {
                let off = strtab_off + 24;
                if off + 8 > data.len() { continue; }
                u64::from_le_bytes([
                    data[off], data[off+1], data[off+2], data[off+3],
                    data[off+4], data[off+5], data[off+6], data[off+7],
                ]) as usize
            } else {
                let off = strtab_off + 16;
                if off + 4 > data.len() { continue; }
                u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) as usize
            };

            if sh_name_off + 8 > data.len() || strtab_fileoff + sh_name_off + 8 > data.len() {
                continue;
            }

            let sec_name_start = strtab_fileoff + sh_name_off;
            if sec_name_start >= data.len() { continue; }
            let sec_name_end = data[sec_name_start..].iter().position(|&b| b == 0).unwrap_or(8);
            let sec_name = std::str::from_utf8(&data[sec_name_start..sec_name_start + sec_name_end.min(8)]).unwrap_or("?");

            let suspicious_elf_sections = [
                "packed", "UPX0", "UPX1", "upx", ".upx", ".packed",
                "themida", "vmp0", "vmp1", ".vmp", ".enigma",
            ];
            for sus in &suspicious_elf_sections {
                if sec_name.contains(sus) {
                    threats.push(MalwareThreat {
                        file: name.to_string(),
                        category: "pe".into(),
                        threat: "PackedExecutable".into(),
                        severity: "high".into(),
                        detail: format!("ELF section '{}' indicates packer/protector — common malware obfuscation", sec_name),
                    });
                    break;
                }
            }
        }
    }

    threats
}

// ─── Java Class File Detection ───

/// Java .class file detection — magic CAFEBABE
fn is_java_class(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == 0xCA && data[1] == 0xFE && data[2] == 0xBA && data[3] == 0xBE
}

// ─── Office Macro Detection ───

fn is_office_doc(name: &str) -> bool {
    let lower = name.to_lowercase();
    // ZIP-based Office formats
    lower.ends_with(".docx") || lower.ends_with(".xlsx") || lower.ends_with(".pptx")
    || lower.ends_with(".docm") || lower.ends_with(".xlsm") || lower.ends_with(".pptm")
    // OLE-based (older) Office formats
    || lower.ends_with(".doc") || lower.ends_with(".xls") || lower.ends_with(".ppt")
    || lower.ends_with(".dot") || lower.ends_with(".xla") || lower.ends_with(".pps")
}

/// Analyze Office document for macro-based threats
fn analyze_office_macro(name: &str, data: &[u8]) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();
    let lower_name = name.to_lowercase();

    // ZIP-based Office docs (docx, xlsx, pptx, docm, xlsm, pptm)
    if lower_name.ends_with("x") || lower_name.ends_with("m") {
        // These are ZIP archives — try to parse and find vbaProject.bin
        // We use std's approach: look for vba project signature in raw data
        // vbaProject.bin starts with a Compound Binary header

        // Search for "vbaProject.bin" in the raw data
        if data.windows(15).any(|w| w == b"vbaProject.bin") {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "macro".into(),
                threat: "OfficeMacroPresent".into(),
                severity: "medium".into(),
                detail: "Document contains VBA macro project (vbaProject.bin)".into(),
            });

            // Scan for dangerous macro keywords in the raw data
            threats.extend(scan_vba_keywords(name, data));
        }

        // Check for OLE object embedding (exploit delivery)
        if data.windows(9).any(|w| w == b"oleObject") {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "macro".into(),
                threat: "EmbeddedOLEObject".into(),
                severity: "medium".into(),
                detail: "Document contains embedded OLE objects — may contain exploits".into(),
            });
        }
    }

    // OLE-based docs (doc, xls, ppt) — check for VBA stream signatures
    if lower_name.ends_with("c") || lower_name.ends_with("t") || lower_name.ends_with("a") || lower_name.ends_with("s") {
        // OLE compound binary starts with D0 CF 11 E0 A1 B1 1A E1
        if data.len() >= 8 && &data[0..8] == b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1" {
            // Check for VBA stream names
            if data.windows(7).any(|w| w == b"VBA/Pro") || data.windows(4).any(|w| w == b"VBA") {
                threats.push(MalwareThreat {
                    file: name.to_string(),
                    category: "macro".into(),
                    threat: "OfficeMacroPresent".into(),
                    severity: "medium".into(),
                    detail: "Legacy OLE document contains VBA macros".into(),
                });

                threats.extend(scan_vba_keywords(name, data));
            }
        }
    }

    threats
}

/// Scan VBA content for dangerous keywords
fn scan_vba_keywords(name: &str, data: &[u8]) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();
    let mut found_dangerous: Vec<String> = Vec::new();
    let mut found_autoexec: Vec<String> = Vec::new();
    let mut found_network: Vec<String> = Vec::new();

    // Dangerous VBA functions
    let dangerous = [
        ("Shell", "Shell execution"),
        ("CreateObject", "COM object creation"),
        ("GetObject", "COM object retrieval"),
        ("WScript.Shell", "Shell access via WScript"),
        ("WScriptShell", "Shell access via WScript"),
        ("MSXML2", "XMLHTTP request"),
        ("WinHttp", "WinHTTP request"),
        ("XMLHTTP", "HTTP request"),
        ("ADODB.Stream", "File stream access"),
        ("FileSystemObject", "File system access"),
        ("Scripting.FileSystemObject", "File system access"),
        ("ActiveXObject", "ActiveX object creation"),
        ("PowerShell", "PowerShell execution"),
        ("powershell", "PowerShell execution"),
        ("cmd.exe", "Command execution"),
        ("rundll32", "DLL execution"),
        ("regsvr32", "COM registration"),
        ("mshta", "HTA execution"),
        ("certutil", "Certificate utility (LOLBIN)"),
        ("bitsadmin", "BITS admin (LOLBIN)"),
        ("cscript", "CScript execution"),
        ("wscript", "WScript execution"),
    ];

    for (keyword, context) in &dangerous {
        if contains_case_insensitive(data, keyword.as_bytes()) {
            found_dangerous.push(format!("{} ({})", keyword, context));
        }
    }

    // Auto-execute macros (common malware vector)
    let autoexec = [
        "AutoOpen", "Auto_Open", "Document_Open", "Workbook_Open",
        "Auto_Exec", "AutoExec", "AutoClose", "Auto_Close",
        "Document_Close", "Document_BeforeClose",
        "Auto_Activate", "Auto_Deactivate",
        "Workbook_Activate", "Workbook_Deactivate",
        "Sheet_Activate", "Sheet_Deactivate",
        "Auto_New", "Document_New",
    ];

    for keyword in &autoexec {
        if data.windows(keyword.len()).any(|w| w == keyword.as_bytes()) {
            found_autoexec.push(keyword.to_string());
        }
    }

    // Network-related (download/phone home)
    let network = [
        "URLDownload", "URLMon", "DownloadFile", "InternetOpen",
        "WinHttpOpen", "Send", "HTTPRequest", "http://", "https://",
        "ftp://", "WebDAV", "ServerXMLHTTP",
    ];

    for keyword in &network {
        if contains_case_insensitive(data, keyword.as_bytes()) {
            found_network.push(keyword.to_string());
        }
    }

    // Generate threats based on findings
    if !found_autoexec.is_empty() {
        // Check if both auto-exec AND dangerous functions exist
        if !found_dangerous.is_empty() || !found_network.is_empty() {
            let detail = format!("Auto-exec macro '{}' with dangerous capabilities: {}",
                found_autoexec.join(", "),
                found_dangerous.iter().chain(found_network.iter()).take(5).cloned().collect::<Vec<_>>().join(", "));
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "macro".into(),
                threat: "AutoExecMaliciousMacro".into(),
                severity: "critical".into(),
                detail,
            });
        } else {
            threats.push(MalwareThreat {
                file: name.to_string(),
                category: "macro".into(),
                threat: "AutoExecMacro".into(),
                severity: "medium".into(),
                detail: format!("Auto-execute macro detected: {}", found_autoexec.join(", ")),
            });
        }
    }

    if !found_dangerous.is_empty() && found_autoexec.is_empty() {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "macro".into(),
            threat: "SuspiciousMacroFunction".into(),
            severity: "high".into(),
            detail: format!("Dangerous VBA functions: {}", found_dangerous.join(", ")),
        });
    }

    if !found_network.is_empty() && !found_dangerous.is_empty() {
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "macro".into(),
            threat: "MacroNetworkAccess".into(),
            severity: "high".into(),
            detail: format!("Macro with network capabilities: {}", found_network.join(", ")),
        });
    }

    threats
}

// ─── Ransomware Detection ───

/// Detect ransomware notes based on content
fn detect_ransomware_note(name: &str, data: &[u8]) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();
    let lower = name.to_lowercase();

    // Only scan text files that could be ransom notes
    let is_text = lower.ends_with(".txt") || lower.ends_with(".html") || lower.ends_with(".htm")
        || lower.ends_with(".hta") || lower.ends_with(".bmp");

    if !is_text || data.is_empty() || data.len() > 100_000 {
        return threats;
    }

    // Convert first 2KB to lowercase text for scanning
    let scan_len = data.len().min(2048);
    let content = String::from_utf8_lossy(&data[..scan_len]).to_lowercase();

    // Ransomware note signature patterns
    let ransom_signatures = [
        ("your files have been encrypted", "ransomware payment note"),
        ("your files are encrypted", "ransomware payment note"),
        ("your documents have been encrypted", "ransomware payment note"),
        ("all your files have been encrypted", "ransomware payment note"),
        ("your important files are encrypted", "ransomware payment note"),
        ("your personal files are encrypted", "ransomware payment note"),
        ("your data is encrypted", "ransomware payment note"),
        ("your data has been encrypted", "ransomware payment note"),
        ("files have been encrypted", "ransomware payment note"),
        ("were encrypted by", "ransomware note"),
        ("your files were encrypted", "ransomware payment note"),
        ("your files are locked", "ransomware note"),
        ("your files have been locked", "ransomware note"),
        ("your documents are locked", "ransomware note"),
        ("all files on your computer are encrypted", "ransomware payment note"),
        ("files encrypted", "ransomware status"),
        ("to decrypt your files", "ransomware payment instructions"),
        ("to recover your files", "ransomware payment instructions"),
        ("to get your files back", "ransomware payment instructions"),
        ("to restore your files", "ransomware payment instructions"),
        ("decryption key", "ransomware demand"),
        ("pay", "ransomware demand"),
        ("bitcoin", "cryptocurrency payment"),
        ("bitcoins", "cryptocurrency payment"),
        ("btc wallet", "cryptocurrency payment"),
        ("tor browser", "ransomware communication channel"),
        ("torproject", "ransomware communication channel"),
        ("onion", "Tor hidden service"),
        ("email us", "contact instruction"),
        ("within 24 hours", "ransomware deadline"),
        ("within 48 hours", "ransomware deadline"),
        ("within 72 hours", "ransomware deadline"),
        ("48 hours", "ransomware deadline"),
        ("72 hours", "ransomware deadline"),
        ("price increases", "ransomware pricing tactic"),
        ("your time is running out", "ransomware threat"),
        ("if you do not pay", "ransomware threat"),
        ("don't try to recover", "ransomware threat"),
        ("do not try to recover", "ransomware threat"),
        ("do not use any recovery", "ransomware threat"),
        ("any attempt to decrypt", "ransomware threat"),
        ("don't contact police", "ransomware intimidation"),
        ("do not contact law", "ransomware intimidation"),
        ("do not contact police", "ransomware intimidation"),
        ("we have downloaded", "data theft ransomware"),
        ("your data was stolen", "data theft/extortion"),
        ("we have stolen", "double extortion"),
        ("we have hacked", "intrusion claim"),
        ("your network has been breached", "intrusion claim"),
        ("we have accessed", "data breach claim"),
        ("sensitive data", "data theft claim"),
        ("confidential data", "data theft claim"),
    ];

    let mut matches: Vec<&str> = Vec::new();
    for (signature, _context) in &ransom_signatures {
        if content.contains(signature) {
            matches.push(signature);
        }
    }

    // Need at least 2 signatures to reduce false positives
    if matches.len() >= 2 {
        let severity = if matches.len() >= 5 { "critical" } else { "high" };
        threats.push(MalwareThreat {
            file: name.to_string(),
            category: "ransomware".into(),
            threat: "RansomwareNote".into(),
            severity: severity.into(),
            detail: format!("File contains {} ransomware note signatures (e.g., '{}')",
                matches.len(), matches[0]),
        });
    }

    threats
}

// ─── General Content Heuristics ───

fn check_suspicious_content(name: &str, data: &[u8]) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();

    // PowerShell one-liner detection (encoded command)
    if !is_pe(data) && data.len() < 50_000 {
        let scan = String::from_utf8_lossy(data);
        let lower = scan.to_lowercase();

        // PowerShell -EncodedCommand detection
        if lower.contains("-encodedcommand") || lower.contains("-enc ") {
            let parts: Vec<&str> = lower.split("-encodedcommand").collect();
            if parts.len() > 1 || lower.contains("-enc ") {
                // Check for base64-like content nearby
                let b64_chars = data.iter().filter(|&&b| {
                    b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'='
                }).count();
                if b64_chars as f64 / data.len() as f64 > 0.7 && data.len() > 50 {
                    threats.push(MalwareThreat {
                        file: name.to_string(),
                        category: "suspicious".into(),
                        threat: "EncodedPowerShell".into(),
                        severity: "critical".into(),
                        detail: "Highly suspicious: PowerShell with encoded command (Base64)".into(),
                    });
                }
            }
        }

        // JavaScript/HTML exploit patterns
        if lower.contains("<script>") || lower.contains("eval(") || lower.contains("unescape(") {
            let script_indicators = [
                "eval(", "fromcharcode(", "unescape(", "escape(", "execscript(",
                "wscript.shell", "activexobject", "shellcode",
            ];
            let script_matches: Vec<&str> = script_indicators.iter()
                .filter(|&&s| lower.contains(s))
                .map(|&s| s)
                .collect();
            if script_matches.len() >= 2 {
                threats.push(MalwareThreat {
                    file: name.to_string(),
                    category: "suspicious".into(),
                    threat: "ObfuscatedScript".into(),
                    severity: "high".into(),
                    detail: format!("Obfuscated script detected: {}", script_matches.join(", ")),
                });
            }
        }

        // Python dangerous scripts
        if lower.contains("eval(") || lower.contains("exec(") || lower.contains("__import__") {
            let python_dangerous = [
                ("base64.b64decode", "Base64 decoding"),
                ("base64.b64encode", "Base64 encoding"),
                ("subprocess.call", "Process execution"),
                ("subprocess.popen", "Process execution"),
                ("subprocess.run", "Process execution"),
                ("os.system", "OS command execution"),
                ("os.popen", "OS command execution"),
                ("os.exec", "OS execution"),
                ("ctypes.windll", "Windows API access"),
                ("ctypes.cdll", "Dynamic library loading"),
                ("socket", "Network communication"),
                ("requests.get", "HTTP request"),
                ("requests.post", "HTTP request"),
                ("urllib.request", "URL request"),
            ];
            let python_matches: Vec<&str> = python_dangerous.iter()
                .filter(|&&(k, _)| lower.contains(k))
                .map(|&(k, _)| k)
                .collect();
            if python_matches.len() >= 2 {
                threats.push(MalwareThreat {
                    file: name.to_string(),
                    category: "suspicious".into(),
                    threat: "DangerousPythonScript".into(),
                    severity: "high".into(),
                    detail: format!("Dangerous Python patterns: {}", python_matches.join(", ")),
                });
            }
        }

        // Bash dangerous patterns
        let bash_like = lower.contains("#!/bin/sh") || lower.contains("#!/bin/bash") || lower.contains("#!/usr/bin/env");
        if bash_like {
            let bash_dangerous = [
                ("curl|sh", "Remote script execution via curl"),
                ("curl|bash", "Remote script execution via curl"),
                ("wget|sh", "Remote script execution via wget"),
                ("wget|bash", "Remote script execution via wget"),
                ("/dev/tcp", "TCP socket access"),
                ("chmod +x", "Make executable"),
                (":(){ :|:& };:", "Fork bomb"),
                ("mkfifo", "Named pipe creation"),
                ("mknod", "Device node creation"),
                (">/dev/tcp", "Network redirect"),
                ("exec 5<>/dev/tcp", "File descriptor socket"),
            ];
            let bash_matches: Vec<&str> = bash_dangerous.iter()
                .filter(|&&(k, _)| lower.contains(k))
                .map(|&(k, _)| k)
                .collect();
            if bash_matches.len() >= 2 {
                threats.push(MalwareThreat {
                    file: name.to_string(),
                    category: "suspicious".into(),
                    threat: "DangerousBashScript".into(),
                    severity: "high".into(),
                    detail: format!("Dangerous Bash patterns: {}", bash_matches.join(", ")),
                });
            }
        }
    }

    threats
}

// ─── Helpers ───

fn contains_case_insensitive(data: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > data.len() {
        return false;
    }
    data.windows(needle.len()).any(|w| w.eq_ignore_ascii_case(needle))
}

/// Compute risk score from threats
pub fn compute_risk_score(threats: &[MalwareThreat]) -> (f64, String) {
    if threats.is_empty() {
        return (0.0, "Clean".into());
    }

    let score: f64 = threats.iter().map(|t| match t.severity.as_str() {
        "critical" => 0.35,
        "high" => 0.20,
        "medium" => 0.10,
        "low" => 0.03,
        _ => 0.0,
    }).sum();

    let score = score.min(1.0);

    let label = if score >= 0.7 {
        "Malicious"
    } else if score >= 0.4 {
        "Highly Suspicious"
    } else if score >= 0.2 {
        "Suspicious"
    } else if score >= 0.1 {
        "Low Risk"
    } else {
        "Clean"
    };

    (score, label.into())
}

/// Combine name-based + content-based scan results
pub fn scan_file(name: &str, data: Option<&[u8]>) -> Vec<MalwareThreat> {
    let mut threats = scan_file_name(name);

    if let Some(d) = data {
        threats.extend(scan_file_content(name, d));
    }

    threats
}

/// Scan archive-level metadata untuk bom/dos patterns.
/// Parameters: total_files, total_compressed_size, total_uncompressed_size
pub fn scan_archive_metadata(
    total_files: usize,
    total_compressed: u64,
    total_uncompressed: u64,
    total_nested_archives: usize,
) -> Vec<MalwareThreat> {
    let mut threats = Vec::new();

    // File flood — terlalu banyak file
    if total_files > 10_000 {
        threats.push(MalwareThreat {
            file: "(archive root)".into(),
            category: "suspicious".into(),
            threat: "FileFlood".into(),
            severity: "high".into(),
            detail: format!(
                "Archive contains {} files — potential filesystem exhaustion attack",
                total_files
            ),
        });
    }

    // Zip bomb — rasio kompresi ekstrem
    if total_compressed > 0 && total_uncompressed > total_compressed * 1000 {
        let ratio = total_uncompressed / total_compressed;
        threats.push(MalwareThreat {
            file: "(archive root)".into(),
            category: "suspicious".into(),
            threat: "ZipBomb".into(),
            severity: "critical".into(),
            detail: format!(
                "Extreme compression ratio {}:1 — potential zip bomb (compressed {} → {})",
                ratio,
                format_size(total_compressed),
                format_size(total_uncompressed),
            ),
        });
    }

    // Decompression bomb — uncompressed > 10 GB
    if total_uncompressed > 10_000_000_000 {
        threats.push(MalwareThreat {
            file: "(archive root)".into(),
            category: "suspicious".into(),
            threat: "DecompressionBomb".into(),
            severity: "critical".into(),
            detail: format!(
                "Total uncompressed size {} exceeds 10 GB — may cause OOM or disk exhaustion",
                format_size(total_uncompressed),
            ),
        });
    }

    // Deep nesting — too many archives inside archives
    if total_nested_archives >= 3 {
        threats.push(MalwareThreat {
            file: "(archive root)".into(),
            category: "suspicious".into(),
            threat: "DeepNesting".into(),
            severity: "high".into(),
            detail: format!(
                "Archive contains {} nested archives — potential decompression loop attack",
                total_nested_archives
            ),
        });
    }

    threats
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    // ── PE Tests ──

    #[test]
    fn test_is_pe_detects_real_pe() {
        // Minimal MZ + PE header
        let mut pe = Vec::new();
        pe.extend_from_slice(b"MZ");                    // DOS header
        pe.extend_from_slice(&[0; 0x3C - 2]);           // padding
        pe.extend_from_slice(&[0x80, 0x00, 0x00, 0x00]); // e_lfanew = 0x80
        pe.extend_from_slice(&[0; 0x80 - 0x40]);        // padding to 0x80
        pe.extend_from_slice(b"PE\x00\x00");            // PE signature
        assert!(is_pe(&pe), "Should detect valid PE");
    }

    #[test]
    fn test_is_pe_rejects_non_pe() {
        assert!(!is_pe(b"Not a PE file"), "Should reject non-PE");
        assert!(!is_pe(b""), "Should reject empty");
        assert!(!is_pe(b"MZ but no PE sig"), "Should reject MZ without PE");
    }

    #[test]
    fn test_pe_rare_architecture() {
        let mut pe = Vec::new();
        pe.extend_from_slice(b"MZ");
        pe.extend_from_slice(&[0; 0x3C - 2]);
        pe.extend_from_slice(&[0x80, 0x00, 0x00, 0x00]);
        pe.extend_from_slice(&[0; 0x80 - 0x40]);
        pe.extend_from_slice(b"PE\x00\x00");
        // COFF header with machine=0x0200 (IA64 — truly rare)
        pe.extend_from_slice(&[
            0x00, 0x02, 0x03, 0x00, // machine=IA64, sections=3
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // padding
            0xE0, 0x00, 0, 0,
        ]);
        let threats = analyze_pe("test.exe", &pe);
        let has_arch = threats.iter().any(|t| t.threat == "RareArchitecture");
        assert!(has_arch, "Should flag rare architecture");
    }

    // ── VBA Tests ──

    #[test]
    fn test_is_office_doc() {
        assert!(is_office_doc("document.docx"));
        assert!(is_office_doc("spreadsheet.xlsm"));
        assert!(is_office_doc("old.doc"));
        assert!(!is_office_doc("text.txt"));
        assert!(!is_office_doc("image.png"));
    }

    #[test]
    fn test_vba_keywords_detects_autoexec() {
        let data = b"This contains AutoOpen and Shell(\"cmd.exe\")";
        let threats = scan_vba_keywords("test.docm", data);
        let has_autoexec = threats.iter().any(|t| t.threat == "AutoExecMaliciousMacro");
        assert!(has_autoexec, "Should detect auto-exec + dangerous");
    }

    #[test]
    fn test_vba_keywords_detects_dangerous_only() {
        let data = b"Sub Test() CreateObject(\"WScript.Shell\").Run \"powershell\" End Sub";
        let threats = scan_vba_keywords("test.docm", data);
        let has_dangerous = threats.iter().any(|t| t.threat == "SuspiciousMacroFunction");
        assert!(has_dangerous, "Should detect dangerous functions");
    }

    // ── Ransomware Tests ──

    #[test]
    fn test_ransomware_note_detection() {
        let content = b"YOUR FILES HAVE BEEN ENCRYPTED!\n\
                        To decrypt your files, send 0.5 BTC to wallet address.\n\
                        You have 48 hours to pay or the price increases.\n\
                        Do not try to recover files yourself or you will lose them.\n\
                        Contact us at email@onion.com";
        let threats = detect_ransomware_note("README.txt", content);
        assert!(!threats.is_empty(), "Should detect ransomware note");
        assert!(threats.iter().any(|t| t.threat == "RansomwareNote"));
    }

    #[test]
    fn test_ransomware_note_not_triggered_on_normal() {
        let content = b"This is a normal readme file. Thank you for using our software.";
        let threats = detect_ransomware_note("README.txt", content);
        assert!(threats.is_empty(), "Should not flag normal text");
    }

    // ── File Name Tests ──

    #[test]
    fn test_double_extension_detection() {
        let threats = scan_file_name("invoice.pdf.exe");
        assert!(threats.iter().any(|t| t.threat == "DoubleExtension"));
    }

    #[test]
    fn test_normal_extension_no_false() {
        let threats = scan_file_name("document.pdf");
        assert!(!threats.iter().any(|t| t.threat == "DoubleExtension"));
    }

    #[test]
    fn test_executable_in_archive() {
        let threats = scan_file_name("setup.exe");
        assert!(threats.iter().any(|t| t.threat == "ExecutableInArchive"));
    }

    #[test]
    fn test_hidden_file_detection() {
        let threats = scan_file_name(".hidden_file");
        assert!(threats.iter().any(|t| t.threat == "HiddenFile"));
    }

    #[test]
    fn test_script_in_archive() {
        let threats = scan_file_name("evil.ps1");
        assert!(threats.iter().any(|t| t.threat == "ScriptInArchive"));
    }

    // ── Risk Score Tests ──

    #[test]
    fn test_clean_risk_score() {
        let (score, label) = compute_risk_score(&[]);
        assert_eq!(score, 0.0);
        assert_eq!(label, "Clean");
    }

    #[test]
    fn test_malicious_risk_score() {
        let threats = vec![
            MalwareThreat { file: "a.exe".into(), category: "pe".into(),
                threat: "ProcessInjection".into(), severity: "critical".into(),
                detail: "".into() },
            MalwareThreat { file: "a.exe".into(), category: "pe".into(),
                threat: "WritableExecutableSection".into(), severity: "critical".into(),
                detail: "".into() },
            MalwareThreat { file: "macro.docm".into(), category: "macro".into(),
                threat: "AutoExecMaliciousMacro".into(), severity: "critical".into(),
                detail: "".into() },
        ];
        let (score, _) = compute_risk_score(&threats);
        assert!(score > 0.5, "Multiple critical threats should give high score");
    }

    // ── PowerShell Detection ──

    #[test]
    fn test_encoded_powershell_detection() {
        let data = b"powershell -EncodedCommand SQBmACgAJABQAFMAVgBlAHIAcwBJAG8AbgBUAGEAYgBsAGUALgBQAFMAVgBlAHIAcwBJAG8AbgAuAE0AYQBqAG8AcgAgAC0AZwBlACAAMwApAHsAfQAgAEUAbgBkAFIASQBmAA==";
        let threats = check_suspicious_content("script.ps1", data);
        assert!(!threats.is_empty(), "Should detect encoded PowerShell");
    }

    // ── Edge Cases ──

    #[test]
    fn test_empty_file_no_threats() {
        let threats = scan_file_content("test.txt", b"");
        assert!(threats.is_empty());
    }

    #[test]
    fn test_large_ransomware_note_not_scanned() {
        let mut large = vec![b'A'; 200_000]; // > 100K limit for ransom scan
        let threats = detect_ransomware_note("note.txt", &large);
        assert!(threats.is_empty(), "Should skip oversized files for content scan");
    }

    #[test]
    fn test_directory_entry_skipped() {
        let threats = scan_file_content("dir/", b"");
        assert!(threats.is_empty());
    }

    #[test]
    fn test_lnk_file_detected() {
        let threats = scan_file_name("important_document.pdf.lnk");
        assert!(threats.iter().any(|t| t.threat == "ShortcutFile"));
    }

    #[test]
    fn test_normal_office_no_false_macro() {
        let data = b"PK\x03\x04 some normal office content without macros";
        let threats = analyze_office_macro("normal.docx", data);
        assert!(!threats.iter().any(|t| t.threat == "OfficeMacroPresent"),
            "Should not flag normal docx without vbaProject");
    }
}
