use std::path::PathBuf;

use crate::engine::types::DownloadState;
use crate::models::DownloadItem;

fn state_directory() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("RustIDM");
        }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    PathBuf::from(home).join(".rust-idm")
}

pub fn state_file() -> PathBuf {
    state_directory().join("downloads.json")
}

pub fn load_downloads() -> Vec<DownloadItem> {
    let path = state_file();
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };

    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn normalize_after_restart(downloads: &mut [DownloadItem]) {
    for item in downloads {
        match item.status {
            DownloadState::Downloading
            | DownloadState::Probing
            | DownloadState::Pausing
            | DownloadState::Retrying
            | DownloadState::Completing => {
                item.status = DownloadState::Paused;
            }
            _ => {}
        }

        item.transfer_rate = 0.0;
        item.time_left_secs = None;
    }
}

pub fn save_downloads(downloads: &[DownloadItem]) -> std::io::Result<()> {
    let dir = state_directory();
    std::fs::create_dir_all(&dir)?;

    let path = state_file();
    let temp_path = dir.join("downloads.json.tmp");
    let data = serde_json::to_vec_pretty(downloads)
        .map_err(std::io::Error::other)?;

    std::fs::write(&temp_path, data)?;

    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }

    std::fs::rename(temp_path, path)?;
    Ok(())
}

pub fn log_line(message: impl AsRef<str>) {
    let dir = state_directory();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }

    let path = dir.join("rustidm.log");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        use std::io::Write;
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] {}", timestamp, message.as_ref());
    }
}
