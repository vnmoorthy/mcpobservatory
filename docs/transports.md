# Transports

`mcpobs` proxies the three transports defined by MCP spec revision **2025-06-18**.

## stdio

The most common transport for desktop clients. The MCP client spawns `mcpobs proxy --upstream <name>` as a subprocess. The proxy then spawns the configured upstream and bridges stdin/stdout/stderr.

Lifecycle:
- Client EOF → proxy closes upstream stdin → upstream exits → proxy exits.
- Upstream crash → proxy reports exit code on its own stderr → client sees the same behaviour as if it had launched the upstream directly.

Backpressure flows naturally — the proxy never buffers more than one line per direction.

## Streamable HTTP

For self-hosted upstreams that speak HTTP. Configure with:

```toml
[upstreams.github-api]
transport = "http"
url = "http://localhost:9000/mcp"
listen_path = "/mcp/github-api"
```

The daemon listens on `listen_path` and forwards POSTed JSON-RPC requests to `url`. The response body is streamed back as bytes.

If the upstream returns `Content-Type: text/event-stream`, we treat the response as SSE and forward each event as it arrives.

## SSE

A long-lived `GET <url>/sse` connection from the daemon to the upstream. Each event is forwarded to subscribers and recorded as an observation.

## Choosing a transport

|           | stdio | http  | sse   |
|---        |---    |---    |---    |
| Use when  | client launches subprocess | upstream is a self-hosted service | upstream pushes events |
| Latency   | lowest | depends on network | streamed |
| Auth      | env vars | request headers | request headers |
| Backpressure | natural | per-request | applies to GET stream |
