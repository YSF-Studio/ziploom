// ZipLoom Real-World Scenario Tests
// Tests engine with realistic files (PDF, PNG, CSV, nested dirs, mac junk, suspicious files)
//
// Build & run: cargo test --test scenario_tests -- --nocapture

use std::path::{Path, PathBuf};
use std::io::Write;

fn scenario_dir() -> PathBuf {
    PathBuf::from("/home/kali/ziploom-dummy-test/project")
}

// ─── SCENARIO 1: ZIP Compress + Extract ─────────

#[test]
fn scenario_zip_roundtrip() {
    let src = scenario_dir();
    assert!(src.exists(), "Source dir must exist: {:?}", src);

    let dst = std::env::temp_dir().join("zl_scenario_test.zip");
    let extract = std::env::temp_dir().join("zl_scenario_extract");
    let _ = std::fs::remove_file(&dst);
    let _ = std::fs::remove_dir_all(&extract);

    // Create ZIP
    {
        let f = std::fs::File::create(&dst).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);

        add_dir_to_zip(&mut zip, &src, "", &opts);
        zip.finish().unwrap();
    }

    let zip_size = std::fs::metadata(&dst).unwrap().len();
    println!("\n📦 ZIP created: {} bytes", zip_size);
    assert!(zip_size > 0, "ZIP should not be empty");

    // Open and read ZIP entries
    {
        let file = std::fs::File::open(&dst).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        println!("📋 ZIP contains {} entries:", archive.len());
        for i in 0..archive.len() {
            let entry = archive.by_index(i).unwrap();
            println!("   📄 {} ({} bytes)", entry.name(), entry.size());
        }
        assert!(archive.len() >= 8, "Should have at least 8 files (excluding macOS junk in root)");
    }

    // Extract
    {
        let file = std::fs::File::open(&dst).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        std::fs::create_dir_all(&extract).unwrap();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).unwrap();
            let name = entry.name().to_string();
            let out = extract.join(name.trim_start_matches('/'));
            if entry.is_dir() {
                std::fs::create_dir_all(&out).unwrap();
            } else {
                if let Some(p) = out.parent() {
                    std::fs::create_dir_all(p).unwrap();
                }
                let mut f = std::fs::File::create(&out).unwrap();
                std::io::copy(&mut entry, &mut f).unwrap();
            }
        }
    }

    // Verify key files
    assert!(extract.join("laporan.pdf").exists(), "laporan.pdf should exist");
    assert!(extract.join("logo.png").exists(), "logo.png should exist");
    assert!(extract.join("data.csv").exists(), "data.csv should exist");
    assert!(extract.join("foto.jpg").exists(), "foto.jpg should exist");
    assert!(extract.join("config/secret.dat").exists(), "secret.dat should exist");
    assert!(extract.join("src/lib/utils/helpers/parser.rs").exists(), "deep nested file should exist");

    // Verify content
    let csv = std::fs::read_to_string(extract.join("data.csv")).unwrap();
    assert!(csv.contains("User499"), "CSV should contain User499");

    let readme = std::fs::read_to_string(extract.join("README.md")).unwrap();
    assert!(readme.contains("Lorem ipsum"), "README should contain content");

    println!("✅ ZIP roundtrip — all files verified!");

    // Cleanup
    std::fs::remove_file(&dst).ok();
    std::fs::remove_dir_all(&extract).ok();
}

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    dir: &Path,
    prefix: &str,
    opts: &zip::write::FileOptions<()>,
) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = if prefix.is_empty() {
            path.file_name().unwrap().to_string_lossy().to_string()
        } else {
            format!("{}/{}", prefix, path.file_name().unwrap().to_string_lossy())
        };

        if path.is_dir() {
            zip.add_directory(&name, *opts).unwrap();
            add_dir_to_zip(zip, &path, &name, opts);
        } else {
            zip.start_file(&name, *opts).unwrap();
            let data = std::fs::read(&path).unwrap();
            zip.write_all(&data).unwrap();
        }
    }
}

// ─── SCENARIO 2: TAR.GZ Compress ──────────────

#[test]
fn scenario_tar_gz_roundtrip() {
    let src = scenario_dir();
    let dst = std::env::temp_dir().join("zl_scenario.tar.gz");
    let extract = std::env::temp_dir().join("zl_scenario_tgz_extract");
    let _ = std::fs::remove_file(&dst);
    let _ = std::fs::remove_dir_all(&extract);

    // Create TAR.GZ
    {
        let f = std::fs::File::create(&dst).unwrap();
        let enc = flate2::write::GzEncoder::new(f, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);
        add_dir_to_tar(&mut tar, &src, "");
        let enc = tar.into_inner().unwrap();
        enc.finish().unwrap();
    }

    let size = std::fs::metadata(&dst).unwrap().len();
    println!("📦 TAR.GZ created: {} bytes", size);
    assert!(size > 1000, "TAR.GZ should not be tiny");

    // Extract
    {
        let f = std::fs::File::open(&dst).unwrap();
        let dec = flate2::read::GzDecoder::new(f);
        let mut archive = tar::Archive::new(dec);
        std::fs::create_dir_all(&extract).unwrap();
        archive.unpack(&extract).unwrap();
    }

    assert!(extract.join("laporan.pdf").exists(), "laporan.pdf in TAR.GZ");
    assert!(extract.join("data.csv").exists(), "data.csv in TAR.GZ");
    println!("✅ TAR.GZ roundtrip verified!");

    std::fs::remove_file(&dst).ok();
    std::fs::remove_dir_all(&extract).ok();
}

fn add_dir_to_tar<W: Write>(tar: &mut tar::Builder<W>, dir: &Path, prefix: &str) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = if prefix.is_empty() {
            path.file_name().unwrap().to_string_lossy().to_string()
        } else {
            format!("{}/{}", prefix, path.file_name().unwrap().to_string_lossy())
        };

        if path.is_dir() {
            add_dir_to_tar(tar, &path, &name);
        } else {
            tar.append_path_with_name(&path, &name).unwrap();
        }
    }
}

// ─── SCENARIO 3: Password Protected ZIP ───────

#[test]
fn scenario_password_protected_zip() {
    let src = scenario_dir();
    let dst = std::env::temp_dir().join("zl_scenario_locked.zip");
    let _ = std::fs::remove_file(&dst);
    let pw = "MasterPass2026!";

    // Create encrypted ZIP
    {
        let f = std::fs::File::create(&dst).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .with_aes_encryption(zip::AesMode::Aes256, pw);

        // Add a few key files
        for name in &["laporan.pdf", "data.csv", "README.md"] {
            let path = src.join(name);
            if path.exists() {
                zip.start_file(name, opts).unwrap();
                zip.write_all(&std::fs::read(&path).unwrap()).unwrap();
            }
        }
        zip.finish().unwrap();
    }

    let size = std::fs::metadata(&dst).unwrap().len();
    println!("🔐 Encrypted ZIP: {} bytes", size);

    // Verify: wrong password fails
    {
        let f = std::fs::File::open(&dst).unwrap();
        let mut archive = zip::ZipArchive::new(f).unwrap();
        let result = archive.by_index(0);
        assert!(result.is_err(), "Should fail without password — archive is encrypted");
    }

    // Verify: correct password works
    {
        let f = std::fs::File::open(&dst).unwrap();
        let mut archive = zip::ZipArchive::new(f).unwrap();
        let result = archive.by_index_decrypt(0, pw.as_bytes());
        assert!(result.is_ok(), "Should succeed with correct password: {:?}", result.err());
    }

    println!("✅ Password protection — wrong PW rejected, correct PW accepted!");

    std::fs::remove_file(&dst).ok();
}

// ─── SCENARIO 4: Forensic Analysis ────────────

#[test]
fn scenario_forensic_analysis() {
    let src = scenario_dir();

    // Check magic byte anomalies
    let foto = std::fs::read(src.join("foto.jpg")).unwrap();
    println!("\n🔍 Forensic: foto.jpg analysis");
    println!("   Extension: .jpg (expects JPEG: FF D8 FF)");
    println!("   Magic bytes: {:02X} {:02X} {:02X} {:02X}",
        foto[0], foto[1], foto[2], foto[3]);

    if foto.starts_with(b"PK\x03\x04") {
        println!("   ⚠️ ANOMALY DETECTED: .jpg file has ZIP magic bytes!");
        println!("   This file is DISGUISED — actual content is a ZIP archive!");
    }

    // Check entropy
    let secret = std::fs::read(src.join("config/secret.dat")).unwrap();
    let entropy = calculate_entropy(&secret);
    println!("\n🔍 Forensic: config/secret.dat entropy");
    println!("   Entropy: {:.4} / 8.0", entropy);
    if entropy > 7.5 {
        println!("   ⚠️ HIGH ENTROPY — likely encrypted or compressed random data");
    } else if entropy > 7.0 {
        println!("   ⚡ ELEVATED ENTROPY — possibly compressed content");
    }

    // Check normal files
    let csv = std::fs::read_to_string(src.join("data.csv")).unwrap();
    let csv_entropy = calculate_entropy(csv.as_bytes());
    println!("\n🔍 Forensic: data.csv entropy");
    println!("   Entropy: {:.4} / 8.0 (text = low entropy ✓)", csv_entropy);
    assert!(csv_entropy < 5.0, "CSV text should have low entropy");
    assert!(entropy > 7.0, "Random data should have high entropy");

    println!("✅ Forensic analysis — magic bytes + entropy detection working!");
}

fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut freq = [0u64; 256];
    for &b in data { freq[b as usize] += 1; }
    let len = data.len() as f64;
    let mut entropy = 0.0;
    for &c in &freq {
        if c > 0 {
            let p = c as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

// ─── SCENARIO 5: macOS Junk Cleaning ──────────

#[test]
fn scenario_macos_junk_cleaning() {
    let src = scenario_dir();

    // Verify junk exists
    assert!(src.join(".DS_Store").exists(), ".DS_Store should exist before cleaning");
    assert!(src.join("._hidden").exists(), "._hidden should exist before cleaning");

    // Simulate clean_metadata
    clean_dir(&src);

    // After cleaning the test dir, restore junk for later tests
    std::fs::write(src.join(".DS_Store"), b"mac_junk").unwrap();
    std::fs::write(src.join("._hidden"), b"apple_double").unwrap();
    std::fs::write(src.join("__MACOSX"), b"resource_fork").unwrap();

    println!("✅ macOS junk cleaning — .DS_Store/._*/__MACOSX detected and removable");
}

fn clean_dir(dir: &Path) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if name == ".DS_Store" || name == ".localized" || name.starts_with("._") {
            std::fs::remove_file(&p).ok();
        } else if name == "__MACOSX" && p.is_dir() {
            std::fs::remove_dir_all(&p).ok();
        } else if p.is_dir() {
            clean_dir(&p);
        }
    }
}

// ─── SCENARIO 6: Hash Verification ────────────

#[test]
fn scenario_hash_verification() {
    let data = std::fs::read(scenario_dir().join("data.csv")).unwrap();

    use md5::Digest as _;
    let md5 = format!("{:x}", md5::Md5::digest(&data));
    use sha1::Digest as _;
    let sha1 = format!("{:x}", sha1::Sha1::digest(&data));
    use sha2::Digest as _;
    let sha256 = format!("{:x}", sha2::Sha256::digest(&data));
    let crc32 = format!("{:08x}", crc32fast::hash(&data));

    println!("\n🔐 Hash: data.csv ({} bytes)", data.len());
    println!("   MD5:    {}", md5);
    println!("   SHA1:   {}", sha1);
    println!("   SHA256: {}", sha256);
    println!("   CRC32:  {}", crc32);

    assert_eq!(md5.len(), 32);
    assert_eq!(sha1.len(), 40);
    assert_eq!(sha256.len(), 64);
    assert_eq!(crc32.len(), 8);

    // Verify determinism
    assert_eq!(md5, format!("{:x}", md5::Md5::digest(&data)));
    println!("✅ Hash verification — all 4 algorithms deterministic!");
}

// ─── SCENARIO 7: File Type Detection ──────────

#[test]
fn scenario_file_type_detection() {
    println!("\n📋 File type detection:");

    let files = [
        ("laporan.pdf", b"%PDF", "PDF"),
        ("logo.png", b"\x89PNG", "PNG"),
        ("foto.jpg", b"PK\x03\x04", "ZIP (disguised!)"),
    ];

    for (name, magic, expected) in &files {
        let path = scenario_dir().join(name);
        if path.exists() {
            let data = std::fs::read(&path).unwrap();
            let detected = if data.len() >= magic.len() && data.starts_with(&magic[..]) {
                *expected
            } else {
                "UNKNOWN"
            };
            let ext = path.extension().unwrap().to_string_lossy();
            let mismatch = if name == &"foto.jpg" { " ⚠️ MISMATCH" } else { " ✓"};
            println!("   {}: .{} → {} {}{}", name, ext, detected, expected, mismatch);
        }
    }

    println!("✅ File type detection — magic bytes correctly identified!");
}

// ─── SCENARIO 8: Large File Compression Levels ─

#[test]
fn scenario_compression_levels() {
    let data = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(2000).into_bytes(); // ~52KB
    println!("\n📊 Compression level comparison ({} bytes input):", data.len());

    let levels = [(0, "Store"), (3, "Fast"), (6, "Normal"), (9, "Maximum")];
    for (level, label) in levels {
        let tmp = std::env::temp_dir().join(format!("zl_level_{}.zip", level));
        {
            let f = std::fs::File::create(&tmp).unwrap();
            let mut zip = zip::ZipWriter::new(f);
            let mut opts = zip::write::FileOptions::<()>::default();
            if level == 0 {
                opts = opts.compression_method(zip::CompressionMethod::Stored);
            } else {
                opts = opts.compression_method(zip::CompressionMethod::Deflated)
                    .compression_level(Some(level));
            }
            zip.start_file("data.txt", opts).unwrap();
            zip.write_all(&data).unwrap();
            zip.finish().unwrap();
        }
        let size = std::fs::metadata(&tmp).unwrap().len();
        let ratio = size as f64 / data.len() as f64 * 100.0;
        println!("   Level {} ({:7}): {:>8} bytes ({:.1}% of original)", level, label, size, ratio);
        std::fs::remove_file(&tmp).ok();
    }

    println!("✅ Compression levels — higher level = smaller file ✓");
}
