# Contributing to MCP Observatory

Thanks for your interest in helping out. This is a small project with a tight scope; the bar for changes is high but the path is short.

## Ground rules

1. **Transparency above all.** The proxy must never modify, drop, or synthesize MCP messages. Patches that break this property will not be merged.
2. **Single binary, zero dependencies for end users.** No optional Python services, no Node sidecar processes at runtime, no Docker requirement.
3. **Local first, no telemetry.** No phone-home. No analytics. Not even opt-in. If you need data, run a local query against `~/.mcpobs/traces.db`.
4. **Spec-revision discipline.** MCP is a moving target. We pin the supported spec revision in the README and only bump it deliberately.

## Development setup

You need:

- Rust **1.82+** (we test on stable; nightly is not required)
- Node 20+ and pnpm (only if you touch the `ui/` folder)
- SQLite 3.35+ (the bundled `sqlx` driver brings its own, but the CLI is handy)
- A real MCP server to test against — the official `npx @modelcontextprotocol/server-filesystem /tmp` is the easiest

Clone, then:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

For the UI:

```bash
cd ui
pnpm install
pnpm dev    # iterating on the UI
pnpm build  # produces ui/dist that gets embedded into the Rust binary
```

The Rust build re-embeds `ui/dist` automatically at release-build time via `rust-embed`.

## Pull request expectations

- One logical change per PR. Refactors and feature work go in separate PRs.
- New behaviour gets a test. Storage queries get a test. Protocol parsing gets a fixture. Transports get an integration test against a real or mocked upstream.
- `cargo clippy --all-targets -- -D warnings` must be clean. We deliberately keep clippy strict.
- Update `CHANGELOG.md` under `## Unreleased`.
- If you're adding a config field, document it in `docs/config-reference.md`.

## Code style

- `rustfmt` defaults. No bikeshedding.
- Errors use `thiserror` at crate boundaries and `anyhow` inside binaries.
- Logging uses `tracing` with structured fields. No `println!` in library code.
- Public APIs are documented. Private items are not (but a one-liner is appreciated for non-obvious code).

## Reporting security issues

Please do not file public GitHub issues for security problems. Open a [private security advisory](https://github.com/vnmoorthy/mcpobservatory/security/advisories/new) on GitHub. See `SECURITY.md` for the full policy.

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
