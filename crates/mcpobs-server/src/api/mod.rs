//! REST and WebSocket handlers, plus the embedded UI assets.

mod diff;
pub mod http_upstream;
mod messages;
mod replay;
mod search;
mod servers;
mod sessions;
mod ui;
mod ws;

use crate::AppState;
use axum::{routing::get, routing::post, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/servers", get(servers::list))
        .route("/api/servers/:name/sparkline", get(servers::sparkline))
        .route("/api/sessions", get(sessions::list))
        .route("/api/sessions/:id/messages", get(sessions::messages))
        .route("/api/messages/:id", get(messages::get))
        .route("/api/messages/:id/trace", get(messages::trace))
        .route("/api/diff", get(diff::get))
        .route("/api/replay", post(replay::post))
        .route("/api/search", get(search::get))
        .route("/api/settings", get(servers::settings))
        .route("/api/health", get(health))
        .route("/ws/live", get(ws::live))
}

pub fn ui_routes() -> Router<AppState> {
    Router::new().fallback(ui::serve)
}

async fn health() -> &'static str {
    "ok"
}
