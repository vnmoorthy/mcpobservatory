# 01 — CEO Review

> Equivalent to gstack `/office-hours` + `/plan-ceo-review`. Strategy and scope before any code.

## One-line product

`mcpobs` — a single-binary local proxy that sits between an MCP client (Claude Desktop, Cursor, Cline, Continue, Windsurf) and any number of MCP servers, captures every JSON-RPC message, and renders it in a local web UI with timeline, replay, and diff.

## Who is this for

The 50–500k developers running 5–15 MCP servers daily. Concretely:

1. **Cursor / Claude Desktop power users** who hit a misbehaving MCP tool and currently have no debug surface beyond `print` statements in the upstream server.
2. **MCP server authors** shipping packages on npm and getting bug reports they cannot reproduce because they have no trace of the failing client interaction.
3. **Internal dev-tools teams at AI-native startups** wiring up custom MCP servers against Claude or other LLM clients and needing to instrument them in dev.

Excluded from launch audience: enterprise observability buyers (that's the future SaaS layer), end-user non-developers, anyone running fewer than 2 MCP servers.

## Why now

- MCP usage went from "experimental" to "default integration surface for AI clients" in 2025–early 2026. Every coding agent shipped MCP support.
- Tools count per developer grew from ~2 to ~10 between mid-2025 and Q1 2026.
- The pain point ("which server returned malformed output and broke my agent loop") is universal among power users and has no incumbent solution.
- Competitive set is thin: official MCP Inspector (one-server-at-a-time, no history), Langfuse / Helicone / Phoenix (LLM-call-centric, no MCP transport awareness), nothing else.

## What "good" looks like at 30 days

Three concrete metrics. If we hit two, we're on track.

| Metric | Target | Why it matters |
|---|---|---|
| GitHub stars | 1,500 | Validates the narrative resonated with the launch audience. |
| Weekly active installs (anonymous, opt-in counter — *deferred to v0.2*) | n/a in v0 | We deliberately ship no telemetry in v0 to make a marketing point. |
| README quickstart completion rate (proxy: HN comments saying "this just worked") | ≥10 such comments | If install-to-trace takes longer than 60s, the launch fails. |

Stars are a vanity metric, but they're the right vanity metric for an open-source dev tool because they correlate with discoverability via GitHub trending.

## Kill criteria

We do **not** ship if:

1. Quickstart on a clean machine takes >5 minutes from `curl | sh` to "first MCP message visible in the UI."
2. Proxy adds >10ms p99 latency on a localhost stdio message (doubles round-trip time on a hot path).
3. Any of: Claude Desktop, Cursor, Cline integration is broken on launch day.
4. The README cannot honestly claim "no telemetry, no signup, no cloud" — i.e., we accidentally ship any phone-home.

Each kill criterion has a remediation step in the QA doc.

## What's in scope for v0

- stdio, Streamable HTTP, SSE transports.
- SQLite persistence.
- Timeline / Server-detail / Session / Trace / Diff / Replay views.
- CLI: `init`, `start`, `proxy`, `add`, `remove`, `list`, `tail`, `export`, `prune`.
- Cross-compiled binaries: macOS arm64, macOS x86_64, Linux x86_64, Linux arm64, Windows x86_64.
- Apache 2.0 license.

## What's out of scope for v0

Hard bans (say no, even if it seems easy):

- Auth, multi-user, team sharing.
- Cloud sync, S3 export, anything that touches the network egress beyond the proxied connection.
- Email/Slack alerts.
- ML / anomaly detection.
- Non-MCP protocols.
- Homebrew / apt / snap / Chocolatey packages (v0.2).
- Docker / K8s manifests.
- UI theming beyond "clean default + dark mode."
- Plugin system, custom interceptors, scripting hooks.
- Authentication of upstream servers beyond passing through `env`.

## SaaS hint (v1+)

The README will mention "Hosted version with team sharing, alerts, and production tracing — [join waitlist]." The waitlist landing page captures intent. The SaaS architecture is *deliberately not* designed in v0 to avoid coupling decisions in the OSS code. The hosted version will likely be a separate codebase that re-uses `mcpobs-core`'s parsing layer.

## Launch plan

- **Day -7**: Send to 3 developer friends, time their quickstarts, fix friction.
- **Day -3**: Final dogfooding pass against Claude Desktop, Cursor, Cline.
- **Day 0**: Tuesday or Wednesday at 6am Pacific.
  - HN Show post (drafted in `launch/hn.md`).
  - Reddit r/ClaudeAI, r/cursor, r/LocalLLaMA (drafted in `launch/reddit-*.md`).
  - X thread, 6 posts, opens with the GIF (drafted in `launch/x.md`).
  - Discord posts in Anthropic and Cursor servers (drafted in `launch/discord.md`).
  - Landing page live with waitlist form.
- **Day +1 to +7**: Reply to every comment, ship patches for any incompatibility report within 24h.

## Taste decisions surfaced for approval

These are the calls I made that a co-founder might reasonably overrule. Flagged here per gstack convention.

1. **License: Apache 2.0** (vs MIT). Apache 2.0 has explicit patent grant; MIT does not. For an MCP tool that may eventually be acquired, Apache is stronger. AGPL is rejected outright (chills enterprise adoption).
2. **Rust over Go.** Go would ship faster but the proxy hot path benefits from Rust's lower overhead, and `tokio` + `axum` + `sqlx` is mature. The trade-off is contribution friction (smaller pool of Rust devs).
3. **Embedded React UI vs HTMX.** The spec gave us a choice. React was picked because the diff view, replay panel, and live timeline benefit from real client-side state. HTMX would be smaller binary but worse for the inspector experience. The UI bundle is ~250KB gzipped, embedded via `rust-embed`.
4. **SQLite over a custom log file.** SQLite gives us indexed queries, WAL concurrency, and `sqlite3` CLI access for free. The cost is a slightly heavier write path; benchmarked at <1ms per message which is well below the 5ms budget.
5. **Local-first + no telemetry as a *marketing* commitment, not just a *technical* one.** This is part of the brand. We will reject PRs that add telemetry, even opt-in, until v1.

## Decision

Build it. Ship in 8–14 hours of agent time spread across several human-clock days, per the spec.
