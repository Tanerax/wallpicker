use std::path::PathBuf;

pub async fn open_preview(path: PathBuf) -> bool {
    let res = tokio::task::spawn_blocking(move || {
        if let Ok(exe) = std::env::current_exe() {
            let mut cmd = std::process::Command::new(exe);
            cmd.arg("--preview").arg(&path);
            match cmd.status() {
                Ok(status) => {
                    if let Some(code) = status.code() {
                        return code == 10;
                    }
                    false
                }
                Err(_) => false,
            }
        } else {
            false
        }
    })
    .await;

    res.unwrap_or(false)
}
