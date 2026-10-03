pub mod types;
pub mod probe;
pub mod retry;
pub mod segmented;
pub mod downloader;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router, http::{HeaderMap, StatusCode, header}, response::IntoResponse};
    use tokio::net::TcpListener;
    use reqwest::Client;

    async fn mock_server() -> String {
        let app = Router::new()
            .route("/file.txt", get(|headers: HeaderMap| async move {
                let body = "Hello, world! This is a test file.";
                let mut res_headers = HeaderMap::new();
                res_headers.insert(header::CONTENT_LENGTH, body.len().to_string().parse().unwrap());
                res_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
                res_headers.insert(header::ETAG, "\"abcdef123456\"".parse().unwrap());
                res_headers.insert(header::CONTENT_DISPOSITION, "attachment; filename=\"mock.txt\"".parse().unwrap());

                if let Some(range) = headers.get(header::RANGE) {
                    let range_str = range.to_str().unwrap();
                    if range_str == "bytes=0-0" {
                        res_headers.insert(header::CONTENT_RANGE, format!("bytes 0-0/{}", body.len()).parse().unwrap());
                        return (StatusCode::PARTIAL_CONTENT, res_headers, body[0..1].to_string()).into_response();
                    } else if range_str.starts_with("bytes=") {
                        let start: usize = range_str[6..range_str.len()-1].parse().unwrap_or(0);
                        res_headers.insert(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, body.len()-1, body.len()).parse().unwrap());
                        return (StatusCode::PARTIAL_CONTENT, res_headers, body[start..].to_string()).into_response();
                    }
                }

                (StatusCode::OK, res_headers, body.to_string()).into_response()
            }))
            .route("/redirect", get(|| async {
                let mut res_headers = HeaderMap::new();
                res_headers.insert(header::LOCATION, "/file.txt".parse().unwrap());
                (StatusCode::FOUND, res_headers, "").into_response()
            }))
            .route("/no-content-length", get(|| async {
                (StatusCode::OK, "No length".to_string()).into_response()
            }));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://127.0.0.1:{}", addr.port())
    }

    #[tokio::test]
    async fn test_probe_metadata() {
        let base_url = mock_server().await;
        let client = Client::builder().redirect(reqwest::redirect::Policy::limited(5)).build().unwrap();

        let meta = probe::probe_metadata(&client, &format!("{}/file.txt", base_url)).await.unwrap();
        assert_eq!(meta.content_length, Some(34));
        assert_eq!(meta.accept_ranges, true);
        assert_eq!(meta.filename, Some("mock.txt".to_string()));
    }

    #[tokio::test]
    async fn test_probe_redirect() {
        let base_url = mock_server().await;
        let client = Client::builder().redirect(reqwest::redirect::Policy::limited(5)).build().unwrap();

        let meta = probe::probe_metadata(&client, &format!("{}/redirect", base_url)).await.unwrap();
        assert_eq!(meta.content_length, Some(34)); // Resolved via redirect
        assert!(meta.final_url.ends_with("/file.txt"));
    }

    #[tokio::test]
    async fn test_probe_no_content_length() {
        let base_url = mock_server().await;
        let client = Client::new();

        let meta = probe::probe_metadata(&client, &format!("{}/no-content-length", base_url)).await.unwrap();
        // Axum adds Content-Length for an in-memory String response.
        assert_eq!(meta.content_length, Some(9));
        assert_eq!(meta.accept_ranges, false);
    }
}
