# Architecture

`mcpobs` is four Rust crates plus a React UI, shipped as one binary.

```
crates/mcpobs-core    protocol parsing, transports, proxy loop
crates/mcpobs-store   sqlite, redaction, writer task
crates/mcpobs-server  axum + rust-embed + ws
crates/mcpobs-cli     clap, config, daemon entrypoint
ui/                   vite + react + tailwind, embedded into the binary
```

## Two process shapes

1. **`mcpobs start`** — long-lived daemon. Owns the SQLite writer, hosts the web UI, mounts HTTP/SSE upstream listeners.
2. **`mcpobs proxy --upstream <name>`** — short-lived stdio bridge. The MCP client launches it as a subprocess. It parses each JSON-RPC frame, forwards it verbatim, and sends an observation up to the daemon.

If the daemon is down, the proxy still proxies. Observations are dropped with a single `tracing::warn!`.

## Data flow

```
client ──► proxy ──► upstream
              │
              └──► observation channel ──► writer ──► sqlite
                                                       ▲
                                          api routes ──┘
                                          ws fan-out
```

Backpressure: every channel is bounded. The proxy never blocks on observation writes; if the channel is full, the observation is dropped.

## Concurrency

Per stdio session there are three tokio tasks:

1. **client→upstream pump** — read line, parse, emit, write
2. **upstream→client pump** — same in the other direction
3. **stderr passthrough** — bytes only, no observation

When either pump exits, the other is given a brief grace period to drain, then the upstream is shut down (SIGTERM, then SIGKILL after 5s).

## Schema

See `crates/mcpobs-store/src/migrations/0001_initial.sql`. Key choices:

- `messages.payload_json TEXT` so SQLite's JSON1 functions work for ad-hoc queries.
- Indexes on `(session_id)`, `(timestamp_ms)`, `(method)` — covers every UI query.
- `correlated_message_id` is filled lazily by the writer when a response's `rpc_id` matches an open request.
- WAL mode + `synchronous=NORMAL` for concurrent reads.

## Threat model

See [`SECURITY.md`](../SECURITY.md). The headline mitigation is default-on key-name redaction (T9): a developer might `git commit ~/.mcpobs/traces.db` to a bug-report repo and leak a token. Redaction is aggressive by default.
