pub mod preview;
pub mod app;
pub mod icons;

pub use app::run;

#[cfg(target_os = "linux")]
pub fn platform_specific_settings(app_id: &str) -> iced::window::settings::PlatformSpecific {
    iced::window::settings::PlatformSpecific {
        application_id: String::from(app_id),
        override_redirect: false,
    }
}

#[cfg(not(target_os = "linux"))]
pub fn platform_specific_settings(_app_id: &str) -> iced::window::settings::PlatformSpecific {
    iced::window::settings::PlatformSpecific::default()
}
