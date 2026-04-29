//! MCP method names and a small set of typed wrappers.
//!
//! We deliberately avoid modelling every MCP type. The proxy treats payloads
//! as opaque JSON. These wrappers are only what we need to drive the UI and
//! the safety-rated replay confirmation gate.

use crate::protocol::jsonrpc::JsonRpcMessage;

pub const METHOD_INITIALIZE: &str = "initialize";
pub const METHOD_INITIALIZED: &str = "notifications/initialized";
pub const METHOD_PING: &str = "ping";

pub const METHOD_TOOLS_LIST: &str = "tools/list";
pub const METHOD_TOOLS_CALL: &str = "tools/call";

pub const METHOD_RESOURCES_LIST: &str = "resources/list";
pub const METHOD_RESOURCES_READ: &str = "resources/read";
pub const METHOD_RESOURCES_TEMPLATES_LIST: &str = "resources/templates/list";

pub const METHOD_PROMPTS_LIST: &str = "prompts/list";
pub const METHOD_PROMPTS_GET: &str = "prompts/get";

pub const METHOD_LOGGING_SET_LEVEL: &str = "logging/setLevel";

pub const METHOD_NOTIFICATIONS_PROGRESS: &str = "notifications/progress";
pub const METHOD_NOTIFICATIONS_CANCELLED: &str = "notifications/cancelled";
pub const METHOD_NOTIFICATIONS_TOOLS_CHANGED: &str = "notifications/tools/list_changed";
pub const METHOD_NOTIFICATIONS_RESOURCES_CHANGED: &str = "notifications/resources/list_changed";
pub const METHOD_NOTIFICATIONS_PROMPTS_CHANGED: &str = "notifications/prompts/list_changed";

/// Returns true if a method name is in the safe-list of side-effect-free
/// operations. Used by the replay confirmation gate.
///
/// Safe by name pattern: `*/list`, `*/get`, `*/read`, and the literal `ping`.
pub fn is_safe_for_replay(method: &str) -> bool {
    if method == METHOD_PING {
        return true;
    }
    if let Some((_, tail)) = method.rsplit_once('/') {
        matches!(tail, "list" | "get" | "read")
    } else {
        false
    }
}

/// True if the message is `tools/call` — these always require explicit
/// confirmation before replay regardless of the tool name.
pub fn is_tools_call(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Request { method, .. } if method == METHOD_TOOLS_CALL)
}

/// Returns the tool name from a `tools/call` request, if present.
pub fn tool_name(msg: &JsonRpcMessage) -> Option<&str> {
    if let JsonRpcMessage::Request { method, params, .. } = msg {
        if method == METHOD_TOOLS_CALL {
            return params
                .as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::jsonrpc::JsonRpcId;
    use serde_json::json;

    #[test]
    fn safe_list_pattern() {
        assert!(is_safe_for_replay("tools/list"));
        assert!(is_safe_for_replay("resources/list"));
        assert!(is_safe_for_replay("resources/read"));
        assert!(is_safe_for_replay("prompts/get"));
        assert!(is_safe_for_replay("ping"));

        assert!(!is_safe_for_replay("tools/call"));
        assert!(!is_safe_for_replay("logging/setLevel"));
        assert!(!is_safe_for_replay("initialize"));
    }

    #[test]
    fn tool_name_extraction() {
        let msg = JsonRpcMessage::Request {
            jsonrpc: "2.0".into(),
            id: JsonRpcId::Num(1),
            method: METHOD_TOOLS_CALL.into(),
            params: Some(json!({"name": "read_file", "arguments": {"path": "/tmp/x"}})),
        };
        assert_eq!(tool_name(&msg), Some("read_file"));
        assert!(is_tools_call(&msg));
    }

    #[test]
    fn tool_name_missing() {
        let msg = JsonRpcMessage::Request {
            jsonrpc: "2.0".into(),
            id: JsonRpcId::Num(1),
            method: METHOD_TOOLS_LIST.into(),
            params: None,
        };
        assert_eq!(tool_name(&msg), None);
        assert!(!is_tools_call(&msg));
    }
}
