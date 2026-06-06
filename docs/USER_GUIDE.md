# ZipLoom User Guide

ZipLoom is a **100% offline** desktop archive utility for compressing, extracting, and inspecting archives — built with Tauri, Rust, and Svelte.

> **Important:** Forensic analysis results are informational only. Independently verify findings before use in legal proceedings or formal audits.

---

## Table of contents

1. [Installation](#installation)
2. [Running the app](#running-the-app)
3. [General UI](#general-ui)
4. [Compress tab](#compress-tab)
5. [Extract tab](#extract-tab)
6. [Inspect tab](#inspect-tab)
7. [About tab](#about-tab)
8. [Password-protected archives](#password-protected-archives)
9. [Drag & drop](#drag--drop)
10. [Format support](#format-support)
11. [Troubleshooting](#troubleshooting)

---

## Installation

### Option A — Download a release

1. Open [Releases](https://github.com/YSF-Studio/ziploom/releases).
2. Pick **installer** or **portable** for your OS:

| Platform | Installer (system-wide) | Portable (no install) |
|----------|-------------------------|-------------------------|
| **macOS** | `*_macos_installer.dmg` — drag to Applications | `*_macos_portable.zip` — unzip, double-click `ZipLoom.app` |
| **Windows** | `*_windows_installer_x64-setup.exe` — run setup wizard | `*_windows_portable_x64.zip` — unzip, run `ziploom.exe` |
| **Linux** | `*_linux_installer_amd64.deb` — `sudo dpkg -i …` | `*_linux_portable_amd64.AppImage` (recommended) or `*_linux_portable_amd64.tar.gz` |

3. Install or extract using your platform’s usual steps.

### Option B — Build from source

**Prerequisites**

| Platform | Main dependencies |
|----------|-------------------|
| **macOS** | Xcode Command Line Tools, Node.js 22+, Rust stable |
| **Linux** | `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`, `libssl-dev`, `libpcap-dev` |
| **Windows** | Visual Studio Build Tools (C++), Node.js 22+, Rust stable |

```bash
git clone https://github.com/YSF-Studio/ziploom.git
cd ziploom
npm install
npm run tauri:dev      # development
npm run tauri:build    # production installer
```

After `npm run tauri:build`, **installer + portable** packages are collected in `src-tauri/target/release/bundle/releases/`:

| Platform | Installer | Portable |
|----------|-----------|----------|
| **macOS** | `ZipLoom_*_macos_installer.dmg` | `ZipLoom_*_macos_portable.zip` |
| **Windows** | `ZipLoom_*_windows_installer_x64-setup.exe` | `ZipLoom_*_windows_portable_x64.zip` |
| **Linux** | `ZipLoom_*_linux_installer_amd64.deb` | `ZipLoom_*_linux_portable_amd64.AppImage` + `.tar.gz` |

> Do **not** use `npm run dev` alone — archive features require the Tauri app (`npm run tauri:dev`).

---

## Running the app

```bash
npm run tauri:dev
```

Vite dev server: `http://localhost:1422` (UI debugging only; archive operations run in the Tauri process).

---

## General UI

| Area | Purpose |
|------|---------|
| **Traffic lights** (top left) | Close / minimize / maximize window |
| **Tab bar** | Compress · Extract · Inspect · About |
| **Theme toggle** (top right) | Click to cycle **Light mode** → **Dark mode** → **System default** |
| **Status bar** (bottom) | Process status & **Offline** badge |
| **Toast** | Short success / error notifications |

There is no separate Settings tab — theme is controlled from the labeled button in the titlebar.

---

## Compress tab

Compress files and folders into archive formats.

### Steps

1. Open **Compress**.
2. Add sources:
   - **Browse files** — one or more files
   - **Browse folder** — entire folder
   - **Drag & drop** files/folders onto the dropzone
3. Configure options:
   - **Format** — ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, TAR.ZST
   - **Password** — **ZIP only** (AES-256, compatible with 7-Zip / WinRAR)
   - **Clean macOS metadata** — skip `.DS_Store` and `__MACOSX`
   - **Compress slider** — level from Fast → Best
4. Click **Compress**.
5. Choose save location and filename in the **Save** dialog.
6. Results appear below the button (file count & output path).

### Tips

- Passwords are supported **only** for ZIP.
- macOS junk files are filtered when clean metadata is enabled.
- Remove sources with the **×** on each chip.

---

## Extract tab

Extract archive contents to a folder of your choice.

### Steps

1. Open **Extract**.
2. Select an archive:
   - **Choose archive** / click the dropzone
   - Drag & drop an archive file
3. (Optional) check **Remove __MACOSX/ and .DS_Store**.
4. Click **Extract**.
5. Choose the destination folder.
6. Enter a password when prompted for encrypted archives.

### Supported formats

ZIP, TAR, TAR.GZ, TAR.BZ2, TAR.XZ, TAR.ZST, 7z, RAR *(see [platform notes](#format-support))*.

---

## Inspect tab

Inspect archive contents without extracting everything — useful for quick audits and light forensic review.

### Workflow

```
Select archive → Load metadata → (optional) Full Scan → Preview / Export / Extract selected
```

### Basic steps

1. Open **Inspect**.
2. Select an archive (browse or drag & drop).
3. Click **Load** — lists files, sizes, and basic metadata.
4. For password-protected archives, enter the password when prompted.

### Advanced actions

| Button | Purpose |
|--------|---------|
| **Full Scan** | Per-file MD5/SHA1/SHA256, entropy, magic bytes, threat/anomaly detection |
| **Hash All** | Hash the archive file (container) as a whole |
| **Export CSV** | Export a report to CSV |
| **Extract Selected** | Extract only checked files |

### Panel & filters

- **Tree / Flat** — hierarchical or flat listing
- **Search** — filter by path name
- **Flagged only** — show flagged entries only
- **Columns ▾** — toggle Hash, Entropy, Magic, Modified columns
- **Detail panel** — text/hex/image preview (size-limited), risk summary, threats/anomalies tabs

### File preview

Click a row to load a read-only preview in the right panel (no execution).

---

## About tab

App information, feature list, legal disclaimer, and link to [ysfloom.com](https://ysfloom.com).

---

## Password-protected archives

| Operation | Password ZIP |
|-----------|--------------|
| **Compress** | Enable **Password**, enter passphrase, use ZIP format |
| **Extract** | Password dialog appears automatically |
| **Inspect** | Password required for Load / Full Scan |

ZIP encryption uses standard **AES-256** — openable in 7-Zip, WinRAR, and ZipLoom.

> Standalone `.aes256` file encryption exists in the backend for internal/testing use but is **not** exposed in the main UI.

---

## Drag & drop

| Active tab | Drop behavior |
|------------|---------------|
| **Compress** | Adds files/folders to the source queue |
| **Extract** | Sets the first dropped archive path |
| **Inspect** | Loads the dropped archive for inspection |

---

## Format support

| Format | Compress | Extract | Inspect |
|--------|:--------:|:-------:|:-------:|
| ZIP | ✅ | ✅ | ✅ |
| ZIP + password | ✅ | ✅ | ✅ |
| TAR | ✅ | ✅ | ✅ |
| TAR.GZ / .tgz | ✅ | ✅ | ✅ |
| TAR.BZ2 / .tbz2 | ✅ | ✅ | ✅ |
| TAR.XZ / .txz | ✅ | ✅ | ✅ |
| TAR.ZST / .tzst | ✅ | ✅ | ✅ |
| 7z | — | ✅ | ✅ |
| RAR | — | ✅* | ✅* |

\* **RAR is not supported on Windows** (native `unrar` build limitation). macOS and Linux support RAR extract & inspect.

---

## Troubleshooting

| Symptom | Solution |
|---------|----------|
| `frontendDist` / `../dist` doesn't exist | Run `npm run build` once, or use `npm run tauri:dev` (auto-builds `dist/` on first run) |
| `invoke` / features fail in browser | Run `npm run tauri:dev`, not `npm run dev` |
| `zsh: command not found: nvm` | Load nvm: `source ~/.nvm/nvm.sh`, then `nvm install 22 && nvm use 22` |
| Rust install blocked by Homebrew Rust | Answer `y` to rustup, or `brew uninstall rust` first; ensure `~/.cargo/bin` is before `/opt/homebrew/bin` in `PATH` |
| Password rejected | Use ZIP format; check password case |
| RAR fails on Windows | Use 7z/ZIP, or extract on macOS/Linux |
| Full Scan is slow | Expected on large archives; watch the progress bar |
| Linux build fails | Install `libwebkit2gtk-4.1-dev` and GTK deps (see README) |
| macOS: tombol close/minimize tidak jalan / jendela tidak bisa digeser | `git pull` versi terbaru — macOS memakai traffic light native + `data-tauri-drag-region` |
| macOS: DMG tidak bisa dibuka (“damaged” / Gatekeeper) | Klik kanan app → **Open**, atau jalankan: `xattr -cr /path/to/ZipLoom.app` |
| Build installer gagal / bukan file `.dmg`/`.exe`/`.deb` | Pakai `npm run tauri:build` (bukan `cargo build` saja). Cek folder `src-tauri/target/release/bundle/` — jika kosong, pastikan `bundle.active: true` di `tauri.conf.json` dan ikon ada di `src-tauri/icons/` (`npm run icons` dari `logo.svg`) |
| Build installer gagal | Jalankan `npm run tauri:build` (bukan hanya `tauri:dev`); butuh waktu ±10–20 menit pertama kali |

> **Tip:** Run each terminal command on its own line. Do not paste inline `# comments` — zsh may treat them as errors, and npm can forward stray text to `cargo`.

### Sample files for manual testing

- `samples/` — demo documents
- `tests/fixtures/e2e/` — automated fixtures (`sample_alpha.txt`, `nested/sample_gamma.txt`, etc.)

---

## Privacy

ZipLoom **does not send data over the internet**. No telemetry, analytics, or network calls for core features.

---

**© 2026 YSF Studio** · [GitHub](https://github.com/YSF-Studio/ziploom) · [ysfloom.com](https://ysfloom.com)
