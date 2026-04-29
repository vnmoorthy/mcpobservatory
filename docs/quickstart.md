# Quickstart

You should be looking at MCP traffic in your browser within 60 seconds.

## 1. Install

```bash
curl -sSL https://raw.githubusercontent.com/vnmoorthy/mcpobservatory/main/scripts/install.sh | sh
```

If you'd rather build from source:

```bash
git clone https://github.com/vnmoorthy/mcpobservatory
cd mcp-observatory
pnpm --dir ui install && pnpm --dir ui run build
cargo install --path crates/mcpobs-cli
```

## 2. Initialise

```bash
mcpobs init
```

Writes `~/.mcpobs/config.toml` and creates `~/.mcpobs/`. Re-run with `--force` to overwrite.

## 3. Start the daemon

```bash
mcpobs start
```

Listens on `127.0.0.1:7890`. Open it in your browser. You'll see an empty dashboard — that's expected.

## 4. Add an upstream

```bash
mcpobs add filesystem \
  --command npx \
  --args '@modelcontextprotocol/server-filesystem,/Users/me/Documents'
```

`mcpobs` prints a JSON snippet for your client config. Paste it into:
- Claude Desktop: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Cursor: `~/.cursor/mcp.json`
- Cline: `~/.config/cline/mcp.json`

The snippet replaces your existing entry for that server. The proxy is transparent.

## 5. Use your client

Restart your client and trigger a tool call. In the Observatory UI you should see the session appear and messages stream in.

## Troubleshooting

- **Daemon not running**: the proxy still works in passthrough mode but no traces are recorded. `mcpobs start` and try again.
- **Port already in use**: `mcpobs start --listen 127.0.0.1:7891`. Update bookmarks accordingly.
- **No messages appear**: confirm your client config is using `mcpobs proxy --upstream <name>` as the command.
