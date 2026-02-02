use crate::config::{self, Config};
use crate::wallpaper::copy_to_current_wallpaper;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct WallhavenResponse {
    data: Vec<WallhavenItem>,
}

#[derive(serde::Deserialize)]
struct WallhavenItem {
    path: String,
}

pub async fn fetch_wallhaven_wallpaper(cfg: &Config) -> Result<Option<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = cfg.wallhaven_api_key.trim().to_string();
    let purity = cfg.wallhaven_purity.trim().to_string();
    let categories = cfg.wallhaven_categories.trim().to_string();

    let url = format!(
        "https://wallhaven.cc/api/v1/search?apikey={}&categories={}&purity={}&atleast=3840x2160&ratios=landscape&sorting=random",
        urlencoding::encode(&api_key),
        urlencoding::encode(&categories),
        urlencoding::encode(&purity)
    );

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36")
        .build()?;

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }

    let payload: WallhavenResponse = resp.json().await?;
    let first = match payload.data.first() {
        Some(it) => it,
        None => return Ok(None),
    };

    let img_url = &first.path;

    let dest_dir = config::save_wallpaper_path(cfg);
    fs::create_dir_all(&dest_dir)?;

    let filename = file_name_from_url(img_url).unwrap_or_else(|| "wallhaven_random.jpg".to_string());
    let dest_path = dest_dir.join(filename);
    let mut img_resp = client.get(img_url).send().await?;

    if !img_resp.status().is_success() {
        return Ok(None);
    }

    let mut file = fs::File::create(&dest_path)?;
    while let Some(chunk) = img_resp.chunk().await? {
        file.write_all(&chunk)?;
    }

    let _ = copy_to_current_wallpaper(&dest_path);

    Ok(Some(dest_path))
}

fn file_name_from_url(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    parsed
        .path_segments()
        .and_then(|segs| segs.last())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}
