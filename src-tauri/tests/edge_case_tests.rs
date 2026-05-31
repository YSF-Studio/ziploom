// Comprehensive edge case tests for ZipLoom forensic engine
// Tests: .docx files, empty archives, special chars, per-entry failures

#[test]
fn forensic_docx_as_zip_works() {
    // A .docx file IS a ZIP — create a minimal .docx structure
    let dir = std::env::temp_dir().join(format!("zl_docx_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let docx_path = dir.join("report.docx");

    // Create minimal .docx (Office Open XML = ZIP with specific entries)
    {
        use std::io::Write;
        let f = std::fs::File::create(&docx_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(b"<?xml version=\"1.0\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();

        zip.start_file("_rels/.rels", opts).unwrap();
        zip.write_all(b"<?xml version=\"1.0\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>").unwrap();

        zip.start_file("word/document.xml", opts).unwrap();
        zip.write_all(b"<?xml version=\"1.0\"?><document xmlns=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><body><p>Hello World</p></body></document>").unwrap();

        zip.finish().unwrap();
    }

    assert!(docx_path.exists(), ".docx file should exist");

    // Test 1: Detect format — should recognize as ZIP
    let fmt = ziploom_tauri_lib::archive_ops::detect_format_cmd(docx_path.to_string_lossy().to_string());
    assert!(fmt.is_some(), "Should detect .docx format");
    assert_eq!(fmt.unwrap().id, "zip", "Should detect as ZIP");

    // Test 2: Load forensic — should list all entries
    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: docx_path.to_string_lossy().to_string(),
        password: None,
    };
    let result = ziploom_tauri_lib::archive_ops::forensic_load(args);
    assert!(result.is_ok(), "Should load .docx entries: {:?}", result.err());
    let entries = result.unwrap();
    assert_eq!(entries.len(), 3, "Should have 3 entries, got: {:?}", entries.iter().map(|e| &e.path).collect::<Vec<_>>());
    assert!(entries.iter().any(|e| e.path == "word/document.xml"), "Should contain word/document.xml");

    println!("✅ .docx forensic: 3 entries loaded from Office Open XML");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn forensic_xlsx_as_zip_works() {
    let dir = std::env::temp_dir().join(format!("zl_xlsx_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let xlsx_path = dir.join("spreadsheet.xlsx");

    {
        use std::io::Write;
        let f = std::fs::File::create(&xlsx_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("xl/workbook.xml", opts).unwrap();
        zip.write_all(b"<?xml version=\"1.0\"?><workbook/>").unwrap();
        zip.start_file("xl/worksheets/sheet1.xml", opts).unwrap();
        zip.write_all(b"<?xml version=\"1.0\"?><worksheet/>").unwrap();

        zip.finish().unwrap();
    }

    let fmt = ziploom_tauri_lib::archive_ops::detect_format_cmd(xlsx_path.to_string_lossy().to_string());
    assert!(fmt.is_some(), "Should detect .xlsx");
    assert_eq!(fmt.unwrap().id, "zip");

    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: xlsx_path.to_string_lossy().to_string(),
        password: None,
    };
    let entries = ziploom_tauri_lib::archive_ops::forensic_load(args).unwrap();
    assert_eq!(entries.len(), 2);
    println!("✅ .xlsx forensic: 2 entries loaded");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn forensic_empty_zip_does_not_crash() {
    let dir = std::env::temp_dir().join(format!("zl_empty_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let zip_path = dir.join("empty.zip");

    // Create truly empty ZIP
    {
        use std::io::Write;
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        zip.finish().unwrap();
    }

    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: zip_path.to_string_lossy().to_string(),
        password: None,
    };
    let result = ziploom_tauri_lib::archive_ops::forensic_load(args);
    assert!(result.is_ok(), "Empty ZIP should not crash: {:?}", result.err());
    let entries = result.unwrap();
    assert_eq!(entries.len(), 0, "Empty ZIP should have 0 entries");
    println!("✅ Empty ZIP: 0 entries, no crash");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn forensic_mixed_content_zip() {
    // ZIP with various file types: text, binary, empty, special chars in name
    let dir = std::env::temp_dir().join(format!("zl_mixed_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let zip_path = dir.join("mixed.zip");

    {
        use std::io::Write;
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Regular text file
        zip.start_file("readme.txt", opts).unwrap();
        zip.write_all(b"Hello World").unwrap();

        // File with spaces and special chars
        zip.start_file("my documents/report (final).txt", opts).unwrap();
        zip.write_all(b"report content").unwrap();

        // Empty file
        zip.start_file("empty.dat", opts).unwrap();
        zip.write_all(b"").unwrap();

        // Binary file
        zip.start_file("data.bin", opts).unwrap();
        zip.write_all(&[0x00, 0xFF, 0xAB, 0xCD, 0x12, 0x34]).unwrap();

        // Nested directory
        zip.start_file("deep/nested/file.json", opts).unwrap();
        zip.write_all(b"{\"key\": \"value\"}").unwrap();

        zip.finish().unwrap();
    }

    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: zip_path.to_string_lossy().to_string(),
        password: None,
    };
    let entries = ziploom_tauri_lib::archive_ops::forensic_load(args).unwrap();
    assert_eq!(entries.len(), 5, "Should have 5 entries, got: {:?}",
        entries.iter().map(|e| &e.path).collect::<Vec<_>>());
    
    assert!(entries.iter().any(|e| e.path.contains("(final)")), "Should handle parens in name");
    assert!(entries.iter().any(|e| e.size == 0), "Should have empty file");
    assert!(entries.iter().any(|e| e.path == "deep/nested/file.json"), "Should handle deep nesting");

    println!("✅ Mixed content: {} entries (special chars, empty, binary, nested)", entries.len());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn forensic_stored_not_deflated_zip() {
    // ZIP with Stored (no compression) entries
    let dir = std::env::temp_dir().join(format!("zl_stored_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let zip_path = dir.join("stored.zip");

    {
        use std::io::Write;
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("uncompressed.txt", opts).unwrap();
        zip.write_all(b"This file is NOT compressed at all").unwrap();

        zip.start_file("also_stored.dat", opts).unwrap();
        zip.write_all(&[1u8, 2, 3, 4, 5]).unwrap();

        zip.finish().unwrap();
    }

    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: zip_path.to_string_lossy().to_string(),
        password: None,
    };
    let entries = ziploom_tauri_lib::archive_ops::forensic_load(args).unwrap();
    assert_eq!(entries.len(), 2, "Stored ZIP should have 2 entries");
    assert_eq!(entries[0].size, 34); // "This file is NOT compressed at all" = 34 bytes
    println!("✅ Stored ZIP: 2 entries, sizes correct");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn forensic_corrupted_zip_handled() {
    let dir = std::env::temp_dir().join(format!("zl_corrupt_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let zip_path = dir.join("corrupt.zip");

    // Write garbage that looks like ZIP header but isn't valid
    std::fs::write(&zip_path, b"PK\x03\x04garbage not a real zip").unwrap();

    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: zip_path.to_string_lossy().to_string(),
        password: None,
    };
    let result = ziploom_tauri_lib::archive_ops::forensic_load(args);
    // Should error gracefully, NOT crash
    assert!(result.is_err(), "Corrupted ZIP should produce an error");
    println!("✅ Corrupted ZIP: error handled gracefully: {}", result.unwrap_err());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn forensic_nonexistent_file() {
    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: "/tmp/definitely_does_not_exist_xyz.zip".to_string(),
        password: None,
    };
    let result = ziploom_tauri_lib::archive_ops::forensic_load(args);
    assert!(result.is_err(), "Nonexistent file should error");
    println!("✅ Nonexistent file: error handled");

}

#[test]
fn forensic_large_number_of_entries() {
    let dir = std::env::temp_dir().join(format!("zl_many_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let zip_path = dir.join("many.zip");

    {
        use std::io::Write;
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Stored);

        for i in 0..100 {
            zip.start_file(format!("file_{:03}.txt", i), opts).unwrap();
            zip.write_all(format!("content {}", i).as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }

    let args = ziploom_tauri_lib::ForensicLoadArgs {
        source: zip_path.to_string_lossy().to_string(),
        password: None,
    };
    let entries = ziploom_tauri_lib::archive_ops::forensic_load(args).unwrap();
    assert_eq!(entries.len(), 100, "Should have 100 entries");
    println!("✅ 100-entry ZIP: all {} loaded", entries.len());

    std::fs::remove_dir_all(&dir).ok();
}
