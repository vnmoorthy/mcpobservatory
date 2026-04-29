use crate::config;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Limit to a specific session id.
    #[arg(long)]
    pub session: Option<String>,

    /// Limit to messages newer than this many seconds (e.g. `3600`).
    #[arg(long)]
    pub since_seconds: Option<i64>,
}

pub async fn run(args: Args) -> Result<()> {
    let cfg = config::load().await?;
    let db_path = config::db_path(&cfg).await?;
    let store = mcpobs_store::open(&db_path).await?;

    let rows = if let Some(sid) = args.session.as_deref() {
        mcpobs_store::queries::list_session_messages(&store, sid, 100_000, None).await?
    } else {
        let since_ms = args
            .since_seconds
            .map(|s| chrono::Utc::now().timestamp_millis() - s * 1000);
        mcpobs_store::queries::search_messages(&store, None, since_ms, None, 100_000).await?
    };

    let mut stdout = tokio::io::stdout();
    use tokio::io::AsyncWriteExt;
    for r in rows {
        let line = serde_json::to_string(&r)?;
        stdout.write_all(line.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
    }
    stdout.flush().await?;
    Ok(())
}
