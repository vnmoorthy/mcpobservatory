use crate::config::{self, Upstream};
use anyhow::{Context, Result};
use clap::Parser;
use mcpobs_core::observation::ObservationSink;
use mcpobs_core::transport::http::HttpUpstream;
use mcpobs_core::transport::sse::SseUpstream;
use mcpobs_server::{AppState, ReplayCoordinator, ServerConfig};
use mcpobs_store::RedactionConfig;
use std::sync::Arc;

#[derive(Parser, Debug)]
pub struct Args {
    /// Override the listen address (default `127.0.0.1:7890`).
    #[arg(long)]
    pub listen: Option<String>,

    /// Required confirmation when binding to a non-loopback address.
    #[arg(long)]
    pub accept_network_exposure_risk: bool,
}

pub async fn run(args: Args) -> Result<()> {
    let cfg = config::load().await?;
    let db_path = config::db_path(&cfg).await?;
    let store = mcpobs_store::open(&db_path).await?;

    let listen_str = args.listen.clone().unwrap_or(cfg.server.listen.clone());
    let addr: std::net::SocketAddr = listen_str
        .parse()
        .with_context(|| format!("parse listen address `{listen_str}`"))?;

    if !addr.ip().is_loopback() && !args.accept_network_exposure_risk {
        anyhow::bail!(
            "refusing to bind to non-loopback address {addr} without --accept-network-exposure-risk;\n\
             see SECURITY.md for what this exposes."
        );
    }
    if !addr.ip().is_loopback() {
        eprintln!(
            "WARNING: mcpobs is bound to {addr} (non-loopback). Anyone on your network can read traces."
        );
    }

    let live = mcpobs_server::make_live_bus(1024);
    let mut origins: Vec<String> = vec![
        format!("http://{addr}"),
        "http://127.0.0.1:7890".into(),
        "http://localhost:7890".into(),
    ];
    origins.sort();
    origins.dedup();
    let server_cfg = ServerConfig {
        listen: addr,
        allowed_origins: origins,
        retention_days: cfg.server.retention_days,
    };

    // Seed the servers table so the dashboard lists configured upstreams
    // before any traffic arrives.
    for (name, up) in &cfg.upstreams {
        let _ = mcpobs_store::queries::upsert_server(&store, name, up.transport_str(), "{}").await;
    }

    // Spawn the persistence writer with the live bus as a sink.
    let (sink, rx) = ObservationSink::new(8192);
    let redaction = RedactionConfig {
        keys: cfg.redaction.keys.clone(),
        placeholder: cfg.redaction.placeholder.clone(),
    };
    let _writer = mcpobs_store::spawn_writer_with_sink(store.clone(), rx, redaction, live.clone());

    // Mount HTTP forwarders for any HTTP upstreams; spawn SSE subscribers.
    let mut replay_coord = ReplayCoordinator::new();
    let mut http_routes: Vec<(String, String)> = Vec::new();
    for (name, up) in &cfg.upstreams {
        match up {
            Upstream::Http {
                url,
                headers,
                listen_path,
            } => {
                let path = listen_path
                    .clone()
                    .unwrap_or_else(|| format!("/mcp/{name}"));
                let forwarder = mcpobs_core::proxy::build_http_forwarder(
                    name.clone(),
                    HttpUpstream {
                        url: url.clone(),
                        headers: headers.clone(),
                    },
                    sink.clone(),
                )?;
                replay_coord.register_http(name.clone(), forwarder);
                http_routes.push((path, name.clone()));
            }
            Upstream::Sse { url, headers } => {
                let sub = mcpobs_core::proxy::build_sse_subscriber(
                    name.clone(),
                    SseUpstream {
                        url: url.clone(),
                        headers: headers.clone(),
                    },
                    sink.clone(),
                )?;
                tokio::spawn(async move {
                    if let Err(e) = sub.run().await {
                        tracing::warn!(error = %e, "sse subscriber exited");
                    }
                });
            }
            Upstream::Stdio { .. } => {
                // stdio is handled by `mcpobs proxy`, not the daemon.
            }
        }
    }

    let state = AppState {
        store: store.clone(),
        live: live.clone(),
        config: Arc::new(server_cfg),
        replay: Arc::new(replay_coord),
    };

    println!("mcpobs daemon listening on http://{addr}");
    println!("traces.db: {}", db_path.display());
    if !http_routes.is_empty() {
        println!("http upstreams mounted:");
        for (path, name) in &http_routes {
            println!("  POST  http://{addr}{path}  →  {name}");
        }
    }
    println!("ctrl-c to stop");

    mcpobs_server::serve(state, addr, http_routes).await?;
    Ok(())
}
