use crate::config::{self, Upstream};
use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Transport {
    Stdio,
    Http,
    Sse,
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Upstream name (e.g. `filesystem`).
    pub name: String,

    /// Transport.
    #[arg(long, value_enum, default_value = "stdio")]
    pub transport: Transport,

    /// Command to spawn (stdio).
    #[arg(long)]
    pub command: Option<String>,

    /// Comma-separated args for the command (stdio). Hyphenated values
    /// like `-c,echo hi` are allowed without escaping.
    #[arg(long, allow_hyphen_values = true)]
    pub args: Option<String>,

    /// URL (http/sse).
    #[arg(long)]
    pub url: Option<String>,

    /// Repeatable env or header `KEY=VALUE` pairs.
    #[arg(short = 'e', long = "env")]
    pub env: Vec<String>,
}

pub async fn run(args: Args) -> Result<()> {
    let mut cfg = config::load().await?;
    if cfg.upstreams.contains_key(&args.name) {
        bail!(
            "upstream `{}` already exists; remove it first or use a different name",
            args.name
        );
    }

    let entry = match args.transport {
        Transport::Stdio => {
            let command = args
                .command
                .clone()
                .ok_or_else(|| anyhow::anyhow!("--command required for stdio transport"))?;
            let arg_vec = args
                .args
                .as_deref()
                .map(|s| {
                    s.split(',')
                        .map(|p| p.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let env = parse_kv(&args.env)?;
            Upstream::Stdio {
                command,
                args: arg_vec,
                env,
                cwd: None,
            }
        }
        Transport::Http => {
            let url = args
                .url
                .clone()
                .ok_or_else(|| anyhow::anyhow!("--url required for http"))?;
            Upstream::Http {
                url,
                headers: parse_kv(&args.env)?,
                listen_path: Some(format!("/mcp/{}", args.name)),
            }
        }
        Transport::Sse => {
            let url = args
                .url
                .clone()
                .ok_or_else(|| anyhow::anyhow!("--url required for sse"))?;
            Upstream::Sse {
                url,
                headers: parse_kv(&args.env)?,
            }
        }
    };

    cfg.upstreams.insert(args.name.clone(), entry);
    let path = config::save(&cfg).await?;

    println!("added `{}`. config: {}", args.name, path.display());
    println!();
    println!("paste into your client config (Claude Desktop / Cursor):");
    println!();
    print_client_snippet(&args.name);
    Ok(())
}

fn parse_kv(items: &[String]) -> Result<std::collections::HashMap<String, String>> {
    let mut map = std::collections::HashMap::new();
    for item in items {
        let (k, v) = item
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected KEY=VALUE in `{item}`"))?;
        map.insert(k.to_string(), v.to_string());
    }
    Ok(map)
}

fn print_client_snippet(name: &str) {
    println!("{{");
    println!("  \"mcpServers\": {{");
    println!("    \"{name}\": {{");
    println!("      \"command\": \"mcpobs\",");
    println!("      \"args\": [\"proxy\", \"--upstream\", \"{name}\"]");
    println!("    }}");
    println!("  }}");
    println!("}}");
}
