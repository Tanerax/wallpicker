pub mod preview;
pub mod app;
pub mod icons;

pub use app::run;

pub fn platform_specific_settings(app_id: &str) -> iced::window::settings::PlatformSpecific {
    crate::platform::window_settings(app_id)
}
