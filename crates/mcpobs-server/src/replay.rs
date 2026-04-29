//! Replay support. The daemon owns a [`ReplayCoordinator`] that holds an
//! [`HttpForwarder`] per HTTP upstream. When the UI POSTs to `/api/replay`,
//! we look up the original message, pick the right forwarder by
//! `server_name`, re-issue the JSON-RPC payload (rewriting the `id` so the
//! upstream doesn't collide with the live session), and tag the resulting
//! observation with `replay_of=<original_id>` in the `metadata` column.

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use mcpobs_core::observation::Observation;
use mcpobs_core::protocol::mcp::is_safe_for_replay;
use mcpobs_core::session::SessionId;
use mcpobs_core::transport::http::{HttpForwardResult, HttpForwarder};
use reqwest::header::HeaderMap;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct ReplayCoordinator {
    http: RwLock<HashMap<String, HttpForwarder>>,
}

impl Default for ReplayCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayCoordinator {
    pub fn new() -> Self {
        Self {
            http: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_http(&mut self, name: String, forwarder: HttpForwarder) {
        if let Ok(mut g) = self.http.write() {
            g.insert(name, forwarder);
        }
    }

    /// Replay an HTTP-transport upstream call. Returns the upstream's
    /// response body as bytes.
    pub async fn replay_http(&self, server: &str, payload: serde_json::Value) -> Result<Bytes> {
        let forwarder = self.lookup_http(server)?;
        let body = Bytes::from(serde_json::to_vec(&payload).context("serialize replay payload")?);
        let res = forwarder
            .forward(SessionId::new(), body, HeaderMap::new())
            .await?;
        Ok(res.body)
    }

    /// Forward a fresh client request through an HTTP upstream — used by
    /// the daemon's `/mcp/<name>` route handler.
    pub async fn forward_http(
        &self,
        server: &str,
        session: SessionId,
        body: Bytes,
        client_headers: axum::http::HeaderMap,
    ) -> Result<HttpForwardResult> {
        let forwarder = self.lookup_http(server)?;
        let mut headers = HeaderMap::new();
        for (k, v) in client_headers.iter() {
            // Strip headers that don't make sense to forward.
            let name = k.as_str().to_ascii_lowercase();
            if matches!(
                name.as_str(),
                "host" | "connection" | "content-length" | "transfer-encoding"
            ) {
                continue;
            }
            if let Ok(name_h) = reqwest::header::HeaderName::try_from(k.as_str()) {
                if let Ok(val) = reqwest::header::HeaderValue::from_bytes(v.as_bytes()) {
                    headers.insert(name_h, val);
                }
            }
        }
        forwarder.forward(session, body, headers).await
    }

    fn lookup_http(&self, server: &str) -> Result<HttpForwarder> {
        let g = self
            .http
            .read()
            .map_err(|_| anyhow!("replay coord poisoned"))?;
        Ok(g.get(server)
            .ok_or_else(|| anyhow!("no http forwarder for upstream `{}`", server))?
            .clone_handle())
    }

    pub fn registered_servers(&self) -> Vec<String> {
        self.http
            .read()
            .map(|g| g.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// Simple safe-list check (used in addition to the explicit-confirmation
/// gate the UI shows). Anything not safe by name pattern requires
/// `confirmed=true` in the replay request.
pub fn is_method_safe_to_replay_unconfirmed(method: &str) -> bool {
    is_safe_for_replay(method)
}

/// Marker so the UI can group an observation as a replay result.
pub fn replay_metadata(replay_of: i64) -> serde_json::Value {
    serde_json::json!({ "replay_of": replay_of })
}

#[allow(dead_code)]
fn _phantom(_: Observation) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_methods_dont_need_confirm() {
        assert!(is_method_safe_to_replay_unconfirmed("tools/list"));
        assert!(is_method_safe_to_replay_unconfirmed("resources/read"));
        assert!(is_method_safe_to_replay_unconfirmed("ping"));
    }

    #[test]
    fn dangerous_methods_need_confirm() {
        assert!(!is_method_safe_to_replay_unconfirmed("tools/call"));
        assert!(!is_method_safe_to_replay_unconfirmed("logging/setLevel"));
    }

    #[test]
    fn empty_coord_has_no_servers() {
        let c = ReplayCoordinator::new();
        assert!(c.registered_servers().is_empty());
    }
}
