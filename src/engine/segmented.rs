use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::time::Instant;

use futures_util::StreamExt;
use reqwest::Client;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::engine::types::{DownloadError, DownloadId, EngineEvent, RequestContext};

const MIN_SEGMENT_SIZE: u64 = 512 * 1024;
const MAX_CONNECTIONS: u8 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentOutcome {
    Completed,
    Paused,
}

struct ProgressClock {
    at: Instant,
    bytes: u64,
}

fn suffixed_path(path: &Path, suffix: &str) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

pub async fn download_to_part(
    client: Client,
    id: DownloadId,
    url: String,
    final_path: &Path,
    total_size: u64,
    requested_connections: u8,
    cancel_flag: Arc<AtomicBool>,
    event_tx: mpsc::Sender<EngineEvent>,
    etag: Option<String>,
    last_modified: Option<String>,
    request_context: RequestContext,
) -> Result<SegmentOutcome, DownloadError> {
    let requested = requested_connections.clamp(1, MAX_CONNECTIONS) as u64;
    let by_size = ((total_size + MIN_SEGMENT_SIZE - 1) / MIN_SEGMENT_SIZE).max(1);
    let connection_count = requested.min(by_size).max(1) as usize;

    if connection_count <= 1 {
        return Err(DownloadError::NetworkError(
            "Segmented download not useful for this file size".to_string(),
        ));
    }

    let base = total_size / connection_count as u64;
    let remainder = total_size % connection_count as u64;
    let mut ranges = Vec::with_capacity(connection_count);
    let mut cursor = 0_u64;

    for i in 0..connection_count {
        let len = base + if (i as u64) < remainder { 1 } else { 0 };
        let start = cursor;
        let end = start + len - 1;
        ranges.push((start, end, len));
        cursor = end + 1;
    }

    let downloaded = Arc::new(AtomicU64::new(0));
    let mut existing_total = 0_u64;

    for (index, (_, _, segment_len)) in ranges.iter().enumerate() {
        let path = suffixed_path(final_path, &format!(".part.{}", index));
        let existing = tokio::fs::metadata(&path).await.map(|m| m.len()).unwrap_or(0);

        if existing > *segment_len {
            let _ = tokio::fs::remove_file(&path).await;
        } else {
            existing_total += existing;
        }
    }

    downloaded.store(existing_total, Ordering::Relaxed);
    let progress_clock = Arc::new(Mutex::new(ProgressClock {
        at: Instant::now(),
        bytes: existing_total,
    }));

    let _ = event_tx.send(EngineEvent::Progress {
        id,
        downloaded: existing_total,
        total_size: Some(total_size),
        transfer_rate: 0.0,
        time_left_secs: None,
    });

    let mut tasks = tokio::task::JoinSet::new();

    for (index, (range_start, range_end, segment_len)) in ranges.iter().copied().enumerate() {
        let path = suffixed_path(final_path, &format!(".part.{}", index));
        let existing = tokio::fs::metadata(&path).await.map(|m| m.len()).unwrap_or(0);

        if existing == segment_len {
            continue;
        }

        let client = client.clone();
        let url = url.clone();
        let cancel_flag = cancel_flag.clone();
        let downloaded = downloaded.clone();
        let progress_clock = progress_clock.clone();
        let event_tx = event_tx.clone();
        let etag = etag.clone();
        let last_modified = last_modified.clone();
        let request_context = request_context.clone();

        tasks.spawn(async move {
            let absolute_start = range_start + existing;
            let mut request = request_context.apply(
                client
                    .get(&url)
                    .header(reqwest::header::RANGE, format!("bytes={}-{}", absolute_start, range_end))
            );

            if let Some(value) = etag {
                request = request.header(reqwest::header::IF_RANGE, value);
            } else if let Some(value) = last_modified {
                request = request.header(reqwest::header::IF_RANGE, value);
            }

            let response = request.send().await.map_err(|e| {
                if e.is_timeout() {
                    DownloadError::Timeout
                } else {
                    DownloadError::NetworkError(e.to_string())
                }
            })?;

            let status = response.status().as_u16();
            if status == 200 {
                // Server ignored the Range request. Caller can fall back to one connection.
                return Err(DownloadError::ResourceChanged);
            }
            if status != 206 {
                return Err(DownloadError::HttpError(status));
            }

            let mut options = tokio::fs::OpenOptions::new();
            options.create(true).write(true).append(true);
            let file = options.open(&path).await
                .map_err(|e| DownloadError::DiskError(e.to_string()))?;
            let mut writer = BufWriter::with_capacity(128 * 1024, file);
            let mut stream = response.bytes_stream();
            let mut segment_downloaded = existing;

            while let Some(chunk_result) = stream.next().await {
                if cancel_flag.load(Ordering::Relaxed) {
                    writer.flush().await
                        .map_err(|e| DownloadError::DiskError(e.to_string()))?;
                    return Err(DownloadError::Cancelled);
                }

                let chunk = chunk_result.map_err(|e| {
                    if e.is_timeout() {
                        DownloadError::Timeout
                    } else {
                        DownloadError::NetworkError(e.to_string())
                    }
                })?;

                segment_downloaded += chunk.len() as u64;
                if segment_downloaded > segment_len {
                    return Err(DownloadError::ResourceChanged);
                }

                writer.write_all(&chunk).await
                    .map_err(|e| DownloadError::DiskError(e.to_string()))?;

                let global = downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed)
                    + chunk.len() as u64;

                if let Ok(mut clock) = progress_clock.lock() {
                    let elapsed = clock.at.elapsed();
                    if elapsed >= std::time::Duration::from_millis(250) {
                        let new_bytes = global.saturating_sub(clock.bytes);
                        let rate = if elapsed.as_secs_f64() > 0.0 {
                            new_bytes as f64 / elapsed.as_secs_f64()
                        } else {
                            0.0
                        };
                        let time_left_secs = if rate > 0.0 && global < total_size {
                            Some((total_size - global) as f64 / rate)
                        } else {
                            None
                        };

                        let _ = event_tx.send(EngineEvent::Progress {
                            id,
                            downloaded: global,
                            total_size: Some(total_size),
                            transfer_rate: rate,
                            time_left_secs,
                        });

                        clock.at = Instant::now();
                        clock.bytes = global;
                    }
                }
            }

            writer.flush().await
                .map_err(|e| DownloadError::DiskError(e.to_string()))?;

            if segment_downloaded != segment_len {
                return Err(DownloadError::NetworkError(format!(
                    "Segment {} ended early: got {} of {} bytes",
                    index, segment_downloaded, segment_len
                )));
            }

            Ok::<(), DownloadError>(())
        });
    }

    let mut paused = false;
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(DownloadError::Cancelled)) => paused = true,
            Ok(Err(error)) => {
                tasks.abort_all();
                return Err(error);
            }
            Err(error) => {
                tasks.abort_all();
                return Err(DownloadError::NetworkError(error.to_string()));
            }
        }
    }

    if paused || cancel_flag.load(Ordering::Relaxed) {
        return Ok(SegmentOutcome::Paused);
    }

    let part_path = suffixed_path(final_path, ".part");
    let output = tokio::fs::File::create(&part_path).await
        .map_err(|e| DownloadError::DiskError(e.to_string()))?;
    let mut writer = BufWriter::with_capacity(256 * 1024, output);

    for index in 0..connection_count {
        let segment_path = suffixed_path(final_path, &format!(".part.{}", index));
        let mut segment = tokio::fs::File::open(&segment_path).await
            .map_err(|e| DownloadError::DiskError(e.to_string()))?;
        tokio::io::copy(&mut segment, &mut writer).await
            .map_err(|e| DownloadError::DiskError(e.to_string()))?;
    }

    writer.flush().await
        .map_err(|e| DownloadError::DiskError(e.to_string()))?;
    drop(writer);

    let assembled_size = tokio::fs::metadata(&part_path).await
        .map_err(|e| DownloadError::DiskError(e.to_string()))?
        .len();

    if assembled_size != total_size {
        return Err(DownloadError::ResourceChanged);
    }

    for index in 0..connection_count {
        let segment_path = suffixed_path(final_path, &format!(".part.{}", index));
        let _ = tokio::fs::remove_file(segment_path).await;
    }

    let _ = event_tx.send(EngineEvent::Progress {
        id,
        downloaded: total_size,
        total_size: Some(total_size),
        transfer_rate: 0.0,
        time_left_secs: Some(0.0),
    });

    Ok(SegmentOutcome::Completed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::State,
        http::{header, HeaderMap, StatusCode},
        response::IntoResponse,
        routing::get,
        Router,
    };

    async fn range_file(
        State(data): State<Arc<Vec<u8>>>,
        headers: HeaderMap,
    ) -> axum::response::Response {
        let Some(range) = headers
            .get(header::RANGE)
            .and_then(|value| value.to_str().ok())
        else {
            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::CONTENT_LENGTH,
                data.len().to_string().parse().unwrap(),
            );
            response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
            return (StatusCode::OK, response_headers, data.as_ref().clone()).into_response();
        };

        let Some(spec) = range.strip_prefix("bytes=") else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        let mut pieces = spec.splitn(2, '-');
        let start = pieces.next()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let end = pieces.next()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or_else(|| data.len().saturating_sub(1))
            .min(data.len().saturating_sub(1));

        if start > end || start >= data.len() {
            return StatusCode::RANGE_NOT_SATISFIABLE.into_response();
        }

        let body = data[start..=end].to_vec();
        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{}", start, end, data.len()).parse().unwrap(),
        );
        response_headers.insert(
            header::CONTENT_LENGTH,
            body.len().to_string().parse().unwrap(),
        );
        response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());

        (StatusCode::PARTIAL_CONTENT, response_headers, body).into_response()
    }

    #[tokio::test]
    async fn segmented_download_assembles_exact_bytes() {
        let size = 4 * 1024 * 1024 + 137;
        let data = Arc::new(
            (0..size)
                .map(|index| (index % 251) as u8)
                .collect::<Vec<u8>>(),
        );

        let app = Router::new()
            .route("/file.bin", get(range_file))
            .with_state(data.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let final_path = std::env::temp_dir()
            .join(format!("rust-idm-segmented-{}-{}.bin", std::process::id(), unique));

        let (tx, _rx) = std::sync::mpsc::channel();
        let outcome = download_to_part(
            Client::new(),
            DownloadId(99),
            format!("http://{}/file.bin", address),
            &final_path,
            data.len() as u64,
            8,
            Arc::new(AtomicBool::new(false)),
            tx,
            None,
            None,
            RequestContext::default(),
        )
        .await
        .unwrap();

        assert_eq!(outcome, SegmentOutcome::Completed);

        let part_path = suffixed_path(&final_path, ".part");
        let actual = tokio::fs::read(&part_path).await.unwrap();
        assert_eq!(actual.as_slice(), data.as_slice());

        let _ = tokio::fs::remove_file(part_path).await;
        for index in 0..32 {
            let segment_path = suffixed_path(&final_path, &format!(".part.{}", index));
            let _ = tokio::fs::remove_file(segment_path).await;
        }
    }
}
