use serde::Serialize;
use std::sync::atomic::Ordering;

#[derive(Debug, Clone, Serialize)]
pub struct NetworkCaptureConfig {
    pub interface: String,
    pub bpf_filter: Option<String>,
    pub output_file: String,
    pub ring_buffer_size: u64,
    pub max_duration_secs: u64,
}

/// Start network capture using pcap crate
pub fn start_capture(config: NetworkCaptureConfig, cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<String, String> {
    use pcap::{Capture, Device};

    let device = Device::list()
        .map_err(|e| format!("Cannot list network devices: {}", e))?
        .into_iter()
        .find(|d| d.name == config.interface)
        .ok_or_else(|| format!("Interface '{}' not found", config.interface))?;

    let mut cap = Capture::from_device(device)
        .map_err(|e| format!("Cannot open device: {}", e))?
        .promisc(true)
        .snaplen(65535) // Max packet size
        .timeout(1000)
        .open()
        .map_err(|e| format!("Cannot start capture: {}", e))?;

    // Apply BPF filter if specified
    if let Some(ref filter) = config.bpf_filter {
        cap.filter(filter, true)
            .map_err(|e| format!("Invalid BPF filter: {}", e))?;
    }

    // Save to file
    let mut savefile = cap.savefile(&config.output_file)
        .map_err(|e| format!("Cannot create savefile: {}", e))?;

    let mut packet_count: u64 = 0;
    let mut bytes_captured: u64 = 0;
    let started = std::time::Instant::now();

    loop {
        if cancel_flag.load(Ordering::SeqCst) { break; }
        if config.max_duration_secs > 0
            && started.elapsed().as_secs() >= config.max_duration_secs { break; }

        match cap.next_packet() {
            Ok(packet) => {
                savefile.write(&packet);
                packet_count += 1;
                bytes_captured += packet.len() as u64;

                super::progress::update_progress(
                    (bytes_captured as f64 / config.ring_buffer_size.max(1) as f64 * 100.0).min(100.0),
                    &format!("Capturing: {} pkt, {:.1} MB", packet_count, bytes_captured as f64 / 1e6),
                    bytes_captured,
                    config.ring_buffer_size.max(1),
                );
            }
            Err(pcap::Error::TimeoutExpired) => continue,
            Err(e) => {
                return Err(format!("Capture error: {}", e));
            }
        }
    }

    // Auto-hash the capture file
    let hash = super::hashing::multi_hash(
        std::path::Path::new(&config.output_file),
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let result = format!(
        "Capture complete: {} packets, {:.1} MB, SHA256: {}",
        packet_count,
        bytes_captured as f64 / 1e6,
        hash.sha256.unwrap_or_default()
    );

    super::progress::finish_progress(Ok(result.clone()));
    Ok(result)
}

/// List available network interfaces
pub fn list_interfaces() -> Result<Vec<String>, String> {
    pcap::Device::list()
        .map(|devices| devices.into_iter().map(|d| d.name).collect())
        .map_err(|e| format!("Cannot list devices: {}", e))
}
