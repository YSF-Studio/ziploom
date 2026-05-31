# 🔍 ZipLoom Detection Capabilities

> **100% Offline — Zero Network, Zero DB, Zero Updates**
> All detection is heuristic-based: structural analysis, statistics, and pattern matching.
> No malware signature database required. No internet connection needed.

---

## 📊 By the Numbers

| Metric | Value |
|--------|-------|
| **Total detection types** | **51** |
| **Lines of detection code** | ~1,900 (scanner.rs) |
| **Zero dependencies** | Pure Rust — no external binaries |
| **False positive rate** | Low — all heuristics have severity tiers |

---

## 🗂️ Threat Categories

### 1. 🔴 PE Executable Analysis (Windows)

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `WritableExecutableSection` | 🔴 Critical | Section has both W+ X permissions — classic shellcode injection |
| `ProcessInjection` | 🔴 Critical | ≥3 injection APIs: VirtualAllocEx, WriteProcessMemory, CreateRemoteThread |
| `ProcessHollowing` | 🔴 Critical | ZwUnmapViewOfSection / NtUnmapViewOfSection detected |
| `Keylogger` | 🔴 Critical | GetAsyncKeyState + GetForegroundWindow + GetWindowText |
| `PackedExecutable` | 🟠 High | Section name matches UPX/VMProtect/Themida/Enigma/etc. |
| `SectionSizeAnomaly` | 🟠 High | Virtual size >> raw size (packed/overlapping sections) |
| `AntiDebugging` | 🟠 High | ≥3 anti-debug APIs: IsDebuggerPresent, NtGlobalFlag, etc. |
| `PersistenceMechanism` | 🟠 High | Registry persistence + startup folder APIs |
| `SuspiciousImport` | 🟠 High | Injection APIs present (1-2) |
| `NativeSubsystem` | 🟡 Medium | PE has Native subsystem (driver-level binary) |
| `NoDataExecutionPrevention` | 🟡 Medium | NX_COMPAT flag missing — DEP disabled |
| `RareArchitecture` | 🟢 Low | Uncommon CPU architecture in PE header |

### 2. 🔴 ELF Executable Analysis (Linux)

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `SuspiciousInterpreter` | 🟠 High | ELF interpreter points to `/tmp/` or `/dev/` |
| `SuspiciousRpath` | 🟠 High | RPATH/RUNPATH points to temporary directory (DLL hijacking) |
| `PackedExecutable` | 🟠 High | Section names match UPX/packed/themida/etc. |
| `ZeroEntryPoint` | 🟠 High | ET_EXEC with entry point at 0 — heavily packed |
| `RareELFabI` | 🟢 Low | Uncommon OS/ABI value |

### 3. 🔴 Mach-O Executable Analysis (macOS)

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `MachOExecutable` | 🟡 Medium | Magic FEEDFACE / FEEDFACF / BEBAFECA detected |

### 4. 🟠 Macro & Office Analysis

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `AutoExecMaliciousMacro` | 🔴 Critical | Auto-exec macro (AutoOpen) + dangerous VBA functions |
| `SuspiciousMacroFunction` | 🟠 High | Dangerous functions: Shell, CreateObject, PowerShell, etc. |
| `MacroNetworkAccess` | 🟠 High | Macro with network capabilities (URLDownload, WinHttp) |
| `OfficeMacroPresent` | 🟡 Medium | VBA project detected (vbaProject.bin) |
| `EmbeddedOLEObject` | 🟡 Medium | Embedded OLE objects — potential exploit delivery |
| `AutoExecMacro` | 🟡 Medium | Auto-exec macro present (no dangerous functions) |

### 5. 🟠 Script & Code Analysis

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `EncodedPowerShell` | 🔴 Critical | PowerShell -EncodedCommand with Base64 payload |
| `DangerousPythonScript` | 🟠 High | ≥2 dangerous patterns: eval, exec, subprocess, os.system, socket, ctypes |
| `DangerousBashScript` | 🟠 High | ≥2 dangerous patterns: curl|sh, wget|bash, /dev/tcp, fork bomb |
| `DangerousAHKScript` | 🟠 High | ≥3 dangerous AHK patterns: Run, DllCall, URLDownload, keystroke injection |
| `ObfuscatedScript` | 🟠 High | ≥2 obfuscation techniques: eval, fromCharCode, unescape, shellcode |
| `ScriptInArchive` | 🟡 Medium | .ps1/.vbs/.bat/.js/.ahk file inside archive |
| `ExecutableInArchive` | 🟡 Medium | .exe/.msi/.scr/.dll file inside archive |

### 6. 🟠 PDF Analysis

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `PDFwithJavaScript` | 🔴 Critical | PDF contains /JavaScript with ≥2 indicators |
| `PDFAutoOpenAction` | 🔴 Critical | /OpenAction present — auto-executes when opened |
| `PDFLaunchAction` | 🔴 Critical | /Launch action — may execute external program |
| `PDFObfuscatedJavaScript` | 🔴 Critical | JavaScript with encoded stream (FlateDecode/ASCIIHexDecode) |
| `PDFEmbeddedFile` | 🟠 High | /EmbeddedFile detected — possible malware delivery |

### 7. 🟠 Android / Java

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `JavaClassFile` | 🟡 Medium | Magic CAFEBABE — Java bytecode in archive |

### 8. 🔴 Archive-Level Threats

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `PathTraversal` | 🔴 Critical | Entry name contains `../` or `..\\` |
| `AbsolutePath` | 🔴 Critical | Entry uses absolute path (`/etc/passwd`, `C:\Windows\`) |
| `ZipBomb` | 🔴 Critical | Compression ratio > 1000:1 (e.g., 42KB → 4.5PB) |
| `DecompressionBomb` | 🔴 Critical | Total uncompressed size > 10 GB |
| `DoubleExtension` | 🟠 High | `invoice.pdf.exe` — hides executable behind innocent extension |
| `FileFlood` | 🟠 High | > 10,000 files in one archive (filesystem exhaustion) |
| `DeepNesting` | 🟠 High | ≥3 nested archives inside archive (decompression loop) |
| `SymlinkInArchive` | 🟠 High | Symlink entry pointing outside — may read sensitive files |
| `ZeroByteDropper` | 🟠 High | 0-byte file at system path (`/etc/`, `C:\Windows\`) |
| `NestedArchive` | 🟡 Medium | Archive contains another archive (ZIP/7z/RAR/GZ/BZ2/XZ/ZSTD) |
| `HiddenFile` | 🟢 Low | `.hidden_file` — may conceal malicious content |
| `ShortcutFile` | 🟢 Low | .lnk inside archive — common in malware delivery |

### 9. 🔴 Filename Anomalies

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `Typosquatting` | 🟡 Medium | `Facebok_Setup.exe`, `GoogIe_Update.msi`, etc. (30+ patterns) |
| `HomoglyphFilename` | 🟡 Medium | Cyrillic/Greek chars mimicking Latin (`goоgle` with Cyrillic 'о') |
| `RansomwareNote` | 🟡 Medium | Filename matches ransom note pattern (`README.txt`, `HOW_TO_DECRYPT`) |

### 10. 🟠 Content Anomalies

| Threat | Severity | How It Works |
|--------|----------|-------------|
| `ImageSizeAnomaly` | 🟡 Medium | Image dimensions vs file size mismatch — possible steganography |
| `HighEntropyText` | 🟡 Medium | Text file with entropy > 7.3 — possible encrypted/encoded payload |

### 11. 🟢 Risk Scoring

| Score | Label | Condition |
|-------|-------|-----------|
| 0.0 | Clean | No threats detected |
| 0.1 — 0.19 | Low Risk | Minor anomalies (hidden files, uncommon arch) |
| 0.2 — 0.39 | Suspicious | Medium threats present |
| 0.4 — 0.69 | Highly Suspicious | High + medium threats |
| ≥ 0.7 | Malicious | Multiple critical threats |

---

## 🔧 Technical Architecture

```
scan_file_content(name, data)          ← Full scan with data
├── is_pe() → analyze_pe()              PE headers + sections + imports
├── is_office_doc() → analyze_office_macro()  VBA project + dangerous functions
├── detect_ransomware_note()            Content-based ransom note signatures
├── check_suspicious_content()          Scripts, PowerShell, entropy, AHK
├── is_elf() → analyze_elf()            ELF headers + interpreter + sections
├── is_java_class()                     Java magic bytes
├── is_pdf() → scan_pdf()               PDF JS, Launch, OpenAction, Embedded
├── is_macho()                          Apple executable magic
├── check_image_anomaly()               PNG/JPG/BMP dimension vs size
└── zero-byte dropper at suspicious paths

scan_file_name(name)                   ← Name-only scan (no data)
├── Hidden files, double extension
├── Executable/script in archive
├── Path traversal, absolute path
├── Ransomware note filenames
├── Shortcut files (.lnk)
├── Typosquatting (30+ brand patterns)
├── Homoglyph Unicode (Cyrillic/Greek)
└── Symlink entries (via entry metadata)

scan_archive_metadata(files, comp, uncomp, nests) ← Archive-level
├── File flood (>10K files)
├── Zip bomb (>1000:1 ratio)
├── Decompression bomb (>10 GB)
└── Deep nesting (≥3 nested archives)
```

---

## 📋 Roadmap V2 (Planned But Not Implemented)

- **LNK binary parser** — extract target path from Windows .lnk files
- **OLE deep VBA extract** — parse VBA source from OLE compound documents
- **PDF stream decompression** — flate-decode PDF streams for JS content
- **Android APK manifest scan** — parse AndroidManifest.xml for dangerous permissions
- **ISO/VHD disk image detection** — detect disk image formats inside archives
- **Firmware image detection** — .bin/.hex firmware analysis
- **ZIP symlink detection** — parse ZIP external attributes for symlinks
- **Entropy baseline fine-tuning** — per-filetype dynamic thresholds
- **Advanced AHK keylogger detection** — deeper AutoHotKey script analysis

---

*Last updated: May 2026*
