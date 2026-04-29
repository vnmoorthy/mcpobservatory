use crate::config;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    pub name: String,
}

pub async fn run(args: Args) -> Result<()> {
    let mut cfg = config::load().await?;
    if cfg.upstreams.remove(&args.name).is_none() {
        println!("no upstream named `{}`", args.name);
        return Ok(());
    }
    let path = config::save(&cfg).await?;
    println!("removed `{}`. config: {}", args.name, path.display());
    Ok(())
}
