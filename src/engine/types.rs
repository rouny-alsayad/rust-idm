use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DownloadId(pub u64);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestContext {
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub cookies: Option<String>,
    #[serde(default)]
    pub referer: Option<String>,
    #[serde(default)]
    pub basic_username: Option<String>,
    #[serde(default)]
    pub basic_password: Option<String>,
}

impl RequestContext {
    pub fn apply(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(value) = self.user_agent.as_deref().filter(|v| !v.trim().is_empty()) {
            request = request.header(reqwest::header::USER_AGENT, value);
        }
        if let Some(value) = self.cookies.as_deref().filter(|v| !v.trim().is_empty()) {
            request = request.header(reqwest::header::COOKIE, value);
        }
        if let Some(value) = self.referer.as_deref().filter(|v| !v.trim().is_empty()) {
            request = request.header(reqwest::header::REFERER, value);
        }
        if let Some(username) = self.basic_username.as_deref().filter(|v| !v.is_empty()) {
            request = request.basic_auth(username, self.basic_password.as_deref());
        }
        request
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadState {
    Queued,
    Probing,
    Downloading,
    Pausing,
    Paused,
    Retrying,
    Completing,
    Complete,
    Failed,
    Cancelled,
}

impl fmt::Display for DownloadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "Queued"),
            Self::Probing => write!(f, "Probing"),
            Self::Downloading => write!(f, "Downloading"),
            Self::Pausing => write!(f, "Pausing"),
            Self::Paused => write!(f, "Paused"),
            Self::Retrying => write!(f, "Retrying"),
            Self::Completing => write!(f, "Completing"),
            Self::Complete => write!(f, "Complete"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    pub user_agent: Option<String>,
    pub cookies: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadMetadata {
    pub final_url: String,
    pub content_length: Option<u64>,
    pub content_type: Option<String>,
    pub filename: Option<String>,
    pub accept_ranges: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadError {
    NetworkError(String),
    DiskError(String),
    HttpError(u16),
    ResourceChanged,
    Timeout,
    MaxRetriesExceeded,
    Cancelled,
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(s) => write!(f, "Network error: {}", s),
            Self::DiskError(s) => write!(f, "Disk error: {}", s),
            Self::HttpError(c) => write!(f, "HTTP error: {}", c),
            Self::ResourceChanged => write!(f, "Resource changed on server"),
            Self::Timeout => write!(f, "Timeout"),
            Self::MaxRetriesExceeded => write!(f, "Max retries exceeded"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    Probed {
        id: DownloadId,
        metadata: DownloadMetadata,
    },
    Progress {
        id: DownloadId,
        downloaded: u64,
        total_size: Option<u64>,
        transfer_rate: f64,
        time_left_secs: Option<f64>,
    },
    StateChanged {
        id: DownloadId,
        state: DownloadState,
    },
    Error {
        id: DownloadId,
        error: DownloadError,
    },
    Complete {
        id: DownloadId,
        final_path: PathBuf,
    },
}

