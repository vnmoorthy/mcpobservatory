import { useQuery, useQueries } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import {
  Activity,
  AlertCircle,
  Clock,
  Hash,
  Server as ServerIcon,
  ArrowRight,
} from "lucide-react";
import { api } from "../lib/api";
import { Sparkline } from "../components/Sparkline";
import { StatusDot } from "../components/StatusDot";
import { StatTile } from "../components/StatTile";
import { PageHeader } from "../components/PageHeader";
import { HealthBanner } from "../components/HealthBanner";
import { ActivityHeatmap } from "../components/ActivityHeatmap";
import { MethodBadge, KindBadge, DirectionBadge } from "../components/Badge";
import { formatDuration, formatTime } from "../lib/format";

export function Dashboard() {
  const servers = useQuery({ queryKey: ["servers"], queryFn: api.servers });
  const sessions = useQuery({
    queryKey: ["sessions", "all"],
    queryFn: () => api.sessions(undefined, 200),
    refetchInterval: 5000,
  });
  const recent = useQuery({
    queryKey: ["search", "recent"],
    queryFn: () => api.search({ sinceSeconds: 3600, limit: 12 }),
    refetchInterval: 4000,
  });

  const sparklines = useQueries({
    queries: (servers.data ?? []).map((s) => ({
      queryKey: ["sparkline", s.name],
      queryFn: () => api.sparkline(s.name),
      refetchInterval: 30000,
    })),
  });

  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const sessionsToday =
    sessions.data?.filter((s) => new Date(s.started_at) >= today).length ?? 0;
  const errorsToday = (servers.data ?? []).reduce((n, s) => n + s.errors_today, 0);
  const totalMessages =
    sessions.data?.reduce((n, s) => n + s.message_count, 0) ?? 0;

  const latencies = (servers.data ?? [])
    .map((s) => s.p50_latency_ms)
    .filter((v): v is number => v != null);
  const avgP50 =
    latencies.length === 0
      ? null
      : Math.round(latencies.reduce((a, b) => a + b, 0) / latencies.length);

  // 24-hour activity heatmap (synthesise from recent sparklines if available).
  const hours = (() => {
    const buckets = new Array(24).fill(0);
    const now = new Date();
    for (const m of recent.data ?? []) {
      const d = new Date(m.timestamp);
      const diffH = Math.floor((now.getTime() - d.getTime()) / 3_600_000);
      if (diffH >= 0 && diffH < 24) {
        buckets[23 - diffH]++;
      }
    }
    return buckets;
  })();

  // Roll up sparkline buckets across servers for the totalMessages tile.
  const aggregateSpark = (() => {
    const buckets = new Array(60).fill(0);
    for (const sp of sparklines) {
      if (!sp.data) continue;
      sp.data.buckets.forEach((v, i) => {
        buckets[i] += v;
      });
    }
    return buckets;
  })();

  return (
    <div>
      <PageHeader
        title="Dashboard"
        subtitle="Live MCP traffic across all configured upstreams."
        crumbs={[{ label: "Home", to: "/" }]}
      />

      <div className="p-4 sm:p-6 flex flex-col gap-4 sm:gap-6 max-w-[1400px]">
        <HealthBanner
          errors={errorsToday}
          servers={servers.data?.length ?? 0}
          sessions={sessionsToday}
        />

        <section className="grid grid-cols-2 lg:grid-cols-4 gap-3">
          <StatTile
            icon={<Clock className="w-3.5 h-3.5" />}
            label="Sessions today"
            value={sessionsToday}
            hint="across all upstreams"
          />
          <StatTile
            icon={<AlertCircle className="w-3.5 h-3.5" />}
            label="Errors today"
            value={errorsToday}
            tone={errorsToday > 0 ? "err" : "default"}
            hint={errorsToday > 0 ? "investigate via Search" : "looking good"}
          />
          <StatTile
            icon={<Activity className="w-3.5 h-3.5" />}
            label="Avg p50 latency"
            value={avgP50 == null ? "—" : formatDuration(avgP50)}
            hint="median across servers, today"
          />
          <StatTile
            icon={<Hash className="w-3.5 h-3.5" />}
            label="Total messages"
            value={totalMessages.toLocaleString()}
            spark={aggregateSpark}
            hint="last 60 minutes"
          />
        </section>

        <section className="card p-4 sm:p-5">
          <div className="flex items-baseline justify-between mb-4">
            <h2 className="text-fg0 font-medium">Activity (last 24h)</h2>
            <span className="text-fg2 text-xs">
              {hours.reduce((a, b) => a + b, 0)} messages
            </span>
          </div>
          <ActivityHeatmap hours={hours} />
          <div className="flex justify-between text-xs text-fg2 mt-2">
            <span>24h ago</span>
            <span>now</span>
          </div>
        </section>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 sm:gap-6">
          <section className="lg:col-span-2 card p-4 sm:p-5 overflow-x-auto">
            <div className="flex items-baseline justify-between mb-4">
              <h2 className="text-fg0 font-medium">Servers</h2>
              <span className="text-fg2 text-xs">
                {servers.data?.length ?? 0} configured
              </span>
            </div>
            {servers.data && servers.data.length === 0 ? (
              <EmptyState />
            ) : (
              <table className="w-full text-sm min-w-[520px]">
                <thead className="text-fg2 text-xs uppercase tracking-wider">
                  <tr className="border-b border-border1">
                    <th className="text-left pb-2 font-medium">Server</th>
                    <th className="text-left pb-2 font-medium">Transport</th>
                    <th className="text-right pb-2 font-medium">Today</th>
                    <th className="text-right pb-2 font-medium">Err</th>
                    <th className="text-right pb-2 font-medium">p50</th>
                    <th className="text-right pb-2 font-medium">p99</th>
                    <th className="text-right pb-2 font-medium">60m</th>
                  </tr>
                </thead>
                <tbody>
                  {servers.data?.map((s, i) => {
                    const sp = sparklines[i]?.data;
                    return (
                      <tr
                        key={s.name}
                        className="border-b border-border1 hover:bg-bg2 transition-colors"
                      >
                        <td className="py-2.5">
                          <Link
                            to={`/servers/${s.name}`}
                            className="flex items-center gap-2 text-fg0 hover:text-accent group"
                          >
                            <StatusDot
                              state={
                                s.errors_today > 0
                                  ? "err"
                                  : s.sessions_today > 0
                                  ? "ok"
                                  : "idle"
                              }
                              pulse={s.sessions_today > 0 && s.errors_today === 0}
                            />
                            <ServerIcon className="w-3.5 h-3.5 text-fg2 group-hover:text-fg0" />
                            <span className="font-medium">{s.name}</span>
                          </Link>
                        </td>
                        <td className="py-2.5">
                          <span className="mono text-fg1 text-xs">{s.transport}</span>
                        </td>
                        <td className="py-2.5 text-right mono text-fg1">
                          {s.sessions_today}
                        </td>
                        <td
                          className={`py-2.5 text-right mono ${
                            s.errors_today > 0 ? "text-err font-medium" : "text-fg1"
                          }`}
                        >
                          {s.errors_today}
                        </td>
                        <td className="py-2.5 text-right mono text-fg1 text-xs">
                          {formatDuration(s.p50_latency_ms ?? null)}
                        </td>
                        <td className="py-2.5 text-right mono text-fg1 text-xs">
                          {formatDuration(s.p99_latency_ms ?? null)}
                        </td>
                        <td className="py-2.5 text-right">
                          <div className="inline-block">
                            <Sparkline values={sp?.buckets ?? []} width={120} height={26} />
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            )}
          </section>

          <section className="card p-4 sm:p-5 flex flex-col max-h-[640px] overflow-hidden">
            <div className="flex items-baseline justify-between mb-4">
              <h2 className="text-fg0 font-medium">Recent activity</h2>
              <Link to="/search" className="text-fg2 text-xs hover:text-fg0 inline-flex items-center gap-1">
                view all <ArrowRight className="w-3 h-3" />
              </Link>
            </div>
            <div className="overflow-y-auto -mx-1 px-1">
            {(recent.data ?? []).length === 0 ? (
              <div className="text-fg2 text-sm py-8 text-center">
                no traffic yet — start a session in your client.
              </div>
            ) : (
              <ul className="flex flex-col">
                {recent.data?.slice(0, 10).map((m) => (
                  <li
                    key={m.id}
                    className="py-2 border-b border-border1 last:border-0 flex flex-col gap-1 group"
                  >
                    <div className="flex items-center gap-2 text-xs">
                      <DirectionBadge direction={m.direction as "c2s" | "s2c"} />
                      <KindBadge kind={m.kind} />
                      <span className="flex-1" />
                      <span className="text-fg2 mono">{formatTime(m.timestamp)}</span>
                    </div>
                    <Link
                      to={`/sessions/${m.session_id}`}
                      className="flex items-center gap-2 group-hover:text-accent text-fg0 transition-colors"
                    >
                      <MethodBadge method={m.method} />
                      <span className="text-fg2 text-xs truncate">{m.server_name}</span>
                    </Link>
                  </li>
                ))}
              </ul>
            )}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

function EmptyState() {
  return (
    <div className="py-10 flex flex-col items-center text-center gap-3">
      <div className="w-12 h-12 rounded-lg bg-bg2 flex items-center justify-center">
        <ServerIcon className="w-6 h-6 text-fg2" />
      </div>
      <div className="max-w-sm">
        <p className="text-fg0 font-medium">No upstreams yet</p>
        <p className="text-fg2 text-sm mt-1">
          Register your first MCP server, then point your client at the proxy.
        </p>
      </div>
      <pre className="mono text-xs bg-bg0 border border-border1 rounded-md px-3 py-2 select-all text-fg0 mt-2">
{`mcpobs add filesystem \\
  --command npx \\
  --args '@modelcontextprotocol/server-filesystem,/Users/me/Documents'`}
      </pre>
    </div>
  );
}
