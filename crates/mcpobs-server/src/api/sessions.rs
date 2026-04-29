use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListQuery {
    server: Option<String>,
    limit: Option<i64>,
}

pub async fn list(State(state): State<AppState>, Query(q): Query<ListQuery>) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    match mcpobs_store::queries::list_sessions(&state.store, q.server.as_deref(), limit).await {
        Ok(rows) => Json(rows).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "list_sessions");
            (StatusCode::INTERNAL_SERVER_ERROR, "list_sessions failed").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    after: Option<i64>,
    limit: Option<i64>,
}

pub async fn messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<MessagesQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(200).clamp(1, 2000);
    match mcpobs_store::queries::list_session_messages(&state.store, &id, limit, q.after).await {
        Ok(rows) => Json(rows).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "list_session_messages");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "list_session_messages failed",
            )
                .into_response()
        }
    }
}
