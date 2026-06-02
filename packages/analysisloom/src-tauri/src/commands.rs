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
