use crate::config;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// How far back to print before tailing live (default 10m).
    #[arg(long, default_value = "10m")]
    pub since: String,

    /// Stop after printing the historical window (don't follow live).
    #[arg(long)]
    pub no_follow: bool,

    /// Filter to a specific upstream server.
    #[arg(long)]
    pub server: Option<String>,
}

pub async fn run(args: Args) -> Result<()> {
    let cfg = config::load().await?;
    let db_path = config::db_path(&cfg).await?;
    let store = mcpobs_store::open(&db_path).await?;

    let secs = parse_duration(&args.since)?;
    let since_ms = chrono::Utc::now().timestamp_millis() - (secs * 1000) as i64;
    let rows =
        mcpobs_store::queries::search_messages(&store, None, Some(since_ms), None, 1000).await?;
    let mut last_id: i64 = 0;
    for r in rows.iter().rev() {
        if let Some(server) = args.server.as_deref() {
            if r.server_name != server {
                continue;
            }
        }
        print_row(r);
        last_id = last_id.max(r.id);
    }

    if args.no_follow {
        return Ok(());
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let new_rows = mcpobs_store::queries::messages_since_id(&store, last_id, 200).await;
        let Ok(rows) = new_rows else { continue };
        for r in rows.iter() {
            if let Some(server) = args.server.as_deref() {
                if r.server_name != server {
                    continue;
                }
            }
            print_row(r);
            last_id = last_id.max(r.id);
        }
    }
}

fn print_row(r: &mcpobs_store::MessageRow) {
    println!(
        "{} {} {:>12} {:<32} {} {}",
        r.timestamp.format("%H:%M:%S%.3f"),
        match r.direction.as_str() {
            "c2s" => "▶",
            "s2c" => "◀",
            _ => "?",
        },
        r.kind,
        r.method.as_deref().unwrap_or("-"),
        r.server_name,
        &r.session_id[..r.session_id.len().min(8)],
    );
}

fn parse_duration(s: &str) -> Result<u64> {
    let s = s.trim();
    let split_at = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let (num, unit) = s.split_at(split_at);
    if num.is_empty() {
        anyhow::bail!(
            "invalid duration `{}` — expected NUMBER+unit, e.g. `30s`, `5m`, `2h`, `1d`",
            s
        );
    }
    let n: u64 = num.parse().map_err(|_| {
        anyhow::anyhow!(
            "invalid duration `{}` — number `{}` out of range; use a smaller value",
            s,
            num
        )
    })?;
    let mult = match unit {
        "" | "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        _ => anyhow::bail!(
            "invalid duration `{}` — unknown unit `{}`; use s, m, h, or d",
            s,
            unit
        ),
    };
    Ok(n * mult)
}

#[cfg(test)]
mod tests {
    use super::parse_duration;

    #[test]
    fn parses_units() {
        assert_eq!(parse_duration("30").unwrap(), 30);
        assert_eq!(parse_duration("30s").unwrap(), 30);
        assert_eq!(parse_duration("5m").unwrap(), 300);
        assert_eq!(parse_duration("2h").unwrap(), 7200);
        assert_eq!(parse_duration("1d").unwrap(), 86400);
    }

    #[test]
    fn rejects_garbage_with_helpful_error() {
        let e = parse_duration("bogus").unwrap_err().to_string();
        assert!(e.contains("expected NUMBER+unit"), "got: {e}");
    }

    #[test]
    fn rejects_unknown_unit() {
        let e = parse_duration("5w").unwrap_err().to_string();
        assert!(e.contains("unknown unit"), "got: {e}");
    }
}
