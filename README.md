# ZipLoom 📦

[![Build](https://github.com/YSF-Studio/ziploom/actions/workflows/build.yml/badge.svg)](https://github.com/YSF-Studio/ziploom/actions)
[![Audit](https://github.com/YSF-Studio/ziploom/actions/workflows/audit.yml/badge.svg)](https://github.com/YSF-Studio/ziploom/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

> Archive compression, extraction & forensic inspection — **100% offline**, built with **Tauri v2 + Rust + SvelteKit**.

## Screenshots

| Compress | Extract |
|:--------:|:-------:|
| ![Compress](screenshots/compress.png) | ![Extract](screenshots/extract.png) |

| Inspect | About | Password ZIP |
|:-------:|:-----:|:------------:|
| ![Inspect](screenshots/inspect.png) | ![About](screenshots/about.png) | ![Password ZIP](screenshots/encrypt.png) |

## ✨ Features

| Tab | Capabilities |
|-----|-------------|
| **Compress** | ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, TAR.ZST — drag & drop files/folders, optional password-protected ZIP (AES-256, 7-Zip / WinRAR compatible) |
| **Extract** | Pure Rust: ZIP, TAR, GZ, BZ2, XZ, ZST, **7z**, **RAR** — no external CLI |
| **Inspect** | Metadata load → Full Scan (per-file MD5/SHA1/SHA256, entropy, magic bytes), tree/flat view, file preview, CSV export |
| **Security** | Password-protected archives, threat/anomaly detection, read-only preview (no execution) |

### Inspect highlights

- Split layout: scrollable file table + detail/findings panel
- Progress bar for long scans, hash, and extract operations
- Preview file contents inside archives (text / hex / image, size-capped)
- Flagged-only filter, sticky columns, hash copy-on-click

## Sample & Test Files

| Path | Purpose |
|------|---------|
| [`samples/`](samples/) | Demo documents for manual testing |
| [`tests/fixtures/e2e/`](tests/fixtures/e2e/) | Automated E2E fixtures (`sample_alpha.txt`, `nested/…`) |

## 🚀 Quick Start

```bash
git clone https://github.com/YSF-Studio/ziploom.git
cd ziploom
npm install
npm run tauri:dev
```

Dev server: `http://localhost:1422`

Or download a release from [Releases](https://github.com/YSF-Studio/ziploom/releases).

## 🧪 Tests

```bash
# Backend E2E — real files (compress, inspect, extract, password ZIP)
npm run test:e2e

# Frontend GUI smoke (Playwright + mocked Tauri IPC)
npm run test:gui

# Both
npm run test:all

# Regenerate README screenshots
npm run screenshots
```

| Suite | Result |
|-------|--------|
| E2E (Rust) | 7/7 — ZIP/TAR/TAR.GZ compress→inspect→extract, password-protected ZIP |
| GUI smoke | 15/15 — all tabs, compress, extract, inspect scan/hash/export |

## 🏗️ Tech Stack

- **Shell:** Tauri v2
- **Backend:** Rust (`ysf-core` — zip, tar, sevenz-rust, unrar)
- **Frontend:** Svelte 5 + Vite
- **Forensic:** Streaming hashes (256 KB buffer), entropy, magic-byte DB

## 📦 Shared Core

`ysf-core` lives in `src-tauri/crates/ysf-core/` (vendored, standalone build). Sync forensic changes with [CollectionLoom](https://github.com/YSF-Studio/collectionloom) and [AnalysisLoom](https://github.com/YSF-Studio/analysisloom) manually.

## 📄 License

MIT © YSF Studio
