# Config reference

Default location: `~/.mcpobs/config.toml`. Override with `MCPOBS_CONFIG=/path/to/file`.

## `[server]`

```toml
[server]
listen          = "127.0.0.1:7890"
data_dir        = "~/.mcpobs"
retention_days  = 7
```

| key | default | description |
|---|---|---|
| `listen` | `127.0.0.1:7890` | address the daemon binds. Loopback only by default. |
| `data_dir` | `~/.mcpobs` | where `traces.db` lives. |
| `retention_days` | `7` | input to `mcpobs prune` defaults. |

Binding to a non-loopback address requires `--accept-network-exposure-risk` on the command line.

## `[redaction]`

```toml
[redaction]
keys        = ["password", "token", "secret", "api_key", "apikey", "authorization"]
placeholder = "[redacted]"
```

Case-insensitive substring match against object keys at any depth.

## `[upstreams.<name>]`

stdio:
```toml
[upstreams.filesystem]
transport = "stdio"
command   = "npx"
args      = ["@modelcontextprotocol/server-filesystem", "/Users/me/Documents"]
env       = { LOG_LEVEL = "debug" }
```

http:
```toml
[upstreams.github-api]
transport   = "http"
url         = "http://localhost:9000/mcp"
listen_path = "/mcp/github-api"
headers     = { "X-Auth" = "..." }
```

sse:
```toml
[upstreams.events]
transport = "sse"
url       = "http://localhost:9001/sse"
headers   = {}
```
