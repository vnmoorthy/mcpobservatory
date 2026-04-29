import type { MessageRow } from "./api";

/// Build an approximate cURL command that re-issues a captured message
/// against the daemon's HTTP forwarder. Useful for sharing repros.
export function asCurl(m: MessageRow): string {
  const url = `http://127.0.0.1:7890/mcp/${m.server_name}`;
  const body = m.payload_json.replace(/'/g, "'\\''");
  return `curl -sS -X POST '${url}' \\\n  -H 'Content-Type: application/json' \\\n  -d '${body}'`;
}
