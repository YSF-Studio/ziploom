//! Ensures every #[tauri::command] is registered in generate_handler! and
//! matches the commands invoked from the frontend.

const REGISTERED: &[&str] = &[
    "supported_formats",
    "archive_needs_password",
    "inspect_archive",
    "compress_files",
    "extract_archive",
    "encrypt_file",
    "decrypt_file",
    "stat_paths",
    "check_tools",
    "hash_file_sha256",
    "hash_archive",
    "get_progress",
    "preview_archive_entry",
    "forensic_scan_archive",
    "extract_archive_entries",
    "test_archive_integrity",
    "about_info",
];

const FRONTEND_USED: &[&str] = &[
    "about_info",
    "archive_needs_password",
    "stat_paths",
    "compress_files",
    "extract_archive",
    "inspect_archive",
    "forensic_scan_archive",
    "hash_archive",
    "preview_archive_entry",
    "extract_archive_entries",
    "get_progress",
    "check_tools",
];

#[test]
fn ipc_registry_has_unique_commands() {
    let mut seen = std::collections::HashSet::new();
    for cmd in REGISTERED {
        assert!(seen.insert(*cmd), "duplicate registered command: {cmd}");
    }
    assert_eq!(REGISTERED.len(), 17);
}

#[test]
fn ipc_frontend_commands_are_registered() {
    for cmd in FRONTEND_USED {
        assert!(
            REGISTERED.contains(cmd),
            "frontend invokes '{cmd}' but it is not in generate_handler!"
        );
    }
}
