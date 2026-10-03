use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG, LAST_MODIFIED, RANGE};
use reqwest::Client;

use crate::engine::types::{DownloadError, DownloadMetadata, RequestContext};

/// Probe the server to get file metadata.
/// Tries HEAD first. If it fails or doesn't return Content-Length, tries a GET with Range: bytes=0-0.
pub async fn probe_metadata(
    client: &Client,
    url: &str,
) -> Result<DownloadMetadata, DownloadError> {
    probe_metadata_with_context(client, url, &RequestContext::default()).await
}

pub async fn probe_metadata_with_context(
    client: &Client,
    url: &str,
    request_context: &RequestContext,
) -> Result<DownloadMetadata, DownloadError> {
    // 1. Try HEAD request
    let head_resp = request_context.apply(client.head(url)).send().await;

    let mut metadata = DownloadMetadata {
        final_url: url.to_string(),
        content_length: None,
        content_type: None,
        filename: None,
        accept_ranges: false,
        etag: None,
        last_modified: None,
    };

    let mut head_failed = true;

    if let Ok(resp) = head_resp {
        if resp.status().is_success() {
            head_failed = false;
            metadata.final_url = resp.url().to_string();
            metadata.content_length = resp.headers().get(CONTENT_LENGTH).and_then(|h| h.to_str().ok()).and_then(|s| s.parse().ok());
            metadata.content_type = resp.headers().get(CONTENT_TYPE).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
            metadata.etag = resp.headers().get(ETAG).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
            metadata.last_modified = resp.headers().get(LAST_MODIFIED).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
            metadata.accept_ranges = resp.headers().get(ACCEPT_RANGES).and_then(|h| h.to_str().ok()) == Some("bytes");

            if let Some(cd) = resp.headers().get("content-disposition") {
                metadata.filename = parse_content_disposition(cd.to_str().unwrap_or(""));
            }
        }
    }

    // 2. If HEAD failed or gave no content-length, fallback to GET Range: bytes=0-0
    if head_failed || metadata.content_length.is_none() {
        let get_resp = request_context.apply(client.get(url).header(RANGE, "bytes=0-0")).send().await;
        if let Ok(resp) = get_resp {
            if resp.status().is_success() || resp.status().as_u16() == 206 {
                metadata.final_url = resp.url().to_string();
                metadata.content_type = resp.headers().get(CONTENT_TYPE).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                metadata.etag = resp.headers().get(ETAG).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                metadata.last_modified = resp.headers().get(LAST_MODIFIED).and_then(|h| h.to_str().ok()).map(|s| s.to_string());

                if let Some(cd) = resp.headers().get("content-disposition") {
                    metadata.filename = parse_content_disposition(cd.to_str().unwrap_or(""));
                }

                if resp.status().as_u16() == 206 {
                    metadata.accept_ranges = true;
                    // Parse Content-Range: bytes 0-0/12345
                    if let Some(cr) = resp.headers().get(CONTENT_RANGE).and_then(|h| h.to_str().ok()) {
                        if let Some(total) = cr.split('/').last().and_then(|s| s.parse::<u64>().ok()) {
                            metadata.content_length = Some(total);
                        }
                    }
                } else if resp.status().is_success() {
                    // Full response returned, ranges not supported
                    metadata.accept_ranges = false;
                    metadata.content_length = resp.headers().get(CONTENT_LENGTH).and_then(|h| h.to_str().ok()).and_then(|s| s.parse().ok());
                }
            } else {
                return Err(DownloadError::HttpError(resp.status().as_u16()));
            }
        } else if let Err(e) = get_resp {
            return Err(DownloadError::NetworkError(e.to_string()));
        }
    }

    Ok(metadata)
}

fn parse_content_disposition(cd: &str) -> Option<String> {
    if let Some(idx) = cd.find("filename=") {
        let mut name = cd[idx + 9..].trim();
        if name.starts_with('"') && name.ends_with('"') {
            name = &name[1..name.len()-1];
        }
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

