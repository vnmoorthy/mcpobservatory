# mcpobs

Local-first proxy and trace viewer for the [Model Context Protocol](https://modelcontextprotocol.io). See every JSON-RPC message your AI agents send.

Drop it between your MCP client (Claude Desktop, Cursor, Cline, Continue, Windsurf) and your servers. Every message lands in a real-time timeline you can replay and diff.

## Install

```bash
npm install -g @vnmoorthy/mcpobs
```

The CLI command is `mcpobs` (the npm name is scoped because the unscoped `mcpobs` collides with an unrelated package).

This downloads the prebuilt binary for your platform from the matching [GitHub release](https://github.com/vnmoorthy/mcpobservatory/releases) and verifies its sha256.

Supported platforms: macOS arm64/x64, Linux arm64/x64, Windows x64.

## Quickstart

```bash
mcpobs init                                                   # ~/.mcpobs/config.toml
mcpobs start &                                                # daemon on :7890
mcpobs add filesystem --command npx \
  --args '@modelcontextprotocol/server-filesystem,/Users/me'  # add an upstream
open http://localhost:7890                                    # browser UI
```

## Links

- Source: https://github.com/vnmoorthy/mcpobservatory
- Issues: https://github.com/vnmoorthy/mcpobservatory/issues
- License: Apache-2.0

## Troubleshooting

**`mcpobs: binary not found`** — install completed but the postinstall step was skipped (e.g. `--ignore-scripts`). Re-install: `npm install -g @vnmoorthy/mcpobs`.

**`mcpobs: unsupported platform`** — only the platforms listed above ship prebuilts. File an issue with your `uname -a`.

**Behind a firewall** — set `MCPOBS_REPO=your/mirror` in the environment before install to point at a release mirror.

**Skip download** — set `MCPOBS_SKIP_DOWNLOAD=1` in CI to skip the binary download (use the binary you provided yourself in `node_modules/mcpobs/bin/`).
