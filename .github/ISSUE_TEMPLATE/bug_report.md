---
name: Bug report
about: Something is broken in mcpobs
title: ""
labels: bug
assignees: ""
---

## What happened

A clear, single-paragraph description.

## Reproduction

1.
2.
3.

## Expected vs actual

- Expected: ...
- Actual: ...

## Environment

- `mcpobs --version`:
- OS + arch (e.g. macOS 14.5 arm64):
- MCP client (Claude Desktop / Cursor / Cline / other):
- Upstream MCP server (name + version):
- Transport (stdio / http / sse):

## Logs

Attach the output of `mcpobs tail --since 5m` from the moment of the bug, or paste a relevant excerpt below.

```text
```

## Trace export (optional but very helpful)

If the bug is in a specific session, run `mcpobs export --session <id> > trace.jsonl` and attach `trace.jsonl`.

> Reminder: trace exports may contain payloads from the affected MCP server. Redact anything you do not want public before attaching.
