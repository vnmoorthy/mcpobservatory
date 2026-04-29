// Tiny fetch wrapper so we don't sprinkle URLs everywhere.

export type Direction = "c2s" | "s2c";

export interface ServerRow {
  name: string;
  transport: string;
  config_json: string;
  sessions_today: number;
  errors_today: number;
  p50_latency_ms: number | null;
  p99_latency_ms: number | null;
}

export interface SessionRow {
  id: string;
  server_name: string;
  transport: string;
  started_at: string;
  ended_at: string | null;
  message_count: number;
  error_count: number;
}

export interface MessageRow {
  id: number;
  session_id: string;
  server_name: string;
  direction: Direction;
  kind: string;
  method: string | null;
  rpc_id: string | null;
  timestamp: string;
  payload_size_bytes: number;
  payload_json: string;
  parse_error: string | null;
  metadata_json: string;
  correlated_message_id: number | null;
  latency_ms: number | null;
}

export interface DiffChange {
  tag: "delete" | "insert" | "equal";
  text: string;
}

export interface DiffResponse {
  a: MessageRow;
  b: MessageRow;
  changes: DiffChange[];
}

export interface SparklineResponse {
  buckets: number[];
  bucket_seconds: number;
}

export interface SettingsResponse {
  listen: string;
  allowed_origins: string[];
  retention_days: number;
  upstreams: { name: string; transport: string }[];
  mcp_spec_revision: string;
  version: string;
}

export interface TraceTreeNode {
  message: MessageRow;
  children: TraceTreeNode[];
}

export interface ReplayResult {
  status: string;
  original_id?: number;
  upstream?: string;
  method?: string;
  response_body?: unknown;
  reason?: string;
}

async function getJson<T>(url: string): Promise<T> {
  const r = await fetch(url, { credentials: "same-origin" });
  if (!r.ok) {
    // Try to extract the structured error message; fall back to status text.
    let msg = `${r.status} ${r.statusText}`;
    try {
      const body = await r.json();
      if (body && typeof body === "object" && "error" in body) {
        msg = String((body as { error: unknown }).error);
      }
    } catch {
      // not JSON
    }
    throw new Error(msg);
  }
  return (await r.json()) as T;
}

export const api = {
  servers: () => getJson<ServerRow[]>("/api/servers"),
  sparkline: (name: string) =>
    getJson<SparklineResponse>(`/api/servers/${encodeURIComponent(name)}/sparkline`),
  sessions: (server?: string, limit = 50) => {
    const params = new URLSearchParams({ limit: String(limit) });
    if (server) params.set("server", server);
    return getJson<SessionRow[]>(`/api/sessions?${params}`);
  },
  sessionMessages: (id: string, after?: number, limit = 200) => {
    const params = new URLSearchParams({ limit: String(limit) });
    if (after !== undefined) params.set("after", String(after));
    return getJson<MessageRow[]>(`/api/sessions/${encodeURIComponent(id)}/messages?${params}`);
  },
  message: (id: number) => getJson<MessageRow>(`/api/messages/${id}`),
  trace: (id: number) => getJson<TraceTreeNode>(`/api/messages/${id}/trace`),
  diff: (a: number, b: number) => getJson<DiffResponse>(`/api/diff?a=${a}&b=${b}`),
  search: (params: {
    method?: string;
    sinceSeconds?: number;
    errorsOnly?: boolean;
    limit?: number;
  }) => {
    const u = new URLSearchParams();
    if (params.method) u.set("method", params.method);
    if (params.sinceSeconds !== undefined) u.set("since_seconds", String(params.sinceSeconds));
    if (params.errorsOnly) u.set("errors_only", "true");
    if (params.limit !== undefined) u.set("limit", String(params.limit));
    return getJson<MessageRow[]>(`/api/search?${u}`);
  },
  settings: () => getJson<SettingsResponse>("/api/settings"),
  replay: async (
    of: number,
    confirmed: boolean,
    overrideParams?: unknown,
  ): Promise<ReplayResult> => {
    const r = await fetch("/api/replay", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ of, confirmed, override_params: overrideParams }),
    });
    return r.json() as Promise<ReplayResult>;
  },
};
