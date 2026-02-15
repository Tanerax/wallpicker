pub mod tray;

use iced::window::settings::PlatformSpecific;
use std::io;
use std::path::Path;
use std::process::ExitStatus;

pub fn set_wallpaper(path: &Path) -> io::Result<ExitStatus> {
    let script = format!(
        "tell application \"Finder\" to set desktop picture to POSIX file \"{}\"",
        path.display()
    );
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
}

pub fn window_settings(_app_id: &str) -> PlatformSpecific {
    PlatformSpecific::default()
}
