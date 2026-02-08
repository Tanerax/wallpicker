mod config;
mod cache;
mod scanner;
mod image;
mod commands;
mod ui;
mod wallhaven;
mod wallpaper;

use std::path::PathBuf;

enum Mode {
    Ui,
    Preview(PathBuf),
    Random,
    Clean,
    Generate,
    Dedupe,
}

fn parse_args() -> Result<Mode, String> {
    let mut args = std::env::args().skip(1);

    let mut selected: Option<Mode> = None;

    while let Some(arg) = args.next() {
        let next_mode = match arg.as_str() {
            "--preview" => {
                let p = args
                    .next()
                    .ok_or_else(|| "Missing value after --preview".to_string())?;
                Mode::Preview(PathBuf::from(p))
            }
            "--random" => Mode::Random,
            "--clean" => Mode::Clean,
            "--generate" => Mode::Generate,
            "--dedupe" => Mode::Dedupe,
            "--help" | "-h" => {
                return Err(
                    "Usage:\n  wallpicker [--preview <path> | --random | --clean | --generate | --dedupe]\n"
                        .to_string(),
                );
            }
            _ => {
                return Err(format!("Unknown argument: {arg}"));
            }
        };

        if selected.is_some() {
            return Err("Only one mode can be specified (e.g. use exactly one of --preview/--random/--clean/--generate)".to_string());
        }

        selected = Some(next_mode);
    }

    Ok(selected.unwrap_or(Mode::Ui))
}

fn run_async<F>(f: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build();

    match rt {
        Ok(rt) => rt.block_on(f),
        Err(e) => eprintln!("Failed to build Tokio runtime: {e}"),
    }
}

fn enforce_single_instance() -> Result<(), String> {
    let inst = single_instance::SingleInstance::new("wallpicker-main")
        .map_err(|e| format!("{e}"))?;
    if !inst.is_single() {
        return Err("already-running".into());
    }

    std::mem::forget(inst);
    Ok(())
}

fn screen_size() -> (u32, u32) {
    let scale = 0.8;
    match screen_size::get_primary_screen_size() {
        Ok((w, h)) => (
            (w as f64 * scale) as u32,
            (h as f64 * scale) as u32,
        ),
        Err(_) => (1280, 800),
    }
}

fn main() -> iced::Result {
    let mode = match parse_args() {
        Ok(m) => m,
        Err(msg) => {
            eprintln!("{msg}");
            return Ok(());
        }
    };

    match mode {
        Mode::Random => {
            let cfg = crate::config::load_or_create_config();
            run_async(async move {
                let _ = crate::commands::set_random_wallpaper_via_wallhaven(cfg).await;
            });
            Ok(())
        }
        Mode::Clean => {
            let cfg = crate::config::load_or_create_config();
            run_async(async move {
                let imgs = crate::scanner::scan_directories(cfg.folders.clone()).await;
                let removed = crate::cache::clean_orphan_thumbnails_for_images(
                    &imgs,
                    crate::image::THUMB_SIZE,
                )
                    .unwrap_or(0);
                println!("Removed {} orphaned thumbnail(s)", removed);
            });
            Ok(())
        }
        Mode::Generate => {
            let cfg = crate::config::load_or_create_config();
            run_async(async move {
                let imgs = crate::scanner::scan_directories(cfg.folders.clone()).await;

                println!("Found {} image(s) to generate thumbnails for.", imgs.len());

                let res = tokio::task::spawn_blocking(move || {
                    let mut generated = 0usize;
                    let mut skipped = 0usize;
                    let mut failed = 0usize;

                    for p in imgs {
                        match crate::image::ensure_thumb_cached(&p, crate::image::THUMB_SIZE) {
                            Ok(true) => generated += 1,
                            Ok(false) => skipped += 1,
                            Err(_) => failed += 1,
                        }
                    }

                    (generated, skipped, failed)
                })
                    .await
                    .unwrap_or((0, 0, 0));

                println!(
                    "Generated {} thumbnail(s); {} already cached; {} failed",
                    res.0, res.1, res.2
                );
            });
            Ok(())
        }
        Mode::Preview(p) => {
            ui::preview::run(p)
        }
        Mode::Dedupe => {
            let cfg = crate::config::load_or_create_config();
            run_async(async move {
                crate::commands::run_dedupe(cfg).await;
            });
            Ok(())
        }
        Mode::Ui => {
            if let Err(e) = enforce_single_instance() {
                if e == "already-running" {
                    eprintln!("wallpicker is already running.");
                    return Ok(());
                } else {
                    eprintln!(
                        "Warning: Unable to enforce single-instance (continuing anyway): {}",
                        e
                    );
                }
            }

            let (w, h) = screen_size();
            ui::run(w, h)
        }
    }
}