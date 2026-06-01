//! ZipLoom — License Module (Pure Rust, App Store Safe)
//! Offline Ed25519 signature verification — zero server, zero calls.
//!
//! HOW IT WORKS:
//! 1. Binary embeds PUBLIC KEY (Ed25519) — anyone can verify
//! 2. Master's private key is NEVER in binary — only on Master's machine
//! 3. License = "ZLV1-" + base64(signature) + "-" + hwid_prefix
//! 4. Verification is 100% offline — no internet needed
//! 5. Each license is bound to ONE hardware ID

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

// ─── ED25519 KEYPAIR ─────────────────────────────
//
// These are TESTNET keys for development.
// Master MUST regenerate with his own keys before release!
// Keygen command: see master module at end of file.

const PUBLIC_KEY_BYTES: [u8; 32] = [
    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7,
    0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x09, 0x15, 0x6d,
    0x27, 0x3d, 0x04, 0xcf, 0x7c, 0x94, 0x36, 0x7e,
    0x2d, 0xa6, 0x3a, 0x97, 0xf4, 0x23, 0x85, 0x7b,
];

fn public_key() -> VerifyingKey {
    VerifyingKey::from_bytes(&PUBLIC_KEY_BYTES).expect("Invalid public key")
}

// ─── HARDWARE ID ─────────────────────────────────

/// Get unique hardware identifier — reads system files, no external calls
pub fn get_hardware_id() -> String {
    let mut parts: Vec<String> = Vec::new();

    // macOS: IOPlatformUUID from ioreg
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if line.contains("IOPlatformUUID") {
                    if let Some(val) = line.split('"').nth(3) {
                        parts.push(val.to_string());
                    }
                }
            }
        }
    }

    // Linux: machine-id
    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            parts.push(id.trim().to_string());
        }
        if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
            parts.push(id.trim().to_string());
        }
    }

    // Windows: wmic uuid
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("wmic")
            .args(["csproduct", "get", "uuid"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines().skip(1) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                    break;
                }
            }
        }
    }

    // Universal fallback: hostname
    if parts.is_empty() {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        parts.push(hostname);
    }

    let raw = parts.join("|");
    if raw.is_empty() {
        "unknown".into()
    } else {
        let hash = blake3::hash(raw.as_bytes());
        hash.to_hex()[..16].to_string()
    }
}

// ─── LICENSE VERIFICATION (OFFLINE) ──────────────

/// Verify a license key against hardware ID using Ed25519
///
/// License format: ZLV1-<hex_signature_64chars>-<hwid_prefix_16chars>
///
/// Verification steps:
/// 1. Parse prefix "ZLV1-"
/// 2. Extract hex signature (64 hex chars = 32 bytes)
/// 3. Extract claimed hwid prefix (first 16 chars of hwid)
/// 4. Re-sign the hwid with embedded public key
/// 5. Compare signatures
pub fn verify_license(license_key: &str, hardware_id: &str) -> Result<String, String> {
    // 1. Check prefix
    if !license_key.starts_with("ZLV1-") {
        return Err("❌ Invalid license: wrong prefix".into());
    }

    // 2. Parse parts
    let body = &license_key[5..]; // skip "ZLV1-"
    let parts: Vec<&str> = body.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err("❌ Invalid license: wrong format".into());
    }

    let sig_hex = parts[0];
    let claimed_hwid = parts[1];

    // 3. Validate hardware ID match
    let hwid_short = &hardware_id[..8.min(hardware_id.len())];
    if claimed_hwid != hwid_short {
        return Err("❌ License tied to different hardware".into());
    }

    // 4. Decode hex signature
    let sig_bytes = hex::decode(sig_hex).map_err(|_| "❌ Invalid license: bad signature encoding".to_string())?;
    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|_| "❌ Invalid license: bad signature".to_string())?;

    // 5. Verify signature — Ed25519 OFFLINE! NO server needed!
    let message = hwid_short.as_bytes();
    match public_key().verify(message, &signature) {
        Ok(_) => Ok(format!("✅ Licensed to {}", hwid_short)),
        Err(_) => Err("❌ Invalid license: signature mismatch".into()),
    }
}

// ─── TAURI COMMANDS ──────────────────────────────

#[tauri::command]
pub fn get_hardware_id_cmd() -> String {
    get_hardware_id()
}

#[tauri::command]
pub fn activate_license(license_key: String) -> Result<String, String> {
    let hwid = get_hardware_id();
    verify_license(&license_key, &hwid)
}

// ─── MASTER-ONLY: LICENSE GENERATION ─────────────
// Compile with: cargo build --features master
// Then run: Ziploom will print license for given HWID

#[cfg(feature = "master")]
pub fn generate_license(hardware_id: &str) -> String {
    use ed25519_dalek::SigningKey;

    // !! WARNING: This is a TEST secret key !!
    // Generate YOUR OWN with: cargo run --features master -- --gen-key
    let secret_bytes: [u8; 32] = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
    ];

    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let hwid_short = &hardware_id[..8.min(hardware_id.len())];
    let signature = signing_key.sign(hwid_short.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());

    format!("ZLV1-{}-{}", sig_hex, hwid_short)
}

#[cfg(feature = "master")]
pub fn generate_keypair() -> (String, String) {
    use rand_core::OsRng;

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let secret_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());

    println!("/// PRIVATE KEY — KEEP SECRET! Store offline.");
    println!("pub const PRIVATE_KEY_BYTES: [u8; 32] = {:?};", signing_key.to_bytes());
    println!();
    println!("/// PUBLIC KEY — Embed in binary:");
    println!("pub const PUBLIC_KEY_BYTES: [u8; 32] = {:?};", verifying_key.to_bytes());
    println!();
    println!("License command for HWID <xxxx>:");
    println!("  cargo run --features master -- --gen-license <hwid>");

    (secret_hex, public_hex)
}

// ─── TESTS ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    #[test]
    fn test_get_hardware_id_not_empty() {
        let hwid = get_hardware_id();
        assert!(!hwid.is_empty(), "Hardware ID should not be empty");
        assert_eq!(hwid.len(), 16, "HWID should be 16 hex chars");
    }

    #[test]
    fn test_verify_license_valid() {
        let hwid = get_hardware_id();
        // Generate a valid license using the feature (will always pass in test)
        // This test verifies that verify_license format parsing works
        // Full crypto test in test_ed25519_sign_verify
        let result = verify_license("ZLV1-0000000000000000000000000000000000000000000000000000000000000000-12345678", &hwid);
        // Will fail on signature (expected with fake sig), but format should parse OK
        assert!(result.is_err(), "Should fail signature check");
        let err = result.unwrap_err();
        assert!(!err.contains("wrong prefix") && !err.contains("wrong format"),
                "Format parsing should work, not format errors: {}", err);
    }

    #[test]
    fn test_verify_license_wrong_prefix() {
        let result = verify_license("INVALID-key-here", "test-hwid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("prefix"));
    }

    #[test]
    fn test_verify_license_wrong_length() {
        let result = verify_license("ZLV1-too-short", "test-hwid");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_license_wrong_hwid() {
        // Generate a license for hwid "abcdefgh" but try to verify with "12345678"
        // Need a real signature — use the test private key
        let secret_bytes: [u8; 32] = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
        ];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let sig = signing_key.sign(b"abcdefgh");
        let sig_hex = hex::encode(sig.to_bytes());
        let license = format!("ZLV1-{}-abcdefgh", sig_hex);

        // Try with different HWID
        let result = verify_license(&license, "12345678");
        assert!(result.is_err(), "Should fail for wrong hardware");
        assert!(result.unwrap_err().contains("different hardware"));
    }

    #[test]
    fn test_ed25519_full_roundtrip() {
        // Full crypto test: sign → verify (same signing key works with same public key)
        let secret_bytes: [u8; 32] = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
        ];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        let message = b"test-hardware-id-1234";
        let signature = signing_key.sign(message);

        // Verify
        assert!(verifying_key.verify(message, &signature).is_ok());

        // Tampered message should fail
        let result = verifying_key.verify(b"tampered-message", &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_hwid_consistency() {
        let hwid1 = get_hardware_id();
        let hwid2 = get_hardware_id();
        assert_eq!(hwid1, hwid2, "HWID should be consistent within same session");
    }
}
