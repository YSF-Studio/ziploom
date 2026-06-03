# ZipLoom 📦

[![Build](https://github.com/YSF-Studio/ziploom/actions/workflows/build.yml/badge.svg)](https://github.com/YSF-Studio/ziploom/actions)
[![Audit](https://github.com/YSF-Studio/ziploom/actions/workflows/audit.yml/badge.svg)](https://github.com/YSF-Studio/ziploom/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

> Archive compression, extraction & inspection tool — 100% offline, built with **Tauri v2 + Rust + SvelteKit**.

## Screenshots

| Compress | Extract |
|:--------:|:-------:|
| ![Compress](https://raw.githubusercontent.com/YSF-Studio/ziploom/main/screenshots/compress.png) | ![Extract](https://raw.githubusercontent.com/YSF-Studio/ziploom/main/screenshots/extract.png) |

| Inspect | About |
|:-------:|:-----:|
| ![Inspect](https://raw.githubusercontent.com/YSF-Studio/ziploom/main/screenshots/inspect.png) | ![About](https://raw.githubusercontent.com/YSF-Studio/ziploom/main/screenshots/about.png) |

## ✨ Features

| Feature | Details |
|---------|---------|
| **Compress** | ZIP, TAR, TAR.GZ with drag-and-drop |
| **Extract** | Multi-format extraction (ZIP, TAR, GZ, BZ2, XZ, RAR) |
| **Inspect** | Preview archive contents without extracting — compression ratios, metadata, tree view |
| **AES-256 Encryption** | PBKDF2 + AES-256-GCM for password-protected archives |
| **Full Scan** | Recursive scan with progress bar, ETA, and cancel support |
| **Buffer Optimization** | 256KB buffer for 4x faster processing |
| **100% Offline** | All processing runs locally — zero telemetry |

## Sample Files

Sample files are included in the [`samples/`](samples/) directory for testing:

| File | Description |
|------|-------------|
| `test_sample.zip` | Sample ZIP containing 3 forensic documents |
| `confidential_report.txt` | Confidential financial report |
| `evidence_manifest.txt` | Evidence hash manifest |
| `case_metadata.xml` | Forensic case metadata |

## 🚀 Quick Start

```bash
# Build from source
git clone https://github.com/YSF-Studio/ziploom.git
cd ziploom/packages/ziploom
npm install
npm run tauri dev
```

Or download the latest release from the [Releases](https://github.com/YSF-Studio/ziploom/releases) page.

## 🏗️ Tech Stack

- **Backend:** Rust with Tauri v2
- **Frontend:** SvelteKit 5
- **Encryption:** AES-256-GCM via Rust `aes-gcm` crate
- **Archives:** `zip`, `tar`, `flate2`, `bzip2`, `xz2`, `unrar`

## 📄 License

MIT © YSF Studio — Built with ❤️ by Yusuf Shalahuddin
