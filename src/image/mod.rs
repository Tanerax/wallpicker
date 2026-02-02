use iced::widget::image::Handle as IcedImageHandle;
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

pub const THUMB_SIZE: u32 = 200;

pub async fn load_thumb(path: PathBuf) -> Option<IcedImageHandle> {
    tokio::task::spawn_blocking(move || {
        let cache_path = crate::cache::cached_thumb_path(&path, THUMB_SIZE);

        if let Some(cache) = cache_path {
            if cache.exists() {
                if let Ok(img) = image::open(&cache) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    return Some(IcedImageHandle::from_rgba(w, h, rgba.into_raw()));
                }
            }
        }

        match image::open(&path) {
            Ok(img) => {
                let thumb = img.thumbnail(THUMB_SIZE, THUMB_SIZE).to_rgba8();

                if let Some(cache) = crate::cache::cached_thumb_path(&path, THUMB_SIZE) {
                    if let Some(parent) = cache.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let dynimg = image::DynamicImage::ImageRgba8(thumb.clone());
                    let _ = dynimg.save(&cache);
                }

                let (w, h) = thumb.dimensions();
                let bytes = thumb.into_raw();
                Some(IcedImageHandle::from_rgba(w, h, bytes))
            }
            Err(_e) => None,
        }
    }).await.unwrap_or_else(|_| None)
}

pub fn ensure_thumb_cached<P: AsRef<Path>>(path: P, size: u32) -> io::Result<bool> {
    let path = path.as_ref();
    let Some(cache_path) = crate::cache::cached_thumb_path(path, size) else {
        return Ok(false);
    };

    if cache_path.exists() {
        return Ok(false);
    }

    let img = match image::open(path) {
        Ok(i) => i,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    };
    let thumb = img.thumbnail(size, size).to_rgba8();

    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let tmp_path = cache_path.with_extension(format!("{}.png", nanos));
    let dynamic_image = image::DynamicImage::ImageRgba8(thumb);

    dynamic_image.save(&tmp_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::rename(&tmp_path, &cache_path)?;

    Ok(true)
}