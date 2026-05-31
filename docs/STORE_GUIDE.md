# Store Distribution Guide

This guide covers how to distribute ZipLoom through different stores and platforms.

## 📦 Build for Each Platform

### Linux (Current)
```bash
cd src-tauri
cargo tauri build
# Output:
#   bundle/appimage/ZipLoom_1.0.0_amd64.AppImage
#   bundle/deb/ZipLoom_1.0.0_amd64.deb
```

### macOS (Future — on Master's MacBook)
```bash
# Prerequisites: Xcode, Apple Developer account
cd src-tauri
cargo tauri build --bundles dmg
# Output: bundle/dmg/ZipLoom_1.0.0_x64.dmg

# For App Store:
cargo tauri build --bundles mas
# Output: bundle/mas/ZipLoom.pkg
```

**macOS Signing Checklist:**
- [ ] Apple Developer account ($99/yr)
- [ ] Code signing certificate in Keychain
- [ ] Notarization (auto via `cargo tauri build`)
- [ ] Sandbox entitlements (if App Store)

### Windows (Future — on Master's Windows)
```bash
cd src-tauri
cargo tauri build --bundles msi
# Output: bundle/msi/ZipLoom_1.0.0_x64.msi
```

**Windows Signing Checklist:**
- [ ] Microsoft Partner account
- [ ] Code signing certificate ($200–300/yr)
- [ ] Windows Defender SmartScreen review

## 🏪 Store Distribution

### Lynkid / Itch.io (Easiest — $0 upfront)
1. Create seller account
2. Create product with screenshots + description
3. Upload binary file
4. Set price $1.99
5. Share purchase link

**Files to upload:**
```
ZipLoom_1.0.0_amd64.AppImage   (Linux portable)
ZipLoom_1.0.0_amd64.deb        (Debian/Ubuntu)
```

### GitHub Releases (Free)
1. Tag a release: `git tag v1.0.0 && git push --tags`
2. Attach binaries to the release
3. Users pay via Lynkid link in README

### macOS App Store
1. Register on Apple Developer ($99/yr)
2. Create app entry in App Store Connect
3. Build with `--bundles mas`
4. Submit for review (3-7 days)

### Microsoft Store
1. Register as Microsoft Partner (~$200 one-time)
2. Package as MSIX
3. Submit for certification

## ✅ Quality Checklist Before Store Submission

- [ ] All 126+ tests pass
- [ ] Zero compiler warnings
- [ ] App icon renders correctly
- [ ] Legal disclaimer visible in About tab
- [ ] Version number matches `Cargo.toml` and `tauri.conf.json`
- [ ] No network calls at startup (confirmed offline)
- [ ] Clean uninstall (no leftover files)
- [ ] Crash-free on basic operations (compress small + large, extract password-protected, forensic inspect)
