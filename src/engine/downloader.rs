use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncWriteExt, BufWriter};
use futures_util::StreamExt;
use reqwest::Client;

use crate::engine::types::{DownloadError, DownloadId, DownloadMetadata, DownloadState, EngineEvent, RequestContext};
use crate::engine::retry::RetryPolicy;
use crate::engine::segmented::{download_to_part, SegmentOutcome};

const BUFFER_SIZE: usize = 128 * 1024; // 128 KB buffer for disk writes

pub struct Downloader {
    pub client: Client,
    pub id: DownloadId,
    pub final_path: PathBuf,
    pub metadata: DownloadMetadata,
    pub event_tx: mpsc::Sender<EngineEvent>,
    pub cancel_flag: Arc<AtomicBool>,
    pub connections: u8,
    pub request_context: RequestContext,
    pub retry_policy: RetryPolicy,
}

impl Downloader {
    pub fn new(
        id: DownloadId,
        final_path: PathBuf,
        metadata: DownloadMetadata,
        event_tx: mpsc::Sender<EngineEvent>,
        cancel_flag: Arc<AtomicBool>,
        connections: u8,
        request_context: RequestContext,
    ) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) RustIDM/0.1")
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            client,
            id,
            final_path,
            metadata,
            event_tx,
            cancel_flag,
            connections: connections.clamp(1, 32),
            request_context,
            retry_policy: RetryPolicy::default(),
        }
    }

    fn part_path(&self) -> PathBuf {
        let mut path = self.final_path.clone();
        if let Some(ext) = path.extension() {
            let mut new_ext = ext.to_os_string();
            new_ext.push(".part");
            path.set_extension(new_ext);
        } else {
            path.set_extension("part");
        }
        path
    }

    fn send_event(&self, event: EngineEvent) {
        let _ = self.event_tx.send(event);
    }

    fn send_state(&self, state: DownloadState) {
        self.send_event(EngineEvent::StateChanged { id: self.id, state });
    }

    pub async fn run(&mut self) {
        let mut attempt = 0;
        loop {
            if self.cancel_flag.load(Ordering::Relaxed) {
                self.send_state(DownloadState::Cancelled);
                return;
            }

            if attempt > 0 {
                self.send_state(DownloadState::Retrying);
                self.retry_policy.wait(attempt).await;
            }

            match self.try_download().await {
                Ok(()) => {
                    return; // Completed or legitimately cancelled inside try_download
                }
                Err(e) => {
                    attempt += 1;
                    if attempt > self.retry_policy.max_attempts || is_permanent_error(&e) {
                        self.send_event(EngineEvent::Error { id: self.id, error: e });
                        self.send_state(DownloadState::Failed);
                        return;
                    }
                    // Else retry loop continues
                }
            }
        }
    }

    async fn try_download(&mut self) -> Result<(), DownloadError> {
        if let Some(parent) = self.final_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| DownloadError::DiskError(format!(
                    "Cannot create download folder {}: {}",
                    parent.display(),
                    e
                )))?;
        }

        let part_path = self.part_path();

        // IDM-style multi-connection mode for servers that support byte ranges.
        if self.connections > 1 && self.metadata.accept_ranges {
            if let Some(total_size) = self.metadata.content_length {
                if total_size >= 1024 * 1024 {
                    match download_to_part(
                        self.client.clone(),
                        self.id,
                        self.metadata.final_url.clone(),
                        &self.final_path,
                        total_size,
                        self.connections,
                        self.cancel_flag.clone(),
                        self.event_tx.clone(),
                        self.metadata.etag.clone(),
                        self.metadata.last_modified.clone(),
                        self.request_context.clone(),
                    ).await {
                        Ok(SegmentOutcome::Completed) => {
                            return self.finalize_download(&part_path).await;
                        }
                        Ok(SegmentOutcome::Paused) => {
                            self.send_state(DownloadState::Paused);
                            return Ok(());
                        }
                        Err(DownloadError::ResourceChanged) => {
                            // Some servers advertise byte ranges but ignore them in practice.
                            // Clean segmented temp files and continue with the safe single-stream path.
                            for index in 0..32 {
                                let mut name = self.final_path.as_os_str().to_os_string();
                                name.push(format!(".part.{}", index));
                                let _ = tokio::fs::remove_file(PathBuf::from(name)).await;
                            }
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
        }

        // Single-connection fallback and resume path.
        // Determine existing size for resume
        let mut start_offset = 0;
        let mut open_options = tokio::fs::OpenOptions::new();
        open_options.write(true).create(true);

        if part_path.exists() {
            if let Ok(meta) = tokio::fs::metadata(&part_path).await {
                let len = meta.len();
                if self.metadata.accept_ranges {
                    start_offset = len;
                    open_options.append(true);
                } else {
                    // Cannot resume, must truncate and start over
                    open_options.truncate(true);
                }
            }
        } else {
            open_options.truncate(true);
        }

        // If we already have the whole file, we might be done, but verify.
        if let Some(total) = self.metadata.content_length {
            if start_offset == total && total > 0 {
                return self.finalize_download(&part_path).await;
            } else if start_offset > total {
                // File on disk is larger than total? Corrupt, start over.
                start_offset = 0;
                open_options.truncate(true);
                open_options.append(false);
            }
        }

        let mut req = self.request_context.apply(self.client.get(&self.metadata.final_url));

        if start_offset > 0 {
            req = req.header(reqwest::header::RANGE, format!("bytes={}-", start_offset));
            if let Some(ref etag) = self.metadata.etag {
                req = req.header(reqwest::header::IF_RANGE, etag);
            } else if let Some(ref last_mod) = self.metadata.last_modified {
                req = req.header(reqwest::header::IF_RANGE, last_mod);
            }
        }

        self.send_state(DownloadState::Downloading);

        let response = req.send().await.map_err(|e| {
            if e.is_timeout() { DownloadError::Timeout } else { DownloadError::NetworkError(e.to_string()) }
        })?;

        let status = response.status();

        if !status.is_success() {
            return Err(DownloadError::HttpError(status.as_u16()));
        }

        // If we requested a range but got 200 OK, the server ignored Range or If-Range failed
        if start_offset > 0 && status.as_u16() == 200 {
            start_offset = 0;
            open_options.truncate(true);
            open_options.append(false); // Reset to overwrite
        } else if start_offset > 0 && status.as_u16() != 206 {
            return Err(DownloadError::HttpError(status.as_u16()));
        }

        let mut file = open_options.open(&part_path).await.map_err(|e| DownloadError::DiskError(e.to_string()))?;
        let mut buf_writer = BufWriter::with_capacity(BUFFER_SIZE, file);

        let mut stream = response.bytes_stream();
        let mut downloaded = start_offset;
        let mut last_update = Instant::now();
        let mut bytes_since_last_update: u64 = 0;
        let update_interval = std::time::Duration::from_millis(250);
        let start_time = Instant::now();

        while let Some(chunk_result) = stream.next().await {
            if self.cancel_flag.load(Ordering::Relaxed) {
                buf_writer.flush().await.map_err(|e| DownloadError::DiskError(e.to_string()))?;
                self.send_state(DownloadState::Paused);
                return Ok(());
            }

            let chunk = chunk_result.map_err(|e| {
                if e.is_timeout() { DownloadError::Timeout } else { DownloadError::NetworkError(e.to_string()) }
            })?;

            buf_writer.write_all(&chunk).await.map_err(|e| DownloadError::DiskError(e.to_string()))?;

            downloaded += chunk.len() as u64;
            bytes_since_last_update += chunk.len() as u64;

            let now = Instant::now();
            if now.duration_since(last_update) >= update_interval {
                let elapsed = now.duration_since(last_update).as_secs_f64();
                let transfer_rate = if elapsed > 0.0 { bytes_since_last_update as f64 / elapsed } else { 0.0 };
                let time_left_secs = self.metadata.content_length.and_then(|total| {
                    if transfer_rate > 0.0 && total > downloaded {
                        Some((total - downloaded) as f64 / transfer_rate)
                    } else { None }
                });

                self.send_event(EngineEvent::Progress {
                    id: self.id,
                    downloaded,
                    total_size: self.metadata.content_length,
                    transfer_rate,
                    time_left_secs,
                });

                bytes_since_last_update = 0;
                last_update = now;
            }
        }

        buf_writer.flush().await.map_err(|e| DownloadError::DiskError(e.to_string()))?;

        // Finalize
        self.finalize_download(&part_path).await
    }

    async fn finalize_download(&mut self, part_path: &Path) -> Result<(), DownloadError> {
        self.send_state(DownloadState::Completing);

        let final_dest = resolve_duplicate_filename(&self.final_path).await;

        tokio::fs::rename(part_path, &final_dest)
            .await
            .map_err(|e| DownloadError::DiskError(format!("Atomic rename failed: {}", e)))?;

        self.send_event(EngineEvent::Complete {
            id: self.id,
            final_path: final_dest,
        });
        self.send_state(DownloadState::Complete);

        Ok(())
    }
}

fn is_permanent_error(e: &DownloadError) -> bool {
    match e {
        DownloadError::HttpError(code) => {
            // 4xx errors are usually permanent (except maybe 408, 429)
            if *code >= 400 && *code < 500 && *code != 408 && *code != 429 {
                true
            } else {
                false
            }
        },
        DownloadError::DiskError(_) => true, // Usually no space left or permission denied
        _ => false,
    }
}

pub async fn resolve_duplicate_filename(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let mut counter = 1;
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("download");
    let ext = path.extension().and_then(|s| s.to_str()).map(|s| format!(".{}", s)).unwrap_or_default();

    loop {
        let new_name = format!("{} ({}){}", file_stem, counter, ext);
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}


#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::{
        extract::State,
        http::{header, HeaderMap, Method, StatusCode},
        response::IntoResponse,
        routing::any,
        Router,
    };

    async fn protected_range_file(
        State(data): State<Arc<Vec<u8>>>,
        method: Method,
        headers: HeaderMap,
    ) -> axum::response::Response {
        let cookie_ok = headers
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("session=ok"))
            .unwrap_or(false);
        let ua_ok = headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            == Some("RustIDM-Test");
        let referer_ok = headers
            .get(header::REFERER)
            .and_then(|v| v.to_str().ok())
            == Some("https://example.test/page");

        if !(cookie_ok && ua_ok && referer_ok) {
            return StatusCode::FORBIDDEN.into_response();
        }

        let mut response_headers = HeaderMap::new();
        response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
        response_headers.insert(
            header::CONTENT_LENGTH,
            data.len().to_string().parse().unwrap(),
        );

        if method == Method::HEAD {
            return (StatusCode::OK, response_headers, Vec::<u8>::new()).into_response();
        }

        if let Some(range) = headers.get(header::RANGE).and_then(|v| v.to_str().ok()) {
            if let Some(spec) = range.strip_prefix("bytes=") {
                let mut parts = spec.splitn(2, '-');
                let start = parts.next().and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
                let end = parts
                    .next()
                    .and_then(|v| if v.is_empty() { None } else { v.parse::<usize>().ok() })
                    .unwrap_or_else(|| data.len().saturating_sub(1))
                    .min(data.len().saturating_sub(1));

                if start > end || start >= data.len() {
                    return StatusCode::RANGE_NOT_SATISFIABLE.into_response();
                }

                let body = data[start..=end].to_vec();
                response_headers.insert(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, data.len()).parse().unwrap(),
                );
                response_headers.insert(
                    header::CONTENT_LENGTH,
                    body.len().to_string().parse().unwrap(),
                );
                return (StatusCode::PARTIAL_CONTENT, response_headers, body).into_response();
            }
        }

        (StatusCode::OK, response_headers, data.as_ref().clone()).into_response()
    }

    #[tokio::test]
    async fn protected_download_creates_missing_folder_and_uses_browser_context() {
        let size = 2 * 1024 * 1024 + 123;
        let data = Arc::new((0..size).map(|i| (i % 239) as u8).collect::<Vec<u8>>());

        let app = Router::new()
            .route("/protected.bin", any(protected_range_file))
            .with_state(data.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let request_context = RequestContext {
            user_agent: Some("RustIDM-Test".to_string()),
            cookies: Some("session=ok".to_string()),
            referer: Some("https://example.test/page".to_string()),
            ..Default::default()
        };

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        let url = format!("http://{}/protected.bin", address);
        let metadata = crate::engine::probe::probe_metadata_with_context(
            &client,
            &url,
            &request_context,
        )
        .await
        .unwrap();

        assert_eq!(metadata.content_length, Some(data.len() as u64));
        assert!(metadata.accept_ranges);

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "rust-idm-e2e-{}-{}",
            std::process::id(),
            unique
        ));
        let final_path = root.join("missing").join("folder").join("protected.bin");

        assert!(!final_path.parent().unwrap().exists());

        let (tx, rx) = std::sync::mpsc::channel();
        let mut downloader = Downloader::new(
            DownloadId(777),
            final_path.clone(),
            metadata,
            tx,
            Arc::new(AtomicBool::new(false)),
            8,
            request_context,
        );

        downloader.run().await;

        let mut completed_path = None;
        while let Ok(event) = rx.try_recv() {
            match event {
                EngineEvent::Complete { final_path, .. } => completed_path = Some(final_path),
                EngineEvent::Error { error, .. } => panic!("download failed: {}", error),
                _ => {}
            }
        }

        let completed_path = completed_path.expect("download did not complete");
        let actual = tokio::fs::read(&completed_path).await.unwrap();
        assert_eq!(actual.as_slice(), data.as_slice());
        assert!(final_path.parent().unwrap().exists());

        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    #[ignore = "requires public internet access"]
    async fn real_network_smoke_download() {
        let url = "https://proof.ovh.net/files/1Mb.dat".to_string();
        let request_context = RequestContext::default();

        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        let metadata = crate::engine::probe::probe_metadata_with_context(
            &client,
            &url,
            &request_context,
        )
        .await
        .unwrap();

        assert_eq!(metadata.content_length, Some(1_048_576));

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "rust-idm-real-smoke-{}-{}",
            std::process::id(),
            unique
        ));
        let final_path = root.join("nested").join("1Mb.dat");

        let (tx, rx) = std::sync::mpsc::channel();
        let mut downloader = Downloader::new(
            DownloadId(778),
            final_path.clone(),
            metadata,
            tx,
            Arc::new(AtomicBool::new(false)),
            8,
            request_context,
        );

        downloader.run().await;

        let mut completed_path = None;
        while let Ok(event) = rx.try_recv() {
            match event {
                EngineEvent::Complete { final_path, .. } => completed_path = Some(final_path),
                EngineEvent::Error { error, .. } => panic!("real network download failed: {}", error),
                _ => {}
            }
        }

        let completed_path = completed_path.expect("real network download did not complete");
        let size = tokio::fs::metadata(&completed_path).await.unwrap().len();
        assert_eq!(size, 1_048_576);

        let _ = tokio::fs::remove_dir_all(root).await;
    }
}
