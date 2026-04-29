use clap::{Parser, Subcommand};

pub mod add;
pub mod export;
pub mod init;
pub mod list;
pub mod proxy;
pub mod prune;
pub mod remove;
pub mod start;
pub mod tail;

#[derive(Parser)]
#[command(
    name = "mcpobs",
    about = "Local-first MCP proxy and trace viewer",
    long_about = "Local-first MCP proxy and trace viewer.\n\
                  \n\
                  Captures every JSON-RPC message your MCP client (Claude Desktop, Cursor,\n\
                  Cline, ...) exchanges with its servers and renders it in a local web UI\n\
                  at http://127.0.0.1:7890. No telemetry, no signup, no cloud.",
    version,
    after_help = "EXAMPLES:\n\
                  \n  \
                  Quickstart (60s):\n\
                      $ mcpobs init\n\
                      $ mcpobs start &\n\
                      $ mcpobs add filesystem --command npx \\\n\
                            --args '@modelcontextprotocol/server-filesystem,/Users/me/Documents'\n\
                      $ open http://localhost:7890\n\
                  \n  \
                  Tail live traffic from your terminal:\n\
                      $ mcpobs tail --since 30m\n\
                  \n  \
                  Export the last hour as JSONL:\n\
                      $ mcpobs export --since-seconds 3600 > traces.jsonl\n\
                  \n  \
                  Compact the database (default 7-day retention):\n\
                      $ mcpobs prune --older-than 7\n\
                  \n\
                  Detailed walkthrough: docs/quickstart.md\n\
                  Source: https://github.com/vnmoorthy/mcpobservatory"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialise the data directory and write a default config.
    Init(init::Args),
    /// Run the daemon (HTTP/SSE listeners + web UI on 127.0.0.1:7890).
    Start(start::Args),
    /// Add an upstream MCP server to the config.
    Add(add::Args),
    /// Remove an upstream from the config.
    Remove(remove::Args),
    /// List configured upstreams.
    List(list::Args),
    /// Run as a stdio proxy bridge (invoked by your MCP client).
    Proxy(proxy::Args),
    /// Tail the live message log.
    Tail(tail::Args),
    /// Export traces as JSONL.
    Export(export::Args),
    /// Delete traces older than N days.
    Prune(prune::Args),
}
