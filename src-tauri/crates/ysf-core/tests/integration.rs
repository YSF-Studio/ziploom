/// Full integration test suite — ZipLoom-compatible patterns
/// Creates real sample archives and runs forensic_load + forensic_report

use ysf_core::*;
use std::path::{Path, PathBuf};
use std::io::Write;

// ═══ Test Helpers ═══
fn tempdir(name: &str) -> PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target").join("test_tmp").join(format!("ysf_test_{}_{}", name, uuid::Uuid::new_v4()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn create_sample_files(dir: &Path) {
    std::fs::write(dir.join("readme.txt"), b"CollectionLoom forensic test file.\nThis is a sample text document.\n").unwrap();
    std::fs::write(dir.join("data.csv"), b"id,name,value\n1,alpha,100\n2,beta,200\n3,gamma,300\n").unwrap();
    std::fs::write(dir.join("empty.dat"), b"").unwrap();
    std::fs::write(dir.join("binary.bin"), &[0u8; 1024]).unwrap();
}

fn create_test_zip(zip_path: &Path, src_dir: &Path, _password: Option<&str>) {
    use std::fs::File;
    use zip::write::ZipWriter;
    use zip::CompressionMethod;

    let file = File::create(zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    for entry in std::fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            zip.start_file(path.file_name().unwrap().to_string_lossy(), options.clone()).unwrap();
            let content = std::fs::read(&path).unwrap();
            zip.write_all(&content).unwrap();
        }
    }
    zip.finish().unwrap();
}

fn create_test_tar(tar_path: &Path, src_dir: &Path) {
    use tar::Builder;
    let file = std::fs::File::create(tar_path).unwrap();
    let mut tar = Builder::new(file);
    for entry in std::fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path != tar_path {
            tar.append_path_with_name(&path, path.file_name().unwrap()).unwrap();
        }
    }
    tar.finish().unwrap();
}

// ═══ Archive Forensic Tests ═══

#[test]
fn forensic_load_zip_normal() {
    let dir = tempdir("zip_normal");
    create_sample_files(&dir);
    let zip_path = dir.join("test.zip");
    create_test_zip(&zip_path, &dir, None);
    
    let entries = archive::forensic_load(&zip_path.to_string_lossy(), None).unwrap();
    assert!(entries.len() >= 4, "Expected >=4 files, got {}", entries.len());
    
    // Verify entries have required fields
    for e in &entries {
        assert!(!e.path.is_empty(), "Empty path for entry");
    }
}

#[test]
fn forensic_load_tar_normal() {
    let dir = tempdir("tar_normal");
    create_sample_files(&dir);
    let tar_path = dir.join("test.tar");
    create_test_tar(&tar_path, &dir);
    
    let entries = archive::forensic_load(&tar_path.to_string_lossy(), None).unwrap();
    assert!(entries.len() >= 4);
}

#[test]
fn forensic_load_tar_gz() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    
    let dir = tempdir("tar_gz");
    create_sample_files(&dir);
    let tar_path = dir.join("test.tar");
    create_test_tar(&tar_path, &dir);
    
    // Compress with gzip
    let tar_bytes = std::fs::read(&tar_path).unwrap();
    let gz_path = dir.join("test.tar.gz");
    let mut encoder = GzEncoder::new(std::fs::File::create(&gz_path).unwrap(), Compression::default());
    encoder.write_all(&tar_bytes).unwrap();
    encoder.finish().unwrap();
    
    let entries = archive::forensic_load(&gz_path.to_string_lossy(), None).unwrap();
    assert!(entries.len() >= 4);
}

#[test]
fn forensic_load_format_detection() {
    assert_eq!(archive::detect_format("test.zip"), Some("zip"));
    assert_eq!(archive::detect_format("test.7z"), Some("7z"));
    assert_eq!(archive::detect_format("test.rar"), Some("rar"));
    assert_eq!(archive::detect_format("test.tar"), Some("tar"));
    assert_eq!(archive::detect_format("test.tar.gz"), Some("tar.gz"));
    assert_eq!(archive::detect_format("test.tar.bz2"), Some("tar.bz2"));
    assert_eq!(archive::detect_format("test.tar.xz"), Some("tar.xz"));
    assert_eq!(archive::detect_format("test.unknown"), None);
}

#[test]
fn forensic_report_basic() {
    let dir = tempdir("report");
    create_sample_files(&dir);
    let zip_path = dir.join("test.zip");
    create_test_zip(&zip_path, &dir, None);
    
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let report = archive::generate_forensic_report(
        &zip_path.to_string_lossy(), None, &cancel
    ).unwrap();
    
    assert!(report.total_files >= 4);
    assert!(!report.format.is_empty());
    assert!(report.risk_label.len() > 0);
}

#[test]
fn forensic_report_empty_archive() {
    let dir = tempdir("empty_report");
    std::fs::create_dir_all(&dir).unwrap();
    let zip_path = dir.join("empty.zip");
    
    // Create minimal empty zip
    use zip::write::ZipWriter;
    let file = std::fs::File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    zip.finish().unwrap();
    
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let report = archive::generate_forensic_report(
        &zip_path.to_string_lossy(), None, &cancel
    ).unwrap();
    
    assert_eq!(report.total_files, 0);
    assert_eq!(report.risk_label, "Clean");
}

// ═══ NTFS Tests ═══

#[test]
fn ntfs_parse_invalid_boot_sector() {
    let dir = tempdir("ntfs_invalid");
    // Create a file with garbage data (not a valid NTFS image)
    let invalid_path = dir.join("not_ntfs.dd");
    std::fs::write(&invalid_path, &[0u8; 4096]).unwrap();
    
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let result = ntfs::parse_mft(&invalid_path.to_string_lossy(), &cancel);
    // Should either error gracefully or return empty
    match result {
        Ok(entries) => assert!(entries.is_empty(), "Invalid NTFS should return empty"),
        Err(_) => {} // Error is acceptable
    }
}

// ═══ Carving Tests ═══

#[test]
fn carving_finds_png_in_raw_data() {
    let dir = tempdir("carve");
    let out_dir = dir.join("output");
    std::fs::create_dir_all(&out_dir).unwrap();
    
    // Create raw data with embedded PNG header
    let mut data = vec![0u8; 4096];
    data[1000..1008].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    let raw_path = dir.join("raw.bin");
    std::fs::write(&raw_path, &data).unwrap();
    
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let result = carving::carve_files(
        &raw_path.to_string_lossy(), &out_dir.to_string_lossy(), &cancel
    ).unwrap();
    
    assert!(result.files_found >= 1, "Should find at least 1 file");
    assert!(!result.files.is_empty());
}

#[test]
fn carving_finds_jpeg_header() {
    let dir = tempdir("carve_jpg");
    let out_dir = dir.join("output");
    std::fs::create_dir_all(&out_dir).unwrap();
    
    let mut data = vec![0u8; 4096];
    data[500..503].copy_from_slice(b"\xff\xd8\xff");
    let raw_path = dir.join("data.bin");
    std::fs::write(&raw_path, &data).unwrap();
    
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let result = carving::carve_files(
        &raw_path.to_string_lossy(), &out_dir.to_string_lossy(), &cancel
    ).unwrap();
    
    assert!(result.files_found >= 1);
}

// ═══ Magic Byte Property Tests ═══

#[test]
fn property_magic_byte_exe_header_known() {
    let data = b"MZ\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00\xff\xff\x00\x00";
    let (m, detected, _) = check_magic_bytes(data, "program.exe");
    assert_eq!(m, Some(true));
    assert_eq!(detected.as_deref(), Some("PE (EXE/DLL)"));
}

#[test]
fn property_magic_byte_elf_header() {
    let data = b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    let (m, detected, _) = check_magic_bytes(data, "app.elf");
    assert_eq!(m, Some(true));
    assert_eq!(detected.as_deref(), Some("ELF"));
}

#[test]
fn property_magic_byte_rar_header() {
    let data = b"Rar!\x1a\x07\x00\xcf\x90\x73\x00\x00";
    let (m, detected, _) = check_magic_bytes(data, "archive.rar");
    assert_eq!(m, Some(true));
    assert_eq!(detected.as_deref(), Some("RAR"));
}

// ═══ Stress Tests ═══

#[test]
fn stress_forensic_load_3x_repeat() {
    let dir = tempdir("stress");
    create_sample_files(&dir);
    let zip_path = dir.join("stress.zip");
    create_test_zip(&zip_path, &dir, None);
    
    let mut entry_counts = vec![];
    for run in 1..=3 {
        let entries = archive::forensic_load(&zip_path.to_string_lossy(), None).unwrap();
        assert!(entries.len() >= 4, "Run {}: expected >=4 entries", run);
        entry_counts.push(entries.len());
    }
    // All runs should return same count
    assert_eq!(entry_counts[0], entry_counts[1]);
    assert_eq!(entry_counts[1], entry_counts[2]);
}

#[test]
fn stress_all_formats_load() {
    let dir = tempdir("all_formats");
    create_sample_files(&dir);
    
    // Test each compressible TAR format
    for (ext, compressor) in [("bz2", "bz2"), ("xz", "xz")].iter() {
        let tar_path = dir.join("test.tar");
        create_test_tar(&tar_path, &dir);
        let tar_bytes = std::fs::read(&tar_path).unwrap();
        
        let comp_path = dir.join(&format!("test.tar.{}", ext));
        match *compressor {
            "bz2" => {
                use bzip2::write::BzEncoder;
                let mut enc = BzEncoder::new(std::fs::File::create(&comp_path).unwrap(), bzip2::Compression::default());
                enc.write_all(&tar_bytes).unwrap();
                enc.finish().unwrap();
            }
            "xz" => {
                use xz2::write::XzEncoder;
                let mut enc = XzEncoder::new(std::fs::File::create(&comp_path).unwrap(), 6);
                enc.write_all(&tar_bytes).unwrap();
                enc.finish().unwrap();
            }
            _ => {}
        }
        
        let entries = archive::forensic_load(&comp_path.to_string_lossy(), None).unwrap();
        assert!(entries.len() >= 4, "tar.{} format should load", ext);
    }
}

// ═══ Entropy Property Tests ═══

#[test]
fn property_entropy_encrypted_like_data() {
    // Random-looking data (encrypted/compressed) should have entropy > 7.5
    let data: Vec<u8> = (0i32..10000).map(|i| (i.wrapping_mul(17) ^ 0x55) as u8).collect();
    let e = compute_entropy(&data);
    assert!(e > 7.0, "High-entropy data should score > 7.0, got {}", e);
}

#[test]
fn property_entropy_text_data() {
    let text = b"Hello, this is a standard text file with readable content.".repeat(50);
    let e = compute_entropy(&text);
    assert!(e < 6.5, "Text entropy should be < 6.5, got {}", e);
}

// ═══ Evidence/Metadata Tests ═══

#[test]
fn evidence_id_unique_across_calls() {
    let id1 = evidence::EvidenceId::new("COL").to_string();
    let id2 = evidence::EvidenceId::new("COL").to_string();
    // Each call should increment sequence
    let seq1: u16 = id1.rsplit('-').next().unwrap().parse().unwrap();
    let seq2: u16 = id2.rsplit('-').next().unwrap().parse().unwrap();
    assert!(seq2 > seq1, "Sequences should increment: {} → {}", seq1, seq2);
}

#[test]
fn chain_of_custody_signing() {
    let mut coc = evidence::ChainOfCustody::new("SignCase", "Signer", "/dev/sdb", 2048);
    coc.add_action("start", "Imaging", None);
    coc.add_action("done", "Complete", Some("abc123"));
    
    let kp = crypto::generate_keypair();
    coc.sign(&kp.private_key).unwrap();
    assert!(coc.signature.is_some());
}

// ═══ Progress/Cancel in Long Operation ═══

#[test]
fn cancel_flag_stops_operation() {
    let cancel = std::sync::atomic::AtomicBool::new(true); // Already cancelled
    
    // Try to hash — should fail immediately
    let dir = tempdir("cancel");
    std::fs::write(dir.join("f.txt"), b"data").unwrap();
    let result = hashing::multi_hash(&dir.join("f.txt"), &cancel);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "CANCELLED");
}
