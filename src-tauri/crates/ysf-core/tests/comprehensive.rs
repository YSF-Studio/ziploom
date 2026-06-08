/// YSF Core Comprehensive Test Suite
/// Located in tests/ for integration test access

use ysf_core::*;
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;

// ═══ Entropy Tests ═══

#[test]
fn entropy_empty() {
    assert!((compute_entropy(b"") - 0.0).abs() < 0.01);
}

#[test]
fn entropy_single_char() {
    assert!((compute_entropy(b"A") - 0.0).abs() < 0.01);
}

#[test]
fn entropy_all_zeros() {
    let data = vec![0u8; 1000];
    assert!((compute_entropy(&data) - 0.0).abs() < 0.01);
}

#[test]
fn entropy_all_unique() {
    let data: Vec<u8> = (0..=255).collect();
    assert!((compute_entropy(&data) - 8.0).abs() < 0.01);
}

#[test]
fn entropy_two_values_alternating() {
    let data: Vec<u8> = (0..1000).map(|i| (i % 2) as u8).collect();
    let e = compute_entropy(&data);
    assert!((e - 1.0).abs() < 0.05, "Got {} expected ~1.0", e);
}

#[test]
fn entropy_english_text_approx_range() {
    let text = b"The quick brown fox jumps over the lazy dog. ".repeat(10);
    let e = compute_entropy(&text);
    assert!(e > 3.0 && e < 6.0, "English text entropy {} not in [3,6]", e);
}

// ═══ Magic Byte Tests ═══

#[test]
fn magic_png_match() {
    let data = b"\x89PNG\r\n\x1a\nhello";
    let (m, detected, _) = check_magic_bytes(data, "photo.png");
    assert_eq!(m, Some(true));
    assert_eq!(detected.as_deref(), Some("PNG"));
}

#[test]
fn magic_png_disguised_as_jpg() {
    let data = b"\x89PNG\r\n\x1a\nsneaky";
    let (m, detected, expected) = check_magic_bytes(data, "photo.jpg");
    eprintln!("detected={:?} expected={:?}", detected, expected);
    assert_eq!(m, Some(false));
    assert_eq!(detected.as_deref(), Some("PNG"));
    assert_eq!(expected.as_deref(), Some("png")); // Canonical ext of detected type
}

#[test]
fn magic_jpeg_match() {
    let data = b"\xff\xd8\xff\xe0...";
    let (m, _, _) = check_magic_bytes(data, "photo.jpg");
    assert_eq!(m, Some(true));
}

#[test]
fn magic_pdf_match() {
    let data = b"%PDF-1.4 content";
    let (m, _, _) = check_magic_bytes(data, "doc.pdf");
    assert_eq!(m, Some(true));
}

#[test]
fn magic_pdf_as_exe_mismatch() {
    let data = b"%PDF-1.4 content";
    let (m, detected, _) = check_magic_bytes(data, "malware.exe");
    assert_eq!(m, Some(false));
    assert_eq!(detected.as_deref(), Some("PDF"));
}

#[test]
fn magic_zip_match() {
    let data = b"PK\x03\x04zipcontent";
    let (m, _, _) = check_magic_bytes(data, "archive.zip");
    assert_eq!(m, Some(true));
}

#[test]
fn magic_unknown_extension_not_flagged() {
    let data = b"\x89PNG\r\n\x1a\ndata";
    let (m, _, _) = check_magic_bytes(data, "unknown.xyz");
    assert_eq!(m, Some(false));
}

#[test]
fn magic_empty_file() {
    let (m, _, _) = check_magic_bytes(b"", "empty.txt");
    assert_eq!(m, None);
}

#[test]
fn magic_db_has_25_plus_signatures() {
    assert!(
        hashing::MAGIC_DB.len() >= 25,
        "expected at least 25 signatures, got {}",
        hashing::MAGIC_DB.len()
    );
}

#[test]
fn scanner_flags_pdf_lnk_and_ole_markers() {
    let dir = tempdir().unwrap();
    let archive_path = dir.path().join("scanner-fixture.zip");

    let file = std::fs::File::create(&archive_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default();

    zip.start_file("docs/report.pdf", opts).unwrap();
    zip.write_all(b"%PDF-1.4\n1 0 obj\n<< /OpenAction << /S /JavaScript >> >>\nstream\nfunction test(){app.alert('x');}\nendstream\n%%EOF").unwrap();

    zip.start_file("shortcuts/dropper.lnk", opts).unwrap();
    zip.write_all(&[
        0x4C, 0x00, 0x00, 0x00,
        0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
        0x22, 0x00, 0x00, 0x00,
        0x43, 0x00, 0x00, 0x00,
    ]).unwrap();
    let lnk_target: Vec<u8> = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
        .encode_utf16()
        .flat_map(|u| u.to_le_bytes())
        .collect();
    zip.write_all(&lnk_target).unwrap();

    zip.start_file("office/macro.doc", opts).unwrap();
    zip.write_all(b"\xd0\xcf\x11\xe0macro header WordDocument _VBA_PROJECT AutoOpen WScript.Shell CreateObject").unwrap();

    zip.finish().unwrap();

    let entries = forensic_load(archive_path.to_str().unwrap(), None).unwrap();
    let metadata = scan_archive_metadata(&entries);
    assert!(!metadata.threats.iter().any(|t| t.threat == "File flood"), "unexpected file flood");

    let content_threats = scan_archive_content(archive_path.to_str().unwrap(), None, &entries);
    assert!(content_threats.iter().any(|t| t.threat == "PDF with JavaScript"), "missing PDF heuristic");
    assert!(content_threats.iter().any(|t| t.threat.contains("LNK")), "missing LNK heuristic");
    assert!(content_threats.iter().any(|t| t.threat == "OLE/VBA macro markers"), "missing OLE/VBA heuristic");
}

// ═══ Hash Tests ═══

#[test]
fn hash_known_vector_md5() {
    let hs = hashing::multi_hash_buffer(b"hello world");
    assert_eq!(hs.md5.as_deref(), Some("5eb63bbbe01eeed093cb22bb8f5acdc3"));
}

#[test]
fn hash_known_vector_sha256() {
    let hs = hashing::multi_hash_buffer(b"hello world");
    assert_eq!(hs.sha256.as_deref(), Some("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"));
}

#[test]
fn hash_empty_file_known() {
    let hs = hashing::multi_hash_buffer(b"");
    assert_eq!(hs.md5.as_deref(), Some("d41d8cd98f00b204e9800998ecf8427e"));
}

// ═══ Crypto Tests ═══

#[test]
fn crypto_sign_verify_roundtrip() {
    let kp = crypto::generate_keypair();
    let sig = crypto::sign_data(&kp.private_key, b"CollectionLoom evidence").unwrap();
    assert!(crypto::verify_signature(&kp.public_key, b"CollectionLoom evidence", &sig).unwrap());
}

#[test]
fn crypto_tampered_data_fails() {
    let kp = crypto::generate_keypair();
    let sig = crypto::sign_data(&kp.private_key, b"original").unwrap();
    assert!(!crypto::verify_signature(&kp.public_key, b"tampered", &sig).unwrap());
}

#[test]
fn crypto_wrong_key_fails() {
    let kp1 = crypto::generate_keypair();
    let kp2 = crypto::generate_keypair();
    let sig = crypto::sign_data(&kp1.private_key, b"data").unwrap();
    assert!(!crypto::verify_signature(&kp2.public_key, b"data", &sig).unwrap());
}

// ═══ Evidence Tests ═══

#[test]
fn evidence_id_format() {
    let eid = evidence::EvidenceId::new("COL");
    let s = eid.to_string();
    assert!(s.starts_with("COL-"), "Got: {}", s);
    assert!(s.len() >= 15, "Too short: {}", s);
}

#[test]
fn chain_of_custody_actions() {
    let mut coc = evidence::ChainOfCustody::new("TestCase", "Tester", "/dev/sda", 1024);
    assert!(!coc.evidence_id.is_empty());
    coc.add_action("imaging_start", "Started DD imaging", None);
    assert_eq!(coc.actions.len(), 1);
    coc.add_action("imaging_done", "Hash: abc123", Some("abc123"));
    assert_eq!(coc.actions.len(), 2);
}

// ═══ Progress Tests ═══

#[test]
fn progress_default_state() {
    let p = ProgressState::default();
    assert_eq!(p.percent, 0.0);
    assert!(!p.is_done);
}

#[test]
fn cancel_flag_lifecycle() {
    let cf = progress::CancelFlag::new();
    assert!(!cf.is_cancelled());
    cf.cancel();
    assert!(cf.is_cancelled());
    cf.reset();
    assert!(!cf.is_cancelled());
}
