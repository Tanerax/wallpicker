use crate::config::Config;
use crate::wallpaper::copy_to_current_wallpaper;
use std::path::PathBuf;

pub async fn set_wallpaper(path: PathBuf, copy_to_tmp: bool) -> () {
    if copy_to_tmp {
        let _ = copy_to_current_wallpaper(&path);
    }

    let _ = tokio::task::spawn_blocking(move || crate::platform::set_wallpaper(&path)).await;
}

pub async fn set_random_wallpaper(cfg: Config) -> Option<PathBuf> {
    match crate::wallpaper::find_random_wallpaper(&cfg).await {
        Ok(Some(path)) => {
            let ret = path.clone();
            set_wallpaper(path.clone(), cfg.copy_to_tmp).await;

            Some(ret)
        }
        _ => None,
    }
}

pub async fn set_random_wallpaper_via_wallhaven(cfg: Config) -> Option<PathBuf> {
    match crate::wallhaven::fetch_wallhaven_wallpaper(&cfg).await {
        Ok(Some(path)) => {
            let ret = path.clone();
            set_wallpaper(path.clone(), cfg.copy_to_tmp).await;
            Some(ret)
        }
        _ => None,
    }
}
