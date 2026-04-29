# Spec revisions

| revision | status | notes |
|---|---|---|
| `2025-06-18` | **pinned** | Current build target. |

## Upgrade policy

We pin against one MCP spec revision at a time and ratchet forward deliberately. To bump:

1. Read the diff between revisions on `modelcontextprotocol.io`.
2. Decide if any of our typed wrappers in `crates/mcpobs-core/src/protocol/mcp.rs` need updates.
3. Add fixtures under `crates/mcpobs-core/tests/fixtures/`.
4. Run `cargo test --workspace`.
5. Update this file and the constant `MCP_SPEC_REVISION` in `crates/mcpobs-core/src/lib.rs`.

We deliberately don't try to be revision-agnostic in v0. The cost of "track every revision" is too high relative to the win.
