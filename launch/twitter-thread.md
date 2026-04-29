# Twitter / X thread draft (6 posts)

**1/** I shipped `mcpobs` today. It's a local-first proxy and trace viewer for MCP — sits between your client (Claude Desktop, Cursor, Cline) and your MCP servers and shows every JSON-RPC frame in your browser.

**2/** Why it exists: I had ~10 MCP servers configured. When one of them returned weird output, I had no way to know which one. The official Inspector is one-server-at-a-time. Print statements are not a debugger.

**3/** What it does:
- live timeline of every request and response
- per-session view with full payload inspection
- replay a request, diff against the original
- supports stdio + Streamable HTTP + SSE

**4/** What it deliberately doesn't do:
- no telemetry. ever. it's a marketing commitment.
- no signup, no cloud, no auth.
- no plugin system. one binary.

**5/** Stack: Rust + axum + sqlx + sqlite (WAL) + a React UI embedded with rust-embed. Single binary, ~250KB UI bundle, p99 added latency under 5ms on stdio.

**6/** Apache 2.0. Quickstart is `curl | sh` + four commands. Hosted version (team sharing, alerts) is a separate codebase later — the OSS will never phone home. Repo: https://github.com/vnmoorthy/mcpobservatory
