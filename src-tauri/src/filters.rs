/// ZipLoom — Filters Module
/// Menangani pembersihan metadata macOS dari file archive.
use std::path::PathBuf;

/// Bersihkan semua junk file macOS dari direktori
/// Menghapus: .DS_Store, .localized, ._* (Apple Double), __MACOSX
pub fn clean_metadata(dir: &PathBuf) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name == ".DS_Store" || name == ".localized" || name.starts_with("._") {
                let _ = std::fs::remove_file(&p);
            } else if name == "__MACOSX" && p.is_dir() {
                let _ = std::fs::remove_dir_all(&p);
            } else if p.is_dir() {
                clean_metadata(&p);
            }
        }
    }
}

/// Versi boolean — cek apakah ada junk file dalam direktori
pub fn has_junk_files(dir: &PathBuf) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name == ".DS_Store"
                || name == ".localized"
                || name.starts_with("._")
                || (name == "__MACOSX" && p.is_dir())
            {
                return true;
            }
            if p.is_dir() && has_junk_files(&p) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_metadata_removes_ds_store() {
        let dir =
            std::env::temp_dir().join(format!("zl_test_ds_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".DS_Store"), "fake").unwrap();
        std::fs::write(dir.join("real.txt"), "real").unwrap();

        clean_metadata(&dir);

        assert!(!dir.join(".DS_Store").exists(), ".DS_Store should be deleted");
        assert!(dir.join("real.txt").exists(), "real files should remain");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_clean_metadata_removes_apple_double() {
        let dir =
            std::env::temp_dir().join(format!("zl_test_ad_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("._test"), "fake").unwrap();
        std::fs::write(dir.join("test.txt"), "real").unwrap();

        clean_metadata(&dir);

        assert!(!dir.join("._test").exists(), "._ files should be deleted");
        assert!(dir.join("test.txt").exists(), "real files should remain");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_clean_metadata_cleans_nested() {
        let dir =
            std::env::temp_dir().join(format!("zl_test_nest_{}", uuid::Uuid::new_v4()));
        let sub = dir.join("__MACOSX");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join(".DS_Store"), "fake").unwrap();
        std::fs::write(dir.join("real.txt"), "real").unwrap();

        clean_metadata(&dir);

        assert!(!sub.join(".DS_Store").exists(), "nested .DS_Store should be deleted");
        assert!(dir.join("real.txt").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_has_junk_files_detects_ds_store() {
        let dir =
            std::env::temp_dir().join(format!("zl_test_hj_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".DS_Store"), "fake").unwrap();
        assert!(has_junk_files(&dir));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_has_junk_files_clean_dir() {
        let dir =
            std::env::temp_dir().join(format!("zl_test_hj2_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("real.txt"), "hello").unwrap();
        assert!(!has_junk_files(&dir));
        std::fs::remove_dir_all(&dir).ok();
    }
}
