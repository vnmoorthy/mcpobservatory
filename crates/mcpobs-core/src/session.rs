//! Session metadata. A session is one bidirectional conversation between
//! a client and an upstream MCP server.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Opaque session id. New value per `mcpobs proxy` invocation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Static metadata about a session, recorded at start.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: SessionId,
    pub server_name: String,
    pub transport: TransportKind,
    pub started_at: DateTime<Utc>,
    pub client_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    Stdio,
    Http,
    Sse,
}

impl TransportKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stdio => "stdio",
            Self::Http => "http",
            Self::Sse => "sse",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_ids_are_unique() {
        let a = SessionId::new();
        let b = SessionId::new();
        assert_ne!(a.0, b.0);
    }

    #[test]
    fn transport_round_trips() {
        for k in [
            TransportKind::Stdio,
            TransportKind::Http,
            TransportKind::Sse,
        ] {
            let s = serde_json::to_string(&k).unwrap();
            let back: TransportKind = serde_json::from_str(&s).unwrap();
            assert_eq!(k, back);
        }
    }
}
