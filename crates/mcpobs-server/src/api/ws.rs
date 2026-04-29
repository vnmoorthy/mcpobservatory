use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn live(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Origin check on upgrade — browsers don't enforce same-origin on WS
    // handshakes, so the server must. Local non-browser clients (curl,
    // tooling) typically send no Origin header; we allow that case so users
    // can still tail traces from a script. Browser-issued requests must
    // match the configured allowlist.
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        if !state.config.allowed_origins.iter().any(|o| o == origin) {
            return (
                StatusCode::FORBIDDEN,
                format!("origin `{origin}` not allowed"),
            )
                .into_response();
        }
    }
    ws.on_upgrade(move |socket| handle(socket, state))
        .into_response()
}

async fn handle(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.live.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(ev) = rx.recv().await {
            let payload = match serde_json::to_string(&ev) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if sender.send(Message::Text(payload)).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) | Err(_) => break,
                Ok(_) => {}
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}
