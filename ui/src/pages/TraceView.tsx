import { useQuery } from "@tanstack/react-query";
import { useParams } from "react-router-dom";
import { api } from "../lib/api";
import { MessageRow } from "../components/MessageRow";
import { PageHeader } from "../components/PageHeader";
import { Badge, MethodBadge } from "../components/Badge";
import { formatDuration, formatTime } from "../lib/format";

export function TraceView() {
  const { id = "" } = useParams();
  const trace = useQuery({
    queryKey: ["trace", id],
    queryFn: () => api.trace(Number(id)),
    enabled: !!id,
  });

  if (trace.isError) {
    return (
      <div className="p-6">
        <div className="card p-4 border-err/40 bg-err-soft text-err inline-flex items-center gap-2">
          <span className="font-medium">Trace not available:</span>
          <span className="mono text-sm">
            {trace.error instanceof Error ? trace.error.message : "unknown error"}
          </span>
        </div>
      </div>
    );
  }
  if (!trace.data) return <div className="p-6 text-fg2">…</div>;

  const root = trace.data.message;
  const childCount = trace.data.children.length;

  return (
    <div>
      <PageHeader
        crumbs={[
          { label: "Home", to: "/" },
          { label: "Sessions", to: `/sessions/${root.session_id}` },
          { label: `Trace #${id}` },
        ]}
        title={
          <span className="flex items-center gap-2">
            <span>Trace</span>
            <span className="mono text-fg2 text-base">#{id}</span>
          </span>
        }
        subtitle={`${root.server_name} · session ${root.session_id.slice(0, 8)} · ${formatTime(root.timestamp)}`}
        meta={
          <>
            <MethodBadge method={root.method} />
            <Badge tone="neutral">
              {childCount} correlated message{childCount === 1 ? "" : "s"}
            </Badge>
            {root.latency_ms != null && (
              <span className="mono">latency {formatDuration(root.latency_ms)}</span>
            )}
          </>
        }
      />
      <div className="p-4 sm:p-6">
        <div className="card overflow-hidden">
          <div className="border-b border-border1 px-4 py-2 text-fg2 text-xs uppercase tracking-wider">
            Root
          </div>
          <MessageRow row={root} />
          {childCount > 0 && (
            <>
              <div className="border-y border-border1 px-4 py-2 text-fg2 text-xs uppercase tracking-wider">
                Correlated ({childCount})
              </div>
              {trace.data.children.map((c) => (
                <MessageRow key={c.message.id} row={c.message} />
              ))}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
