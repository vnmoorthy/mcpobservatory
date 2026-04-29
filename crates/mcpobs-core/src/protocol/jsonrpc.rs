//! JSON-RPC 2.0 framing as used by MCP.
//!
//! We parse leniently. Anything we cannot classify is preserved as
//! [`JsonRpcMessage::Unknown`] with the original bytes; callers forward it
//! verbatim and emit an observation with `parse_error` set.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// JSON-RPC ids may be number, string, or null. We keep them in the
/// canonical wire form so equality round-trips.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Num(i64),
    Str(String),
    Null,
}

impl JsonRpcId {
    pub fn as_string(&self) -> String {
        match self {
            Self::Num(n) => n.to_string(),
            Self::Str(s) => s.clone(),
            Self::Null => "null".to_string(),
        }
    }
}

/// One JSON-RPC frame. Untagged because the spec discriminates on which of
/// `method`, `result`, `error` is present rather than a tag field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request {
        jsonrpc: String,
        id: JsonRpcId,
        method: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
    Response {
        jsonrpc: String,
        id: JsonRpcId,
        result: Value,
    },
    Error {
        jsonrpc: String,
        id: JsonRpcId,
        error: JsonRpcError,
    },
    Notification {
        jsonrpc: String,
        method: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
    /// Anything that does not fit. The original `Value` is kept so we can
    /// re-emit it without information loss.
    Unknown(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid utf-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("invalid json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result of parsing a single frame. We keep raw bytes so the proxy can
/// forward them verbatim regardless of parse outcome.
#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub raw: Vec<u8>,
    pub parsed: Option<JsonRpcMessage>,
    pub parse_error: Option<String>,
}

impl ParsedMessage {
    pub fn parse(raw: Vec<u8>) -> Self {
        match serde_json::from_slice::<Value>(&raw) {
            Ok(value) => match serde_json::from_value::<JsonRpcMessage>(value.clone()) {
                Ok(parsed) => Self {
                    raw,
                    parsed: Some(parsed),
                    parse_error: None,
                },
                Err(_) => Self {
                    raw,
                    parsed: Some(JsonRpcMessage::Unknown(value)),
                    parse_error: None,
                },
            },
            Err(e) => Self {
                raw,
                parsed: None,
                parse_error: Some(e.to_string()),
            },
        }
    }

    pub fn method(&self) -> Option<&str> {
        match self.parsed.as_ref()? {
            JsonRpcMessage::Request { method, .. }
            | JsonRpcMessage::Notification { method, .. } => Some(method.as_str()),
            _ => None,
        }
    }

    pub fn id(&self) -> Option<JsonRpcId> {
        match self.parsed.as_ref()? {
            JsonRpcMessage::Request { id, .. }
            | JsonRpcMessage::Response { id, .. }
            | JsonRpcMessage::Error { id, .. } => Some(id.clone()),
            _ => None,
        }
    }

    pub fn is_response(&self) -> bool {
        matches!(
            self.parsed,
            Some(JsonRpcMessage::Response { .. } | JsonRpcMessage::Error { .. })
        )
    }

    pub fn is_request(&self) -> bool {
        matches!(self.parsed, Some(JsonRpcMessage::Request { .. }))
    }

    pub fn is_notification(&self) -> bool {
        matches!(self.parsed, Some(JsonRpcMessage::Notification { .. }))
    }

    pub fn is_error(&self) -> bool {
        matches!(self.parsed, Some(JsonRpcMessage::Error { .. }))
    }

    pub fn kind_str(&self) -> &'static str {
        match self.parsed {
            Some(JsonRpcMessage::Request { .. }) => "request",
            Some(JsonRpcMessage::Response { .. }) => "response",
            Some(JsonRpcMessage::Error { .. }) => "error",
            Some(JsonRpcMessage::Notification { .. }) => "notification",
            Some(JsonRpcMessage::Unknown(_)) => "unknown",
            None => "unparsed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_request() {
        let raw = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#.to_vec();
        let m = ParsedMessage::parse(raw);
        assert!(m.is_request());
        assert_eq!(m.method(), Some("initialize"));
        assert_eq!(m.id(), Some(JsonRpcId::Num(1)));
    }

    #[test]
    fn parses_response() {
        let raw = br#"{"jsonrpc":"2.0","id":"abc","result":{"ok":true}}"#.to_vec();
        let m = ParsedMessage::parse(raw);
        assert!(m.is_response());
        assert_eq!(m.id(), Some(JsonRpcId::Str("abc".into())));
    }

    #[test]
    fn parses_error_response() {
        let raw =
            br#"{"jsonrpc":"2.0","id":2,"error":{"code":-32601,"message":"Method not found"}}"#
                .to_vec();
        let m = ParsedMessage::parse(raw);
        assert!(m.is_error());
        assert_eq!(m.id(), Some(JsonRpcId::Num(2)));
    }

    #[test]
    fn parses_notification() {
        let raw = br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#.to_vec();
        let m = ParsedMessage::parse(raw);
        assert!(m.is_notification());
        assert_eq!(m.method(), Some("notifications/initialized"));
    }

    #[test]
    fn malformed_json_keeps_raw() {
        let raw = b"{not json".to_vec();
        let m = ParsedMessage::parse(raw.clone());
        assert!(m.parsed.is_none());
        assert!(m.parse_error.is_some());
        assert_eq!(m.raw, raw);
    }

    #[test]
    fn null_id() {
        let raw = br#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#
            .to_vec();
        let m = ParsedMessage::parse(raw);
        assert_eq!(m.id(), Some(JsonRpcId::Null));
    }
}
