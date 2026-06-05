//! YSF Core — Shared forensic library for ZipLoom, CollectionLoom, AnalysisLoom
//!
//! All modules are pure Rust with no Tauri dependencies — testable independently.

pub mod progress;
pub mod hashing;
pub mod crypto;
pub mod evidence;
pub mod encryption_detect;
pub mod imaging;
pub mod write_blocker;
pub mod ram;
pub mod mobile;
pub mod cloud;
pub mod network;
pub mod archive;
pub mod ntfs;
pub mod carving;
pub mod report;
pub mod snapshot;
pub mod preview;

// Re-export commonly used types
pub use progress::{ProgressState, CancelFlag, set_cancel_flag, is_cancelled};
pub use hashing::{multi_hash, compute_entropy, check_magic_bytes, HASH_BUFFER_SIZE};
pub use crypto::{sign_data, verify_signature, generate_keypair, KeypairStore};
pub use evidence::{EvidenceId, ActionLog, ChainOfCustody, generate_qr_label};
pub use encryption_detect::{EncryptionReport, FdeType, scan_encryption};
pub use imaging::{DiskImager, AcquisitionState};
pub use write_blocker::{enable_write_blocker, disable_write_blocker, check_write_blocker};
pub use archive::{
    forensic_load, generate_forensic_report, ForensicReport, FileEntry, Anomaly, Threat,
    FORMATS_SUPPORTED,
};
pub use ntfs::{parse_mft, MftEntry, FileAttribute, DeletedFile};
pub use carving::{carve_files, CarvingResult, CarvedFile, MAGIC_SIGNATURES};
pub use report::generate_pdf_report;

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

/// Global cancel flag shared across all modules
pub static CANCEL_FLAG: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
pub static PROGRESS_STATE: Lazy<Mutex<ProgressState>> = Lazy::new(|| {
    Mutex::new(ProgressState::default())
});
pub static OPERATION_RESULT: Lazy<Mutex<Option<Result<String, String>>>> = Lazy::new(|| Mutex::new(None));

/// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SUITE_NAME: &str = "YSF Forensic Suite";
