use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

pub async fn scan_directories(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    tokio::task::spawn_blocking(move || {
        let mut files: Vec<PathBuf> = Vec::new();
        let mut seen: HashSet<PathBuf> = HashSet::new();
        for dir in dirs {
            if !dir.exists() {
                continue;
            }
            for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
                if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        let ext = ext.to_lowercase();
                        if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp" {
                            let pb = path.to_path_buf();
                            if seen.insert(pb.clone()) {
                                files.push(pb);
                            }
                        }
                    }
                }
            }
        }

        files.sort_by(|a, b| {
            let at = file_time(a);
            let bt = file_time(b);

            bt.cmp(&at)
        });
        files
    })
    .await
    .unwrap_or_default()
}

fn file_time(p: &PathBuf) -> SystemTime {
    std::fs::metadata(p)
        .and_then(|m| m.created().or_else(|_| m.modified()))
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
