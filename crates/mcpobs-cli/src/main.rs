use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod commands;
mod config;

use commands::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let cli = Cli::parse();
    match cli.command {
        Command::Init(c) => commands::init::run(c).await,
        Command::Start(c) => commands::start::run(c).await,
        Command::Add(c) => commands::add::run(c).await,
        Command::Remove(c) => commands::remove::run(c).await,
        Command::List(c) => commands::list::run(c).await,
        Command::Proxy(c) => commands::proxy::run(c).await,
        Command::Tail(c) => commands::tail::run(c).await,
        Command::Export(c) => commands::export::run(c).await,
        Command::Prune(c) => commands::prune::run(c).await,
    }
}
