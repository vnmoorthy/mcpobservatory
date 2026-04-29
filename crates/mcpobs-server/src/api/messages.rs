use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn get(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    match mcpobs_store::queries::get_message(&state.store, id).await {
        Ok(Some(m)) => Json(m).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "message not found").into_response(),
        Err(e) => {
            tracing::error!(error = %e, "get_message");
            (StatusCode::INTERNAL_SERVER_ERROR, "get_message failed").into_response()
        }
    }
}

pub async fn trace(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    match mcpobs_store::queries::get_trace_tree(&state.store, id).await {
        Ok(Some(tree)) => Json(tree).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "trace not found").into_response(),
        Err(e) => {
            tracing::error!(error = %e, "get_trace");
            (StatusCode::INTERNAL_SERVER_ERROR, "get_trace failed").into_response()
        }
    }
}
