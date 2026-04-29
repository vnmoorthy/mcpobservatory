# Security Policy

## Reporting a vulnerability

Please **do not** file a public GitHub issue for a security problem. Open a [private security advisory](https://github.com/vnmoorthy/mcpobservatory/security/advisories/new) on GitHub with:

- A description of the issue and the version of `mcpobs` it affects.
- A minimal reproduction (commands, config, or a small fixture).
- Your name and how you'd like to be credited (or "anonymous").

We will acknowledge receipt within 48 hours and aim to ship a fix within 14 days for high-severity issues.

## Scope

`mcpobs` is a local-first developer tool. The threat model is:

1. **Trusted operator.** The user runs the tool on their own machine, with their own configs.
2. **Untrusted upstream MCP servers.** The proxy must not crash, OOM, or be exploited by a malicious payload from an upstream.
3. **Untrusted MCP payload contents.** Payload bodies (tool-call args, response content) are arbitrary JSON.
4. **Trusted localhost.** The local web UI on `127.0.0.1:7890` is treated as same-origin to the user's browser.

In scope:
- Crashes / DoS via crafted MCP payloads.
- Path traversal in `data_dir` or related config.
- SQL injection.
- XSS in the local UI.
- CSRF on the local API.
- Unintentional exposure of `:7890` to the network.
- Secret-leak patterns in stored payloads (we redact aggressively by default).
- Symlink / file-permission attacks on `~/.mcpobs/`.
- Supply-chain CVEs in dependencies (we run `cargo audit` and `pnpm audit` in CI).

Out of scope:
- Anything requiring physical access to the user's machine.
- Side channels in localhost timing.
- Issues in upstream MCP servers themselves (file those upstream).
- "User mis-configures `--listen 0.0.0.0` and exposes the UI to the internet" — we make this hard but ultimately respect the user's choice.

## Security features in place

- All deserialisation of untrusted bytes is fallible; parse failures are logged and forwarded as-is. The proxy never `unwrap`s on untrusted input.
- All SQL uses parameter binding via `sqlx`. CI greps for `format!.*SELECT` etc. as a guardrail.
- Default bind is `127.0.0.1:7890`. Binding to a non-loopback address requires a flag and prints a warning.
- The local API checks the `Origin` header on POST endpoints to prevent CSRF.
- Payloads are rendered in the React UI as text content, never as HTML; no `dangerouslySetInnerHTML` is used anywhere.
- `~/.mcpobs/` is created with `mode 0700`. We refuse to start if its ownership doesn't match the running user.
- Default redaction strips values for keys `password`, `token`, `secret`, `api_key`, `Authorization`, and any key matching the regex `(?i)(password|token|secret|key|cred)`.
- Cargo workspace pins versions; `cargo audit` is a required CI check.

## Updates

When a security issue is fixed, we publish a release with `[security]` in the changelog and a CVE if applicable.
