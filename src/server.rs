use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use std::sync::Arc;
use crossbeam_channel::Sender;
use eframe::egui::Context;

#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    pub url: String,
    pub user_agent: Option<String>,
    pub cookies: Option<String>,
    pub referer: Option<String>,
}

pub struct ServerState {
    pub sender: Sender<DownloadRequest>,
    pub ctx: Context,
}

pub async fn start_server(state: Arc<ServerState>) {
    let app = Router::new()
        .route("/download", post(handle_download))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:8765").await {
        Ok(l) => { println!("Server listening on 8765"); l },
        Err(e) => { println!("Server bind failed: {}", e); return; }
    };
    axum::serve(listener, app).await.unwrap();
}

async fn handle_download(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Json(payload): Json<DownloadRequest>,
) -> Result<&'static str, StatusCode> {
    // Validate request via custom header (prevents simple CSRF from arbitrary websites)
    if headers.get("X-Rust-IDM-Auth").and_then(|v| v.to_str().ok()) != Some("secure-local-token") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Validate URL scheme
    if !payload.url.starts_with("http://") && !payload.url.starts_with("https://") {
        return Err(StatusCode::BAD_REQUEST);
    }

    if state.sender.send(payload).is_ok() {
        state.ctx.request_repaint();
        Ok("OK")
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
