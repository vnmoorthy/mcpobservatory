use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match mcpobs_store::queries::list_servers_with_latency(&state.store).await {
        Ok(rows) => Json(rows).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "list_servers");
            (StatusCode::INTERNAL_SERVER_ERROR, "list_servers failed").into_response()
        }
    }
}

#[derive(Serialize)]
pub struct SparklineResponse {
    /// 60 buckets, each is the count of messages in that bucket.
    pub buckets: Vec<i64>,
    /// Width of each bucket in seconds.
    pub bucket_seconds: i64,
}

pub async fn sparkline(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match mcpobs_store::queries::server_sparkline(&state.store, &name, 60, 60).await {
        Ok(buckets) => Json(SparklineResponse {
            buckets,
            bucket_seconds: 60,
        })
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "sparkline");
            (StatusCode::INTERNAL_SERVER_ERROR, "sparkline failed").into_response()
        }
    }
}

#[derive(Serialize)]
pub struct SettingsResponse {
    pub listen: String,
    pub allowed_origins: Vec<String>,
    pub retention_days: u32,
    pub upstreams: Vec<UpstreamRow>,
    pub mcp_spec_revision: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct UpstreamRow {
    pub name: String,
    pub transport: String,
}

pub async fn settings(State(state): State<AppState>) -> impl IntoResponse {
    let upstreams = mcpobs_store::queries::list_servers_with_latency(&state.store)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|s| UpstreamRow {
            name: s.name,
            transport: s.transport,
        })
        .collect();

    Json(SettingsResponse {
        listen: state.config.listen.to_string(),
        allowed_origins: state.config.allowed_origins.clone(),
        retention_days: state.config.retention_days,
        upstreams,
        mcp_spec_revision: mcpobs_core::MCP_SPEC_REVISION,
        version: env!("CARGO_PKG_VERSION"),
    })
    .into_response()
}
