use serde::{Deserialize, Serialize};
use std::fmt;
use crate::engine::types::{DownloadState, RequestContext};

/// Categories for organizing downloads
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Category {
    #[default]
    All,
    Compressed,
    Documents,
    Music,
    Programs,
    Video,
}

impl Category {
    /// Auto-detect category from a file extension
    pub fn from_extension(ext: &str) -> Category {
        match ext.to_lowercase().as_str() {
            "zip" | "rar" | "7z" | "gz" | "tar" | "bz2" | "xz" | "lzh" | "ace" | "sit" | "sitx" | "iso" | "img" => Category::Compressed,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "odt" | "ods" | "csv" => Category::Documents,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "aif" => Category::Music,
            "exe" | "msi" | "msu" | "dmg" | "deb" | "rpm" | "appimage" | "apk" => Category::Programs,
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | "3gp" | "ogv" | "rmvb" => Category::Video,
            _ => Category::All,
        }
    }
}

/// Main structure representing a single download item in the list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: u64,
    pub file_name: String,
    pub url: String,
    pub size: Option<u64>,
    pub downloaded: u64,
    pub status: DownloadState,
    pub category: Category,
    pub transfer_rate: f64, // bytes per second
    pub time_left_secs: Option<f64>,
    pub last_try_date: Option<String>,
    pub description: String,
    pub save_path: String,
    pub connections: u8,
    #[serde(default, skip_serializing, skip_deserializing)]
    pub request_context: RequestContext,
}

impl Default for DownloadItem {
    fn default() -> Self {
        Self {
            id: 0,
            file_name: String::new(),
            url: String::new(),
            size: None,
            downloaded: 0,
            status: DownloadState::Queued,
            category: Category::All,
            transfer_rate: 0.0,
            time_left_secs: None,
            last_try_date: None,
            description: String::new(),
            save_path: String::new(),
            connections: 8,
            request_context: RequestContext::default(),
        }
    }
}

impl DownloadItem {
    /// Calculate the completion fraction [0.0, 1.0] for progress bars
    pub fn progress_fraction(&self) -> f32 {
        if let Some(size) = self.size {
            if size == 0 {
                0.0
            } else {
                (self.downloaded as f32) / (size as f32)
            }
        } else {
            0.0
        }
    }

    /// Format the size nicely in Bytes/KB/MB/GB
    pub fn size_display(&self) -> String {
        fn format_bytes(bytes: u64) -> String {
            const KB: u64 = 1024;
            const MB: u64 = 1024 * KB;
            const GB: u64 = 1024 * MB;

            if bytes >= GB {
                format!("{:.2} GB", bytes as f64 / GB as f64)
            } else if bytes >= MB {
                format!("{:.2} MB", bytes as f64 / MB as f64)
            } else if bytes >= KB {
                format!("{:.2} KB", bytes as f64 / KB as f64)
            } else {
                format!("{} bytes", bytes)
            }
        }

        if let Some(size) = self.size {
            format_bytes(size)
        } else {
            "Unknown".to_string()
        }
    }

    /// Format transfer rate (bytes/sec) into KB/s, MB/s, etc.
    pub fn rate_display(&self) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = 1024.0 * KB;
        const GB: f64 = 1024.0 * MB;

        let rate = self.transfer_rate;
        if rate >= GB {
            format!("{:.2} GB/sec", rate / GB)
        } else if rate >= MB {
            format!("{:.2} MB/sec", rate / MB)
        } else if rate >= KB {
            format!("{:.2} KB/sec", rate / KB)
        } else {
            format!("{:.0} bytes/sec", rate)
        }
    }

    /// Format time left into readable format (e.g. "1h 20m" or "45 sec")
    pub fn time_left_display(&self) -> String {
        if let Some(secs) = self.time_left_secs {
            let hours = (secs / 3600.0).floor() as u64;
            let minutes = ((secs % 3600.0) / 60.0).floor() as u64;
            let seconds = (secs % 60.0).floor() as u64;

            if hours > 0 {
                format!("{}h {}m", hours, minutes)
            } else if minutes > 0 {
                format!("{}m {}s", minutes, seconds)
            } else {
                format!("{} sec", seconds)
            }
        } else {
            "Unknown".to_string()
        }
    }
}

/// Sidebar categories typically found in IDM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SidebarCategory {
    AllDownloads,
    Compressed,
    Documents,
    Music,
    Programs,
    Video,
    Unfinished,
    Finished,
    GrabberProjects,
    Queues,
}

impl fmt::Display for SidebarCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SidebarCategory::AllDownloads => write!(f, "All Downloads"),
            SidebarCategory::Compressed => write!(f, "Compressed"),
            SidebarCategory::Documents => write!(f, "Documents"),
            SidebarCategory::Music => write!(f, "Music"),
            SidebarCategory::Programs => write!(f, "Programs"),
            SidebarCategory::Video => write!(f, "Video"),
            SidebarCategory::Unfinished => write!(f, "Unfinished"),
            SidebarCategory::Finished => write!(f, "Finished"),
            SidebarCategory::GrabberProjects => write!(f, "Grabber Projects"),
            SidebarCategory::Queues => write!(f, "Queues"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingFileInfo {
    pub url: String,
    pub file_name: String,
    pub save_path: String,
    pub category: Category,
    pub size: Option<u64>,
    pub description: String,
    pub remember_path: bool,
    pub request_context: RequestContext,
}
