use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

#[derive(Deserialize)]
pub struct DiffQuery {
    a: i64,
    b: i64,
}

#[derive(Serialize)]
pub struct DiffResponse {
    a: mcpobs_store::MessageRow,
    b: mcpobs_store::MessageRow,
    changes: Vec<DiffChange>,
}

#[derive(Serialize)]
pub struct DiffChange {
    tag: String,
    text: String,
}

pub async fn get(State(state): State<AppState>, Query(q): Query<DiffQuery>) -> impl IntoResponse {
    if q.a == q.b {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("a and b must differ — both are #{}", q.a)
            })),
        )
            .into_response();
    }
    let a = match mcpobs_store::queries::get_message(&state.store, q.a).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("message #{} not found", q.a)
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "get_message a");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("get_message a: {}", e)})),
            )
                .into_response();
        }
    };
    let b = match mcpobs_store::queries::get_message(&state.store, q.b).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("message #{} not found", q.b)
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "get_message b");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("get_message b: {}", e)})),
            )
                .into_response();
        }
    };

    let pretty_a = pretty(&a.payload_json);
    let pretty_b = pretty(&b.payload_json);

    let diff = TextDiff::from_lines(&pretty_a, &pretty_b);
    let mut changes = Vec::new();
    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Delete => "delete",
            ChangeTag::Insert => "insert",
            ChangeTag::Equal => "equal",
        };
        changes.push(DiffChange {
            tag: tag.into(),
            text: change.value().to_string(),
        });
    }

    Json(DiffResponse { a, b, changes }).into_response()
}

fn pretty(s: &str) -> String {
    serde_json::from_str::<serde_json::Value>(s)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| s.to_string())
}
