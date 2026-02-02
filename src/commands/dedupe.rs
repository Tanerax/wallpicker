use crate::config::Config;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct DedupeKey {
    size: u64,
    hash: blake3::Hash,
}

pub async fn run_dedupe(cfg: Config) {
    let images = crate::scanner::scan_directories(cfg.folders.clone()).await;
    if images.is_empty() {
        println!("No images found.");
        return;
    }

    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for p in images {
        if let Ok(meta) = std::fs::metadata(&p) {
            let len = meta.len();
            by_size.entry(len).or_default().push(p);
        }
    }

    let mut groups: HashMap<DedupeKey, Vec<PathBuf>> = HashMap::new();
    for (size, paths) in by_size.into_iter() {
        if paths.len() < 2 {
            continue;
        }

        for p in paths {
            let key_opt = compute_hash_key(p.clone(), size).await;
            if let Some(key) = key_opt {
                groups.entry(key).or_default().push(p);
            }
        }
    }

    let mut dup_groups: Vec<(u64, blake3::Hash, Vec<PathBuf>)> = groups
        .into_iter()
        .filter_map(|(k, v)| if v.len() >= 2 { Some((k.size, k.hash, v)) } else { None })
        .collect();

    if dup_groups.is_empty() {
        println!("No exact duplicates found.");
        return;
    }

    dup_groups.sort_by(|a, b| b.0.cmp(&a.0));

    let mut total_deleted = 0usize;
    let mut bytes_reclaimed: u64 = 0;
    let mut groups_processed = 0usize;

    'outer: for (idx, (size, _hash, mut files)) in dup_groups.into_iter().enumerate() {
        files.retain(|p| p.exists());
        if files.len() < 2 {
            continue;
        }

        groups_processed += 1;
        loop {
            println!("\nDuplicate group {} ({} file(s), {})", idx + 1, files.len(), format_size(size));
            for (i, p) in files.iter().enumerate() {
                let mt = read_modified_time(p);
                println!("  [{}] {} | modified {}", i + 1, p.display(), mt);
            }
            println!("Options: k <n>=keep n, kn=keep newest, ko=keep oldest, p <n>=preview n, s=skip, q=quit");
            print!("dedupe> ");
            let _ = std::io::stdout().flush();

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!("Failed to read input. Skipping group.");
                break;
            }
            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }

            let mut parts = trimmed.split_whitespace();
            let cmd = parts.next().unwrap_or("");
            match cmd {
                "k" => {
                    if let Some(nstr) = parts.next() {
                        if let Ok(n) = nstr.parse::<usize>() {
                            if n >= 1 && n <= files.len() {
                                let keep_index = n - 1;
                                let keep_path = files[keep_index].clone();

                                if !confirm_action(&format!(
                                    "Delete {} other file(s) and keep '{}' [y/N] ",
                                    files.len() - 1,
                                    keep_path.display()
                                )) {
                                    continue;
                                }
                                
                                let (del_count, bytes) = delete_all_except(&files, keep_index, size);
                                total_deleted += del_count;
                                bytes_reclaimed += bytes;
                                break;
                            }
                        }
                    }
                    println!("Usage: k <index>");
                }
                "kn" => {
                    if let Some(keep_idx) = newest_index(&files) {
                        let (del_count, bytes) = delete_all_except(&files, keep_idx, size);
                        total_deleted += del_count;
                        bytes_reclaimed += bytes;
                        break;
                    }
                }
                "ko" => {
                    if let Some(keep_idx) = oldest_index(&files) {
                        let (del_count, bytes) = delete_all_except(&files, keep_idx, size);
                        total_deleted += del_count;
                        bytes_reclaimed += bytes;
                        break;
                    }
                }
                "p" => {
                    if let Some(nstr) = parts.next() {
                        if let Ok(n) = nstr.parse::<usize>() {
                            if n >= 1 && n <= files.len() {
                                let p = files[n - 1].clone();
                                let deleted = crate::commands::open_preview(p.clone()).await;

                                if deleted {
                                    files.retain(|x| x != &p);
                                    total_deleted += 1;
                                    bytes_reclaimed += size;
                                    if files.len() < 2 {
                                        break;
                                    }
                                }
                                continue;
                            }
                        }
                    }
                    println!("Usage: p <index>");
                }
                "s" => {
                    break;
                }
                "q" => {
                    println!("Quitting early.");
                    break 'outer;
                }
                _ => {
                    println!("Unknown command: {}", cmd);
                }
            }
        }
    }

    println!(
        "\nDedupe summary: processed {} group(s), deleted {} file(s), reclaimed {}.",
        groups_processed,
        total_deleted,
        format_size(bytes_reclaimed)
    );
}

fn confirm_action(prompt: &str) -> bool {
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let mut s = String::new();
    if std::io::stdin().read_line(&mut s).is_ok() {
        let c = s.trim().to_lowercase();
        return c == "y" || c == "yes";
    }
    false
}

fn newest_index(files: &[PathBuf]) -> Option<usize> {
    let mut best: Option<(SystemTime, usize)> = None;
    for (i, p) in files.iter().enumerate() {
        let t = file_time(p).unwrap_or(UNIX_EPOCH);
        if let Some((bt, _)) = best {
            if t > bt {
                best = Some((t, i));
            }
        } else {
            best = Some((t, i));
        }
    }
    best.map(|(_, i)| i)
}

fn oldest_index(files: &[PathBuf]) -> Option<usize> {
    let mut best: Option<(SystemTime, usize)> = None;
    for (i, p) in files.iter().enumerate() {
        let t = file_time(p).unwrap_or(UNIX_EPOCH);
        if let Some((bt, _)) = best {
            if t < bt {
                best = Some((t, i));
            }
        } else {
            best = Some((t, i));
        }
    }
    best.map(|(_, i)| i)
}

fn delete_all_except(files: &[PathBuf], keep_index: usize, size_per_file: u64) -> (usize, u64) {
    let mut deleted = 0usize;
    let mut bytes = 0u64;
    for (i, p) in files.iter().enumerate() {
        if i == keep_index {
            continue;
        }
        match std::fs::remove_file(p) {
            Ok(_) => {
                deleted += 1;
                bytes += size_per_file;
                println!("Deleted {}", p.display());
            }
            Err(e) => {
                eprintln!("Failed to delete {}: {}", p.display(), e);
            }
        }
    }
    (deleted, bytes)
}

async fn compute_hash_key(path: PathBuf, size: u64) -> Option<DedupeKey> {
    tokio::task::spawn_blocking(move || {
        let mut f = File::open(&path).ok()?;
        let mut hasher = blake3::Hasher::new();
        let mut buf = [0u8; 1024 * 64];
        loop {
            match f.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    hasher.update(&buf[..n]);
                }
                Err(_) => return None,
            }
        }
        let hash = hasher.finalize();
        Some(DedupeKey { size, hash })
    })
    .await
    .ok()
    .flatten()
}

fn file_time(p: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(p)
        .ok()
        .and_then(|m| m.created().ok().or_else(|| m.modified().ok()))
}

fn read_modified_time(p: &PathBuf) -> String {
    if let Some(t) = file_time(p) {
        if let Ok(dur) = t.duration_since(UNIX_EPOCH) {
            let secs = dur.as_secs();
            let tm = chrono_like(secs);
            return tm;
        }
    }
    "unknown".into()
}

fn chrono_like(secs: u64) -> String {
    format!("{}s", secs)
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GiB", b / GB)
    } else if b >= MB {
        format!("{:.2} MiB", b / MB)
    } else if b >= KB {
        format!("{:.2} KiB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}
