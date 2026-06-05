
/// Enable write blocker for a device
pub fn enable_write_blocker(device: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let file = std::fs::OpenOptions::new()
            .read(true).write(true).open(device)
            .map_err(|e| format!("Cannot open {}: {}", device, e))?;
        let ro: i32 = 1;
        // BLKROSET = 0x125D
        let ret = unsafe { libc::ioctl(file.as_raw_fd(), 0x0000125D, &ro) };
        if ret != 0 { return Err(format!("BLKROSET failed on {} (errno: {})", device, ret)); }
        // Verify
        let test = std::fs::OpenOptions::new().write(true).create(false).truncate(false).open(device);
        if test.is_ok() {
            // Can still write — blocker may not be effective
            return Err(format!("Write blocker MAY NOT be active on {} — proceed with caution", device));
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let status = Command::new("diskutil").args(["mountDisk", "readOnly", device])
            .status().map_err(|e| e.to_string())?;
        if !status.success() { return Err("Failed to set read-only mode on macOS".into()); }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: DeviceIoControl(FSCTL_LOCK_VOLUME) + FSCTL_SET_READ_ONLY_MODE
        Err("Windows write blocker requires administrator elevation via GUI".into())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    { Err("Unsupported platform".into()) }
}

/// Disable write blocker
pub fn disable_write_blocker(device: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let file = std::fs::OpenOptions::new()
            .read(true).write(true).open(device)
            .map_err(|e| format!("Cannot open {}: {}", device, e))?;
        let rw: i32 = 0;
        let ret = unsafe { libc::ioctl(file.as_raw_fd(), 0x0000125D, &rw) };
        if ret != 0 { return Err(format!("BLKROSET clear failed on {}", device)); }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("diskutil").args(["mountDisk", device])
            .status().map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    { Err("Unsupported platform".into()) }
}

/// Check if write blocker is currently active
pub fn check_write_blocker(_device: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("lsblk").args(["-o", "NAME,RO", _device]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().any(|l| l.trim().ends_with('1'))
        } else { false }
    }
    #[cfg(not(target_os = "linux"))]
    { false }
}
