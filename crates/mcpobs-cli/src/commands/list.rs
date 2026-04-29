use crate::config;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {}

pub async fn run(_args: Args) -> Result<()> {
    let cfg = config::load().await?;
    if cfg.upstreams.is_empty() {
        println!("no upstreams configured. try `mcpobs add <name> --command <cmd>`.");
        return Ok(());
    }
    let mut names: Vec<_> = cfg.upstreams.keys().cloned().collect();
    names.sort();
    for name in names {
        let upstream = &cfg.upstreams[&name];
        println!("{:<24} {}", name, upstream.transport_str());
    }
    Ok(())
}
