# Client integrations

`mcpobs` is a transparent proxy. Configure your client to launch `mcpobs proxy --upstream <name>` instead of the upstream directly. That's it.

## Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "mcpobs",
      "args": ["proxy", "--upstream", "filesystem"]
    }
  }
}
```

Quit and restart Claude Desktop.

## Cursor

Edit `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "mcpobs",
      "args": ["proxy", "--upstream", "filesystem"]
    }
  }
}
```

Reload window or restart Cursor.

## Cline (VS Code)

Cline reads `mcp.json` from its workspace settings. Use the same shape as above.

## Continue

Add an `mcp` entry in `~/.continue/config.json` pointing to the same `mcpobs proxy` command.

## Windsurf

Add to your Windsurf MCP config; the command/args fields are identical.

## Generic

Any client that launches an MCP server as a subprocess can route through `mcpobs`. The proxy reads JSON-RPC frames on stdin, forwards them, and writes responses to stdout. It never modifies bytes.
