# CollectionLoom 📀

[![Build](https://github.com/YSF-Studio/collectionloom/actions/workflows/build.yml/badge.svg)](https://github.com/YSF-Studio/collectionloom/actions)
[![Audit](https://github.com/YSF-Studio/collectionloom/actions/workflows/audit.yml/badge.svg)](https://github.com/YSF-Studio/collectionloom/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

> Portable forensic acquisition tool — evidence collection compliant with ISO 27037, built with **Tauri v2 + Rust + SvelteKit**.

## ✨ Features

| Feature | Details |
|---------|---------|
| **Disk Imaging** | Bit-for-bit acquisition with SHA-256 verification |
| **RAM Capture** | Memory acquisition (avml, winpmem support) |
| **Mobile Triage** | Android & iOS logical acquisition |
| **Cloud Snapshots** | AWS EBS, Azure Disk, GCP Persistent Disk |
| **Network Capture** | Live packet acquisition with BPF filtering |
| **Hash Verification** | Compare SHA-256/SHA-1/MD5 against expected values |
| **Write Blocker** | Hardware & software write protection |
| **Chain of Custody** | Evidence tracking with Ed25519 signatures |
| **System Snapshot** | Point-in-time file/process/network capture |

## 🖥️ Screenshots

| Disk Imaging | RAM Capture | Cloud Snapshot |
|--------------|-------------|----------------|
| _Coming soon_ | _Coming soon_ | _Coming soon_ |

## 🚀 Quick Start

```bash
# Build from source
git clone https://github.com/YSF-Studio/collectionloom.git
cd collectionloom/packages/collectionloom
npm install
npm run tauri dev
```

Or download the latest release from the [Releases](https://github.com/YSF-Studio/collectionloom/releases) page.

## 🏗️ Tech Stack

- **Backend:** Rust with Tauri v2
- **Frontend:** SvelteKit 5
- **Hashing:** SHA-256, SHA-1, MD5 via Rust `sha2`, `sha1`, `md-5` crates
- **Cloud:** AWS SDK, Azure SDK, GCP SDK via Rust
- **Capture:** BPF, avml integration

## 📄 License

MIT © YSF Studio — Built with ❤️ by Yusuf Shalahuddin
