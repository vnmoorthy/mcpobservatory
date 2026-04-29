//! Streamable HTTP transport.
//!
//! The daemon listens on `[upstreams.<name>] listen_path` (default
//! `/mcp/<name>`). It accepts POST with JSON-RPC bodies and forwards them to
//! `upstream_url`. Responses may be either JSON or SSE; we sniff the first
//! few bytes of the response body to decide.

use crate::observation::{Direction, Observation, ObservationKind, ObservationSink};
use crate::protocol::jsonrpc::ParsedMessage;
use crate::session::{SessionId, SessionMeta, TransportKind};
use anyhow::{Context, Result};
use bytes::Bytes;
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

/// Configuration for an HTTP upstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpUpstream {
    pub url: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

/// One forwarded request: JSON-in, JSON-or-SSE-out. Cloning is cheap
/// (reqwest::Client is internally `Arc`'d and the rest is small).
#[derive(Clone)]
pub struct HttpForwarder {
    client: reqwest::Client,
    upstream: HttpUpstream,
    server_name: String,
    sink: ObservationSink,
}

impl HttpForwarder {
    /// Returns a fresh handle pointing at the same upstream and sink.
    pub fn clone_handle(&self) -> Self {
        self.clone()
    }
}

impl HttpForwarder {
    pub fn new(server_name: String, upstream: HttpUpstream, sink: ObservationSink) -> Result<Self> {
        let client = reqwest::Client::builder()
            // No timeout — we let the client's own JSON-RPC layer handle that.
            .build()
            .context("reqwest client build")?;
        Ok(Self {
            client,
            upstream,
            server_name,
            sink,
        })
    }

    /// Forward one request and return the upstream response body. Emits two
    /// observations (c2s and s2c). The session id is supplied so the daemon
    /// can re-use sessions across multiple requests on the same connection.
    pub async fn forward(
        &self,
        session_id: SessionId,
        body: Bytes,
        extra_headers: HeaderMap,
    ) -> Result<HttpForwardResult> {
        let session = SessionMeta {
            id: session_id.clone(),
            server_name: self.server_name.clone(),
            transport: TransportKind::Http,
            started_at: Utc::now(),
            client_hint: None,
        };

        // Emit c2s observation.
        emit(
            &self.sink,
            &session,
            Direction::C2s,
            body.clone(),
            serde_json::json!({}),
        );

        // Build the request.
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        for (k, v) in &self.upstream.headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::try_from(k),
                HeaderValue::from_str(v),
            ) {
                headers.insert(name, val);
            }
        }
        for (k, v) in extra_headers.iter() {
            headers.insert(k, v.clone());
        }

        let resp = self
            .client
            .post(&self.upstream.url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .with_context(|| format!("POST {}", self.upstream.url))?;

        let status = resp.status();
        let resp_headers = resp.headers().clone();
        let resp_body = resp.bytes().await.context("read upstream body")?;

        let is_sse = resp_headers
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.contains("text/event-stream"))
            .unwrap_or_else(|| resp_body.starts_with(b"event:") || resp_body.starts_with(b"data:"));

        emit(
            &self.sink,
            &session,
            Direction::S2c,
            resp_body.clone(),
            serde_json::json!({"http_status": status.as_u16(), "sse": is_sse}),
        );

        Ok(HttpForwardResult {
            status,
            headers: resp_headers,
            body: resp_body,
            is_sse,
        })
    }
}

pub struct HttpForwardResult {
    pub status: reqwest::StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub is_sse: bool,
}

fn emit(
    sink: &ObservationSink,
    session: &SessionMeta,
    direction: Direction,
    bytes: Bytes,
    extra_metadata: serde_json::Value,
) {
    let parsed = ParsedMessage::parse(bytes.to_vec());
    let kind = ObservationKind::from(&parsed);
    let method = parsed.method().map(|s| s.to_string());
    let rpc_id = parsed.id().map(|i| i.as_string());
    let payload_json = parsed
        .parsed
        .as_ref()
        .and_then(|m| serde_json::to_value(m).ok())
        .unwrap_or(serde_json::Value::Null);

    let obs = Observation {
        session_id: session.id.clone(),
        server_name: session.server_name.clone(),
        direction,
        kind,
        method,
        rpc_id,
        timestamp: Utc::now(),
        payload_size_bytes: bytes.len() as u64,
        payload_json,
        parse_error: parsed.parse_error.clone(),
        metadata: extra_metadata,
    };
    if sink.try_send(obs).is_err() {
        tracing::warn!("observation channel full; dropping http observation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_serde() {
        let u = HttpUpstream {
            url: "http://localhost:9000/mcp".into(),
            headers: [("X-Auth".into(), "abc".into())].into_iter().collect(),
        };
        let s = serde_json::to_string(&u).unwrap();
        let back: HttpUpstream = serde_json::from_str(&s).unwrap();
        assert_eq!(u.url, back.url);
    }
}
