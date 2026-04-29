//! `~/.mcpobs/config.toml` loader.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerSection,
    #[serde(default)]
    pub redaction: RedactionSection,
    #[serde(default)]
    pub upstreams: HashMap<String, Upstream>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

impl Default for ServerSection {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            data_dir: default_data_dir(),
            retention_days: default_retention_days(),
        }
    }
}

fn default_listen() -> String {
    "127.0.0.1:7890".into()
}
fn default_data_dir() -> String {
    "~/.mcpobs".into()
}
fn default_retention_days() -> u32 {
    7
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionSection {
    #[serde(default = "default_redact_keys")]
    pub keys: Vec<String>,
    #[serde(default = "default_placeholder")]
    pub placeholder: String,
}

impl Default for RedactionSection {
    fn default() -> Self {
        Self {
            keys: default_redact_keys(),
            placeholder: default_placeholder(),
        }
    }
}

fn default_redact_keys() -> Vec<String> {
    vec![
        "password".into(),
        "token".into(),
        "secret".into(),
        "api_key".into(),
        "apikey".into(),
        "authorization".into(),
    ]
}
fn default_placeholder() -> String {
    "[redacted]".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "lowercase")]
pub enum Upstream {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
        #[serde(default)]
        cwd: Option<String>,
    },
    Http {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
        #[serde(default)]
        listen_path: Option<String>,
    },
    Sse {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

impl Upstream {
    pub fn transport_str(&self) -> &'static str {
        match self {
            Self::Stdio { .. } => "stdio",
            Self::Http { .. } => "http",
            Self::Sse { .. } => "sse",
        }
    }
}

pub fn config_path() -> PathBuf {
    let raw = std::env::var("MCPOBS_CONFIG").unwrap_or_else(|_| "~/.mcpobs/config.toml".into());
    expand(&raw)
}

pub fn data_dir(cfg: &Config) -> PathBuf {
    expand(&cfg.server.data_dir)
}

fn expand(p: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(p).into_owned())
}

pub async fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let s = tokio::fs::read_to_string(&path)
        .await
        .with_context(|| format!("read {}", path.display()))?;
    let cfg: Config = toml::from_str(&s).with_context(|| format!("parse {}", path.display()))?;
    Ok(cfg)
}

pub async fn save(cfg: &Config) -> Result<PathBuf> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let s = toml::to_string_pretty(cfg).context("serialize config")?;
    tokio::fs::write(&path, s)
        .await
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

pub async fn ensure_data_dir(cfg: &Config) -> Result<PathBuf> {
    let dir = data_dir(cfg);
    tokio::fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("create {}", dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = tokio::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).await;
    }
    Ok(dir)
}

pub async fn db_path(cfg: &Config) -> Result<PathBuf> {
    let dir = ensure_data_dir(cfg).await?;
    Ok(dir.join("traces.db"))
}
