use crate::config::Config;
use crate::wallpaper::copy_to_current_wallpaper;
use std::path::PathBuf;

pub async fn set_wallpaper(path: PathBuf) -> () {
    let _ = copy_to_current_wallpaper(&path);

    let _ = tokio::task::spawn_blocking(move || {
        if cfg!(target_os = "macos") {
            let script = format!(
                "tell application \"Finder\" to set desktop picture to POSIX file \"{}\"",
                path.display()
            );
            std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .status()
        } else {
            std::process::Command::new("swww")
                .arg("img")
                .arg(&path)
                .arg("--transition-type")
                .arg("outer")
                .arg("--transition-fps")
                .arg("60")
                .status()
        }
    })
    .await;
}

pub async fn set_random_wallpaper(cfg: Config) -> Option<PathBuf> {
    match crate::wallpaper::find_random_wallpaper(&cfg).await {
        Ok(Some(path)) => {
            let ret = path.clone();
            set_wallpaper(path.clone()).await;

            Some(ret)
        }
        _ => None,
    }
}

pub async fn set_random_wallpaper_via_wallhaven(cfg: Config) -> Option<PathBuf> {
    match crate::wallhaven::fetch_wallhaven_wallpaper(&cfg).await {
        Ok(Some(path)) => {
            let ret = path.clone();
            set_wallpaper(path.clone()).await;
            Some(ret)
        }
        _ => None,
    }
}
