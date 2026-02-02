pub mod wallpaper;
pub mod preview;
pub mod dedupe;

pub use wallpaper::{set_random_wallpaper, set_random_wallpaper_via_wallhaven, set_wallpaper};
pub use preview::open_preview;
pub use dedupe::run_dedupe;
