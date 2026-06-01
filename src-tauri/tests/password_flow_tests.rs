// Password-specific forensic test
// Tests: password popup → correct PW → entries load
//        password popup → wrong PW → error
//        password popup → correct PW on retry → entries load

#[test]
fn forensic_password_correct_loads_entries() {
    let dir = std::env::temp_dir().join(format!("zl_fpw_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("secret.txt"), b"TOP SECRET DATA").unwrap();
    std::fs::write(dir.join("notes.txt"), b"regular notes").unwrap();

    let zip_path = dir.join("locked.zip");
    let pw = "TestPass123!";

    // Create encrypted ZIP
    {
        use std::io::Write;
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .with_aes_encryption(zip::AesMode::Aes256, pw);
        zip.start_file("secret.txt", opts).unwrap();
        zip.write_all(b"TOP SECRET DATA").unwrap();
        zip.start_file("notes.txt", opts).unwrap();
        zip.write_all(b"regular notes").unwrap();
        zip.finish().unwrap();
    }

    // Test 1: No password → should return PASSWORD_NEEDED
    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: zip_path.to_string_lossy().to_string(),
        password: None,
    };
    let result = ziploom_tauri_lib::archive_ops::forensic_load(args.source, args.password);
    assert!(result.is_err(), "Should fail without password");
    assert_eq!(result.unwrap_err(), "PASSWORD_NEEDED", "Should say PASSWORD_NEEDED");

    // Test 2: Correct password → should load entries
    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: zip_path.to_string_lossy().to_string(),
        password: Some(pw.to_string()),
    };
    let result = ziploom_tauri_lib::archive_ops::forensic_load(args.source, args.password);
    assert!(result.is_ok(), "Should succeed with correct password: {:?}", result.err());
    let entries = result.unwrap();
    assert_eq!(entries.len(), 2, "Should have 2 entries");
    assert!(entries.iter().any(|e| e.path == "secret.txt"), "Should contain secret.txt");
    assert!(entries.iter().any(|e| e.path == "notes.txt"), "Should contain notes.txt");

    // Test 3: Wrong password → should return PASSWORD_NEEDED
    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: zip_path.to_string_lossy().to_string(),
        password: Some("wrongpassword".to_string()),
    };
    let result = ziploom_tauri_lib::archive_ops::forensic_load(args.source, args.password);
    assert!(result.is_err(), "Should fail with wrong password");
    assert_eq!(result.unwrap_err(), "PASSWORD_NEEDED", "Should say PASSWORD_NEEDED for wrong PW");

    println!("✅ Password forensic flow: NO PW→blocked, CORRECT→loaded, WRONG→blocked");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn forensic_report_password_works() {
    let dir = std::env::temp_dir().join(format!("zl_frpw_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("data.txt"), b"encrypted report test data here!").unwrap();

    let zip_path = dir.join("report_locked.zip");
    let pw = "ReportPass456!";

    // Create encrypted ZIP
    {
        use std::io::Write;
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .with_aes_encryption(zip::AesMode::Aes256, pw);
        zip.start_file("data.txt", opts).unwrap();
        zip.write_all(b"encrypted report test data here!").unwrap();
        zip.finish().unwrap();
    }

    // Test: Full report with password
    let report = ziploom_tauri_lib::archive_ops::generate_forensic_report(
        zip_path.to_string_lossy().to_string(),
        Some(pw.to_string()),
    );
    assert!(report.is_ok(), "Full report should work with password: {:?}", report.err());
    let r = report.unwrap();
    assert_eq!(r.entries.len(), 1, "Should have 1 entry");
    assert_eq!(r.total_files, 1);
    println!("✅ Full report: {} files, {} format, {} total size", r.total_files, r.format, r.total_size);

    std::fs::remove_dir_all(&dir).ok();
}
