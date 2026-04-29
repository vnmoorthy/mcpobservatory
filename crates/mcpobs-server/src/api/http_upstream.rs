//! Mounted HTTP upstream forwarder. The CLI's `start` command registers
//! one of these per HTTP upstream in the config; this handler does the
//! per-request forward.

use crate::AppState;
use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use mcpobs_core::session::SessionId;

pub async fn forward(
    state: AppState,
    server_name: String,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Pull the forwarder out of the replay coordinator (it's the same
    // instance — the daemon registers each HTTP upstream there once).
    let res = state
        .replay
        .forward_http(&server_name, SessionId::new(), body, headers)
        .await;

    match res {
        Ok(r) => {
            let mut response = axum::response::Response::new(axum::body::Body::from(r.body));
            *response.status_mut() = r.status;
            for (k, v) in r.headers.iter() {
                if k.as_str().eq_ignore_ascii_case("transfer-encoding") {
                    continue;
                }
                response.headers_mut().insert(k.clone(), v.clone());
            }
            response.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, server = %server_name, "http forward failed");
            (
                StatusCode::BAD_GATEWAY,
                format!("upstream forward failed: {e}"),
            )
                .into_response()
        }
    }
}
