## What

One paragraph on what this PR changes.

## Why

What problem this solves, or what bug it fixes. Link issues with `Fixes #123`.

## How

Brief notes on the approach. Anything subtle reviewers should look at first?

## Verification

- [ ] `cargo fmt --all` clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo test --workspace` passes
- [ ] `pnpm run build` succeeds (if `ui/` touched)
- [ ] Quickstart still works on a clean machine (if proxy or CLI touched)

## Out-of-scope check

This PR does **not** add (check that none apply):

- [ ] Telemetry, analytics, or phone-home of any kind
- [ ] Network egress beyond the proxied connection
- [ ] Auth, multi-user, or team-sharing surface
- [ ] Plugin / scripting hooks
- [ ] Non-MCP protocol support

If any are checked, this PR will be closed. See `planning/01-ceo-review.md`.

## Screenshots / GIFs

If UI changed, attach a before/after.
