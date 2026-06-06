# ZipLoom — Developer Documentation

Technical reference for contributors and maintainers.

## Repository layout

```
ziploom/
├── src/                    # Svelte 5 frontend
│   ├── App.svelte          # Shell, tabs, theme toggle
│   └── lib/
│       ├── components/     # CompressTab, ExtractTab, InspectTab, …
│       ├── tauri.js        # Safe invoke/open wrappers
│       ├── theme.js        # Light / dark / system
│       └── prefs.js        # localStorage preferences
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs     # Tauri IPC commands
│   │   └── lib.rs
│   ├── tests/
│   │   └── e2e_workflow_test.rs
│   └── crates/ysf-core/    # Shared forensic/archive library
├── tests/
│   ├── fixtures/e2e/       # Rust E2E fixtures
│   ├── gui-smoke.mjs       # Playwright smoke (mocked IPC)
│   └── capture-screenshots.mjs
├── docs/
│   ├── USER_GUIDE.md       # User guide (English)
│   └── DEVELOPER.md        # This file
└── screenshots/            # README assets
```

## Architecture

```
┌─────────────────────────────────────────┐
│  Svelte UI (Compress / Extract / Inspect)│
└──────────────────┬──────────────────────┘
                   │ Tauri invoke
┌──────────────────▼──────────────────────┐
│  ziploom commands.rs                    │
│  compress_zip, extract_archive, inspect…│
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│  ysf-core (archive, forensic, preview)  │
│  zip v1 · tar · sevenz-rust · unrar*    │
└─────────────────────────────────────────┘
  * unrar gated off on Windows targets
```

### Key crates

| Crate | Role |
|-------|------|
| `zip` (v2 in app, v1 in ysf-core) | Write/read ZIP, AES password |
| `tar` + `flate2` / `bzip2` / `xz2` / `zstd` | TAR variants |
| `sevenz-rust` | 7z read/extract |
| `unrar` | RAR (non-Windows only) |
| `pcap` | Network capture (Unix only, ysf-core) |

## Commands (npm)

| Script | Description |
|--------|-------------|
| `npm run tauri:dev` | Run desktop app (port 1422); auto-runs `npm run build` if `dist/` is missing |
| `npm run tauri:build` | Production installer (DMG / NSIS / deb / AppImage) |
| `npm run icons` | Regenerate `src-tauri/icons/*` from `logo.svg` |
| `npm run build` | Frontend only (Vite) |
| `npm run test:e2e` | 7 Rust integration tests |
| `npm run test:gui` | 15 Playwright UI smoke tests |
| `npm run test:all` | E2E + GUI |
| `npm run screenshots` | Regenerate `screenshots/*.png` |

## Rust tests

```bash
# App E2E (compress, inspect, extract, password ZIP)
cargo test --manifest-path src-tauri/Cargo.toml --test e2e_workflow_test --locked

# Full ysf-core suite
cargo test --manifest-path src-tauri/crates/ysf-core/Cargo.toml --locked
```

E2E temp dirs use per-test unique paths (`AtomicU64` counter) to avoid parallel CI races.

## macOS local setup

1. **Xcode Command Line Tools** — `xcode-select --install`
2. **Rust** — prefer [rustup](https://rustup.rs) over Homebrew `rust`; put `~/.cargo/bin` early in `PATH`
3. **Node 22+** — nvm: after install, `source ~/.nvm/nvm.sh` (or restart Terminal) before `nvm install 22`
4. **First dev run** — `npm run tauri:dev` ensures `dist/` exists; without it, `generate_context!()` fails with `frontendDist ... doesn't exist`
5. **Paste hygiene** — do not append `# comment` on the same line as npm/cargo commands (zsh/npm may pass junk to `cargo run`)

## Production bundles

Tauri bundling is enabled in `src-tauri/tauri.conf.json` (`bundle.active: true`). Each platform builds **installer + portable** artifacts:

| Platform | Installer target | Portable target | Packaged as |
|----------|------------------|-----------------|-------------|
| macOS | `dmg` | `app` → zipped | `*_macos_installer.dmg`, `*_macos_portable.zip` |
| Windows | `nsis` | release exe + DLLs → zipped | `*_windows_installer_x64-setup.exe`, `*_windows_portable_x64.zip` |
| Linux | `deb` | `appimage` + tar.gz from deb | `*_linux_installer_amd64.deb`, `*_linux_portable_amd64.AppImage` |

`npm run tauri:build` runs `scripts/package-releases.mjs` to copy/rename outputs into `bundle/releases/` for local use. Pre-built binaries are **not** published to GitHub Releases.

Linux release links `libc++` for `unrar_sys` — see `src-tauri/.cargo/config.toml`.

## CI workflows

| Workflow | Trigger | Jobs |
|----------|---------|------|
| `ci.yml` | push/PR `main` | Secret scan (gitleaks CLI), ysf-core tests, ZipLoom build+test |
| `build.yml` | push/PR `main` | Matrix: ubuntu / macos / windows |
| `audit.yml` | schedule + Cargo changes | `cargo audit`, SBOM |

Windows CI notes:

- `CARGO_TARGET_DIR=C:\t` — short path for native deps
- `unrar` disabled — stub RAR handlers return clear errors
- `write_blocker` / `pcap` — Unix-oriented; Windows stubs avoid `-D warnings` failures

## Tauri commands (selection)

| Command | Purpose |
|---------|---------|
| `compress_files` | Create archive (ZIP/TAR/…) |
| `extract_archive` | Full extraction |
| `inspect_archive` | Metadata listing |
| `forensic_scan_archive` | Full Scan |
| `extract_archive_entries` | Partial extract from Inspect |
| `preview_archive_entry` | In-archive preview |
| `hash_archive` | Container hash |
| `about_info` | About tab JSON |

## Syncing ysf-core

`ysf-core` is vendored under `src-tauri/crates/ysf-core/`. Forensic changes may need manual sync with [CollectionLoom](https://github.com/YSF-Studio/collectionloom) and [AnalysisLoom](https://github.com/YSF-Studio/analysisloom).

## Regenerating screenshots

```bash
npm run screenshots
git add screenshots/
```

Requires a running or buildable app context per `tests/capture-screenshots.mjs`.

## License

MIT © YSF Studio — see [LICENSE](../LICENSE).
