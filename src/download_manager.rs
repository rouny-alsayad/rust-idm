use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

use crate::engine::types::{DownloadId, DownloadMetadata, EngineEvent};
use crate::engine::downloader::Downloader;

struct DownloadHandle {
    cancel_flag: Arc<AtomicBool>,
}

pub struct DownloadManager {
    runtime: tokio::runtime::Runtime,
    tx: mpsc::Sender<EngineEvent>,
    rx: mpsc::Receiver<EngineEvent>,
    handles: HashMap<u64, DownloadHandle>,
}

impl DownloadManager {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        let (tx, rx) = mpsc::channel();

        Self {
            runtime,
            tx,
            rx,
            handles: HashMap::new(),
        }
    }

    pub fn spawn_task<F>(&self, future: F) where F: std::future::Future<Output = ()> + Send + 'static {
        self.runtime.spawn(future);
    }

    pub fn start_download(
        &mut self,
        id: u64,
        url: String,
        save_path: PathBuf,
        connections: u8,
    ) {
        self.start_download_with_context(
            id,
            url,
            save_path,
            connections,
            crate::engine::types::RequestContext::default(),
        );
    }

    pub fn start_download_with_context(
        &mut self,
        id: u64,
        url: String,
        save_path: PathBuf,
        connections: u8,
        request_context: crate::engine::types::RequestContext,
    ) {
        if let Some(handle) = self.handles.get(&id) {
            handle.cancel_flag.store(true, Ordering::Relaxed);
        }

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let tx = self.tx.clone();
        let flag = cancel_flag.clone();

        self.handles.insert(id, DownloadHandle { cancel_flag: flag.clone() });

        self.runtime.spawn(async move {
            if let Some(parent) = save_path.parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    let _ = tx.send(EngineEvent::Error {
                        id: DownloadId(id),
                        error: crate::engine::types::DownloadError::DiskError(
                            format!("Cannot create download folder {}: {}", parent.display(), e)
                        ),
                    });
                    return;
                }
            }

            let client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) RustIDM/0.1")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default();

            // Probe metadata first
            match crate::engine::probe::probe_metadata_with_context(&client, &url, &request_context).await {
                Ok(metadata) => {
                    let _ = tx.send(EngineEvent::Probed { id: DownloadId(id), metadata: metadata.clone() });
                    let mut downloader = Downloader::new(DownloadId(id), save_path, metadata, tx.clone(), flag, connections, request_context);
                    downloader.run().await;
                },
                Err(e) => {
                    let _ = tx.send(EngineEvent::Error { id: DownloadId(id), error: e });
                }
            }
        });
    }

    pub fn pause_download(&mut self, id: u64) {
        if let Some(handle) = self.handles.get(&id) {
            handle.cancel_flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn resume_download(&mut self, id: u64, url: String, save_path: PathBuf, connections: u8) {
        self.start_download(id, url, save_path, connections);
    }

    pub fn resume_download_with_context(
        &mut self,
        id: u64,
        url: String,
        save_path: PathBuf,
        connections: u8,
        request_context: crate::engine::types::RequestContext,
    ) {
        self.start_download_with_context(id, url, save_path, connections, request_context);
    }

    pub fn cancel_download(&mut self, id: u64, delete_file: bool, save_path: &PathBuf) {
        if let Some(handle) = self.handles.remove(&id) {
            handle.cancel_flag.store(true, Ordering::Relaxed);
        }
        if delete_file {
            let mut part_path = save_path.clone();
            if let Some(ext) = part_path.extension() {
                let mut new_ext = ext.to_os_string();
                new_ext.push(".part");
                part_path.set_extension(new_ext);
            } else {
                part_path.set_extension("part");
            }
            let _ = std::fs::remove_file(part_path);
            for index in 0..32 {
                let mut segment_name = save_path.as_os_str().to_os_string();
                segment_name.push(format!(".part.{}", index));
                let _ = std::fs::remove_file(PathBuf::from(segment_name));
            }
            let _ = std::fs::remove_file(save_path);
        }
    }

    pub fn stop_all(&mut self) {
        for handle in self.handles.values() {
            handle.cancel_flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn poll_updates(&self) -> Vec<EngineEvent> {
        let mut updates = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            updates.push(event);
        }
        updates
    }
}

pub async fn fetch_file_info(url: String, request_context: crate::engine::types::RequestContext, tx: crossbeam_channel::Sender<crate::models::PendingFileInfo>, ctx: eframe::egui::Context) {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) RustIDM/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    let mut file_name = url.split('?').next().unwrap_or(&url).split('/').next_back().filter(|s| !s.is_empty()).unwrap_or("download").to_string();
    let mut size = None;

    if let Ok(metadata) = crate::engine::probe::probe_metadata_with_context(&client, &url, &request_context).await {
        size = metadata.content_length;
        if let Some(name) = metadata.filename {
            file_name = name;
        }
    }

    let ext = file_name.rsplit('.').next().unwrap_or("");
    let category = crate::models::Category::from_extension(ext);

    let _ = tx.send(crate::models::PendingFileInfo {
        url,
        file_name,
        save_path: String::new(),
        category,
        size,
        description: String::new(),
        remember_path: true,
        request_context,
    });
    ctx.request_repaint();
}
