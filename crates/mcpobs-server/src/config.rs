use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// `127.0.0.1:7890` by default. Override with `--listen`.
    pub listen: SocketAddr,
    /// Origin header values that POST endpoints will accept.
    pub allowed_origins: Vec<String>,
    /// Retention in days for `prune`.
    pub retention_days: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:7890".parse().unwrap(),
            allowed_origins: vec![
                "http://127.0.0.1:7890".into(),
                "http://localhost:7890".into(),
            ],
            retention_days: 7,
        }
    }
}
