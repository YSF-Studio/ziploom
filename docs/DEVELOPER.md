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
│   ├── PANDUAN.md          # User guide (Indonesian)
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
| `npm run tauri:dev` | Run desktop app (port 1422) |
| `npm run tauri:build` | Production installer |
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
