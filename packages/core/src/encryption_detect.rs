use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct EncryptionReport {
    pub platform: String,
    pub has_fde: bool,
    pub fde_type: Option<FdeType>,
    pub tpm_present: bool,
    pub tpm_version: Option<String>,
    pub secure_boot: Option<String>,
    pub fde_protectors: Vec<String>,
    pub encrypted_partitions: Vec<PartitionInfo>,
    pub recommendations: Vec<String>,
    pub requires_ram_capture: bool,
    pub requires_recovery_key: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum FdeType {
    BitLocker { status: String },
    FileVault { status: String },
    Luks { version: u8, cipher: String },
    VeraCrypt,
    DeviceEncryption,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartitionInfo {
    pub device: String,
    pub mount_point: Option<String>,
    pub file_system: String,
    pub is_encrypted: bool,
    pub encryption_type: Option<String>,
    pub size_bytes: u64,
}

/// Scan all detectable encryption on the current system
pub fn scan_encryption() -> EncryptionReport {
    let platform = if cfg!(target_os = "windows") { "Windows" }
        else if cfg!(target_os = "macos") { "macOS" }
        else if cfg!(target_os = "linux") { "Linux" }
        else { "Unknown" };

    let mut report = EncryptionReport {
        platform: platform.to_string(),
        has_fde: false,
        fde_type: None,
        tpm_present: false,
        tpm_version: None,
        secure_boot: None,
        fde_protectors: vec![],
        encrypted_partitions: vec![],
        recommendations: vec![],
        requires_ram_capture: false,
        requires_recovery_key: false,
    };

    #[cfg(target_os = "windows")]
    scan_windows(&mut report);

    #[cfg(target_os = "macos")]
    scan_macos(&mut report);

    #[cfg(target_os = "linux")]
    scan_linux(&mut report);

    // Generate recommendations
    if report.has_fde {
        report.requires_ram_capture = true;
        report.requires_recovery_key = true;
        report.recommendations.push(
            "⚠️ Full Disk Encryption detected — capture RAM BEFORE shutdown!".into()
        );
        report.recommendations.push(
            "🔑 Request recovery key / password from device owner or IT admin.".into()
        );
    }
    if report.tpm_present {
        report.recommendations.push(
            "🔐 TPM present — BitLocker/LUKS key may be sealed to TPM. RAM capture critical.".into()
        );
    }

    report
}

#[cfg(target_os = "windows")]
fn scan_windows(report: &mut EncryptionReport) {
    // --- BitLocker via manage-bde ---
    if let Ok(output) = Command::new("manage-bde").arg("-status").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            if lower.contains("conversion status") && lower.contains("fully encrypted") {
                report.has_fde = true;
                report.fde_type = Some(FdeType::BitLocker { status: "Fully Encrypted".into() });
            } else if lower.contains("conversion status") && lower.contains("encryption in progress") {
                report.has_fde = true;
                report.fde_type = Some(FdeType::BitLocker { status: "Encrypting".into() });
            }
            if lower.contains("protection") && lower.contains("on") {
                report.fde_type = Some(FdeType::BitLocker { status: "Protected".into() });
            }
        }
    }
    // TPM check
    if let Ok(output) = Command::new("tpmtool").arg("getdeviceinformation").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("TPM Present: true") || stdout.contains("TpmReady: true") {
            report.tpm_present = true;
            report.tpm_version = Some("2.0".into()); // tpmtool only exists on Win10+ with TPM 2.0
        }
    }
    // Secure Boot
    if let Ok(output) = Command::new("powershell").args(["-Command", "Confirm-SecureBootUEFI"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        report.secure_boot = if stdout.contains("True") { Some("Enabled".into()) }
            else if stdout.contains("False") { Some("Disabled".into()) }
            else { None }
    }
}

#[cfg(target_os = "macos")]
fn scan_macos(report: &mut EncryptionReport) {
    if let Ok(output) = Command::new("fdesetup").arg("status").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("FileVault is On") {
            report.has_fde = true;
            report.fde_type = Some(FdeType::FileVault { status: "On".into() });
        } else if stdout.contains("FileVault is Off") {
            report.fde_type = Some(FdeType::FileVault { status: "Off".into() });
        }
    }
    // T2 chip/Secure Enclave
    if let Ok(output) = Command::new("system_profiler").args(["SPHardwareDataType"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("T2") || stdout.contains("Apple") {
            report.tpm_present = true;
            report.tpm_version = Some("Apple Secure Enclave".into());
        }
    }
    report.secure_boot = Some("Apple Secure Boot".into());
}

#[cfg(target_os = "linux")]
fn scan_linux(report: &mut EncryptionReport) {
    // --- LUKS detection ---
    // Check for LUKS metadata via dmsetup
    let has_luks = if let Ok(output) = Command::new("dmsetup").args(["ls", "--target", "crypt"]).output() {
        !String::from_utf8_lossy(&output.stdout).trim().is_empty()
    } else { false };

    let has_luks_alt = std::path::Path::new("/dev/mapper").exists()
        && std::fs::read_dir("/dev/mapper").map(|d| d.count() > 3).unwrap_or(false);

    if has_luks || has_luks_alt {
        report.has_fde = true;

        // Check LUKS version via cryptsetup
        let version = if let Ok(output) = Command::new("cryptsetup").arg("--version").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("cryptsetup 2") { 2u8 } else { 1u8 }
        } else { 2u8 };

        report.fde_type = Some(FdeType::Luks {
            version,
            cipher: "aes-xts-plain64".into(), // Most common, placeholder
        });
    }

    // TPM 2.0 via sysfs
    if std::path::Path::new("/sys/class/tpm/tpm0").exists() {
        report.tpm_present = true;
        let version_path = std::path::Path::new("/sys/class/tpm/tpm0/tpm_version_major");
        if let Ok(ver) = std::fs::read_to_string(version_path) {
            report.tpm_version = Some(format!("{}.0", ver.trim()));
        }
    }

    // Secure Boot
    if let Ok(output) = Command::new("mokutil").arg("--sb-state").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        report.secure_boot = if stdout.contains("SecureBoot enabled") {
            Some("Enabled".into())
        } else {
            Some("Disabled".into())
        };
    }
}
