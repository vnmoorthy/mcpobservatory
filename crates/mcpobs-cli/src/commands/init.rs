use crate::config;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Overwrite an existing config file.
    #[arg(long)]
    pub force: bool,
}

pub async fn run(args: Args) -> Result<()> {
    let cfg_path = config::config_path();
    if cfg_path.exists() && !args.force {
        println!("config already exists at {}", cfg_path.display());
        println!("(pass --force to overwrite)");
        return Ok(());
    }

    let cfg = config::Config::default();
    let path = config::save(&cfg).await?;
    let dir = config::ensure_data_dir(&cfg).await?;

    println!("wrote default config: {}", path.display());
    println!("data dir: {}", dir.display());
    println!();
    println!("next: `mcpobs start` (in another terminal) and `mcpobs add <name> --command ...`.");
    Ok(())
}
