# Changelog

All notable changes to `mcpobs` are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-29

The first public release. Local-first MCP proxy and trace viewer.

### Added

- **Proxy** — transparent stdio, Streamable HTTP, and SSE transports.
- **CLI** — `init`, `start`, `add`, `remove`, `list`, `proxy`, `tail`, `export`, `prune`.
- **Storage** — SQLite WAL with default-on key-name redaction (`password`, `token`,
  `secret`, `api_key`, `apikey`, `Authorization`).
- **Server** — local HTTP API on `127.0.0.1:7890`, WebSocket live tail at `/ws/live`,
  embedded React UI.
- **UI** — Dashboard with health banner, stat tiles, 24h activity heatmap, server table
  with sparklines and p50/p99 latency, recent activity feed.
- **UI** — Session view with timing waterfall, filter bar (method/direction/errors-only),
  master-detail layout, JSON tree with syntax highlighting and copy-path, right-click
  context menu (Replay / Diff / Copy as cURL / Copy raw / Open trace), j/k keyboard
  navigation.
- **UI** — Server detail with tabs for Sessions, Tools, Resources, Prompts, Notifications.
  Tools tab auto-discovers from captured `tools/list` responses and renders an auto-generated
  invocation form.
- **UI** — Side-by-side Diff view with line numbers, Replay panel with `params` override,
  cross-session Search, Settings page.
- **UI** — Command palette (`⌘K` / `Ctrl+K`) over pages, servers, sessions. Shortcut
  cheatsheet (`?`).
- **UI** — Light/dark theme toggle (persisted to `localStorage`).
- **Replay** — daemon re-issues captured HTTP-upstream requests with safe-list
  confirmation gate (`*/list`, `*/get`, `*/read`, `ping`).
- **Security** — explicit Origin allowlist on `/api/replay` and `/ws/live`. Default
  loopback bind. `--accept-network-exposure-risk` required for non-loopback.
- **CI** — GitHub Actions for cross-OS build/test/clippy/fmt + cargo audit + pnpm audit
  + telemetry/SQL-injection guardrails. Cross-compiled release artifacts for macOS
  arm64/x86_64, Linux x86_64/arm64, Windows x86_64.

### MCP spec

- Pinned against MCP spec revision **2025-06-18**.

[Unreleased]: https://github.com/vnmoorthy/mcpobservatory/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/vnmoorthy/mcpobservatory/releases/tag/v0.1.0
