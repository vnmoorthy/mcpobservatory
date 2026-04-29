//! Background writer task. Drains an `mpsc::Receiver<Observation>` from the
//! proxy and lands rows in SQLite. Maintains an in-memory map of `(session,
//! rpc_id) → message_id` so the writer can fill `correlated_message_id` and
//! `latency_ms` when a response arrives.

use crate::queries::upsert_server;
use crate::redact::{redact_value, RedactionConfig};
use crate::schema::Store;
use anyhow::Result;
use mcpobs_core::observation::{Direction, Observation, ObservationKind};
use mcpobs_core::session::SessionId;
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Trait for fanning out persisted messages to live subscribers (the
/// `/ws/live` websocket). The writer doesn't depend on `mcpobs-server`, so
/// the server crate provides an impl and passes it in.
pub trait LiveSink: Send + Sync + 'static {
    fn publish(&self, payload: serde_json::Value);
}

/// No-op sink for tests and the `proxy` subcommand (which doesn't host the
/// live UI).
pub struct NullLiveSink;
impl LiveSink for NullLiveSink {
    fn publish(&self, _payload: serde_json::Value) {}
}

/// Hard cap on the in-memory request-correlation map. If `tools/call` fires
/// in a long-running session and never gets a response, we don't want this
/// to grow forever. Oldest-out FIFO when the cap is exceeded.
const MAX_OPEN_REQUESTS: usize = 4096;

#[derive(Debug, Clone)]
pub struct WriterHandle {
    pub join: std::sync::Arc<tokio::sync::Mutex<Option<JoinHandle<Result<()>>>>>,
}

pub fn spawn_writer(
    store: Store,
    rx: mpsc::Receiver<Observation>,
    redaction: RedactionConfig,
) -> WriterHandle {
    spawn_writer_with_sink(store, rx, redaction, NullLiveSink)
}

pub fn spawn_writer_with_sink<L: LiveSink>(
    store: Store,
    mut rx: mpsc::Receiver<Observation>,
    redaction: RedactionConfig,
    live: L,
) -> WriterHandle {
    let handle = tokio::spawn(async move {
        // (session_id, rpc_id) -> (message_id, timestamp_ms). FIFO bounded
        // by `order` to evict oldest when full.
        let mut open_requests: HashMap<(String, String), (i64, i64)> = HashMap::new();
        let mut order: VecDeque<(String, String)> = VecDeque::new();
        let mut open_sessions: std::collections::HashSet<SessionId> = Default::default();

        while let Some(mut obs) = rx.recv().await {
            // Apply redaction in-place before persisting.
            redact_value(&mut obs.payload_json, &redaction);
            redact_value(&mut obs.metadata, &redaction);

            // Open the session lazily. Server row must exist first because
            // sessions.server_name is a foreign key.
            if !open_sessions.contains(&obs.session_id) {
                if let Err(e) = upsert_server(&store, &obs.server_name, "stdio", "{}").await {
                    tracing::warn!(error = %e, "upsert_server failed");
                }
                if let Err(e) = sqlx::query(
                    r#"INSERT OR IGNORE INTO sessions(id, server_name, transport, started_at_ms)
                       VALUES (?1, ?2, ?3, ?4)"#,
                )
                .bind(obs.session_id.as_str())
                .bind(&obs.server_name)
                .bind("stdio")
                .bind(obs.timestamp.timestamp_millis())
                .execute(store.pool())
                .await
                {
                    tracing::warn!(error = %e, "insert session failed");
                }
                open_sessions.insert(obs.session_id.clone());
            }

            let payload_str =
                serde_json::to_string(&obs.payload_json).unwrap_or_else(|_| "{}".to_string());
            let metadata_str =
                serde_json::to_string(&obs.metadata).unwrap_or_else(|_| "{}".to_string());

            let kind_str = match obs.kind {
                ObservationKind::Request => "request",
                ObservationKind::Response => "response",
                ObservationKind::Error => "error",
                ObservationKind::Notification => "notification",
                ObservationKind::Unknown => "unknown",
                ObservationKind::Unparsed => "unparsed",
            };

            let direction_str = match obs.direction {
                Direction::C2s => "c2s",
                Direction::S2c => "s2c",
            };

            let res = sqlx::query(
                r#"INSERT INTO messages
                   (session_id, server_name, direction, kind, method, rpc_id, timestamp_ms,
                    payload_size_bytes, payload_json, parse_error, metadata_json)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            )
            .bind(obs.session_id.as_str())
            .bind(&obs.server_name)
            .bind(direction_str)
            .bind(kind_str)
            .bind(&obs.method)
            .bind(&obs.rpc_id)
            .bind(obs.timestamp.timestamp_millis())
            .bind(obs.payload_size_bytes as i64)
            .bind(&payload_str)
            .bind(&obs.parse_error)
            .bind(&metadata_str)
            .execute(store.pool())
            .await;

            let inserted_id = match res {
                Ok(r) => r.last_insert_rowid(),
                Err(e) => {
                    tracing::warn!(error = %e, "failed to insert message");
                    continue;
                }
            };

            // Correlate: a request opens an entry; a response/error closes it.
            let mut latency_for_publish: Option<i64> = None;
            if let Some(rpc_id) = obs.rpc_id.as_ref() {
                let key = (obs.session_id.as_str().to_string(), rpc_id.clone());
                match obs.kind {
                    ObservationKind::Request => {
                        if open_requests.len() >= MAX_OPEN_REQUESTS {
                            if let Some(stale) = order.pop_front() {
                                open_requests.remove(&stale);
                            }
                        }
                        open_requests
                            .insert(key.clone(), (inserted_id, obs.timestamp.timestamp_millis()));
                        order.push_back(key);
                    }
                    ObservationKind::Response | ObservationKind::Error => {
                        if let Some((req_id, req_ts)) = open_requests.remove(&key) {
                            // Drop the matching key from the FIFO too.
                            if let Some(pos) = order.iter().position(|k| k == &key) {
                                order.remove(pos);
                            }
                            let latency = obs.timestamp.timestamp_millis() - req_ts;
                            latency_for_publish = Some(latency);
                            let _ = sqlx::query(
                                "UPDATE messages SET correlated_message_id = ?1, latency_ms = ?2 WHERE id = ?3",
                            )
                            .bind(req_id)
                            .bind(latency)
                            .bind(inserted_id)
                            .execute(store.pool())
                            .await;
                            let _ = sqlx::query(
                                "UPDATE messages SET correlated_message_id = ?1 WHERE id = ?2",
                            )
                            .bind(inserted_id)
                            .bind(req_id)
                            .execute(store.pool())
                            .await;
                        }
                    }
                    _ => {}
                }
            }

            // Publish to live subscribers AFTER the row is durable, so
            // anyone who responds to the event by re-fetching gets a hit.
            live.publish(serde_json::json!({
                "event": "message",
                "data": {
                    "id": inserted_id,
                    "session_id": obs.session_id.as_str(),
                    "server_name": obs.server_name,
                    "direction": direction_str,
                    "kind": kind_str,
                    "method": obs.method,
                    "rpc_id": obs.rpc_id,
                    "timestamp": obs.timestamp.to_rfc3339(),
                    "latency_ms": latency_for_publish,
                },
            }));
        }

        // Channel closed; mark all open sessions ended.
        for sid in open_sessions {
            let _ = sqlx::query(
                "UPDATE sessions SET ended_at_ms = ?1 WHERE id = ?2 AND ended_at_ms IS NULL",
            )
            .bind(chrono::Utc::now().timestamp_millis())
            .bind(sid.as_str())
            .execute(store.pool())
            .await;
        }

        Ok(())
    });

    WriterHandle {
        join: std::sync::Arc::new(tokio::sync::Mutex::new(Some(handle))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queries::{get_message, list_session_messages};
    use crate::schema::open;
    use chrono::Utc;
    use mcpobs_core::observation::{Direction, Observation, ObservationKind};
    use mcpobs_core::session::SessionId;
    use serde_json::json;

    fn make_obs(
        session: &SessionId,
        direction: Direction,
        kind: ObservationKind,
        method: Option<&str>,
        rpc_id: Option<&str>,
        ts_ms: i64,
    ) -> Observation {
        Observation {
            session_id: session.clone(),
            server_name: "fs".into(),
            direction,
            kind,
            method: method.map(String::from),
            rpc_id: rpc_id.map(String::from),
            timestamp: chrono::DateTime::from_timestamp_millis(ts_ms).unwrap_or_else(Utc::now),
            payload_size_bytes: 0,
            payload_json: json!({"hello": "world"}),
            parse_error: None,
            metadata: json!({}),
        }
    }

    /// Test sink that captures live events for inspection.
    #[derive(Default)]
    struct CapturingLiveSink {
        captured: std::sync::Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
    }
    impl LiveSink for CapturingLiveSink {
        fn publish(&self, payload: serde_json::Value) {
            self.captured.lock().unwrap().push(payload);
        }
    }

    #[tokio::test]
    async fn writer_publishes_to_live_sink() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path().join("t.db")).await.unwrap();
        let (tx, rx) = mpsc::channel(64);
        let captured = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = CapturingLiveSink {
            captured: captured.clone(),
        };
        let _h = spawn_writer_with_sink(store.clone(), rx, RedactionConfig::default(), sink);

        let sid = SessionId::new();
        tx.send(make_obs(
            &sid,
            Direction::C2s,
            ObservationKind::Request,
            Some("ping"),
            Some("1"),
            0,
        ))
        .await
        .unwrap();
        drop(tx);

        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            let cap = captured.lock().unwrap();
            if !cap.is_empty() {
                let event = cap[0].get("event").and_then(|v| v.as_str()).unwrap_or("");
                assert_eq!(event, "message");
                return;
            }
        }
        panic!("writer did not publish to live sink");
    }

    #[tokio::test]
    async fn writer_correlates_request_and_response() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path().join("t.db")).await.unwrap();
        let (tx, rx) = mpsc::channel(64);
        let _h = spawn_writer(store.clone(), rx, RedactionConfig::default());

        let sid = SessionId::new();
        tx.send(make_obs(
            &sid,
            Direction::C2s,
            ObservationKind::Request,
            Some("tools/list"),
            Some("1"),
            1_000,
        ))
        .await
        .unwrap();
        tx.send(make_obs(
            &sid,
            Direction::S2c,
            ObservationKind::Response,
            None,
            Some("1"),
            1_010,
        ))
        .await
        .unwrap();
        drop(tx);

        // Give the writer a moment.
        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            let msgs = list_session_messages(&store, sid.as_str(), 10, None)
                .await
                .unwrap();
            if msgs.len() == 2 && msgs.iter().any(|m| m.latency_ms.is_some()) {
                let resp = msgs.iter().find(|m| m.kind == "response").unwrap();
                assert_eq!(resp.latency_ms, Some(10));
                let req = msgs.iter().find(|m| m.kind == "request").unwrap();
                assert_eq!(req.correlated_message_id, Some(resp.id));
                return;
            }
        }
        panic!("writer did not correlate request and response in time");
    }

    #[tokio::test]
    async fn writer_redacts_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path().join("t.db")).await.unwrap();
        let (tx, rx) = mpsc::channel(64);
        let _h = spawn_writer(store.clone(), rx, RedactionConfig::default());

        let sid = SessionId::new();
        let mut obs = make_obs(
            &sid,
            Direction::C2s,
            ObservationKind::Request,
            Some("x"),
            Some("1"),
            0,
        );
        obs.payload_json = json!({"token": "secret-value", "name": "ok"});
        tx.send(obs).await.unwrap();
        drop(tx);

        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            let msgs = list_session_messages(&store, sid.as_str(), 10, None)
                .await
                .unwrap();
            if !msgs.is_empty() {
                let stored: serde_json::Value =
                    serde_json::from_str(&msgs[0].payload_json).unwrap();
                assert_eq!(stored["token"], "[redacted]");
                assert_eq!(stored["name"], "ok");
                return;
            }
        }
        panic!("writer did not insert in time");
    }

    #[tokio::test]
    async fn writer_handles_unknown_message_id() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path().join("t.db")).await.unwrap();
        let (tx, rx) = mpsc::channel(64);
        let _h = spawn_writer(store.clone(), rx, RedactionConfig::default());

        let sid = SessionId::new();
        // Response without a matching request — should not crash; just no
        // correlation.
        tx.send(make_obs(
            &sid,
            Direction::S2c,
            ObservationKind::Response,
            None,
            Some("99"),
            0,
        ))
        .await
        .unwrap();
        drop(tx);

        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            let msgs = list_session_messages(&store, sid.as_str(), 10, None)
                .await
                .unwrap();
            if !msgs.is_empty() {
                assert!(msgs[0].correlated_message_id.is_none());
                return;
            }
        }
        panic!("writer did not insert in time");
    }

    #[tokio::test]
    async fn get_message_returns_inserted() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(dir.path().join("t.db")).await.unwrap();
        let (tx, rx) = mpsc::channel(64);
        let _h = spawn_writer(store.clone(), rx, RedactionConfig::default());

        let sid = SessionId::new();
        tx.send(make_obs(
            &sid,
            Direction::C2s,
            ObservationKind::Request,
            Some("ping"),
            Some("1"),
            0,
        ))
        .await
        .unwrap();
        drop(tx);

        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            let msgs = list_session_messages(&store, sid.as_str(), 10, None)
                .await
                .unwrap();
            if !msgs.is_empty() {
                let got = get_message(&store, msgs[0].id).await.unwrap().unwrap();
                assert_eq!(got.method.as_deref(), Some("ping"));
                return;
            }
        }
        panic!("writer did not insert in time");
    }
}
