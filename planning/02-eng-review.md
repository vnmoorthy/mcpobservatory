# 02 — Engineering Review

> Equivalent to gstack `/plan-eng-review`. Crate boundaries, contracts, threat model, perf budget. Read this before writing code.

## Architecture diagram (logical)

```
                       ┌─────────────────────────────┐
                       │  MCP client (Claude / Cursor│
                       │  / Cline / Continue / etc.)  │
                       └──────────────┬──────────────┘
                                      │ stdio | http | sse
                                      ▼
        ┌──────────────────────────────────────────────────────┐
        │  mcpobs-cli  →  spawns  mcpobs proxy --upstream X    │
        │                                                       │
        │  ┌─────────────────────────────────────────────────┐ │
        │  │  mcpobs-core  (no I/O dep on store / server)    │ │
        │  │  ├─ protocol::jsonrpc                            │ │
        │  │  ├─ protocol::mcp                                │ │
        │  │  ├─ transport::{stdio, http, sse}                │ │
        │  │  ├─ session                                       │ │
        │  │  └─ proxy   (emits StoreEvent on bounded chan)   │ │
        │  └────────┬────────────────────────────────────────┘ │
        │           │ tokio::mpsc<StoreEvent>                  │
        │           ▼                                           │
        │  ┌─────────────────────────────────────────────────┐ │
        │  │  mcpobs-store    sqlx + SQLite WAL              │ │
        │  │  └─ writer task drains the channel               │ │
        │  └─────────────────────────────────────────────────┘ │
        └──────────────────────────────────────────────────────┘
                                      │
                                      ▼
                          ~/.mcpobs/traces.db  (SQLite, WAL)
                                      ▲
                                      │ reads
                       ┌──────────────┴──────────────┐
                       │  mcpobs-server   (axum)     │
                       │  ├─ REST /api/...           │
                       │  ├─ WebSocket /ws/live      │
                       │  └─ embedded UI (rust-embed)│
                       └──────────────┬──────────────┘
                                      │ http://localhost:7890
                                      ▼
                       ┌─────────────────────────────┐
                       │   browser (developer)       │
                       └─────────────────────────────┘
```

## Crate layout and rationale

| Crate | Depends on | Why a separate crate |
|---|---|---|
| `mcpobs-core` | `tokio`, `serde`, `tracing` only | Pure protocol/proxy. No SQL, no HTTP server. Lets us reuse it later in the SaaS layer without dragging the OSS UI along. |
| `mcpobs-store` | `core`, `sqlx`, `chrono` | Persistence boundary. Swappable later (Postgres in SaaS). |
| `mcpobs-server` | `core`, `store`, `axum`, `rust-embed` | The local web server. The UI bundle is embedded here. |
| `mcpobs-cli` | all of the above | The single binary. Composes the start/proxy/init commands. |

The four-crate split is *not* over-engineering: `mcpobs-core` is genuinely reusable, and `mcpobs-store` will get a sibling `mcpobs-store-pg` in the SaaS layer.

## Process model

There are exactly two `mcpobs` process shapes a user runs:

1. **`mcpobs start`** — long-lived daemon. Hosts the HTTP/SSE upstream listeners (when configured), serves the web UI on `:7890`, owns the SQLite writer.
2. **`mcpobs proxy --upstream <name>`** — short-lived stdio bridge. The MCP client launches this as a subprocess. It connects to the daemon's writer over a Unix socket (or named pipe on Windows) at `~/.mcpobs/writer.sock` to forward observed messages.

The proxy process is intentionally "dumb": parse, forward, emit observation, exit when client closes stdin. All persistence is the daemon's job. This means the daemon must be running for traces to land — `mcpobs proxy` will *still proxy correctly* if the daemon is down (it MUST be transparent), but it will buffer up to 4MB of observations and drop the oldest if the writer socket is unreachable. We log a single `tracing::warn!` and continue.

This is the most subtle decision in the whole architecture. The alternative — every proxy invocation writes directly to SQLite — has two problems:
- SQLite WAL handles concurrent readers + one writer. N MCP servers = N proxy processes = N writers, each contending for the WAL lock.
- Every `proxy` invocation pays sqlx connect-time on every short-lived spawn (in stdio mode, the proxy lives for the whole client session, so this isn't acute, but still).

The Unix-socket bridge solves both.

## Transport contracts

### stdio

```rust
pub struct StdioProxy {
    upstream_cmd: String,
    upstream_args: Vec<String>,
    upstream_env: HashMap<String, String>,
    sink: ObservationSink,
}
```

The proxy:
1. Reads JSON-RPC lines from its own stdin (newline-delimited).
2. Spawns the upstream as a child with piped stdin/stdout/stderr.
3. Forwards every input line to upstream stdin; emits observation `{direction: c2s, raw, parsed}`.
4. Forwards every upstream stdout line to its own stdout; emits observation `{direction: s2c, raw, parsed}`.
5. Forwards upstream stderr lines to its own stderr unchanged (so the client sees them).
6. On client EOF: send SIGTERM to upstream, wait 5s, SIGKILL.
7. On upstream exit: log exit status, send EOF on own stdout, exit 0.

Backpressure: bounded `mpsc::channel(1024)` for both directions. If the upstream is slow, we **block the client's write** rather than buffer unboundedly. This matches the upstream's own backpressure behaviour — transparent.

### Streamable HTTP

The daemon listens on the configured `[upstreams.<name>] listen_path`. Default `/mcp/<name>`. Accepts POST with `Content-Type: application/json`. Body is a JSON-RPC request. We forward to the upstream URL via `reqwest::Client::new().post(url).json(&body)`. The response body is streamed back as we receive it; the client gets the same bytes. Observation is emitted at request and response boundaries with the full payload.

Note: per MCP spec, the upstream may return either a JSON response or an SSE stream. We probe the first byte of the response body — if it's `event:` we treat as SSE; otherwise JSON.

### SSE

The proxy maintains one long-lived `GET <upstream>/sse` and re-emits each event to the client. The client's POST endpoint is bridged to the upstream's POST endpoint via the proxy's reverse-proxy logic. Each SSE event is observed.

## Storage schema rationale

See `crates/mcpobs-store/src/migrations/0001_initial.sql` for the actual SQL. Notes:

- **`messages.payload_json` is `TEXT`, not `BLOB`.** SQLite's JSON1 functions work on TEXT, and we get `sqlite3 ~/.mcpobs/traces.db "SELECT json_extract(payload_json, '$.params.name') FROM messages WHERE method='tools/call' LIMIT 5;"` for free during debugging.
- **`payload_size_bytes` is denormalised** so we can sum total bytes per session without touching the payload.
- **`correlated_message_id`** is filled lazily when a response arrives whose `rpc_id` matches an open request in the session. This is the join key for the trace tree.
- **Indexes**: `(session_id)`, `(timestamp_ms)`, `(method)` cover every query the UI makes. Verified at CHECKPOINT 4 by inserting 10k synthetic rows.
- **WAL mode** turned on at connect time via `PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;`. This buys us concurrent reads while the writer holds a write transaction.
- **Retention**: `mcpobs prune --older-than 7` deletes by `timestamp_ms`. Default 7 days. Configurable in `[server] retention_days`.

## Concurrency model

```
client stdin  ─┐                                         ┌─ stdout to client
               ├─► reader task ──► obs_chan ──┐          │
                                               │          │
upstream stdin ◄┤                              ▼          ├─ upstream stdout
                ▼                          observer task   ▼
              upstream child process       (drains chan,   │
                                            sends to       │
                                            writer sock)   │
```

Three tokio tasks per stdio session:

1. **client→upstream pump**: read line, emit observation, write to upstream stdin.
2. **upstream→client pump**: read line, emit observation, write to client stdout, fill `correlated_message_id` if response.
3. **observer drain**: take from the bounded `mpsc::Receiver<Observation>`, send to writer socket as length-prefixed JSON, retry once on transient failure, drop with warning otherwise.

All three are joined on shutdown. Panics in any task are caught at the join boundary and logged; the proxy exits with code 2 to signal a bug.

## Failure modes and how we handle them

| Failure | What we do |
|---|---|
| Upstream subprocess crashes mid-session | Log, send EOF to client, exit cleanly. Client sees the same behaviour as if it had launched the upstream directly. |
| Upstream returns malformed JSON | Forward bytes verbatim. Emit observation with `parsed: null` and `parse_error: "..."`. **We do not synthesise a JSON-RPC error**. |
| Writer daemon down | Buffer up to 4MB of observations, drop oldest with `tracing::warn!`. Proxy continues forwarding correctly. |
| SQLite disk full | Writer logs and exits; daemon restart loop will retry. Proxy keeps working (observations dropped). |
| HTTP upstream returns 5xx | Forward as-is. |
| HTTP upstream times out | We do **not** time out faster than the client would. Default `reqwest` timeout is disabled; we let the client's own JSON-RPC layer handle timeouts. |
| Replay request targets a deleted message | API returns 404. UI shows "trace pruned." |
| Two clients connect to the same upstream simultaneously | Each gets its own `mcpobs proxy` invocation, its own session id, its own subprocess of the upstream. They do not share state. The upstream sees two independent connections (which is what would happen without the proxy). |

## Threat model (security review v1)

This is a *local-first developer tool*. The threat model is correspondingly modest, but not empty. We enumerate threats and mitigations:

| # | Threat | Mitigation |
|---|---|---|
| T1 | A malicious MCP server sends a payload that crashes the proxy. | All deserialisation is fallible; parse errors are logged and forwarded as raw bytes. No `unwrap` on untrusted input. `cargo deny` blocks crates with known panics on bad input. |
| T2 | A malicious MCP server sends a multi-megabyte payload to OOM the proxy. | Per-line cap of 16MB at the line reader; oversize lines are split-logged and forwarded as-is to the client (client's parser will fail safely). Buffered channel size 1024 × 16MB = 16GB worst case theoretical, but `tokio` mpsc `send` is `await`-ed so backpressure throttles before allocation. |
| T3 | SQL injection via a maliciously-crafted MCP method name or session id. | All queries use sqlx's parameter binding. No string interpolation into SQL anywhere. `grep -rn 'format!.*SELECT'` in CI as a guardrail. |
| T4 | Path traversal via a config-supplied `data_dir` or `command`. | `data_dir` is canonicalised on load; we reject paths containing `..` after canonicalisation. `command` is *intentionally* arbitrary — the user is launching a process they chose; the proxy is not the trust boundary here. We document this in `SECURITY.md`. |
| T5 | Replay of an old request hits an upstream that has changed state, causing damage. | Replay is **opt-in per click** in the UI. The replay panel shows a confirmation when the message is a `tools/call` with a non-readonly tool. We mark a few common readonly tools (`*/list`, `*/get`, `*/read`) as safe by name pattern; everything else needs confirmation. |
| T6 | Web UI XSS via raw JSON content from a malicious server. | All payloads are rendered via `JSON.stringify` + `<pre>` text content in React (never `dangerouslySetInnerHTML`). |
| T7 | CSRF on the local API server. | Same-origin policy + `127.0.0.1`-only bind by default + `Origin` header check on `POST /api/replay`. We reject any request whose `Origin` is not `http://localhost:7890` / `http://127.0.0.1:7890`. |
| T8 | Local API server bound to `0.0.0.0` exposes traces to LAN. | Default bind is `127.0.0.1:7890`. `--listen 0.0.0.0` requires the user to pass `--accept-network-exposure-risk` and prints a banner. Documented. |
| T9 | Sensitive secrets (tokens, API keys) in MCP payloads land in `~/.mcpobs/traces.db` in plaintext. | Documented prominently. Add a `[server] redact_keys = ["password", "token", "secret", "api_key", "Authorization"]` config knob (default on). The redactor walks parsed JSON; raw bytes are kept only if `redact_raw = false` (default `true` strips the raw form when a parsed form exists). |
| T10 | Dependency CVE in `tokio`/`axum`/`sqlx` etc. | `cargo audit` runs on every CI build; we fail the build on any unfixed CVE. Renovate-bot keeps deps fresh. |
| T11 | Symlink attack on `~/.mcpobs/`. | Create `~/.mcpobs/` with `mode 0700` if absent. Verify ownership on startup. |
| T12 | Denial-of-service: client floods proxy with notifications. | Bounded channels. Backpressure flows back to client. We do not add per-second rate limits because that would change semantics. |
| T13 | Timing side channels in replay. | Not relevant — local tool, no auth. |
| T14 | Privilege escalation via setuid. | Binary is shipped without setuid. Documented. |
| T15 | Supply-chain attack on UI dependencies (npm). | Lockfile committed (`pnpm-lock.yaml`); `pnpm audit --prod` runs in CI; dependencies pinned to exact versions. |

T9 (secret redaction) is the highest-stakes mitigation because the most realistic harm is "I committed `~/.mcpobs/traces.db` to a bug-report repo and leaked my GitHub token." We make redaction default-on and aggressive.

## Performance budget

- **Proxy added latency p99 ≤ 5ms** on a localhost stdio message. Measured by `cargo bench` with a fixture upstream.
- **Web UI initial paint ≤ 300ms** on a session with 1000 messages.
- **Live tail websocket lag ≤ 50ms** from observation to UI.
- **SQLite writer throughput ≥ 10k messages/sec** on commodity SSD.
- **Memory steady state ≤ 100MB RSS** for the daemon with 7 days of traces in the DB.

These are claims the README will make; CI runs the bench and fails if any regress by >20%.

## Spec revision target

We pin against MCP spec revision **2025-06-18** (latest stable as of this build). Newer revisions are tracked in `docs/spec-revisions.md` with a deliberate-upgrade decision per revision.

## Out-of-scope confirmations

These came up in design discussions and were rejected for v0:

- **No per-tool-call latency budget alerts.** That's anomaly detection; v0.3.
- **No OpenTelemetry export.** Roadmap; the schema is OTel-friendly but we don't ship the exporter.
- **No multi-upstream "fan-in" trace correlation.** Each upstream is its own session/trace tree. A future feature in the SaaS layer.
- **No write-back capability** (i.e., the UI cannot edit a stored trace). Read-only by design.

## Decision

Architecture approved. Proceed to design review.
