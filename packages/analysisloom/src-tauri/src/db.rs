use rusqlite::Connection;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let db_path = dirs_next().unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".ysf").join("analysisloom.db");
    let _ = std::fs::create_dir_all(db_path.parent().unwrap());
    let conn = Connection::open(&db_path).expect("Cannot open database");
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS cases (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            operator TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            status TEXT DEFAULT 'active'
        );
        CREATE TABLE IF NOT EXISTS evidence_items (
            id TEXT PRIMARY KEY,
            case_id TEXT REFERENCES cases(id),
            source_path TEXT,
            type TEXT,
            sha256 TEXT,
            size_bytes INTEGER,
            acquired_at TEXT
        );
        CREATE TABLE IF NOT EXISTS timeline_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            case_id TEXT REFERENCES cases(id),
            timestamp TEXT,
            source TEXT,
            file_path TEXT,
            event_type TEXT
        );
        CREATE TABLE IF NOT EXISTS findings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            case_id TEXT REFERENCES cases(id),
            description TEXT,
            file_path TEXT,
            severity TEXT DEFAULT 'info'
        );
        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            case_id TEXT REFERENCES cases(id),
            timestamp TEXT DEFAULT (datetime('now')),
            action TEXT NOT NULL,
            detail TEXT DEFAULT ''
        );
    ").expect("Schema creation failed");
    Mutex::new(conn)
});

fn dirs_next() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(std::path::PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(std::path::PathBuf::from))
}

pub fn init() -> Result<(), String> { let _ = &*DB; Ok(()) }
pub fn conn() -> std::sync::MutexGuard<'static, Connection> { DB.lock().unwrap() }
