import { useQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Download, Filter, X } from "lucide-react";
import { api, MessageRow } from "../lib/api";
import { MessageRow as Row } from "../components/MessageRow";
import { MessageDetail } from "../components/MessageDetail";
import { useLiveTail } from "../hooks/useLiveTail";
import { Waterfall } from "../components/Waterfall";
import { ContextMenu, MenuItem } from "../components/ContextMenu";
import { PageHeader } from "../components/PageHeader";
import { Badge, KindBadge } from "../components/Badge";
import { SkeletonRows } from "../components/Skeleton";
import { toast } from "../components/Toast";
import { asCurl } from "../lib/curl";

type Direction = "all" | "c2s" | "s2c";

export function SessionView() {
  const { id = "" } = useParams();
  const nav = useNavigate();
  const [follow, setFollow] = useState(true);
  const [methodFilter, setMethodFilter] = useState<string>("");
  const [direction, setDirection] = useState<Direction>("all");
  const [errorsOnly, setErrorsOnly] = useState(false);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [diffPick, setDiffPick] = useState<number | null>(null);

  const messages = useQuery({
    queryKey: ["session-messages", id],
    queryFn: () => api.sessionMessages(id),
    enabled: !!id,
    refetchInterval: follow ? 2000 : false,
  });

  const containerRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    if (!follow) return;
    const el = containerRef.current;
    if (!el) return;
    const fromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    if (fromBottom < 100) el.scrollTop = el.scrollHeight;
  }, [messages.data?.length, follow]);

  const liveEvents = useLiveTail(follow);

  const knownMethods = useMemo(() => {
    const set = new Set<string>();
    for (const m of messages.data ?? []) {
      if (m.method) set.add(m.method);
    }
    return ["", ...Array.from(set).sort()];
  }, [messages.data]);

  const filtered = useMemo(() => {
    return (messages.data ?? []).filter((m) => {
      if (methodFilter && m.method !== methodFilter) return false;
      if (direction !== "all" && m.direction !== direction) return false;
      if (errorsOnly && m.kind !== "error") return false;
      return true;
    });
  }, [messages.data, methodFilter, direction, errorsOnly]);

  // Auto-select first message if nothing is selected.
  useEffect(() => {
    if (selectedId == null && filtered.length > 0) {
      setSelectedId(filtered[0].id);
    }
  }, [filtered, selectedId]);

  // j/k navigation
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement)?.tagName;
      if (["INPUT", "TEXTAREA", "SELECT"].includes(tag)) return;
      const idx = filtered.findIndex((m) => m.id === selectedId);
      if (e.key === "j") {
        e.preventDefault();
        const next = filtered[Math.min(filtered.length - 1, idx + 1)];
        if (next) setSelectedId(next.id);
      } else if (e.key === "k") {
        e.preventDefault();
        const prev = filtered[Math.max(0, idx - 1)];
        if (prev) setSelectedId(prev.id);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [filtered, selectedId]);

  const selected = filtered.find((m) => m.id === selectedId) ?? null;
  const pair = useMemo(() => {
    if (!selected) return null;
    if (selected.correlated_message_id == null) return null;
    return (messages.data ?? []).find((m) => m.id === selected.correlated_message_id) ?? null;
  }, [selected, messages.data]);

  // Right-click context menu
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number; row: MessageRow } | null>(null);
  const ctxItems: MenuItem[] = ctxMenu
    ? [
        {
          label: "Replay",
          hint: "↵",
          onSelect: () => doReplay(ctxMenu.row),
        },
        {
          label: diffPick == null ? "Mark for diff (a)" : `Diff a=${diffPick} ↔ b=${ctxMenu.row.id}`,
          onSelect: () => {
            if (diffPick == null) {
              setDiffPick(ctxMenu.row.id);
              toast(`marked ${ctxMenu.row.id} as diff a — pick another`);
            } else {
              nav(`/diff?a=${diffPick}&b=${ctxMenu.row.id}`);
              setDiffPick(null);
            }
          },
        },
        {
          label: "Copy as cURL",
          onSelect: () => {
            navigator.clipboard.writeText(asCurl(ctxMenu.row));
            toast("copied as cURL", "ok");
          },
        },
        {
          label: "Copy raw JSON",
          onSelect: () => {
            navigator.clipboard.writeText(ctxMenu.row.payload_json);
            toast("copied raw JSON", "ok");
          },
        },
        {
          label: "Open trace",
          onSelect: () => nav(`/traces/${ctxMenu.row.id}`),
        },
      ]
    : [];

  function doReplay(m: MessageRow) {
    nav(`/replay?of=${m.id}`);
  }

  const messageCount = messages.data?.length ?? 0;
  const errorCount = (messages.data ?? []).filter((m) => m.kind === "error").length;
  const session = messages.data?.[0];

  return (
    <div className="flex flex-col h-full">
      <PageHeader
        crumbs={[
          { label: "Home", to: "/" },
          { label: "Sessions" },
          { label: id.slice(0, 8) },
        ]}
        title={
          <span className="flex items-center gap-3">
            <span>Session</span>
            <span className="mono text-fg2 text-base">{id.slice(0, 8)}</span>
          </span>
        }
        subtitle={session ? `${session.server_name} · ${messageCount} messages` : "—"}
        meta={
          <>
            <KindBadge kind={errorCount > 0 ? "error" : "ok"} />
            <span className="mono">{messageCount} msgs</span>
            <span className="mono">{errorCount} err</span>
            {liveEvents.length > 0 && (
              <Badge tone="accent">
                live · {liveEvents.length} event{liveEvents.length === 1 ? "" : "s"}
              </Badge>
            )}
          </>
        }
        actions={
          <>
            <a
              href={`/api/sessions/${id}/messages?limit=2000`}
              target="_blank"
              rel="noreferrer"
              className="btn"
            >
              <Download className="w-3.5 h-3.5" />
              Export
            </a>
            <label className="btn btn-ghost cursor-pointer">
              <input
                type="checkbox"
                checked={follow}
                onChange={(e) => setFollow(e.target.checked)}
              />
              Live tail
            </label>
          </>
        }
      />

      <div className="px-6 pt-4">
        <Waterfall
          messages={messages.data ?? []}
          selectedId={selectedId}
          onSelect={(id) => setSelectedId(id)}
        />
      </div>

      <FilterBar
        methodFilter={methodFilter}
        onMethod={setMethodFilter}
        direction={direction}
        onDirection={setDirection}
        errorsOnly={errorsOnly}
        onErrors={setErrorsOnly}
        knownMethods={knownMethods}
        diffPick={diffPick}
        onClearDiff={() => setDiffPick(null)}
      />

      <div className="flex-1 grid grid-cols-1 lg:grid-cols-[400px_1fr] min-h-0">
        <aside
          className="border-b lg:border-b-0 lg:border-r border-border1 overflow-auto max-h-[40vh] lg:max-h-none"
          ref={containerRef}
        >
          {messages.isPending && (
            <div className="p-4">
              <SkeletonRows rows={8} />
            </div>
          )}
          {!messages.isPending && filtered.length === 0 && (
            <div className="p-6 text-fg2 text-sm">
              {messageCount === 0
                ? "No messages in this session yet."
                : "No messages match the current filter."}
            </div>
          )}
          {filtered.map((m) => (
            <Row
              key={m.id}
              row={m}
              selected={m.id === selectedId}
              onClick={() => setSelectedId(m.id)}
              onContextMenu={(e) => {
                e.preventDefault();
                setCtxMenu({ x: e.clientX, y: e.clientY, row: m });
              }}
            />
          ))}
        </aside>

        <section className="overflow-hidden flex flex-col bg-bg0">
          {selected ? (
            <MessageDetail
              message={selected}
              pair={pair}
              onReplay={doReplay}
              onDiff={(m) => {
                if (diffPick == null) {
                  setDiffPick(m.id);
                  toast(`marked ${m.id} as diff a — pick another`);
                } else {
                  nav(`/diff?a=${diffPick}&b=${m.id}`);
                  setDiffPick(null);
                }
              }}
            />
          ) : (
            <div className="flex-1 flex items-center justify-center text-fg2 text-sm">
              Select a message to inspect.
            </div>
          )}
        </section>
      </div>

      {ctxMenu && (
        <ContextMenu
          items={ctxItems}
          x={ctxMenu.x}
          y={ctxMenu.y}
          onClose={() => setCtxMenu(null)}
        />
      )}
    </div>
  );
}

function FilterBar({
  methodFilter,
  onMethod,
  direction,
  onDirection,
  errorsOnly,
  onErrors,
  knownMethods,
  diffPick,
  onClearDiff,
}: {
  methodFilter: string;
  onMethod: (v: string) => void;
  direction: Direction;
  onDirection: (v: Direction) => void;
  errorsOnly: boolean;
  onErrors: (v: boolean) => void;
  knownMethods: string[];
  diffPick: number | null;
  onClearDiff: () => void;
}) {
  return (
    <div className="px-6 py-3 border-b border-border1 bg-bg0/80 backdrop-blur-sm flex items-center gap-2 flex-wrap text-sm">
      <Filter className="w-3.5 h-3.5 text-fg2" />
      <select
        value={methodFilter}
        onChange={(e) => onMethod(e.target.value)}
        className="bg-bg1 border border-border1 rounded-md px-2 py-1 text-xs"
      >
        {knownMethods.map((m) => (
          <option key={m} value={m}>
            {m || "Method: any"}
          </option>
        ))}
      </select>
      <select
        value={direction}
        onChange={(e) => onDirection(e.target.value as Direction)}
        className="bg-bg1 border border-border1 rounded-md px-2 py-1 text-xs"
      >
        <option value="all">Direction: any</option>
        <option value="c2s">c2s only</option>
        <option value="s2c">s2c only</option>
      </select>
      <label className="btn btn-ghost cursor-pointer text-xs">
        <input
          type="checkbox"
          checked={errorsOnly}
          onChange={(e) => onErrors(e.target.checked)}
        />
        Errors only
      </label>
      <span className="flex-1" />
      {diffPick != null && (
        <span className="flex items-center gap-2 text-xs text-fg1">
          <Badge tone="info">diff a = #{diffPick}</Badge>
          <button onClick={onClearDiff} className="text-fg2 hover:text-fg0">
            <X className="w-3 h-3" />
          </button>
        </span>
      )}
      <span className="text-fg2 text-xs">
        Tip: <span className="kbd">j</span> <span className="kbd">k</span> to navigate
      </span>
    </div>
  );
}
