use serde::{Serialize, Deserialize};
use ysf_core::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Case {
    pub id: String,
    pub name: String,
    pub operator: Option<String>,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub file_path: String,
    pub offset: u64,
    pub context: String,
}

// ─── Case Management ───

#[tauri::command]
pub fn list_cases() -> Result<Vec<Case>, String> {
    let db = crate::db::conn();
    let mut stmt = db.prepare("SELECT id, name, operator, created_at, status FROM cases ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    let cases = stmt.query_map([], |row| {
        Ok(Case {
            id: row.get(0)?,
            name: row.get(1)?,
            operator: row.get(2)?,
            created_at: row.get(3)?,
            status: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(cases)
}

#[tauri::command]
pub fn create_case(name: String, operator: String) -> Result<Case, String> {
    let db = crate::db::conn();
    let id = evidence::EvidenceId::new("ANL").to_string();
    db.execute("INSERT INTO cases (id, name, operator) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, name, operator])
        .map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    Ok(Case { id, name, operator: Some(operator), created_at: now, status: "active".into() })
}

#[tauri::command]
pub fn get_case(id: String) -> Result<Case, String> {
    let db = crate::db::conn();
    db.query_row("SELECT id, name, operator, created_at, status FROM cases WHERE id = ?1",
        [&id], |row| Ok(Case {
            id: row.get(0)?, name: row.get(1)?, operator: row.get(2)?,
            created_at: row.get(3)?, status: row.get(4)?,
        })).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_case(id: String) -> Result<(), String> {
    crate::db::conn().execute("DELETE FROM cases WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── NTFS Browser ───

#[tauri::command]
pub fn parse_mft(image_path: String) -> Result<Vec<ntfs::MftEntry>, String> {
    let cancel = std::sync::atomic::AtomicBool::new(false);
    ntfs::parse_mft(&image_path, &cancel)
}

// ─── File Carving ───

#[tauri::command]
pub async fn start_carving(image_path: String, output_dir: String) -> Result<(), String> {
    CANCEL_FLAG.store(false, std::sync::atomic::Ordering::SeqCst);
    *PROGRESS_STATE.lock().unwrap() = ProgressState::default();
    let cancel = CANCEL_FLAG.clone();

    tokio::task::spawn_blocking(move || {
        let _ = carving::carve_files(&image_path, &output_dir, &cancel);
    });

    Ok(())
}

#[tauri::command]
pub fn get_carving_progress() -> Result<ProgressState, String> {
    PROGRESS_STATE.lock().map(|s| s.clone()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_carving() {
    CANCEL_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
}

// ─── Timeline ───

#[tauri::command]
pub fn get_timeline(case_id: String) -> Result<Vec<serde_json::Value>, String> {
    let db = crate::db::conn();
    let mut stmt = db.prepare(
        "SELECT timestamp, source, file_path, event_type FROM timeline_events WHERE case_id = ?1 ORDER BY timestamp DESC LIMIT 200"
    ).map_err(|e| e.to_string())?;
    let events = stmt.query_map([case_id], |row| {
        Ok(serde_json::json!({
            "timestamp": row.get::<_, String>(0)?,
            "source": row.get::<_, String>(1)?,
            "filePath": row.get::<_, String>(2)?,
            "eventType": row.get::<_, String>(3)?,
        }))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(events)
}

// ─── Keyword Search ───

#[tauri::command]
pub fn keyword_search(case_id: String, query: String) -> Result<Vec<SearchResult>, String> {
    let regex = regex::Regex::new(&format!("(?i){}", regex::escape(&query)))
        .map_err(|e| format!("Invalid regex: {}", e))?;

    // Search evidence items for the case
    let db = crate::db::conn();
    let mut stmt = db.prepare(
        "SELECT source_path, sha256 FROM evidence_items WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;

    let mut results = vec![];
    let items: Vec<(String, Option<String>)> = stmt.query_map([&case_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    for (path, _) in items {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for (line_no, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    results.push(SearchResult {
                        file_path: path.clone(),
                        offset: line_no as u64,
                        context: line.to_string(),
                    });
                }
            }
        }
    }

    Ok(results)
}

// ─── File Preview ───

#[tauri::command]
pub fn preview_file(path: String) -> Result<ysf_core::preview::PreviewResult, String> {
    ysf_core::preview::preview_file(&path)
}

// ─── Report Generation ───

#[tauri::command]
pub fn generate_case_report(case_id: String, format: String) -> Result<String, String> {
    let db = crate::db::conn();

    // Get case info
    let case: Case = db.query_row(
        "SELECT id, name, operator, created_at, status FROM cases WHERE id = ?1",
        [&case_id], |row| Ok(Case {
            id: row.get(0)?, name: row.get(1)?, operator: row.get(2)?,
            created_at: row.get(3)?, status: row.get(4)?,
        }),
    ).map_err(|e| format!("Case not found: {e}"))?;

    // Get timeline events
    let mut stmt = db.prepare(
        "SELECT timestamp, source, file_path, event_type FROM timeline_events WHERE case_id = ?1 ORDER BY timestamp DESC LIMIT 100"
    ).map_err(|e| e.to_string())?;
    let timeline: Vec<String> = stmt.query_map([&case_id], |row| {
        Ok(format!("{} | {} | {} ({})",
            row.get::<_, String>(0)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(1)?,
        ))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // Get evidence items
    let mut stmt = db.prepare(
        "SELECT source_path, type, sha256, size_bytes FROM evidence_items WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;
    let evidence: Vec<String> = stmt.query_map([&case_id], |row| {
        Ok(format!("{} ({}) — {} bytes — SHA256: {}",
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(3).unwrap_or(0),
            row.get::<_, String>(2).unwrap_or_default(),
        ))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // Get findings
    let mut stmt = db.prepare(
        "SELECT description, file_path, severity FROM findings WHERE case_id = ?1"
    ).map_err(|e| e.to_string())?;
    let findings: Vec<String> = stmt.query_map([&case_id], |row| {
        Ok(format!("[{}] {} — {}",
            row.get::<_, String>(2)?,
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
        ))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // Get audit trail
    let mut stmt = db.prepare(
        "SELECT timestamp, action, detail FROM audit_log WHERE case_id = ?1 ORDER BY timestamp DESC LIMIT 50"
    ).map_err(|e| e.to_string())?;
    let audit: Vec<String> = stmt.query_map([&case_id], |row| {
        Ok(format!("{} | {} — {}",
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let operator_name = case.operator.clone().unwrap_or_default();

    if format == "html" {
        // Generate HTML report
        let html = generate_html_report(&case, &timeline, &evidence, &findings, &audit, &now);
        let dir = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let out_path = format!("{}/analysisloom_report_{}.html", dir, &case_id[..8.min(case_id.len())]);
        std::fs::write(&out_path, &html).map_err(|e| format!("Write error: {e}"))?;
        Ok(out_path)
    } else {
        // Generate PDF
        let sections = vec![
            report::ReportSection {
                heading: "Case Information".into(),
                content: format!("Case: {}\nOperator: {}\nStatus: {}\nCreated: {}",
                    case.name, operator_name, case.status, case.created_at),
            },
            report::ReportSection {
                heading: "Timeline Events".into(),
                content: if timeline.is_empty() { "No timeline events recorded.".into() }
                    else { timeline.join("\n") },
            },
            report::ReportSection {
                heading: "Evidence Items".into(),
                content: if evidence.is_empty() { "No evidence items recorded.".into() }
                    else { evidence.join("\n") },
            },
            report::ReportSection {
                heading: "Findings".into(),
                content: if findings.is_empty() { "No findings recorded.".into() }
                    else { findings.join("\n") },
            },
            report::ReportSection {
                heading: "Audit Trail".into(),
                content: if audit.is_empty() { "No audit log entries.".into() }
                    else { audit.join("\n") },
            },
        ];

        let pdf = report::generate_pdf_report(&report::PdfReport {
            title: format!("Forensic Analysis Report — {}", case.name),
            evidence_id: case_id.clone(),
            operator: operator_name.clone(),
            case_name: case.name,
            device: "AnalysisLoom Workstation".into(),
            date: now,
            sections,
        })?;

        let dir = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let out_path = format!("{}/analysisloom_report_{}.pdf", dir, &case_id[..8.min(case_id.len())]);
        std::fs::write(&out_path, &pdf).map_err(|e| format!("Write error: {e}"))?;
        Ok(out_path)
    }
}

fn generate_html_report(
    case: &Case, timeline: &[String], evidence: &[String],
    findings: &[String], audit: &[String], now: &str,
) -> String {
    let list = |items: &[String]| -> String {
        if items.is_empty() { "<p><em>None recorded</em></p>".into() }
        else { items.iter().map(|i| format!("<li>{}</li>", html_escape(i))).collect::<Vec<_>>().join("\n") }
    };
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><title>AnalysisLoom Report — {name}</title>
<style>
  body {{ font-family: -apple-system, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px;
         background: #0a0a0a; color: #e0e0e0; }}
  h1 {{ border-bottom: 2px solid #3b82f6; padding-bottom: 8px; }}
  h2 {{ color: #3b82f6; margin-top: 28px; }}
  .meta {{ color: #888; font-size: 13px; margin-bottom: 24px; }}
  ul {{ background: #111; border: 1px solid #222; border-radius: 8px; padding: 12px 32px; }}
  li {{ margin: 4px 0; font-size: 12px; font-family: monospace; }}
  .footer {{ margin-top: 40px; padding-top: 12px; border-top: 1px solid #222; font-size: 11px; color: #555; }}
</style></head>
<body>
  <h1>Forensic Analysis Report</h1>
  <div class="meta">
    <strong>Case:</strong> {name} &nbsp;|&nbsp;
    <strong>ID:</strong> {id} &nbsp;|&nbsp;
    <strong>Operator:</strong> {op} &nbsp;|&nbsp;
    <strong>Status:</strong> {status}<br/>
    <strong>Generated:</strong> {now}
  </div>

  <h2>📊 Timeline Events</h2>
  <ul>{timeline}</ul>

  <h2>📦 Evidence Items</h2>
  <ul>{evidence}</ul>

  <h2>🔍 Findings</h2>
  <ul>{findings}</ul>

  <h2>📋 Audit Trail</h2>
  <ul>{audit}</ul>

  <div class="footer">
    Generated by AnalysisLoom — YSF Studio | 100% Offline Forensic Workstation<br/>
    This report is provided AS-IS. Verify independently before use in legal proceedings.
  </div>
</body></html>"#,
    name = case.name, id = case.id, op = case.operator.as_deref().unwrap_or("—"),
    status = case.status, now = now, timeline = list(timeline),
    evidence = list(evidence), findings = list(findings), audit = list(audit),
  )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('"', "&quot;").replace('\'', "&#39;")
}

// ─── Audit Logging ───

#[tauri::command]
pub fn log_action(case_id: String, action: String, detail: String) -> Result<(), String> {
    crate::db::conn().execute(
        "INSERT INTO audit_log (case_id, action, detail) VALUES (?1, ?2, ?3)",
        rusqlite::params![case_id, action, detail],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_audit_log(case_id: String) -> Result<Vec<serde_json::Value>, String> {
    let db = crate::db::conn();
    let mut stmt = db.prepare(
        "SELECT timestamp, action, detail FROM audit_log WHERE case_id = ?1 ORDER BY timestamp DESC LIMIT 100"
    ).map_err(|e| e.to_string())?;
    let entries = stmt.query_map([case_id], |row| {
        Ok(serde_json::json!({
            "timestamp": row.get::<_, String>(0)?,
            "action": row.get::<_, String>(1)?,
            "detail": row.get::<_, String>(2)?,
        }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(entries)
}

#[tauri::command]
pub fn about_info() -> serde_json::Value {
    serde_json::json!({
        "appName": "AnalysisLoom",
        "version": "0.1.0",
        "developer": "YSF Studio — Built with ❤️ by Yusuf Shalahuddin",
        "build": "Master Build — All Features Unlocked",
        "features": [
            "Forensic-grade NTFS/MFT Parser & File Browser",
            "File Carving with multi-format signature detection",
            "Timeline Analysis & Event Correlation",
            "Multi-format file preview (text, image, hex, archive)",
            "SQLite-based Case Management with Audit Trail",
            "Chain of Custody tracking with SHA-256 verification",
            "100% Offline — Zero Data Collection. All processing runs locally."
        ],
        "disclaimer": "This software is provided 'AS-IS'. Results should be independently verified before use in legal proceedings.",
        "offline": true,
        "privacy": "100% offline — zero data collection. No telemetry, no analytics, no external network calls."
    })
}
