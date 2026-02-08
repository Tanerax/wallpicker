use crate::config::Config;
use std::ffi::c_void;
use std::sync::Arc;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

// Objective-C runtime FFI
unsafe extern "C" {
    fn objc_getClass(name: *const u8) -> *mut c_void;
    fn sel_registerName(name: *const u8) -> *mut c_void;
    fn objc_msgSend();
}

fn msg_send_ptr() -> *const () {
    objc_msgSend as *const ()
}

/// Initialize NSApplication with Accessory activation policy, returns the app pointer.
unsafe fn init_macos_app() -> *mut c_void {
    unsafe {
        let msg_send = msg_send_ptr();
        let cls = objc_getClass(b"NSApplication\0".as_ptr());
        let sel_shared = sel_registerName(b"sharedApplication\0".as_ptr());

        let shared_app: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(msg_send);
        let app = shared_app(cls, sel_shared);

        // [app setActivationPolicy:NSApplicationActivationPolicyAccessory]  (1 = Accessory)
        let sel_policy = sel_registerName(b"setActivationPolicy:\0".as_ptr());
        let set_policy: unsafe extern "C" fn(*mut c_void, *mut c_void, i64) -> u8 =
            std::mem::transmute(msg_send);
        set_policy(app, sel_policy, 1);

        // [app finishLaunching]
        let sel_finish = sel_registerName(b"finishLaunching\0".as_ptr());
        let finish: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(msg_send);
        finish(app, sel_finish);

        app
    }
}

/// Call [NSApp run] — blocks forever, events are handled via callbacks.
unsafe fn run_macos_app(app: *mut c_void) {
    unsafe {
        let msg_send = msg_send_ptr();
        let sel_run = sel_registerName(b"run\0".as_ptr());
        let run_fn: unsafe extern "C" fn(*mut c_void, *mut c_void) =
            std::mem::transmute(msg_send);
        run_fn(app, sel_run);
    }
}

fn build_icon() -> Icon {
    let size: u32 = 22;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[0] = 100; // R
        pixel[1] = 149; // G
        pixel[2] = 237; // B  (cornflower blue)
        pixel[3] = 255; // A
    }
    Icon::from_rgba(rgba, size, size).expect("failed to create tray icon")
}

pub fn run(cfg: Config) {
    let app = unsafe { init_macos_app() };

    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime"),
    );

    let menu = Menu::new();
    let open_item = MenuItem::new("Open Wallpicker", true, None);
    let random_item = MenuItem::new("Random Wallhaven", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&open_item).unwrap();
    menu.append(&random_item).unwrap();
    menu.append(&quit_item).unwrap();

    let _tray = TrayIconBuilder::new()
        .with_icon(build_icon())
        .with_icon_as_template(true)
        .with_tooltip("Wallpicker")
        .with_menu(Box::new(menu))
        .build()
        .expect("failed to build tray icon");

    // Use callback-based event handling since [NSApp run] blocks forever
    let open_id = open_item.id().clone();
    let random_id = random_item.id().clone();
    let quit_id = quit_item.id().clone();

    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == open_id {
            spawn_ui();
        } else if event.id == random_id {
            let cfg = cfg.clone();
            rt.spawn(async move {
                match crate::commands::set_random_wallpaper_via_wallhaven(cfg).await {
                    Some(p) => println!("Set wallpaper: {}", p.display()),
                    None => eprintln!("Failed to set random wallhaven wallpaper"),
                }
            });
        } else if event.id == quit_id {
            std::process::exit(0);
        }
    }));

    // Run the macOS event loop (blocks forever, menu events handled by callback above)
    unsafe {
        run_macos_app(app);
    }
}

fn spawn_ui() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).spawn();
    }
}
