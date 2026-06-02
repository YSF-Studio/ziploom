# YSF Forensic Suite

Three professional forensic applications built with **Tauri v2 + Rust + SvelteKit**.

## 🔗 Repositories

| App | Repo | Purpose |
|-----|------|---------|
| 🟢 **ZipLoom** | [YSF-Studio/ziploom](https://github.com/YSF-Studio/ziploom) | Archive inspection & threat detection |
| 🔴 **CollectionLoom** | [YSF-Studio/collectionloom](https://github.com/YSF-Studio/collectionloom) | Portable forensic acquisition (ISO 27037) |
| 🔵 **AnalysisLoom** | [YSF-Studio/analysisloom](https://github.com/YSF-Studio/analysisloom) | Forensic analysis workstation |

## 🏗️ Architecture (Monorepo Dev)

```
ysf-forensic-suite/
├── packages/
│   ├── core/           # Shared library: hashing, crypto, evidence, archive, ntfs, carving...
│   ├── ziploom/        # Archive inspection
│   ├── collectionloom/ # Portable acquisition
│   └── analysisloom/   # Analysis workstation
└── .github/workflows/  # CI/CD across all apps
```

## 🧪 Test Suite

**43 tests** across 3 layers with 3x consecutive stability verification.

```bash
cd packages/core
cargo test --test comprehensive   # 24 unit/comprehensive
cargo test --test integration     # 19 integration (real archives)
```

## 📄 License

MIT © YSF Studio
