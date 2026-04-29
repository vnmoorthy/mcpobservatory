import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Search as SearchIcon } from "lucide-react";
import { api } from "../lib/api";
import { formatTime, formatDuration } from "../lib/format";
import { PageHeader } from "../components/PageHeader";
import { Badge, MethodBadge, KindBadge, DirectionBadge } from "../components/Badge";
import { SkeletonRows } from "../components/Skeleton";

const COMMON_METHODS = [
  "",
  "initialize",
  "tools/list",
  "tools/call",
  "resources/list",
  "resources/read",
  "prompts/list",
  "prompts/get",
  "ping",
];

export function SearchPage() {
  const [method, setMethod] = useState<string>("");
  const [errorsOnly, setErrorsOnly] = useState<boolean>(false);
  const [sinceSeconds, setSinceSeconds] = useState<number>(3600);
  const [text, setText] = useState<string>("");

  const results = useQuery({
    queryKey: ["search", method, errorsOnly, sinceSeconds],
    queryFn: () =>
      api.search({
        method: method || undefined,
        sinceSeconds,
        errorsOnly,
        limit: 200,
      }),
  });

  const filtered = (results.data ?? []).filter((m) => {
    if (!text) return true;
    const t = text.toLowerCase();
    return (
      (m.method ?? "").toLowerCase().includes(t) ||
      (m.server_name ?? "").toLowerCase().includes(t) ||
      m.payload_json.toLowerCase().includes(t)
    );
  });

  return (
    <div>
      <PageHeader
        crumbs={[{ label: "Home", to: "/" }, { label: "Search" }]}
        title="Search"
        subtitle="Cross-session message search and filtering."
      />

      <div className="p-6 flex flex-col gap-4">
        <div className="card p-4 flex flex-wrap gap-3 items-baseline">
          <div className="flex items-center gap-2 px-2 py-1 rounded-md border border-border1 bg-bg0 flex-1 min-w-[240px]">
            <SearchIcon className="w-3.5 h-3.5 text-fg2" />
            <input
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder="full-text filter (method, server, payload)…"
              className="bg-transparent flex-1 outline-none text-sm placeholder:text-fg2"
            />
          </div>
          <label className="text-sm flex items-center gap-2">
            <span className="text-fg2 text-xs">method</span>
            <select
              value={method}
              onChange={(e) => setMethod(e.target.value)}
              className="bg-bg0 border border-border1 rounded-md px-2 py-1 text-xs"
            >
              {COMMON_METHODS.map((m) => (
                <option key={m} value={m}>
                  {m || "any"}
                </option>
              ))}
            </select>
          </label>
          <label className="text-sm flex items-center gap-2">
            <span className="text-fg2 text-xs">since</span>
            <select
              value={sinceSeconds}
              onChange={(e) => setSinceSeconds(Number(e.target.value))}
              className="bg-bg0 border border-border1 rounded-md px-2 py-1 text-xs"
            >
              <option value={300}>5 min</option>
              <option value={3600}>1 hour</option>
              <option value={86400}>24 hours</option>
              <option value={604800}>7 days</option>
            </select>
          </label>
          <label className="text-sm flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={errorsOnly}
              onChange={(e) => setErrorsOnly(e.target.checked)}
            />
            errors only
          </label>
          <span className="ml-auto text-fg2 text-xs">
            {filtered.length} result{filtered.length === 1 ? "" : "s"}
          </span>
        </div>

        <div className="card overflow-hidden">
          {results.isPending && (
            <div className="p-4">
              <SkeletonRows rows={6} />
            </div>
          )}
          {!results.isPending && filtered.length === 0 && (
            <div className="p-10 text-center text-fg2 text-sm">
              No matches in this window.
            </div>
          )}
          {filtered.length > 0 && (
            <table className="w-full text-sm">
              <thead className="text-fg2 text-xs uppercase tracking-wider">
                <tr className="border-b border-border1">
                  <th className="text-left px-4 py-2 font-medium">Time</th>
                  <th className="text-left px-4 py-2 font-medium">Direction</th>
                  <th className="text-left px-4 py-2 font-medium">Kind</th>
                  <th className="text-left px-4 py-2 font-medium">Method</th>
                  <th className="text-left px-4 py-2 font-medium">Server</th>
                  <th className="text-right px-4 py-2 font-medium">Latency</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((m) => (
                  <tr key={m.id} className="border-b border-border1 hover:bg-bg2 transition-colors">
                    <td className="px-4 py-2 mono text-xs">
                      <Link to={`/sessions/${m.session_id}`} className="link">
                        {formatTime(m.timestamp)}
                      </Link>
                    </td>
                    <td className="px-4 py-2">
                      <DirectionBadge direction={m.direction as "c2s" | "s2c"} />
                    </td>
                    <td className="px-4 py-2">
                      <KindBadge kind={m.kind} />
                    </td>
                    <td className="px-4 py-2">
                      <MethodBadge method={m.method} />
                    </td>
                    <td className="px-4 py-2 mono text-xs text-fg1">
                      <Badge tone="neutral">{m.server_name}</Badge>
                    </td>
                    <td
                      className={`px-4 py-2 text-right mono text-xs ${
                        m.kind === "error" ? "text-err" : "text-fg1"
                      }`}
                    >
                      {formatDuration(m.latency_ms ?? null)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
}
