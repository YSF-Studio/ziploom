//! Simulated digital forensic examiner workflow — builds evidence archives and
//! prints a chain-of-custody style report. Run:
//!   cargo test --test forensic_exam_test -- --nocapture

use std::fs;
use std::path::{Path, PathBuf};

use ziploom_lib::commands::{
    compress_files_sync, extract_archive_entries_sync, forensic_scan_archive_sync,
    hash_archive_sync, inspect_archive_sync, preview_archive_entry_sync,
};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn exam_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ziploom-forensic-exam-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create exam dir");
    dir
}

fn build_evidence_staging(staging: &Path) {
    let samples = workspace_root().join("samples");
    let fixtures = workspace_root().join("tests/fixtures/e2e");

    for name in ["evidence_manifest.txt", "confidential_report.txt", "case_metadata.xml"] {
        let src = samples.join(name);
        if src.exists() {
            fs::copy(&src, staging.join(name)).expect("copy sample");
        }
    }
    for name in ["sample_alpha.txt", "sample_beta.txt"] {
        let src = fixtures.join(name);
        if src.exists() {
            fs::copy(&src, staging.join(name)).expect("copy fixture");
        }
    }

    // High-entropy payload (simulates encrypted / packed data)
    let mut rng = [0u8; 4096];
    for (i, b) in rng.iter_mut().enumerate() {
        *b = ((i.wrapping_mul(1103515245).wrapping_add(12345)) & 0xff) as u8;
    }
    fs::write(staging.join("artifacts/encrypted_payload.bin"), &rng).expect("write entropy file");

    // Magic-byte spoof: PE header disguised as PDF name + double extension
    let fake_pe = b"MZ\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00\xff\xff\x00\x00";
    fs::write(staging.join("documents/report.pdf.exe"), fake_pe).expect("write spoof exe");

    // Zero-byte stub (deleted artifact indicator)
    fs::write(staging.join("logs/deleted_stub.log"), b"").expect("write zero-byte");
}

fn zip_staging(staging: &Path, out_zip: &Path) {
    let mut paths: Vec<String> = Vec::new();
    for entry in walkdir::WalkDir::new(staging).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            paths.push(entry.path().to_string_lossy().into_owned());
        }
    }
    compress_files_sync(paths, out_zip.to_string_lossy().into_owned(), "zip".into(), None)
        .expect("build evidence zip");
}

fn print_report(title: &str, report: &ysf_core::archive::ForensicReport) {
    println!("\n{}", "=".repeat(72));
    println!("  {title}");
    println!("  Archive: {}", report.archive_path);
    println!("  Format: {} | Files: {} | Size: {} bytes", report.format, report.total_files, report.total_size);
    println!("  Risk: {} (score {:.2})", report.risk_label, report.risk_score);
    println!("{}", "-".repeat(72));

    println!("\n  [ENTRIES]");
    for e in &report.entries {
        if e.is_dir {
            continue;
        }
        let ent = e.entropy.map(|x| format!("{x:.2}")).unwrap_or_else(|| "-".into());
        let magic = match e.magic_match {
            Some(true) => "OK",
            Some(false) => "MISMATCH",
            None => "?",
        };
        let sha = e.sha256.as_deref().unwrap_or("-");
        let sha_short = if sha.len() > 16 { &sha[..16] } else { sha };
        println!(
            "    {:<36} {:>8}  ent={:<5} magic={:<8} sha256={}…",
            e.path, e.size, ent, magic, sha_short
        );
    }

    if !report.threats.is_empty() {
        println!("\n  [THREATS — {}]", report.threats.len());
        for t in &report.threats {
            println!("    [{}] {} — {} ({})", t.severity, t.file, t.threat, t.category);
        }
    }

    if !report.anomalies.is_empty() {
        println!("\n  [ANOMALIES — {}]", report.anomalies.len());
        for a in &report.anomalies {
            println!("    [{}] {} — {}", a.severity, a.file, a.issue);
        }
    }
}

#[test]
fn forensic_examiner_case_workflow() {
    let dir = exam_dir();
    let staging = dir.join("staging");
    let evidence_zip = dir.join("CASE-2026-0042-evidence.zip");
    let extract_dir = dir.join("extracted_artifacts");
    fs::create_dir_all(staging.join("artifacts")).unwrap();
    fs::create_dir_all(staging.join("documents")).unwrap();
    fs::create_dir_all(staging.join("logs")).unwrap();

    println!("\n>>> PHASE 1 — Evidence acquisition (build examiner test bundle)");
    build_evidence_staging(&staging);
    zip_staging(&staging, &evidence_zip);
    assert!(evidence_zip.exists());

    let zip_path = evidence_zip.to_string_lossy().into_owned();

    println!("\n>>> PHASE 2 — Chain of custody: container hash");
    let container_hashes = hash_archive_sync(zip_path.clone()).expect("hash container");
    println!("  MD5:    {}", container_hashes.md5.as_deref().unwrap_or("-"));
    println!("  SHA-1:  {}", container_hashes.sha1.as_deref().unwrap_or("-"));
    println!("  SHA-256:{}", container_hashes.sha256.as_deref().unwrap_or("-"));

    println!("\n>>> PHASE 3 — Triage: metadata load (Inspect → Load)");
    let meta = inspect_archive_sync(zip_path.clone(), None).expect("inspect metadata");
    println!("  {} files listed, format={}", meta.total_files, meta.format);

    println!("\n>>> PHASE 4 — Full forensic scan (Inspect → Full Scan)");
    let report = forensic_scan_archive_sync(zip_path.clone(), None).expect("full scan");
    print_report("FORENSIC SCAN REPORT — CASE-2026-0042", &report);

    assert!(report.total_files >= 5, "expected multiple evidence files");
    assert!(
        report.threats.iter().any(|t| t.category == "spoofing" || t.category == "executable"),
        "examiner should flag spoofed/double-extension artifacts"
    );
    assert!(
        report.anomalies.iter().any(|a| a.issue.contains("entropy") || a.issue.contains("Zero-byte")),
        "examiner should flag entropy or zero-byte anomalies"
    );

    println!("\n>>> PHASE 5 — Targeted preview (non-destructive examination)");
    let preview = preview_archive_entry_sync(
        zip_path.clone(),
        "confidential_report.txt".into(),
        None,
    )
    .expect("preview confidential report");
    assert!(preview.text.as_ref().map(|t| t.contains("confidential")).unwrap_or(false));
    println!("  Preview confidential_report.txt: {} chars, type={}", preview.text.as_ref().map(|t| t.len()).unwrap_or(0), preview.preview_type);

    println!("\n>>> PHASE 6 — Selective extraction (Extract Selected)");
    fs::create_dir_all(&extract_dir).unwrap();
    let payload_path = report
        .entries
        .iter()
        .find(|e| e.path.contains("encrypted_payload"))
        .map(|e| e.path.clone())
        .expect("encrypted payload in report");

    let selected = vec!["evidence_manifest.txt".into(), payload_path.clone()];
    let partial = extract_archive_entries_sync(
        zip_path.clone(),
        extract_dir.to_string_lossy().into_owned(),
        selected,
        None,
    )
    .expect("partial extract");
    assert!(partial.success);
    assert!(extract_dir.join("evidence_manifest.txt").exists());
    assert!(extract_dir.join(&payload_path).exists());
    println!("  Extracted {} selected artifacts to {}", partial.files_processed, extract_dir.display());

    println!("\n>>> EXAMINER CONCLUSION");
    println!("  Bundle suitable for Inspect-tab forensic workflow.");
    println!("  Risk label: {} — review flagged entries before legal use.", report.risk_label);
    println!("  All operations offline; no network exfiltration.\n");
}
