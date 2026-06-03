# AnalysisLoom 🔬

[![Build](https://github.com/YSF-Studio/analysisloom/actions/workflows/build.yml/badge.svg)](https://github.com/YSF-Studio/analysisloom/actions)
[![Audit](https://github.com/YSF-Studio/analysisloom/actions/workflows/audit.yml/badge.svg)](https://github.com/YSF-Studio/analysisloom/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

> Forensic analysis workstation — NTFS parsing, file carving, timeline analysis & case management, built with **Tauri v2 + Rust + SvelteKit**.

## ✨ Features

| Feature | Details |
|---------|---------|
| **NTFS/MFT Parser** | File browser with sorted deleted file recovery |
| **File Preview** | Text, image, hex (interactive with byte select), archive |
| **File Carving** | Multi-format signature-based recovery with progress |
| **Timeline Analysis** | Chronological event correlation (MACE timestamps) |
| **Keyword Search** | Regex-based search across evidence files |
| **Case Management** | SQLite-based with evidence & findings tracking |
| **Report Generation** | PDF & HTML reports with full audit trail |
| **Bookmarks & Tags** | Mark files of interest with color-coded notes |
| **Hex Viewer** | Interactive byte-level inspection with bookmarks |
| **Audit Trail** | ISO 27042-compliant action logging |

## 🖥️ Screenshots

| NTFS Browser | File Carving | Timeline |
|--------------|--------------|----------|
| _Coming soon_ | _Coming soon_ | _Coming soon_ |

| Keyword Search | Case Dashboard | Report |
|----------------|----------------|--------|
| _Coming soon_ | _Coming soon_ | _Coming soon_ |

## 🚀 Quick Start

```bash
# Build from source
git clone https://github.com/YSF-Studio/analysisloom.git
cd analysisloom/packages/analysisloom
npm install
npm run tauri dev
```

Or download the latest release from the [Releases](https://github.com/YSF-Studio/analysisloom/releases) page.

## 🏗️ Tech Stack

- **Backend:** Rust with Tauri v2
- **Frontend:** SvelteKit 5
- **NTFS:** Custom MFT parser in Rust
- **Carving:** Multi-signature matching engine
- **Storage:** SQLite via `rusqlite`
- **Reports:** PDF generation via Rust

## 📄 License

MIT © YSF Studio — Built with ❤️ by Yusuf Shalahuddin
