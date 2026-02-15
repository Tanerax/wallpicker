use iced::window::settings::PlatformSpecific;
use std::io;
use std::path::Path;
use std::process::ExitStatus;

pub fn set_wallpaper(path: &Path) -> io::Result<ExitStatus> {
    std::process::Command::new("swww")
        .arg("img")
        .arg(path)
        .arg("--transition-type")
        .arg("outer")
        .arg("--transition-fps")
        .arg("60")
        .status()
}

pub fn window_settings(app_id: &str) -> PlatformSpecific {
    PlatformSpecific {
        application_id: String::from(app_id),
        override_redirect: false,
    }
}
