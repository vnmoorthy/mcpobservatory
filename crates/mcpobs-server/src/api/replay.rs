use crate::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ReplayRequest {
    pub of: i64,
    #[serde(default)]
    pub confirmed: bool,
    /// If present, replaces `params` on the original payload before re-issue.
    /// Lets the UI tweak arguments before replaying a `tools/call`.
    #[serde(default)]
    pub override_params: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct ReplayResponse {
    pub status: String,
    pub original_id: i64,
    pub upstream: String,
    pub method: Option<String>,
    pub response_body: serde_json::Value,
}

pub async fn post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ReplayRequest>,
) -> impl IntoResponse {
    // Origin check. Reject unknown origins AND missing Origin — browsers
    // always send Origin, and we want non-browser clients to set it
    // explicitly so /api/replay can never be silently invoked from any
    // local process that happens to know the URL.
    let origin = headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if origin.is_empty() || !state.config.allowed_origins.iter().any(|o| o == origin) {
        return (
            StatusCode::FORBIDDEN,
            format!("origin `{origin}` not allowed"),
        )
            .into_response();
    }

    // Look up the original message and decide whether replay is allowed.
    let original = match mcpobs_store::queries::get_message(&state.store, body.of).await {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "original message not found").into_response(),
        Err(e) => {
            tracing::error!(error = %e, "replay get_message");
            return (StatusCode::INTERNAL_SERVER_ERROR, "lookup failed").into_response();
        }
    };

    let method = original.method.as_deref().unwrap_or("");
    let safe = crate::replay::is_method_safe_to_replay_unconfirmed(method);
    if !safe && !body.confirmed {
        return (
            StatusCode::PRECONDITION_REQUIRED,
            Json(serde_json::json!({
                "status": "needs_confirmation",
                "method": method,
                "reason": "method is not in the readonly safe-list (`*/list`, `*/get`, `*/read`, `ping`)",
            })),
        )
            .into_response();
    }

    // Build the JSON-RPC payload to send. Use a fresh id so we don't
    // collide with the live session's id space.
    let mut payload: serde_json::Value =
        serde_json::from_str(&original.payload_json).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "id".into(),
            serde_json::Value::String(format!("replay-{}", body.of)),
        );
        if let Some(new_params) = body.override_params.clone() {
            obj.insert("params".into(), new_params);
        }
    }

    // We only support replay against HTTP upstreams in v0. stdio replay
    // would need to either reuse the running proxy's process (we don't
    // hold a handle to it) or spawn a fresh one (which would create a
    // separate session). Document this in the response.
    let registered = state.replay.registered_servers();
    if !registered.iter().any(|s| s == &original.server_name) {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "not_supported",
                "reason": format!(
                    "upstream `{}` is not an HTTP upstream registered with the daemon. v0 supports HTTP replay only; stdio replay requires the proxy process to expose a control channel (planned for v0.2).",
                    original.server_name
                ),
            })),
        )
            .into_response();
    }

    let body_bytes = match state
        .replay
        .replay_http(&original.server_name, payload.clone())
        .await
    {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, "replay forward");
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"status": "upstream_error", "error": e.to_string()})),
            )
                .into_response();
        }
    };

    let response_value: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap_or(
        serde_json::Value::String(String::from_utf8_lossy(&body_bytes).into_owned()),
    );

    Json(ReplayResponse {
        status: "ok".into(),
        original_id: body.of,
        upstream: original.server_name,
        method: original.method,
        response_body: response_value,
    })
    .into_response()
}
