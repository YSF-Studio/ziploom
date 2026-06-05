use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Default)]
pub struct ProgressState {
    pub percent: f64,
    pub status: String,
    pub is_done: bool,
    pub error: Option<String>,
    pub eta_secs: Option<f64>,
    pub bytes_processed: u64,
    pub total_bytes: u64,
}

pub struct CancelFlag {
    flag: Arc<AtomicBool>,
}

impl CancelFlag {
    pub fn new() -> Self {
        Self { flag: Arc::new(AtomicBool::new(false)) }
    }
    pub fn cancel(&self) { self.flag.store(true, Ordering::SeqCst); }
    pub fn is_cancelled(&self) -> bool { self.flag.load(Ordering::SeqCst) }
    pub fn reset(&self) { self.flag.store(false, Ordering::SeqCst); }
    pub fn clone_arc(&self) -> Arc<AtomicBool> { self.flag.clone() }
}

pub fn set_cancel_flag(flag: Arc<AtomicBool>) {
    *CANCEL_FLAG_MUTEX.lock().unwrap() = Some(flag);
}

pub fn is_cancelled() -> bool {
    CANCEL_FLAG_MUTEX.lock().unwrap()
        .as_ref().map(|f| f.load(Ordering::SeqCst)).unwrap_or(false)
}

/// Reset progress for a new long-running operation.
pub fn reset_progress(status: &str) {
    if let Ok(mut p) = super::PROGRESS_STATE.lock() {
        *p = ProgressState {
            percent: 0.0,
            status: status.to_string(),
            is_done: false,
            error: None,
            eta_secs: None,
            bytes_processed: 0,
            total_bytes: 0,
        };
    }
    *super::OPERATION_RESULT.lock().unwrap() = None;
}

/// Read current progress snapshot.
pub fn get_progress() -> ProgressState {
    super::PROGRESS_STATE
        .lock()
        .map(|p| p.clone())
        .unwrap_or_default()
}

/// Update progress state (thread-safe)
pub fn update_progress(percent: f64, status: &str, bytes: u64, total: u64) {
    if let Ok(mut p) = super::PROGRESS_STATE.lock() {
        p.percent = percent;
        p.status = status.to_string();
        p.bytes_processed = bytes;
        p.total_bytes = total;
    }
}

/// Mark operation as done
pub fn finish_progress(result: Result<String, String>) {
    if let Ok(mut p) = super::PROGRESS_STATE.lock() {
        p.is_done = true;
        p.percent = 100.0;
        p.status = "Complete".to_string();
        match &result {
            Ok(_) => p.error = None,
            Err(e) => p.error = Some(e.clone()),
        }
    }
    *super::OPERATION_RESULT.lock().unwrap() = Some(result);
}

use once_cell::sync::Lazy;
static CANCEL_FLAG_MUTEX: Lazy<Mutex<Option<Arc<AtomicBool>>>> = Lazy::new(|| Mutex::new(None));
