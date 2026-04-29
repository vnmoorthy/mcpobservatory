//! mcpobs-server
//!
//! Local HTTP + WebSocket server with embedded UI.

pub mod api;
pub mod config;
pub mod live;
pub mod replay;

use anyhow::{Context, Result};
use axum::http::{HeaderValue, Method};
use axum::Router;
use mcpobs_store::Store;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub use config::ServerConfig;
pub use live::{LiveBus, LiveEvent};
pub use replay::ReplayCoordinator;

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub live: LiveBus,
    pub config: Arc<ServerConfig>,
    pub replay: Arc<ReplayCoordinator>,
}

pub fn build_router(state: AppState, http_routes: Vec<(String, String)>) -> Router {
    // Explicit allowlist sourced from config — no wildcard. Same-origin
    // requests from the embedded UI continue to work because they share
    // host:port with the daemon. Cross-origin browsers hit pre-flight,
    // see no allow-origin for their host, and bail.
    let origins: Vec<HeaderValue> = state
        .config
        .allowed_origins
        .iter()
        .filter_map(|s| HeaderValue::from_str(s).ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::ORIGIN]);

    let mut router = Router::new().merge(api::routes());
    for (path, server_name) in http_routes {
        let st = state.clone();
        let name = server_name.clone();
        router = router.route(
            &path,
            axum::routing::post(move |headers, body| {
                let st = st.clone();
                let name = name.clone();
                async move { api::http_upstream::forward(st, name, headers, body).await }
            }),
        );
    }
    router
        .merge(api::ui_routes())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

pub async fn serve(
    state: AppState,
    addr: SocketAddr,
    http_routes: Vec<(String, String)>,
) -> Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    let router = build_router(state, http_routes);
    tracing::info!(%addr, "mcpobs server listening");
    axum::serve(listener, router).await.context("axum serve")?;
    Ok(())
}

pub fn make_live_bus(capacity: usize) -> LiveBus {
    LiveBus::new(capacity)
}
