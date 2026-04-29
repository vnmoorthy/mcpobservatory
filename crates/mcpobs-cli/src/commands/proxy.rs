use crate::config::{self, Upstream};
use anyhow::{bail, Result};
use clap::Parser;
use mcpobs_core::observation::ObservationSink;
use mcpobs_core::transport::stdio::StdioUpstream;

#[derive(Parser, Debug)]
pub struct Args {
    /// Upstream name (must match an entry in config.toml).
    #[arg(long)]
    pub upstream: String,
}

pub async fn run(args: Args) -> Result<()> {
    let cfg = config::load().await?;
    let upstream_cfg = cfg
        .upstreams
        .get(&args.upstream)
        .ok_or_else(|| anyhow::anyhow!("no upstream named `{}` in config", args.upstream))?;

    match upstream_cfg.clone() {
        Upstream::Stdio {
            command,
            args: cmd_args,
            env,
            cwd,
        } => {
            let db_path = config::db_path(&cfg).await?;
            let store = mcpobs_store::open(&db_path).await?;
            let (sink, rx) = ObservationSink::new(1024);
            let redaction = mcpobs_store::RedactionConfig {
                keys: cfg.redaction.keys.clone(),
                placeholder: cfg.redaction.placeholder.clone(),
            };
            let _writer = mcpobs_store::spawn_writer(store, rx, redaction);

            let upstream = StdioUpstream {
                command,
                args: cmd_args,
                env,
                cwd,
            };
            let exit =
                mcpobs_core::proxy::run_stdio_proxy(args.upstream.clone(), upstream, sink).await?;

            tracing::info!(
                session = %exit.session_id,
                exit_code = ?exit.upstream_exit_code,
                "proxy exiting"
            );
        }
        Upstream::Http { .. } | Upstream::Sse { .. } => {
            bail!(
                "upstream `{}` is configured for HTTP/SSE; the daemon mounts those — use `mcpobs start`, not `mcpobs proxy`",
                args.upstream
            );
        }
    }

    Ok(())
}
