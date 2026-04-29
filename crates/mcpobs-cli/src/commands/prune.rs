use crate::config;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Delete traces older than this many days.
    #[arg(long, default_value_t = 7)]
    pub older_than: u32,
}

pub async fn run(args: Args) -> Result<()> {
    let cfg = config::load().await?;
    let db_path = config::db_path(&cfg).await?;
    let store = mcpobs_store::open(&db_path).await?;

    let cutoff = chrono::Utc::now()
        .timestamp_millis()
        .saturating_sub(args.older_than as i64 * 86_400_000);
    let n = mcpobs_store::queries::prune_older_than(&store, cutoff).await?;
    println!(
        "pruned {n} message(s) older than {} day(s)",
        args.older_than
    );
    Ok(())
}
