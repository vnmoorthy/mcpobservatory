//! Observations are what the proxy emits and the store consumes. They are
//! the only contract between the two crates.

use crate::protocol::jsonrpc::ParsedMessage;
use crate::session::SessionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Client to server.
    C2s,
    /// Server to client.
    S2c,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::C2s => "c2s",
            Self::S2c => "s2c",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObservationKind {
    Request,
    Response,
    Error,
    Notification,
    Unknown,
    Unparsed,
}

impl From<&ParsedMessage> for ObservationKind {
    fn from(p: &ParsedMessage) -> Self {
        match p.kind_str() {
            "request" => Self::Request,
            "response" => Self::Response,
            "error" => Self::Error,
            "notification" => Self::Notification,
            "unknown" => Self::Unknown,
            _ => Self::Unparsed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub session_id: SessionId,
    pub server_name: String,
    pub direction: Direction,
    pub kind: ObservationKind,
    pub method: Option<String>,
    pub rpc_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub payload_size_bytes: u64,
    pub payload_json: Value,
    pub parse_error: Option<String>,
    pub metadata: Value,
}

/// Sink that the proxy writes observations into. Backed by a bounded mpsc
/// channel; if it is full, [`ObservationSink::send`] returns the observation
/// for the caller to drop with a single warning. We never block the proxy on
/// observation writes.
#[derive(Clone)]
pub struct ObservationSink {
    tx: mpsc::Sender<Observation>,
}

impl ObservationSink {
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<Observation>) {
        let (tx, rx) = mpsc::channel(capacity);
        (Self { tx }, rx)
    }

    /// Best-effort send. Returns `Err(obs)` if the channel is full or closed.
    /// We return the observation so callers can choose to drop or log it
    /// without an extra allocation; the size of `Observation` is intentional.
    #[allow(clippy::result_large_err)]
    pub fn try_send(&self, obs: Observation) -> Result<(), Observation> {
        self.tx.try_send(obs).map_err(|e| match e {
            mpsc::error::TrySendError::Full(o) | mpsc::error::TrySendError::Closed(o) => o,
        })
    }

    /// Awaitable send. Blocks until the channel has room or is closed.
    #[allow(clippy::result_large_err)]
    pub async fn send(&self, obs: Observation) -> Result<(), Observation> {
        self.tx.send(obs).await.map_err(|e| e.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_round_trips() {
        for d in [Direction::C2s, Direction::S2c] {
            let s = serde_json::to_string(&d).unwrap();
            let back: Direction = serde_json::from_str(&s).unwrap();
            assert_eq!(d, back);
        }
    }

    #[tokio::test]
    async fn sink_full_returns_obs() {
        let (sink, _rx) = ObservationSink::new(1);
        let make = || Observation {
            session_id: SessionId::new(),
            server_name: "x".into(),
            direction: Direction::C2s,
            kind: ObservationKind::Request,
            method: None,
            rpc_id: None,
            timestamp: Utc::now(),
            payload_size_bytes: 0,
            payload_json: serde_json::json!({}),
            parse_error: None,
            metadata: serde_json::json!({}),
        };
        sink.try_send(make()).unwrap();
        let dropped = sink.try_send(make());
        assert!(dropped.is_err());
    }
}
