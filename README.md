# ZipLoom 📦

[![Build](https://github.com/YSF-Studio/ziploom/actions/workflows/build.yml/badge.svg)](https://github.com/YSF-Studio/ziploom/actions)
[![Audit](https://github.com/YSF-Studio/ziploom/actions/workflows/audit.yml/badge.svg)](https://github.com/YSF-Studio/ziploom/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

> Archive compression, extraction & inspection tool — 100% offline, built with **Tauri v2 + Rust + SvelteKit**.

## Screenshots

| Compress | Extract |
|:--------:|:-------:|
| ![Compress](screenshots/compress.png) | ![Extract](screenshots/extract.png) |

| Inspect | About | Encrypt |
|:-------:|:-----:|:-------:|
| ![Inspect](screenshots/inspect.png) | ![About](screenshots/about.png) | ![Encrypt](screenshots/encrypt.png) |

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

Sample files are included in the [`samples/`](samples/) directory for testing.

## 🚀 Quick Start

```bash
git clone https://github.com/YSF-Studio/ziploom.git
cd ziploom
npm install
npm run tauri dev
```

Or download the latest release from the [Releases](https://github.com/YSF-Studio/ziploom/releases) page.

## 🏗️ Tech Stack

- **Backend:** Rust with Tauri v2
- **Frontend:** SvelteKit 5
- **Core library:** `ysf-core` (vendored in `src-tauri/crates/ysf-core/`)

## 📦 Shared Core

This repo includes a **local copy** of `ysf-core` under `src-tauri/crates/ysf-core/`. It is duplicated (not published as a separate crate) so ZipLoom builds standalone without external dependencies.

The same core is also embedded in [CollectionLoom](https://github.com/YSF-Studio/collectionloom) and [AnalysisLoom](https://github.com/YSF-Studio/analysisloom). When updating forensic logic, apply changes here first or sync manually across repos.

## 🧪 Tests

```bash
cargo test --manifest-path src-tauri/crates/ysf-core/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
npm run build
```

## 📄 License

MIT © YSF Studio
