# HN Show post draft

**Title:** Show HN: mcpobs – local-first proxy and trace viewer for MCP

**Body:**

Hi HN — I built `mcpobs` because I got tired of debugging MCP servers with `print` statements.

It's a single binary that sits between your MCP client (Claude Desktop, Cursor, Cline, Continue, Windsurf, ...) and any number of MCP servers. It captures every JSON-RPC frame, stores it in a local SQLite database, and serves a web UI on `localhost:7890` with a timeline, request inspector, replay, and diff.

What it gets you that nothing else does:

- One pane of glass across every MCP server you have configured, not one-server-at-a-time like the official Inspector.
- Historical traces — find what broke yesterday.
- Replay a request against the live upstream and diff the new response.
- Transparent: no bytes are modified on the wire. If your upstream is broken, you see the broken bytes.

Local-first by design. No telemetry. No signup. No cloud. The README will tell you what's in the SaaS version (alerts, team sharing) but that's a separate codebase later — the OSS will never phone home.

Quickstart is one curl + four commands. Took ~60 seconds on my machine.

```
curl -sSL https://raw.githubusercontent.com/vnmoorthy/mcpobservatory/main/scripts/install.sh | sh
mcpobs init
mcpobs start &
mcpobs add filesystem --command npx --args '@modelcontextprotocol/server-filesystem,/'
```

Repo: https://github.com/vnmoorthy/mcpobservatory

Happy to answer anything about the architecture (Rust + axum + sqlite + embedded React) or the tradeoffs.
