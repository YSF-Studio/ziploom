use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceId {
    pub prefix: String,    // "COL" or "ANL"
    pub date: String,      // "20260602"
    pub sequence: u16,     // 0001
}

impl EvidenceId {
    pub fn new(prefix: &str) -> Self {
        let date = Utc::now().format("%Y%m%d").to_string();
        // Read counter from ~/.ysf/evidence_counter.json
        let counter_path = dirs_next().unwrap_or_else(|| PathBuf::from("."))
            .join(".ysf").join("evidence_counter.json");
        let sequence = match std::fs::read_to_string(&counter_path) {
            Ok(s) => {
                let current: u16 = s.trim().parse().unwrap_or(0);
                let next = current + 1;
                let _ = std::fs::create_dir_all(counter_path.parent().unwrap());
                let _ = std::fs::write(&counter_path, next.to_string());
                next
            }
            Err(_) => {
                let _ = std::fs::create_dir_all(counter_path.parent().unwrap());
                let _ = std::fs::write(&counter_path, "1");
                1
            }
        };
        Self { prefix: prefix.to_string(), date, sequence }
    }

    pub fn to_string(&self) -> String {
        format!("{}-{}-{:04}", self.prefix, self.date, self.sequence)
    }
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
}

/// Single action in the chain of custody log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLog {
    pub timestamp: String,
    pub operator: String,
    pub action: String,
    pub details: String,
    pub hash: Option<String>,
}

/// Complete chain of custody for an evidence collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfCustody {
    pub evidence_id: String,
    pub case_name: String,
    pub operator: String,
    pub source_device: String,
    pub source_size_bytes: u64,
    pub actions: Vec<ActionLog>,
    pub final_hashes: Option<super::hashing::HashSet>,
    pub signature: Option<Vec<u8>>,
}

impl ChainOfCustody {
    pub fn new(case_name: &str, operator: &str, source_device: &str, source_size: u64) -> Self {
        let eid = EvidenceId::new("COL");
        Self {
            evidence_id: eid.to_string(),
            case_name: case_name.to_string(),
            operator: operator.to_string(),
            source_device: source_device.to_string(),
            source_size_bytes: source_size,
            actions: vec![],
            final_hashes: None,
            signature: None,
        }
    }

    pub fn add_action(&mut self, action: &str, details: &str, hash: Option<&str>) {
        self.actions.push(ActionLog {
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            operator: self.operator.clone(),
            action: action.to_string(),
            details: details.to_string(),
            hash: hash.map(|h| h.to_string()),
        });
    }

    pub fn set_final_hashes(&mut self, hashes: super::hashing::HashSet) {
        self.final_hashes = Some(hashes);
    }

    pub fn sign(&mut self, private_key: &[u8]) -> Result<(), String> {
        let data = serde_json::to_string(&self).map_err(|e| e.to_string())?;
        let sig = super::crypto::sign_data(private_key, data.as_bytes())?;
        self.signature = Some(sig);
        Ok(())
    }
}

/// Generate QR code PNG for evidence label
pub fn generate_qr_label(evidence_id: &str, device: &str, case: &str) -> Vec<u8> {
    let text = format!("EID:{}\nDEV:{}\nCASE:{}", evidence_id, device, case);
    let code = qrcode::QrCode::new(text.as_bytes()).unwrap();
    let image = code.render::<qrcode::render::unicode::Dense1x2>().build();
    // Return as simple representation
    format!("QR Label for {}\n{}", evidence_id, image).into_bytes()
}
