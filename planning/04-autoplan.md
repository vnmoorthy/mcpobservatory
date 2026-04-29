# 04 — Autoplan

> Equivalent to gstack `/autoplan`. Synthesises CEO + Eng + Design into an ordered build plan. Surfaces only the decisions that need taste.

## Build order (with dependency arrows)

```
[A] Workspace + tooling  ─►  [B] mcpobs-store ────────────►┐
                                                            │
                              [C] mcpobs-core/protocol ────►┤
                                                            │
                              [D] mcpobs-core/transport/std ┤─►  [F] mcpobs-server
                                                            │           │
                              [E] mcpobs-core/transport/http┤           │
                                                            │           ▼
                                                            └─►   [G] UI (ui/)
                                                                        │
                                                                        ▼
                                                                  [H] mcpobs-cli
                                                                        │
                                                                        ▼
                                                            [I] Docs / examples / launch
                                                                        │
                                                                        ▼
                                                            [J] CI / cross-compile / verify
```

A blocks everything. B and C are independent. D needs C. E needs C. F needs B + C. G needs F (compiles HTTP types from the OpenAPI surface). H needs everything. I needs H. J needs the lot.

## Estimated effort

| # | Unit | Files | Est lines | Risk |
|---|---|---|---|---|
| A | Workspace, .github, license, README placeholder | 12 | 600 | low |
| B | Storage + migrations + queries + tests | 6 | 800 | low |
| C | Protocol (jsonrpc, mcp types, fixtures, tests) | 8 | 900 | medium (spec drift) |
| D | stdio transport + integration test | 4 | 700 | high (subprocess lifecycle) |
| E | HTTP + SSE transport + mock server | 5 | 700 | medium |
| F | Server (api/, ws.rs, embed) | 9 | 1100 | medium |
| G | React UI (5 pages + components + hooks) | 22 | 1800 | medium (state mgmt) |
| H | CLI (8 commands + config + bridge to daemon) | 12 | 1100 | medium (UX edge cases) |
| I | README, 4 doc pages, 3 launch drafts, examples | 11 | 1500 | low |
| J | CI workflows, release.yml, cargo audit gate | 4 | 400 | medium |
| **Total** | | **93** | **~9.6k** | |

## Verification gates

The original spec has 7 CHECKPOINTs; we map them to gstack's `/qa` runs:

- After A → CHECKPOINT 0: workspace compiles (`cargo build --workspace`).
- After C → CHECKPOINT 1: protocol parsing tests pass.
- After D → CHECKPOINT 2: real MCP server runs through proxy, traces land in DB. **Requires human at keyboard with Claude Desktop or `npx @modelcontextprotocol/inspector`.**
- After E → CHECKPOINT 3: mock HTTP server round-trip works.
- After B (re-verify) → CHECKPOINT 4: 10k synthetic messages, queries < 100ms.
- After G → CHECKPOINT 5: every UI page renders with real data. **Requires human.**
- After H → CHECKPOINT 6: clean-machine quickstart < 5 minutes. **Requires human.**
- After I → CHECKPOINT 7: 3 friends do the quickstart. **Requires human.**

Checkpoints that say "requires human" are marked in QA as deferred to user.

## Dependency pins

We pin the major versions of every external dep in the workspace `Cargo.toml` and the UI `package.json`. Specifically:

- `tokio = "1.40"` (LTS line)
- `axum = "0.7"` (current stable)
- `sqlx = "0.8"` (current stable)
- `reqwest = "0.12"` with `rustls-tls` (no OpenSSL)
- `clap = "4.5"`
- `serde_json` with `preserve_order` (we want JSON keys in a predictable order in the UI)
- `react = "18"` (we don't chase 19 yet)
- `vite = "5"`
- `tailwindcss = "3.4"`
- `@tanstack/react-query = "5"`

We commit `Cargo.lock` (it's a binary crate) and `pnpm-lock.yaml`.

## Surface-only-taste decisions

Per gstack convention, here are the things that need a human eye. Everything else I'm just doing.

1. **Naming**: tool is `mcpobs` (binary), product is "MCP Observatory". The directory `~/.mcpobs/` is fixed. Domain `mcpobservatory.dev` is the suggested launch URL. Approve or override.
2. **Default port 7890**: arbitrary but memorable. No conflict with common dev ports. Approve or override.
3. **Default retention 7 days**: long enough for "what broke yesterday", short enough to keep the DB small. Approve or override.
4. **Replay safe-list pattern**: `*/list`, `*/get`, `*/read`, `ping`. Anything else needs an explicit confirmation modal. Approve or expand.
5. **Redaction default-on**: `password`, `token`, `secret`, `api_key`, `Authorization`. Approve or expand. (My recommendation: keep aggressive default; the alternative — leaking a token by default — is a much worse failure mode.)
6. **No telemetry, ever, in v0**: this is a marketing commitment. Approve.

If I don't hear back, I'm proceeding with these defaults.

## Out-of-scope guardrail

Per CEO doc, these are banned for v0. The reviewer (and any contributor) should reject PRs that add:

- Auth, multi-user, team sharing
- Cloud sync, S3 export, network egress beyond proxied
- Email/Slack alerts
- ML / anomaly detection
- Non-MCP protocols
- Homebrew / apt / snap / Chocolatey
- Docker / K8s
- Theming beyond "default + dark"
- Plugin system / scripting hooks
- Telemetry of any kind

## Decision

Plan approved. Implementation begins.
