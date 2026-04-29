//! SSE transport.
//!
//! The proxy maintains one long-lived `GET <upstream>/sse` and re-emits each
//! event as an s2c observation. The client's POST endpoint (for sending
//! requests back to the upstream) is bridged via [`super::http`].

use crate::observation::{Direction, Observation, ObservationKind, ObservationSink};
use crate::protocol::jsonrpc::ParsedMessage;
use crate::session::{SessionId, SessionMeta, TransportKind};
use anyhow::{Context, Result};
use chrono::Utc;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseUpstream {
    pub url: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

pub struct SseSubscriber {
    client: reqwest::Client,
    upstream: SseUpstream,
    server_name: String,
    sink: ObservationSink,
}

impl SseSubscriber {
    pub fn new(server_name: String, upstream: SseUpstream, sink: ObservationSink) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .build()
                .context("reqwest client")?,
            upstream,
            server_name,
            sink,
        })
    }

    /// Subscribe to the upstream SSE channel and emit each event as an
    /// observation. Returns when the upstream closes the connection.
    pub async fn run(self) -> Result<()> {
        let session = SessionMeta {
            id: SessionId::new(),
            server_name: self.server_name.clone(),
            transport: TransportKind::Sse,
            started_at: Utc::now(),
            client_hint: None,
        };

        let mut req = self.client.get(&self.upstream.url);
        for (k, v) in &self.upstream.headers {
            req = req.header(k, v);
        }
        let resp = req
            .send()
            .await
            .with_context(|| format!("GET {}", self.upstream.url))?;
        let mut stream = resp.bytes_stream().eventsource();

        while let Some(event) = stream.next().await {
            match event {
                Ok(ev) => {
                    let raw = ev.data.into_bytes();
                    let parsed = ParsedMessage::parse(raw.clone());
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
                        direction: Direction::S2c,
                        kind,
                        method,
                        rpc_id,
                        timestamp: Utc::now(),
                        payload_size_bytes: raw.len() as u64,
                        payload_json,
                        parse_error: parsed.parse_error.clone(),
                        metadata: serde_json::json!({"sse_event": ev.event}),
                    };
                    if self.sink.try_send(obs).is_err() {
                        tracing::warn!("sse observation dropped");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "sse stream error");
                    break;
                }
            }
        }
        Ok(())
    }
}
