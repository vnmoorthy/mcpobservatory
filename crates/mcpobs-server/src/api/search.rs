use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub method: Option<String>,
    pub since_seconds: Option<i64>,
    pub limit: Option<i64>,
    pub errors_only: Option<bool>,
}

pub async fn get(State(state): State<AppState>, Query(q): Query<SearchQuery>) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(200).clamp(1, 1000);
    let since = q
        .since_seconds
        .map(|s| chrono::Utc::now().timestamp_millis() - s.max(0) * 1000);

    let rows = match mcpobs_store::queries::search_messages(
        &state.store,
        q.method.as_deref(),
        since,
        None,
        limit,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "search");
            return (StatusCode::INTERNAL_SERVER_ERROR, "search failed").into_response();
        }
    };

    let filtered: Vec<_> = if q.errors_only.unwrap_or(false) {
        rows.into_iter().filter(|m| m.kind == "error").collect()
    } else {
        rows
    };
    Json(filtered).into_response()
}
