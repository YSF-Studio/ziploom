use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ziploom_lib::commands::{
    about_info, compress_files, decrypt_file, encrypt_file, extract_archive, inspect_archive,
    supported_formats,
};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("e2e")
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_output_dir() -> PathBuf {
    let n = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("ziploom-e2e-{}-{}", std::process::id(), n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp output dir");
    dir
}

fn source_paths() -> Vec<String> {
    let root = fixture_root();
    vec![
        root.join("sample_alpha.txt").to_string_lossy().into_owned(),
        root.join("sample_beta.txt").to_string_lossy().into_owned(),
        root.join("nested").to_string_lossy().into_owned(),
    ]
}

fn read_fixture(name: &str) -> String {
    fs::read_to_string(fixture_root().join(name)).expect("read fixture")
}

#[test]
fn e2e_supported_formats_and_about() {
    let formats = supported_formats();
    assert!(!formats.is_empty(), "supported_formats should return entries");
    assert!(formats.iter().any(|f| f.contains("zip") || f.to_lowercase().contains("zip")));

    let about = about_info();
    assert_eq!(about["appName"], "ZipLoom");
    assert_eq!(about["offline"], true);
    assert!(about["features"].as_array().map(|a| !a.is_empty()).unwrap_or(false));
}

#[test]
fn e2e_compress_inspect_extract_zip() {
    let out_dir = temp_output_dir();
    let archive = out_dir.join("test_bundle.zip");
    let extract_dir = out_dir.join("extracted_zip");

    let result = compress_files(
        source_paths(),
        archive.to_string_lossy().into_owned(),
        "zip".into(),
        None,
    )
    .expect("compress zip");
    assert!(result.success);
    assert!(archive.exists());
    assert!(result.files_processed >= 3);

    let info = inspect_archive(archive.to_string_lossy().into_owned(), None).expect("inspect zip");
    assert_eq!(info.format.to_lowercase(), "zip");
    assert!(info.total_files >= 3);
    assert!(info.total_size > 0);
    assert!(!info.entries.is_empty());

    let extracted = extract_archive(
        archive.to_string_lossy().into_owned(),
        extract_dir.to_string_lossy().into_owned(),
        None,
    )
    .expect("extract zip");
    assert!(extracted.success);
    assert!(extract_dir.join("sample_alpha.txt").exists());
    assert_eq!(
        fs::read_to_string(extract_dir.join("sample_alpha.txt")).unwrap(),
        read_fixture("sample_alpha.txt")
    );
}

#[test]
fn e2e_compress_inspect_extract_tar() {
    let out_dir = temp_output_dir();
    let archive = out_dir.join("test_bundle.tar");
    let extract_dir = out_dir.join("extracted_tar");

    compress_files(
        source_paths(),
        archive.to_string_lossy().into_owned(),
        "tar".into(),
        None,
    )
    .expect("compress tar");

    let info = inspect_archive(archive.to_string_lossy().into_owned(), None).expect("inspect tar");
    assert!(info.format.to_lowercase().contains("tar"));

    extract_archive(
        archive.to_string_lossy().into_owned(),
        extract_dir.to_string_lossy().into_owned(),
        None,
    )
    .expect("extract tar");

    assert!(extract_dir.join("sample_beta.txt").exists());
}

#[test]
fn e2e_compress_inspect_extract_targz() {
    let out_dir = temp_output_dir();
    let archive = out_dir.join("test_bundle.tar.gz");
    let extract_dir = out_dir.join("extracted_targz");

    compress_files(
        source_paths(),
        archive.to_string_lossy().into_owned(),
        "tar.gz".into(),
        None,
    )
    .expect("compress tar.gz");

    let info = inspect_archive(archive.to_string_lossy().into_owned(), None).expect("inspect tar.gz");
    assert!(info.total_files >= 1);

    extract_archive(
        archive.to_string_lossy().into_owned(),
        extract_dir.to_string_lossy().into_owned(),
        None,
    )
    .expect("extract tar.gz");

    assert!(extract_dir.join("nested").join("sample_gamma.txt").exists()
        || extract_dir.join("sample_gamma.txt").exists());
}

#[test]
fn e2e_encrypt_decrypt_roundtrip() {
    let out_dir = temp_output_dir();
    let source = out_dir.join("secret.txt");
    fs::write(&source, b"TOP SECRET - ZipLoom encryption test payload.").unwrap();

    let encrypted_path =
        encrypt_file(source.to_string_lossy().into_owned(), "TestPass123!".into())
            .expect("encrypt file");
    assert!(Path::new(&encrypted_path).exists());
    assert_ne!(
        fs::read(&encrypted_path).unwrap(),
        fs::read(&source).unwrap()
    );

    let decrypted_path =
        decrypt_file(encrypted_path, "TestPass123!".into()).expect("decrypt file");
    assert_eq!(
        fs::read_to_string(decrypted_path).unwrap(),
        "TOP SECRET - ZipLoom encryption test payload."
    );
}

#[test]
fn e2e_compress_password_protected_zip() {
    let out_dir = temp_output_dir();
    let archive = out_dir.join("secure_bundle.zip");
    let extract_dir = out_dir.join("extracted_secure");
    let pw = "TestPass123!";

    compress_files(
        source_paths(),
        archive.to_string_lossy().into_owned(),
        "zip".into(),
        Some(pw.into()),
    )
    .expect("compress password zip");
    assert!(archive.exists());

    let err = inspect_archive(archive.to_string_lossy().into_owned(), None).unwrap_err();
    assert!(
        err.contains("PASSWORD_NEEDED") || err.to_lowercase().contains("password"),
        "expected password error, got: {err}"
    );

    let info = inspect_archive(archive.to_string_lossy().into_owned(), Some(pw.into()))
        .expect("inspect password zip");
    assert!(info.total_files >= 3);

    extract_archive(
        archive.to_string_lossy().into_owned(),
        extract_dir.to_string_lossy().into_owned(),
        Some(pw.into()),
    )
    .expect("extract password zip");
    assert!(extract_dir.join("sample_alpha.txt").exists());
}

#[test]
fn e2e_inspect_rejects_missing_archive() {
    let err = inspect_archive("/nonexistent/missing.zip".into(), None).unwrap_err();
    let lower = err.to_lowercase();
    assert!(
        lower.contains("failed") || lower.contains("read") || lower.contains("cannot open"),
        "expected missing-file error, got: {err}"
    );
}
