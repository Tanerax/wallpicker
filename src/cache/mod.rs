use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use std::collections::HashSet;

pub fn cache_dir() -> PathBuf {
    crate::config::config_dir().join("cache")
}

pub fn cached_thumb_path(img_path: &Path, thumb_size: u32) -> Option<PathBuf> {
    let meta = fs::metadata(img_path).ok();

    let (mtime, len) = if let Some(m) = meta {
        let len = m.len();
        let mt = m
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        (mt, len)
    } else {
        (0, 0)
    };

    let key = format!("{}|{}|{}|{}", img_path.to_string_lossy(), mtime, len, thumb_size);
    let hash = blake3::hash(key.as_bytes()).to_hex().to_string();

    Some(cache_dir().join(format!("{}.png", hash)))
}

pub fn clean_orphan_thumbnails_for_images(images: &[PathBuf], thumb_size: u32) -> std::io::Result<usize> {
    let mut expected: HashSet<PathBuf> = HashSet::new();
    for img in images.iter() {
        if let Some(p) = cached_thumb_path(img, thumb_size) {
            expected.insert(p);
        }
    }

    let cdir = cache_dir();
    let _ = fs::create_dir_all(&cdir);
    let mut removed = 0usize;

    if let Ok(read_dir) = fs::read_dir(&cdir) {
        for ent in read_dir.flatten() {
            let path = ent.path();
            let is_png = path.extension().and_then(|e| e.to_str()).map(|s| s.eq_ignore_ascii_case("png")).unwrap_or(false);
            if !is_png { continue; }

            if !expected.contains(&path) {
                if fs::remove_file(&path).is_ok() {
                    removed += 1;
                }
            }
        }
    }

    Ok(removed)
}
