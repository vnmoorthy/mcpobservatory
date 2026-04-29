# 03 — Design Review

> Equivalent to gstack `/plan-design-review`. Visual + UX system before any pixels.

## Design intent

"A well-made developer tool, not a VC pitch deck." Concretely:

- Monospace for everything that is data (JSON, identifiers, methods, latencies).
- Sans for everything that is chrome (nav, headings, button labels).
- One accent colour. No gradients. No drop shadows. No emoji in chrome.
- Dark mode by default. A light mode exists and is accessible via a single toggle in the bottom-left, persisted to `localStorage`.
- High information density. Plenty of whitespace where the data is sparse, not where it isn't.
- Reading the screen for five seconds should answer "is anything broken?"

The reference aesthetic is `htop` × Linear × Sentry's old issues view, not Vercel dashboards.

## Information architecture

```
/                           Dashboard (default landing)
/servers                    Same as / (alias for clarity in URL bar)
/servers/:name              Server detail
/sessions/:id               Session timeline
/traces/:id                 Trace view (subset of session)
/diff?a=:id&b=:id           Diff view
/replay?of=:id              Replay panel (modal-ish; replaces session right pane)
/settings                   Server config + retention + redaction (read-only in v0)
```

Every URL is bookmarkable and shareable (within the user's machine). No client-side state is required to render any view.

## Layout

Three-column layout on desktop ≥1280px, two-column on 768–1280, single column below.

```
┌─────────────────────────────────────────────────────────────┐
│  ━━ mcpobs                                       [⌘K] [🌓]  │  ← top bar 48px
├──────────────┬──────────────────────────────────────────────┤
│              │                                                │
│  Servers     │   Main content (Dashboard / Session / etc.)   │
│  ────────    │                                                │
│  filesystem  │                                                │
│  github      │                                                │
│  postgres    │                                                │
│              │                                                │
│  Recent      │                                                │
│  ────────    │                                                │
│  17:42 fs    │                                                │
│  17:39 gh    │                                                │
│              │                                                │
└──────────────┴──────────────────────────────────────────────┘
   left rail 240px           main flex-1
```

Left rail collapses below 1024px. The `[⌘K]` button opens a command palette with fuzzy-find over server name, session id, and method name.

## Pages

### Dashboard `/`

Above the fold, four numeric tiles in a single row:

```
┌─────────────────┬─────────────────┬─────────────────┬─────────────────┐
│ Sessions today  │ Errors today    │ Avg latency     │ Total messages  │
│       42        │        3        │     12 ms       │     8,201       │
└─────────────────┴─────────────────┴─────────────────┴─────────────────┘
```

Below: a table of configured servers with one row each:

```
| name        | transport | status      | sessions today | err | p50  | p99  | sparkline (60min)
| filesystem  | stdio     | ● connected |          17    |  0  |  4ms | 18ms | ▁▂▃▂▁▁▁▂▃
| github      | stdio     | ● idle      |           5    |  1  | 122ms| 480ms| ▁▁▁▁▁▂▃▁▁
| postgres    | http      | ○ stale     |           0    |  0  |   —  |   —  | ────────
```

Click a row → server detail.

### Server detail `/servers/:name`

Header: name, transport, command/url, status pill.

A list of recent sessions (default 50), reverse-chrono. Each row:

```
| started        | duration | msgs | errors | p99  | first method
| 17:42:13 today | 4m 12s   |  84  |   0    | 18ms | initialize
| 16:08:01 today | 1m 03s   |  22  |   2    | 480ms| initialize
```

Click a row → session view.

### Session view `/sessions/:id`

The headline view. Vertical timeline:

```
┌─ Session 7e3c… filesystem · started 17:42:13 · 84 msgs · 4m 12s ────────────┐
│                                                                              │
│  17:42:13.041  ▶ c2s  request   id=1     initialize                          │
│  17:42:13.044  ◀ s2c  response  id=1     [OK 3ms]   {server v2026-06-18…}   │
│  17:42:13.045  ▶ c2s  notification       initialized                         │
│  17:42:13.047  ▶ c2s  request   id=2     tools/list                          │
│  17:42:13.051  ◀ s2c  response  id=2     [OK 4ms]   12 tools                 │
│  17:43:01.128  ▶ c2s  request   id=3     tools/call (read_file)              │
│  17:43:01.129  ◀ s2c  notification       progress 0%                         │
│  17:43:01.184  ◀ s2c  notification       progress 100%                       │
│  17:43:01.185  ◀ s2c  response  id=3     [OK 57ms]  {content: [{type:text…}]}│
│  17:45:22.001  ▶ c2s  request   id=4     tools/call (write_file)             │
│  17:45:22.014  ◀ s2c  error     id=4     [ERR 13ms] EACCES /etc/hosts        │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

Visual encoding:
- ▶ green for client→server, ◀ blue for server→client
- request = bold, response = normal weight, notification = grey, error = red
- latency badge `[OK 4ms]` / `[ERR 13ms]` on responses
- click a row → expand inline, full pretty-printed JSON below it
- right-click a row → context menu: Replay, Diff against…, Copy as cURL, Copy raw

Top-right of the timeline:
- "Filter": method dropdown, direction toggle, error-only toggle
- "Export": full session as JSONL
- "Live tail": toggle (default on if session is still active)

When live tail is on, new messages append at the bottom and the page auto-scrolls only if the user is scrolled within 100px of the bottom. This is the "tail -f" behaviour every developer expects.

### Trace view `/traces/:id`

Same as session view but scoped to a single root request and its descendants (notifications, response, any `notifications/cancelled`). Useful for sharing a single failed tool call.

The URL pattern lets us share `…/traces/abc123` in a bug report; the recipient sees just the relevant slice.

### Diff view `/diff?a=…&b=…`

Side-by-side two-pane JSON diff. We use `similar` crate for the actual diff algorithm; the UI renders it as line-level highlight with red/green inline tokens. Top of page: identifiers of both messages with timestamps and a "swap" button.

Use cases:
- Compare two responses to the same tool call (one passing, one failing).
- Compare two `tools/list` responses across server-version upgrades.

### Replay panel `/replay?of=…`

When a request is replayed:
1. UI POSTs to `/api/replay` with the original message id.
2. The daemon sends the same JSON-RPC payload to the upstream (using a fresh `id` to avoid colliding with the live session's id space).
3. The new response is captured as a normal observation **but tagged** `replay_of=<id>` in the `metadata` JSON column.
4. UI shows original response on left, replay response on right, with the diff inlined.

A confirmation gate fires if the original method is not in the safe-list (`*/list`, `*/get`, `*/read`, `ping`).

## Visual tokens

Defined once in `ui/src/lib/tokens.css`. The whole UI references these and only these.

```css
:root {
  /* base hues */
  --bg-0:     #0b0d10;   /* page background, dark */
  --bg-1:     #11151b;   /* card / row background */
  --bg-2:     #1a2028;   /* hover / selected row */
  --border:   #232a33;
  --fg-0:     #e6edf3;   /* primary text */
  --fg-1:     #9aa6b2;   /* secondary text */
  --fg-2:     #6b7682;   /* muted (timestamps, ids) */

  /* accents */
  --accent:        #4cc9a4;   /* mint — the one accent */
  --accent-soft:   #1f3a33;
  --client-direction: #6cd084; /* slightly different from accent so they don't clash */
  --server-direction: #6db6e8;
  --error:         #e06868;
  --warning:       #d4a04a;

  /* type */
  --font-sans: ui-sans-serif, system-ui, -apple-system, "SF Pro Text", sans-serif;
  --font-mono: ui-monospace, "SF Mono", "JetBrains Mono", Menlo, Consolas, monospace;
  --fs-xs: 11px;
  --fs-sm: 12px;
  --fs-md: 13px;
  --fs-lg: 15px;
  --fs-xl: 20px;

  /* spacing */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-6: 24px;
  --space-8: 32px;
}

[data-theme="light"] {
  --bg-0:  #f7f8fa;
  --bg-1:  #ffffff;
  --bg-2:  #eef2f6;
  --border:#dfe5ec;
  --fg-0:  #0d1117;
  --fg-1:  #475467;
  --fg-2:  #98a2b3;
}
```

That's the entire palette. No additional colours allowed without an RFC.

## States

For every page:

- **Empty state**: "No sessions yet. Run `mcpobs add <name> …`, then connect your client. We'll be here." with a copyable command.
- **Loading state**: skeleton bars matching the row height, no spinners. Spinners are a confession of latency.
- **Error state**: a single red banner at the top with the error text and a Retry button. Never a full-page error.

## Accessibility

- All interactive elements reachable by `Tab` in DOM order.
- Color contrast ≥ AA (4.5:1) for body text against background. Verified with axe.
- Focus ring is a 2px `--accent` outline with 1px offset; visible in both themes.
- Keyboard shortcuts: `?` opens cheatsheet, `j/k` next/prev row in any list, `/` focuses search, `e` exports, `r` replays.
- `prefers-reduced-motion` disables the live-tail auto-scroll animation.

## What we're explicitly NOT doing

- No charts beyond the per-server sparkline (Chart.js inflates the bundle; we draw the sparkline as a 60-point inline SVG).
- No avatars, no user mentions, no mascot.
- No animations longer than 150ms.
- No icon library beyond lucide-react.
- No drag-and-drop.
- No tooltips that obscure data; we use inline secondary text instead.

## Handoff to engineering

Tokens are in `ui/src/lib/tokens.css`. The five page components are in `ui/src/pages/`. The shared timeline row component is `ui/src/components/MessageRow.tsx`. The empty-state component is `ui/src/components/EmptyState.tsx`. Engineering should not invent components; if a need arises that isn't covered, raise it as a design RFC before implementing.

## Decision

Design approved. Proceed to autoplan.
