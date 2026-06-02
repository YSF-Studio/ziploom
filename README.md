# YSF Forensic Suite

Three professional forensic applications built with Tauri + Rust + SvelteKit.

## Apps
- **ZipLoom** — Archive inspection & threat detection
- **CollectionLoom** — Portable forensic acquisition toolkit (ISO 27037)
- **AnalysisLoom** — Installed forensic analysis workstation

## Architecture
```
ysf-forensic-suite/
├── packages/
│   ├── core/           # Shared Rust library
│   ├── ziploom/        # Archive forensics
│   ├── collectionloom/ # Acquisition toolkit (portable)
│   └── analysisloom/   # Analysis workstation (installed)
```

## Build
```bash
npm install
cd packages/core && cargo build
npm run build:collectionloom
npm run build:analysisloom
```

## Development
```bash
npm run dev:collectionloom
npm run dev:analysisloom
```

## License
YSF Studio — MIT open-core + premium binary
