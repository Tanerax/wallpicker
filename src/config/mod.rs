use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub folders: Vec<PathBuf>,
    #[serde(default)]
    pub wallhaven_api_key: String,
    #[serde(default)]
    pub wallhaven_categories: String,
    #[serde(default)]
    pub wallhaven_purity: String,
    #[serde(default = "default_wallhaven_resolution")]
    pub wallhaven_resolution: String,
    #[serde(default)]
    pub copy_to_tmp: bool,
}

fn default_wallhaven_resolution() -> String {
    match screen_size::get_primary_screen_size() {
        Ok((w, h)) => format!("{w}x{h}"),
        Err(_) => String::from("1920x1080"),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            folders: vec![default_wallpapers()],
            wallhaven_api_key: String::new(),
            wallhaven_purity: String::from("100"),
            wallhaven_categories: String::from("111"),
            wallhaven_resolution: default_wallhaven_resolution(),
            copy_to_tmp: false,
        }
    }
}

pub fn save_wallpaper_path(cfg: &Config) -> PathBuf{
    cfg.folders.first().unwrap_or(&default_wallpapers()).to_path_buf()
}

pub fn default_wallpapers() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
    Path::new(&home)
        .join("Pictures")
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));

    Path::new(&home)
        .join(".config")
        .join("wallpicker")
}

pub fn config_file_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_or_create_config() -> Config {
    let cfg_dir = config_dir();
    let _ = fs::create_dir_all(&cfg_dir);
    let cf = config_file_path();

    if cf.exists() {
        return load_config();
    }

    let cfg = Config::default();
    let _ = save_config(&cfg);
    cfg
}

pub fn load_config() -> Config {
    let cf = config_file_path();

    if let Ok(mut f) = fs::File::open(&cf) {
        let mut s = String::new();

        if f.read_to_string(&mut s).is_ok() {
            if let Ok(cfg) = serde_json::from_str::<Config>(&s) {
                let mut cfg = cfg;

                if cfg.wallhaven_api_key.is_empty() {
                    cfg.wallhaven_api_key = String::new();
                }

                return cfg;
            }
        }
    }

    Config::default()
}

pub fn save_config(cfg: &Config) -> std::io::Result<()> {
    let cf = config_file_path();

    if let Some(parent) = cf.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(cfg).unwrap_or_else(|_| String::from("{}"));
    fs::write(cf, data)
}
