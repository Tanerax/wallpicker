use std::fs;
use std::path::{Path, PathBuf};
use crate::config::Config;
use crate::scanner::scan_directories;
use rand::seq::SliceRandom;

pub fn copy_to_current_wallpaper<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let target_dir = Path::new("/tmp");
    let _ = fs::create_dir_all(target_dir);
    let target = target_dir.join("current_wallpaper");
    fs::copy(path, target)?;
    Ok(())
}

pub async fn find_random_wallpaper(cfg: &Config) -> Result<Option<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let wallpapers = scan_directories(cfg.folders.clone()).await;

    if wallpapers.is_empty() {
        return Ok(None);
    }

    let mut rng = rand::thread_rng();
    let selected = wallpapers.choose(&mut rng).cloned();

    Ok(selected)
}