import { useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { Wrench, Boxes, FileText, Bell } from "lucide-react";
import { api } from "../lib/api";
import { formatDuration, formatTime } from "../lib/format";
import { Sparkline } from "../components/Sparkline";
import { PageHeader } from "../components/PageHeader";
import { Badge, KindBadge } from "../components/Badge";
import { StatusDot } from "../components/StatusDot";
import { ToolInvoker } from "../components/ToolInvoker";
import { SkeletonRows } from "../components/Skeleton";

interface ToolSchema {
  name: string;
  description?: string;
  inputSchema?: any;
}
interface ResourceItem {
  uri: string;
  name?: string;
  description?: string;
  mimeType?: string;
}
interface PromptItem {
  name: string;
  description?: string;
}

export function ServerDetail() {
  const { name = "" } = useParams();
  const [tab, setTab] = useState<"sessions" | "tools" | "resources" | "prompts" | "notifications">(
    "sessions",
  );

  const sessions = useQuery({
    queryKey: ["sessions", name],
    queryFn: () => api.sessions(name, 100),
    enabled: !!name,
  });
  const sparkline = useQuery({
    queryKey: ["sparkline", name],
    queryFn: () => api.sparkline(name),
    enabled: !!name,
    refetchInterval: 30000,
  });
  const recentResponses = useQuery({
    queryKey: ["server-responses", name],
    queryFn: () =>
      api.search({
        sinceSeconds: 86400 * 7,
        limit: 200,
      }),
    enabled: !!name,
  });

  const tools = useMemo<ToolSchema[]>(() => {
    return latestArrayFor(recentResponses.data ?? [], name, "tools/list", "tools");
  }, [recentResponses.data, name]);
  const resources = useMemo<ResourceItem[]>(() => {
    return latestArrayFor(recentResponses.data ?? [], name, "resources/list", "resources");
  }, [recentResponses.data, name]);
  const prompts = useMemo<PromptItem[]>(() => {
    return latestArrayFor(recentResponses.data ?? [], name, "prompts/list", "prompts");
  }, [recentResponses.data, name]);
  const notifications = useMemo(() => {
    return (recentResponses.data ?? [])
      .filter((m) => m.server_name === name && m.kind === "notification")
      .slice(0, 30);
  }, [recentResponses.data, name]);

  const sessionsToday = (sessions.data ?? []).filter((s) => {
    const t = new Date();
    t.setHours(0, 0, 0, 0);
    return new Date(s.started_at) >= t;
  }).length;
  const errorsToday = (sessions.data ?? [])
    .filter((s) => {
      const t = new Date();
      t.setHours(0, 0, 0, 0);
      return new Date(s.started_at) >= t;
    })
    .reduce((n, s) => n + s.error_count, 0);

  return (
    <div>
      <PageHeader
        crumbs={[
          { label: "Home", to: "/" },
          { label: "Servers" },
          { label: name },
        ]}
        title={
          <span className="flex items-center gap-3">
            <StatusDot
              state={errorsToday > 0 ? "err" : sessionsToday > 0 ? "ok" : "idle"}
              pulse={sessionsToday > 0 && errorsToday === 0}
            />
            <span className="mono">{name}</span>
          </span>
        }
        meta={
          <>
            <Badge tone="neutral">stdio</Badge>
            <span>
              {sessionsToday} session{sessionsToday === 1 ? "" : "s"} today
            </span>
            {errorsToday > 0 && (
              <span className="text-err">{errorsToday} errors</span>
            )}
            <span className="ml-auto">
              <Sparkline values={sparkline.data?.buckets ?? []} width={180} height={30} />
            </span>
          </>
        }
      />

      <div className="px-6 border-b border-border1 flex items-center gap-1">
        <Tab active={tab === "sessions"} onClick={() => setTab("sessions")} icon={<FileText className="w-3.5 h-3.5" />}>
          Sessions <Badge tone="neutral">{sessions.data?.length ?? 0}</Badge>
        </Tab>
        <Tab active={tab === "tools"} onClick={() => setTab("tools")} icon={<Wrench className="w-3.5 h-3.5" />}>
          Tools <Badge tone="neutral">{tools.length}</Badge>
        </Tab>
        <Tab active={tab === "resources"} onClick={() => setTab("resources")} icon={<Boxes className="w-3.5 h-3.5" />}>
          Resources <Badge tone="neutral">{resources.length}</Badge>
        </Tab>
        <Tab active={tab === "prompts"} onClick={() => setTab("prompts")} icon={<FileText className="w-3.5 h-3.5" />}>
          Prompts <Badge tone="neutral">{prompts.length}</Badge>
        </Tab>
        <Tab active={tab === "notifications"} onClick={() => setTab("notifications")} icon={<Bell className="w-3.5 h-3.5" />}>
          Notifications <Badge tone="neutral">{notifications.length}</Badge>
        </Tab>
      </div>

      <div className="p-6">
        {tab === "sessions" && <SessionsTab data={sessions.data} loading={sessions.isPending} />}
        {tab === "tools" && (
          <ToolsTab tools={tools} serverName={name} loading={recentResponses.isPending} />
        )}
        {tab === "resources" && <ResourcesTab items={resources} loading={recentResponses.isPending} />}
        {tab === "prompts" && <PromptsTab items={prompts} loading={recentResponses.isPending} />}
        {tab === "notifications" && (
          <NotificationsTab items={notifications} loading={recentResponses.isPending} />
        )}
      </div>
    </div>
  );
}

function Tab({
  active,
  children,
  onClick,
  icon,
}: {
  active: boolean;
  children: React.ReactNode;
  onClick: () => void;
  icon: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-3 py-3 text-sm border-b-2 transition-colors ${
        active
          ? "border-accent text-fg0"
          : "border-transparent text-fg1 hover:text-fg0"
      }`}
    >
      {icon}
      {children}
    </button>
  );
}

function SessionsTab({ data, loading }: { data?: any[]; loading: boolean }) {
  if (loading) return <SkeletonRows rows={6} />;
  if (!data || data.length === 0) {
    return (
      <div className="card p-8 text-center text-fg2">
        No sessions yet. Restart your client to open one.
      </div>
    );
  }
  return (
    <table className="w-full text-sm">
      <thead className="text-fg2 text-xs uppercase tracking-wider">
        <tr className="border-b border-border1">
          <th className="text-left pb-2 font-medium">Started</th>
          <th className="text-right pb-2 font-medium">Duration</th>
          <th className="text-right pb-2 font-medium">Messages</th>
          <th className="text-right pb-2 font-medium">Errors</th>
        </tr>
      </thead>
      <tbody>
        {data.map((s) => {
          const dur =
            s.ended_at != null
              ? new Date(s.ended_at).getTime() - new Date(s.started_at).getTime()
              : null;
          return (
            <tr key={s.id} className="border-b border-border1 hover:bg-bg2 transition-colors">
              <td className="py-2.5">
                <Link to={`/sessions/${s.id}`} className="link mono">
                  {formatTime(s.started_at)}
                </Link>
              </td>
              <td className="py-2.5 text-right mono text-fg1 text-xs">
                {formatDuration(dur)}
              </td>
              <td className="py-2.5 text-right mono">{s.message_count}</td>
              <td
                className={`py-2.5 text-right mono ${
                  s.error_count > 0 ? "text-err font-medium" : "text-fg1"
                }`}
              >
                {s.error_count}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

function ToolsTab({
  tools,
  serverName,
  loading,
}: {
  tools: ToolSchema[];
  serverName: string;
  loading: boolean;
}) {
  if (loading) return <SkeletonRows rows={6} />;
  if (tools.length === 0) {
    return (
      <div className="card p-8 text-center text-fg2">
        Haven't seen a <code className="mono">tools/list</code> response yet.
        Trigger one from your client and it'll show up here.
      </div>
    );
  }
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
      {tools.map((t) => (
        <ToolInvoker key={t.name} serverName={serverName} tool={t} />
      ))}
    </div>
  );
}

function ResourcesTab({ items, loading }: { items: any[]; loading: boolean }) {
  if (loading) return <SkeletonRows rows={6} />;
  if (items.length === 0) {
    return (
      <div className="card p-8 text-center text-fg2">
        No resources advertised yet.
      </div>
    );
  }
  return (
    <ul className="grid grid-cols-1 md:grid-cols-2 gap-3">
      {items.map((r) => (
        <li key={r.uri} className="card p-3">
          <div className="mono text-fg0 truncate">{r.name ?? r.uri}</div>
          {r.description && (
            <div className="text-fg2 text-xs mt-1 line-clamp-2">{r.description}</div>
          )}
          <div className="mono text-xs text-fg2 mt-2 truncate">{r.uri}</div>
          {r.mimeType && (
            <div className="mt-2">
              <Badge tone="neutral">{r.mimeType}</Badge>
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}

function PromptsTab({ items, loading }: { items: any[]; loading: boolean }) {
  if (loading) return <SkeletonRows rows={6} />;
  if (items.length === 0) {
    return (
      <div className="card p-8 text-center text-fg2">
        No prompts advertised yet.
      </div>
    );
  }
  return (
    <ul className="grid grid-cols-1 md:grid-cols-2 gap-3">
      {items.map((p) => (
        <li key={p.name} className="card p-3">
          <div className="mono text-fg0">{p.name}</div>
          {p.description && (
            <div className="text-fg2 text-xs mt-1 line-clamp-2">{p.description}</div>
          )}
        </li>
      ))}
    </ul>
  );
}

function NotificationsTab({ items, loading }: { items: any[]; loading: boolean }) {
  if (loading) return <SkeletonRows rows={6} />;
  if (items.length === 0) {
    return (
      <div className="card p-8 text-center text-fg2">
        No notifications captured yet.
      </div>
    );
  }
  return (
    <table className="w-full text-sm">
      <tbody>
        {items.map((m) => (
          <tr key={m.id} className="border-b border-border1 hover:bg-bg2">
            <td className="py-2 px-3 mono text-fg2 text-xs whitespace-nowrap">
              {formatTime(m.timestamp)}
            </td>
            <td className="py-2 px-3">
              <KindBadge kind={m.kind} />
            </td>
            <td className="py-2 px-3 mono">{m.method ?? "—"}</td>
            <td className="py-2 px-3 text-right">
              <Link to={`/sessions/${m.session_id}`} className="text-accent text-xs">
                open
              </Link>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function latestArrayFor<T>(
  messages: { server_name: string; method: string | null; kind: string; payload_json: string }[],
  serverName: string,
  method: string,
  arrayKey: string,
): T[] {
  // Find the latest response whose paired request was `method`. We can't
  // distinguish that without joining; approximate: parse each response and
  // look for the array key on `result`.
  for (const m of messages) {
    if (m.server_name !== serverName) continue;
    if (m.kind !== "response") continue;
    try {
      const v = JSON.parse(m.payload_json);
      const arr = v?.result?.[arrayKey];
      if (Array.isArray(arr) && arr.length > 0 && (m.method == null || m.method === method)) {
        return arr as T[];
      }
    } catch {
      // ignore
    }
  }
  return [];
}
