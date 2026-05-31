# ZipLoom — Archive Utility & Forensic Inspector

> **Pure Rust · Offline · Private — No external CLI tools, no network calls**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Tauri v2](https://img.shields.io/badge/Tauri-v2-ffc131)
![Rust](https://img.shields.io/badge/Rust-2021-dea584)

---

## 🔥 Features

### Archive Operations
| Feature | Supported |
|---------|-----------|
| **Compress** | ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, TAR.ZST |
| **Extract** | ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, TAR.ZST, **7z, RAR** |
| **AES-256 Encrypted ZIP** | ✅ Password-protected archives |
| **Split volumes** | ✅ Compress & split into chunks |
| **Compression levels** | 0–9 configurable |
| **Clean macOS junk** | Auto-strips `.DS_Store`, `__MACOSX`, `._` files |
| **Drag & drop** | ✅ Full drag-and-drop support |

### Forensic Inspector
- **Magic byte verification** — detects format mismatch / tampering
- **Entropy analysis** — flags encrypted or compressed content inside archives
- **Batch hashing** — MD5, SHA-1, SHA-256 per file
- **Anomaly detection** — high-entropy files, extension mismatch
- **Malware scoring engine** — heuristic PE, office macro, ransomware & script analysis ([architecture](docs/MALWARE_SCORING_ARCH.md))
- **Exportable CSV reports** with full evidence trail
- **File tree view** with sortable columns

### Privacy & Security
- ✅ **100% offline** — zero network calls, zero telemetry
- ✅ **No data collection** — everything stays on your machine
- ✅ **Pure Rust backend** — memory-safe, no external CLI dependencies
- ✅ **App Store safe** — no shelling out to 7z/bsdtar/unrar

---

## 📸 Screenshots

*Coming soon — see [Screenshot Guide](SCREENSHOT_GUIDE.md)*

---

## 🚀 Download

| Platform | Pre-built Binary | Build from Source |
|----------|-----------------|-------------------|
| **Linux** | [.AppImage] | `cargo build --release` |
| **Linux (Debian/Ubuntu)** | [.deb] | `cargo build --release` |
| **macOS** | *Coming soon* | `cargo build --release` |
| **Windows** | *Coming soon* | `cargo build --release` |

> **Pre-built binaries are $1.99** — you're paying for convenience, not the code.  
> [Buy on Lynkid](https://lynkid.com/...) or build for free below.

### Build from Source

```bash
git clone https://github.com/ysf-studio/ziploom.git
cd ziploom

# Install prerequisites (one-time)
# Linux: sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
#   libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

npm install
cd src-tauri && cargo build --release
```

The binary will be at `src-tauri/target/release/ziploom-tauri`.

---

## 🧪 Run Tests

```bash
cd src-tauri
cargo test
```

---

## 📜 License & Trademark

**Code:** MIT License — see [LICENSE](LICENSE)  
**Brand:** "ZipLoom", "YSF Studio" and the ZipLoom logo are trademarks of Yusuf Shalahuddin — see [TRADEMARK.md](TRADEMARK.md)

---

## 🏗️ Tech Stack

- **Frontend:** SvelteKit + Vite
- **Backend:** Rust via Tauri v2
- **Archive Engine:** Pure Rust (`zip`, `tar`, `flate2`, `bzip2`, `zstd`, `sevenz-rust`, `unrar`) — zero CLI dependencies
- **Hashing:** SHA-2, MD5, BLAKE3 (Rust native)

---

## 🙋 FAQ

**Q: Why $1.99 when the code is MIT?**  
A: You're paying for the pre-built binary — download, click, done. No need to install Rust, no compile time. The source is free forever.

**Q: Can I sell my own compiled version?**  
A: Yes — MIT license allows redistribution. But you **cannot** use the "ZipLoom" name or YSF Studio branding (see [TRADEMARK.md](TRADEMARK.md)).

**Q: Is this court-certified for digital forensics?**  
A: **No.** See the legal disclaimer in-app and in our documentation. All forensic output is informational.

---

*Built with ❤️ by [YSF Studio](https://ysfstudio.com)*
