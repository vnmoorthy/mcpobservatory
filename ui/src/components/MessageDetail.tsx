import { useMemo, useState } from "react";
import {
  Copy,
  Repeat,
  GitCompare,
  Search as SearchIcon,
  ChevronRight,
  Maximize2,
} from "lucide-react";
import type { MessageRow } from "../lib/api";
import { JsonTree } from "./JsonTree";
import { Badge, KindBadge, MethodBadge, DirectionBadge } from "./Badge";
import { formatDuration, formatTime } from "../lib/format";
import { toast } from "./Toast";
import { asCurl } from "../lib/curl";

interface Props {
  message: MessageRow;
  pair?: MessageRow | null; // matched request/response pair
  onReplay?: (m: MessageRow) => void;
  onDiff?: (m: MessageRow) => void;
}

export function MessageDetail({ message, pair, onReplay, onDiff }: Props) {
  const [tab, setTab] = useState<"payload" | "raw" | "metadata">("payload");
  const [search, setSearch] = useState("");

  const parsed = useMemo(() => {
    try {
      return JSON.parse(message.payload_json);
    } catch {
      return message.payload_json;
    }
  }, [message.payload_json]);

  const meta = useMemo(() => {
    try {
      return JSON.parse(message.metadata_json);
    } catch {
      return {};
    }
  }, [message.metadata_json]);

  return (
    <div className="flex flex-col h-full">
      <header className="px-5 py-4 border-b border-border1 flex flex-col gap-3 bg-bg0">
        <div className="flex items-center gap-2 flex-wrap">
          <DirectionBadge direction={message.direction as "c2s" | "s2c"} />
          <KindBadge kind={message.kind} />
          <MethodBadge method={message.method} />
          {message.rpc_id != null && (
            <Badge tone="neutral" title="JSON-RPC id">
              id={message.rpc_id}
            </Badge>
          )}
          <span className="flex-1" />
          <span className="text-fg2 text-xs mono">{formatTime(message.timestamp)}</span>
        </div>

        <div className="flex items-center gap-3 text-xs text-fg2">
          <span title="message id" className="flex items-center gap-1">
            <span>#</span>
            <span className="mono text-fg1">{message.id}</span>
          </span>
          <span>·</span>
          <span>session</span>
          <span className="mono text-fg1">{message.session_id.slice(0, 8)}</span>
          {message.latency_ms != null && (
            <>
              <span>·</span>
              <span className={message.kind === "error" ? "text-err" : "text-fg1"}>
                latency <span className="mono">{formatDuration(message.latency_ms)}</span>
              </span>
            </>
          )}
          <span>·</span>
          <span>
            size <span className="mono text-fg1">{message.payload_size_bytes}B</span>
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => onReplay?.(message)}
            className="btn btn-primary"
            title="Replay this request against the upstream"
          >
            <Repeat className="w-3.5 h-3.5" />
            Replay
          </button>
          <button onClick={() => onDiff?.(message)} className="btn">
            <GitCompare className="w-3.5 h-3.5" />
            Diff against…
          </button>
          <button
            onClick={() => {
              navigator.clipboard.writeText(asCurl(message));
              toast("copied as cURL", "ok");
            }}
            className="btn"
          >
            <Copy className="w-3.5 h-3.5" />
            Copy as cURL
          </button>
          <button
            onClick={() => {
              navigator.clipboard.writeText(message.payload_json);
              toast("copied raw JSON", "ok");
            }}
            className="btn btn-ghost"
            title="Copy raw payload"
          >
            <Copy className="w-3.5 h-3.5" />
          </button>
        </div>

        {pair && (
          <div className="text-xs text-fg2 flex items-center gap-2">
            <ChevronRight className="w-3 h-3" />
            paired with{" "}
            <Badge tone="neutral">
              #{pair.id} {pair.kind}
            </Badge>
            {pair.latency_ms != null && (
              <span>
                · <span className="mono">{formatDuration(pair.latency_ms)}</span>
              </span>
            )}
          </div>
        )}
      </header>

      <div className="border-b border-border1 px-5 flex items-center gap-1">
        <TabBtn active={tab === "payload"} onClick={() => setTab("payload")}>
          Payload
        </TabBtn>
        <TabBtn active={tab === "raw"} onClick={() => setTab("raw")}>
          Raw
        </TabBtn>
        <TabBtn active={tab === "metadata"} onClick={() => setTab("metadata")}>
          Metadata
        </TabBtn>
        <span className="flex-1" />
        {tab === "payload" && (
          <div className="flex items-center gap-1.5 px-2 py-1 rounded-md border border-border1 bg-bg1 text-xs">
            <SearchIcon className="w-3 h-3 text-fg2" />
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="filter keys / values"
              className="bg-transparent outline-none placeholder:text-fg2 w-40"
            />
          </div>
        )}
      </div>

      <div className="flex-1 overflow-auto p-4 sm:p-5 bg-bg0">
        <div className="max-w-3xl">
          {tab === "payload" && (
            <JsonTree value={parsed} query={search || undefined} initialDepth={3} />
          )}
          {tab === "raw" && (
            <pre className="mono text-xs leading-relaxed text-fg0 whitespace-pre-wrap break-all">
              {message.payload_json}
            </pre>
          )}
          {tab === "metadata" && <JsonTree value={meta} initialDepth={5} />}
          {message.parse_error && (
            <div className="mt-4 p-3 border border-warn/40 bg-warn-soft rounded-md text-warn text-xs flex items-start gap-2">
              <Maximize2 className="w-3.5 h-3.5 shrink-0 mt-0.5" />
              <div>
                <div className="font-medium">parse error</div>
                <div className="mono mt-1">{message.parse_error}</div>
              </div>
            </div>
          )}

          {pair && tab === "payload" && (
            <section className="mt-6 border-t border-border1 pt-5">
              <div className="flex items-center gap-2 mb-3">
                <KindBadge kind={pair.kind} />
                <span className="text-fg2 text-xs">paired</span>
                <span className="mono text-fg2 text-xs">#{pair.id}</span>
                {pair.latency_ms != null && (
                  <span className="mono text-fg2 text-xs">
                    {formatDuration(pair.latency_ms)}
                  </span>
                )}
                <span className="flex-1" />
                <span className="text-fg2 text-xs mono">
                  {formatTime(pair.timestamp)}
                </span>
              </div>
              <JsonTree
                value={(() => {
                  try {
                    return JSON.parse(pair.payload_json);
                  } catch {
                    return pair.payload_json;
                  }
                })()}
                initialDepth={2}
              />
            </section>
          )}
        </div>
      </div>
    </div>
  );
}

function TabBtn({
  active,
  children,
  onClick,
}: {
  active: boolean;
  children: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`px-3 py-2.5 text-sm border-b-2 transition-colors ${
        active
          ? "border-accent text-fg0"
          : "border-transparent text-fg1 hover:text-fg0"
      }`}
    >
      {children}
    </button>
  );
}
