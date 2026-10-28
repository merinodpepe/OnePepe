// OnePepe: automatic backups with versions. Before a page is overwritten on save, the previous version is copied
// to a backup folder (outside the notebooks), keeping the newest ones.

// Imports
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How many versions are kept of each page.
pub const KEEP_DEFAULT: usize = 10;
/// A new version is made at most this often, so autosave does not fill the backups in minutes.
pub const MIN_INTERVAL_DEFAULT: Duration = Duration::from_secs(20 * 60);

/// The backup folder of a page: one folder per notebook folder, with a readable name.
pub fn backup_dir_for(file: &Path, backup_root: &Path) -> PathBuf {
    let parent = file
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let name: String = parent
        .trim_start_matches('/')
        .chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == ':' {
                '_'
            } else {
                c
            }
        })
        .collect();
    backup_root.join(if name.is_empty() {
        "_".to_string()
    } else {
        name
    })
}

fn versions_of(dir: &Path, stem: &str, ext: &str) -> Vec<PathBuf> {
    let prefix = format!("{stem}.");
    let suffix = format!(".{ext}");
    let mut versions: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.file_name()
                        .map(|n| n.to_string_lossy())
                        .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(&suffix))
                })
                .collect()
        })
        .unwrap_or_default();
    // the timestamp in the name sorts chronologically
    versions.sort();
    versions
}

/// Copies `file` (if it exists) to its backup folder as `<name>.<YYYYmmdd-HHMMSS>.<ext>`, unless the newest backup is
/// younger than `min_interval`, and deletes the oldest versions beyond `keep`.
///
/// Returns the path of the new backup, if one was made.
pub fn backup_before_save(
    file: &Path,
    backup_root: &Path,
    keep: usize,
    min_interval: Duration,
    now: chrono::NaiveDateTime,
) -> std::io::Result<Option<PathBuf>> {
    if !file.is_file() {
        return Ok(None);
    }
    let stem = file
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "pagina".to_string());
    let ext = file
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "rnote".to_string());
    let dir = backup_dir_for(file, backup_root);
    std::fs::create_dir_all(&dir)?;

    let versions = versions_of(&dir, &stem, &ext);
    if let Some(newest) = versions.last()
        && let Ok(modified) = std::fs::metadata(newest).and_then(|m| m.modified())
        && modified.elapsed().unwrap_or(Duration::MAX) < min_interval
    {
        return Ok(None);
    }

    let target = dir.join(format!("{stem}.{}.{ext}", now.format("%Y%m%d-%H%M%S")));
    std::fs::copy(file, &target)?;

    let versions = versions_of(&dir, &stem, &ext);
    if versions.len() > keep {
        for old in &versions[..versions.len() - keep] {
            let _ = std::fs::remove_file(old);
        }
    }
    Ok(Some(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(min: u32) -> chrono::NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(2026, 10, 28)
            .unwrap()
            .and_hms_opt(10, min, 0)
            .unwrap()
    }

    #[test]
    fn keeps_the_newest_versions() {
        let tmp = std::env::temp_dir().join(format!("onepepe-backup-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let notebook = tmp.join("cuaderno");
        std::fs::create_dir_all(&notebook).unwrap();
        let page = notebook.join("tema 1.rnote");
        let root = tmp.join("copias");

        // no file yet: nothing to back up
        assert!(
            backup_before_save(&page, &root, 3, Duration::ZERO, time(0))
                .unwrap()
                .is_none()
        );
        std::fs::write(&page, b"v1").unwrap();
        for m in 0..5 {
            backup_before_save(&page, &root, 3, Duration::ZERO, time(m)).unwrap();
        }
        let dir = backup_dir_for(&page, &root);
        let versions = versions_of(&dir, "tema 1", "rnote");
        assert_eq!(versions.len(), 3);
        assert!(
            versions[0]
                .to_string_lossy()
                .ends_with("tema 1.20261028-100200.rnote")
        );
        assert_eq!(std::fs::read(&versions[2]).unwrap(), b"v1");

        // a fresh backup blocks a new one within the interval
        assert!(
            backup_before_save(&page, &root, 3, Duration::from_secs(3600), time(9))
                .unwrap()
                .is_none()
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn readable_folder_names() {
        let dir = backup_dir_for(
            Path::new("/home/pepe/Documentos/Biofísica/tema.rnote"),
            Path::new("/copias"),
        );
        assert_eq!(dir, Path::new("/copias/home_pepe_Documentos_Biofísica"));
    }
}
