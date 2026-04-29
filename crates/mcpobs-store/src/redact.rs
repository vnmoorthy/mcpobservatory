//! Recursive JSON redaction for sensitive keys.
//!
//! The proxy stores everything by default, but the threat model
//! (`planning/02-eng-review.md` T9) is that a developer might send their
//! traces.db along with a bug report and accidentally publish a token. So
//! redaction is on by default and aggressive.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionConfig {
    /// Case-insensitive substring matches against object keys. If a key
    /// contains any of these substrings, the value is replaced with the
    /// placeholder.
    pub keys: Vec<String>,

    /// Placeholder string used in place of redacted values.
    #[serde(default = "default_placeholder")]
    pub placeholder: String,
}

fn default_placeholder() -> String {
    "[redacted]".into()
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            keys: vec![
                "password".into(),
                "token".into(),
                "secret".into(),
                "api_key".into(),
                "apikey".into(),
                "authorization".into(),
            ],
            placeholder: default_placeholder(),
        }
    }
}

impl RedactionConfig {
    pub fn matches(&self, key: &str) -> bool {
        let key_lc = key.to_ascii_lowercase();
        self.keys
            .iter()
            .any(|k| key_lc.contains(&k.to_ascii_lowercase()))
    }
}

/// Walk `value` and replace the value of any matching key with the
/// placeholder string. Operates in place.
pub fn redact_value(value: &mut Value, cfg: &RedactionConfig) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if cfg.matches(k) {
                    *v = Value::String(cfg.placeholder.clone());
                } else {
                    redact_value(v, cfg);
                }
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                redact_value(v, cfg);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redacts_top_level_token() {
        let cfg = RedactionConfig::default();
        let mut v = json!({"token": "abc123", "name": "foo"});
        redact_value(&mut v, &cfg);
        assert_eq!(v["token"], "[redacted]");
        assert_eq!(v["name"], "foo");
    }

    #[test]
    fn redacts_nested() {
        let cfg = RedactionConfig::default();
        let mut v = json!({
            "headers": {"Authorization": "Bearer xyz"},
            "data": [{"api_key": "k"}, {"safe": 1}]
        });
        redact_value(&mut v, &cfg);
        assert_eq!(v["headers"]["Authorization"], "[redacted]");
        assert_eq!(v["data"][0]["api_key"], "[redacted]");
        assert_eq!(v["data"][1]["safe"], 1);
    }

    #[test]
    fn case_insensitive_substring() {
        let cfg = RedactionConfig::default();
        let mut v = json!({
            "MY_API_KEY": "x",
            "user_password_hash": "y",
            "OTHER": "z"
        });
        redact_value(&mut v, &cfg);
        assert_eq!(v["MY_API_KEY"], "[redacted]");
        assert_eq!(v["user_password_hash"], "[redacted]");
        assert_eq!(v["OTHER"], "z");
    }

    #[test]
    fn passthrough_primitives() {
        let cfg = RedactionConfig::default();
        let mut v = json!(42);
        redact_value(&mut v, &cfg);
        assert_eq!(v, json!(42));
    }
}
