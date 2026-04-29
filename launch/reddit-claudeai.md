# r/ClaudeAI / r/cursor / r/LocalLLaMA draft

**Title:** I built a local-first inspector for MCP servers — every JSON-RPC message Claude Desktop or Cursor sends, in your browser, with replay and diff

**Body:**

If you've added more than one MCP server to Claude Desktop or Cursor, you've probably hit one of these:

- A tool call returned something weird and you don't know which server caused it.
- A server worked yesterday and doesn't today.
- A server crashed and you don't have a stack trace.

`mcpobs` is what I built to fix that for myself. It's a transparent proxy between your client and your MCP servers. It logs every JSON-RPC message, shows you a real-time timeline, lets you replay any request, and diffs two responses side-by-side.

It's local-first — no telemetry, no signup, no cloud. The whole thing is one binary. Quickstart is roughly 60 seconds.

Repo, screenshots, install instructions: https://github.com/vnmoorthy/mcpobservatory

Apache 2.0. Issues and PRs welcome.
