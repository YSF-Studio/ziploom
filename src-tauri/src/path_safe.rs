/// Sanitasi path entry archive untuk mencegah path traversal.
/// - Strips leading `/` dan `\`
/// - Membuang komponen `..` (path traversal)
/// - Menolak absolute paths (`/...`, `C:\...`)
/// - Return None kalau path traversal terdeteksi
pub fn safe_entry_path(name: &str) -> Option<String> {
    let name = name.trim_start_matches('/').trim_start_matches('\\');

    // Absolute path detection
    if name.starts_with('/') || name.starts_with('\\') {
        return None;
    }
    #[cfg(windows)]
    {
        if name.len() >= 2 && name.as_bytes()[1] == b':' {
            return None; // C:\... or D:\...
        }
    }

    // Path traversal: ../ or ..\\
    let mut safe = String::new();
    for component in name.split(|c| c == '/' || c == '\\') {
        match component {
            "." | "" => continue, // skip current dir dan empty
            ".." => {
                // Pop komponen terakhir kalau ada
                if let Some(pos) = safe.rfind(|c| c == '/' || c == '\\') {
                    safe.truncate(pos);
                } else {
                    safe.clear();
                }
            }
            _ => {
                if !safe.is_empty() {
                    safe.push('/');
                }
                safe.push_str(component);
            }
        }
    }

    if safe.is_empty() { None } else { Some(safe) }
}
