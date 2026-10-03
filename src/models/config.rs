use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub file_types: FileTypesConfig,
    pub save: SaveConfig,
    pub download_behavior: DownloadBehaviorConfig,
    pub connection: ConnectionConfig,
    pub proxy: ProxyConfig,
    pub site_logins: Vec<SiteLogin>,
    pub dial_up: DialUpConfig,
    pub sounds: SoundConfig,
}

/// Represents a browser entry in the integration list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserEntry {
    pub name: String,
    pub enabled: bool,
}

/// General settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub launch_on_startup: bool,
    pub clipboard_monitoring: bool,
    pub browsers: Vec<BrowserEntry>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            launch_on_startup: true,
            clipboard_monitoring: true,
            browsers: vec![
                BrowserEntry { name: "Google Chrome".into(), enabled: true },
                BrowserEntry { name: "Mozilla Firefox".into(), enabled: true },
                BrowserEntry { name: "Microsoft Edge".into(), enabled: true },
            ],
        }
    }
}

/// File Types configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypesConfig {
    pub auto_download_types: String,
    pub exception_sites: String,
    pub exception_addresses: Vec<String>,
}

impl Default for FileTypesConfig {
    fn default() -> Self {
        Self {
            auto_download_types: "3GP 7Z AAC ACE AIF ARJ ASF AVI BIN BZ2 EXE GZ GZIP IMG ISO LZH M4A M4V MKV MOV MP3 MP4 MPA MPE MPEG MPG MSI MSU OGG OGV PDF PLJ PPS PPT QT R0* R1* RA RAR RM RMVB SEA SIT SITX TAR TIF TIFF WMA WMV Z ZIP".into(),
            exception_sites: "".into(),
            exception_addresses: vec![],
        }
    }
}

/// Save settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveConfig {
    pub category: String,
    pub default_directory: String,
    pub temp_directory: String,
    pub change_folder_on_last_selected: bool,
    pub set_server_date: bool,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            category: "General".into(),
            default_directory: "".into(),
            temp_directory: "".into(),
            change_folder_on_last_selected: true,
            set_server_date: true,
        }
    }
}

/// Download behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadBehaviorConfig {
    pub show_start_dialog: bool,
    pub show_complete_dialog: bool,
    pub start_immediately: bool,
    pub show_queue_on_later: bool,
    pub show_queue_on_batch_close: bool,
    pub ignore_time_changes: bool,
    pub duplicate_action: String,
    pub user_agent: String,
    pub show_speed_limiter_tab: bool,
    pub show_options_on_completion: bool,
    pub show_hide_tab_buttons: bool,
    pub start_view: String,
}

impl Default for DownloadBehaviorConfig {
    fn default() -> Self {
        Self {
            show_start_dialog: true,
            show_complete_dialog: true,
            start_immediately: true,
            show_queue_on_later: true,
            show_queue_on_batch_close: true,
            ignore_time_changes: false,
            duplicate_action: "ask".into(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".into(),
            show_speed_limiter_tab: true,
            show_options_on_completion: true,
            show_hide_tab_buttons: true,
            start_view: "normal".into(),
        }
    }
}

/// Represents an exception to server connection limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerException {
    pub server: String,
    pub number: u8,
}

/// Connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub max_connections: u8,
    pub download_limit_enabled: bool,
    pub download_limit_mb: u32,
    pub download_limit_hours: u32,
    pub show_limit_warning: bool,
    pub exceptions: Vec<ServerException>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            max_connections: 8,
            download_limit_enabled: false,
            download_limit_mb: 200,
            download_limit_hours: 5,
            show_limit_warning: true,
            exceptions: vec![],
        }
    }
}

/// Proxy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub mode: String, // "none", "system", "auto", "manual"
    pub use_browser_proxy: bool,
    pub auto_config_url: String,
    pub proxy_address: String,
    pub proxy_port: String,
    pub proxy_user: String,
    pub proxy_pass: String,
    pub use_http: bool,
    pub use_https: bool,
    pub use_ftp: bool,
    pub use_pasv: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            mode: "none".into(),
            use_browser_proxy: true,
            auto_config_url: "".into(),
            proxy_address: "".into(),
            proxy_port: "".into(),
            proxy_user: "".into(),
            proxy_pass: "".into(),
            use_http: true,
            use_https: true,
            use_ftp: true,
            use_pasv: true,
        }
    }
}

/// Configuration for site logins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteLogin {
    pub protocol: String,
    pub server_path: String,
    pub user: String,
    pub password: String,
}

impl Default for SiteLogin {
    fn default() -> Self {
        Self {
            protocol: "https://".into(),
            server_path: "".into(),
            user: "".into(),
            password: "".into(),
        }
    }
}

/// Dial Up / VPN settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialUpConfig {
    pub enabled: bool,
    pub connection: String,
    pub user_name: String,
    pub password: String,
    pub save_password: bool,
    pub redial_attempts: u32,
    pub redial_interval: u32,
}

impl Default for DialUpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            connection: "".into(),
            user_name: "".into(),
            password: "".into(),
            save_password: false,
            redial_attempts: 0,
            redial_interval: 30,
        }
    }
}

/// Sound event settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEvent {
    pub enabled: bool,
    pub sound_file: String,
}

impl Default for SoundEvent {
    fn default() -> Self {
        Self {
            enabled: false,
            sound_file: "".into(),
        }
    }
}

/// Sounds configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoundConfig {
    pub download_complete: SoundEvent,
    pub download_failed: SoundEvent,
    pub queue_started: SoundEvent,
    pub queue_stopped: SoundEvent,
}
