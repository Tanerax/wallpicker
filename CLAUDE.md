# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Run Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo check                          # Type-check without building
cargo run                            # Run GUI mode
cargo run -- --preview <path>        # Preview a single image
cargo run -- --random                # Set random wallpaper from configured folders
cargo run -- --clean                 # Remove orphaned cache entries
cargo run -- --generate              # Pre-generate all thumbnails
cargo run -- --dedupe                # Interactive duplicate finder
cargo clippy                         # Lint
cargo fmt                            # Format code
```

No test suite exists yet.

## Architecture

Wallpicker is a Rust desktop wallpaper manager with both a GUI (Iced framework) and CLI modes. It scans local folders for images, generates/caches thumbnails, and sets wallpapers via the `swww` command-line tool.

### Module Layout

- **`main.rs`** — CLI argument parsing (clap-style manual parsing), single-instance enforcement, mode dispatch
- **`ui/app.rs`** — Main Iced application: message-driven state machine (`WallpaperApp`), thumbnail grid, Wallhaven integration
- **`ui/preview.rs`** — Full-screen image preview window (separate Iced app)
- **`ui/icons.rs`** — Inline SVG icon definitions
- **`commands/`** — Side-effect operations: setting wallpaper (`swww img`), spawning preview, deduplication
- **`scanner/`** — Recursive directory walker; finds PNG/JPEG/WebP files
- **`image/`** — Thumbnail generation (200×200px) with async loading
- **`cache/`** — Thumbnail cache keyed by BLAKE3 hash of `path|mtime|size|thumbsize`, stored as PNGs in `~/.config/wallpicker/cache/`
- **`config/`** — JSON config at `~/.config/wallpicker/config.json` with fields: `folders`, `wallhaven_api_key`, `wallhaven_categories`, `wallhaven_purity`
- **`wallpaper/`** — Random wallpaper selection logic
- **`wallhaven/`** — Wallhaven.cc API client (search endpoint)

### Key Patterns

- **Iced message passing**: UI updates flow through `Message` enum → `update()` → `view()` cycle in `app.rs`
- **Async thumbnail loading**: Thumbnails load via Tokio tasks, with cache hits returning immediately and misses generating + caching in the background
- **Single instance**: Uses `single-instance` crate to prevent duplicate windows

### External Dependencies

- **`swww`** must be installed and in PATH (Wayland wallpaper setter)
- Config/cache lives under `~/.config/wallpicker/`
- Wallpaper copy goes to `/tmp/current_wallpaper`
