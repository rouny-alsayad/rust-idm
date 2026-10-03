use eframe::egui;
use crate::models;
use crate::download_manager::DownloadManager;

pub struct AddUrlState {
    pub url: String,
    pub use_auth: bool,
    pub login: String,
    pub password: String,
}

impl Default for AddUrlState {
    fn default() -> Self { Self {
            url: String::new(),
            use_auth: false,
            login: String::new(),
            password: String::new(),
        }
    }
}

pub struct App {
    pub downloads: Vec<models::DownloadItem>,
    pub selected_category: models::SidebarCategory,
    pub selected_downloads: std::collections::HashSet<usize>,
    pub config: models::AppConfig,
    pub show_add_url_dialog: bool,
    pub show_options_dialog: bool,
    pub show_scheduler_dialog: bool,
    pub show_move_rename_dialog: bool,
    pub move_rename_target: Option<usize>,
    pub move_rename_mode: u8,
    pub move_rename_text: String,
    pub ext_rx: Option<crossbeam_channel::Receiver<crate::server::DownloadRequest>>,
    pub options_tab: usize,
    pub add_url_state: AddUrlState,
    pub search_text: String,
    pub show_search: bool,
    pub download_manager: DownloadManager,
    pub next_id: u64,
    pub pending_file_info: Option<crate::models::PendingFileInfo>,
    pub active_progress_window_id: Option<u64>,
    pub fetching_info: bool,
    pub info_rx: Option<crossbeam_channel::Receiver<crate::models::PendingFileInfo>>,
    pub info_tx: Option<crossbeam_channel::Sender<crate::models::PendingFileInfo>>,
    pub ext_needs_focus: bool,
    pub completed_downloads_to_show: Vec<u64>,
    pub last_persist_at: std::time::Instant,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let (tx, rx) = crossbeam_channel::unbounded();
        let (info_tx, info_rx) = crossbeam_channel::unbounded();
        let server_state = std::sync::Arc::new(crate::server::ServerState {
            sender: tx,
            ctx: cc.egui_ctx.clone(),
        });

        let mut downloads = crate::persistence::load_downloads();
        crate::persistence::normalize_after_restart(&mut downloads);

        // Migrate older builds that stored relative paths like "Downloads\file.bin".
        // Relative paths depend on the process working directory and caused os error 3.
        let default_download_dir = dirs_fallback();
        for item in &mut downloads {
            let current = std::path::PathBuf::from(&item.save_path);
            if current.is_relative() {
                item.save_path = default_download_dir
                    .join(&item.file_name)
                    .to_string_lossy()
                    .to_string();
            }
        }

        let next_id = downloads.iter().map(|d| d.id).max().unwrap_or(0) + 1;

        let app = Self {
            downloads,
            selected_category: models::SidebarCategory::AllDownloads,
            selected_downloads: std::collections::HashSet::new(),
            config: models::AppConfig::default(),
            show_add_url_dialog: false,
            show_options_dialog: false,
            show_scheduler_dialog: false,
            show_move_rename_dialog: false,
            move_rename_target: None,
            move_rename_mode: 0,
            move_rename_text: String::new(),
            ext_rx: Some(rx),
            options_tab: 0,
            add_url_state: AddUrlState::default(),
            search_text: String::new(),
            show_search: false,
            download_manager: DownloadManager::new(),
            next_id,
            pending_file_info: None,
            active_progress_window_id: None,
            fetching_info: false,
            info_rx: Some(info_rx),
            info_tx: Some(info_tx),
            ext_needs_focus: false,
            completed_downloads_to_show: Vec::new(),
            last_persist_at: std::time::Instant::now(),
        };
        app.download_manager.spawn_task(crate::server::start_server(server_state));
        app
    }

    pub fn get_save_path(&self, file_name: &str) -> std::path::PathBuf {
        let dir = if self.config.save.default_directory.is_empty() {
            dirs_fallback()
        } else {
            std::path::PathBuf::from(&self.config.save.default_directory)
        };
        dir.join(file_name)
    }

    pub fn next_download_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn apply_progress_updates(&mut self) {
        let events = self.download_manager.poll_updates();
        for event in events {
            match event {
                crate::engine::types::EngineEvent::Probed { id, metadata } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id.0) {
                        if let Some(size) = metadata.content_length {
                            item.size = Some(size);
                        }
                    }
                }
                crate::engine::types::EngineEvent::Progress { id, downloaded, total_size, transfer_rate, time_left_secs } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id.0) {
                        item.downloaded = downloaded;
                        if let Some(total) = total_size { item.size = Some(total); }
                        item.transfer_rate = transfer_rate;
                        item.time_left_secs = time_left_secs;
                    }
                }
                crate::engine::types::EngineEvent::StateChanged { id, state } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id.0) {
                        item.status = state;
                    }
                }
                crate::engine::types::EngineEvent::Error { id, error } => {
                    crate::persistence::log_line(format!("download {} failed: {}", id.0, error));
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id.0) {
                        item.status = crate::engine::types::DownloadState::Failed;
                        item.description = error.to_string();
                    }
                }
                crate::engine::types::EngineEvent::Complete { id, final_path } => {
                    crate::persistence::log_line(format!(
                        "download {} complete: {}",
                        id.0,
                        final_path.display()
                    ));
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == id.0) {
                        item.status = crate::engine::types::DownloadState::Complete;
                        item.save_path = final_path.to_string_lossy().to_string();
                        self.completed_downloads_to_show.push(item.id);
                        if self.active_progress_window_id == Some(item.id) {
                            self.active_progress_window_id = None;
                        }
                    }
                }
            }
        }
    }

    pub fn active_download_count(&self) -> usize {
        self.downloads.iter().filter(|d| d.status == crate::engine::types::DownloadState::Downloading).count()
    }

    pub fn total_speed(&self) -> f64 {
        self.downloads.iter()
            .filter(|d| d.status == crate::engine::types::DownloadState::Downloading)
            .map(|d| d.transfer_rate)
            .sum()
    }
}

fn dirs_fallback() -> std::path::PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join("Downloads")
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = crate::persistence::save_downloads(&self.downloads);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = self.info_rx.as_ref() {
            while let Ok(mut info) = rx.try_recv() {
                self.fetching_info = false;
                info.save_path = self.get_save_path(&info.file_name).to_string_lossy().to_string();
                self.pending_file_info = Some(info);
            }
        }
        if let Some(rx) = self.ext_rx.as_ref() {
            while let Ok(req) = rx.try_recv() {
                self.fetching_info = true;
                let tx = self.info_tx.as_ref().unwrap().clone();
                let ctx_clone = ctx.clone();
                let request_context = crate::engine::types::RequestContext {
                    user_agent: req.user_agent,
                    cookies: req.cookies,
                    referer: req.referer,
                    ..Default::default()
                };
                self.download_manager.spawn_task(crate::download_manager::fetch_file_info(
                    req.url,
                    request_context,
                    tx,
                    ctx_clone,
                ));
            }
        }
        crate::theme::IdmTheme::apply(ctx);

        self.apply_progress_updates();

        if self.last_persist_at.elapsed() >= std::time::Duration::from_secs(2) {
            let _ = crate::persistence::save_downloads(&self.downloads);
            self.last_persist_at = std::time::Instant::now();
        }

        if self.active_download_count() > 0 {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        crate::ui::menu_bar::render_menu_bar(ctx, self);
        crate::ui::toolbar::render_toolbar(ctx, self);

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let active = self.active_download_count();
                let status_text = if active > 0 {
                    let speed = format_speed(self.total_speed());
                    format!("Downloading {} file(s) - {}", active, speed)
                } else {
                    "Ready".to_string()
                };
                ui.label(
                    egui::RichText::new(status_text)
                        .color(crate::theme::IdmTheme::TEXT_SECONDARY)
                        .size(12.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} downloads", self.downloads.len()))
                            .color(crate::theme::IdmTheme::TEXT_SECONDARY)
                            .size(12.0),
                    );
                });
            });
        });

        crate::ui::sidebar::render_sidebar(ctx, self);
        crate::ui::download_table::render_download_table(ctx, self);

        if self.show_add_url_dialog {
            crate::ui::dialogs::add_url::render_add_url_dialog(ctx, self);
        }
        if self.show_options_dialog {
            crate::ui::dialogs::options::render_options_dialog(ctx, self);
        }
        if self.show_scheduler_dialog {
            crate::ui::dialogs::scheduler::render_scheduler_dialog(ctx, self);
        }
        if self.show_move_rename_dialog {
            crate::ui::dialogs::move_rename::render_move_rename_dialog(ctx, self);
        }
        crate::ui::dialogs::file_info::render_file_info_dialog(ctx, self);
        crate::ui::dialogs::progress_window::render_progress_window(ctx, self);
        crate::ui::dialogs::complete_window::render_complete_windows(ctx, self);
    }
}

pub fn format_speed(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * KB;
    const GB: f64 = 1024.0 * MB;

    if bytes_per_sec >= GB {
        format!("{:.2} GB/s", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.2} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

