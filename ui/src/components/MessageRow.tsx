import type { MessageRow as Row } from "../lib/api";
import { formatTime, formatDuration } from "../lib/format";
import { MethodBadge } from "./Badge";

interface Props {
  row: Row;
  selected?: boolean;
  onClick?: () => void;
  onContextMenu?: (e: React.MouseEvent) => void;
}

export function MessageRow({ row, selected, onClick, onContextMenu }: Props) {
  const arrow = row.direction === "c2s" ? "→" : "←";
  const dirColor = row.direction === "c2s" ? "text-client" : "text-server";
  const isError = row.kind === "error";
  return (
    <button
      onClick={onClick}
      onContextMenu={onContextMenu}
      className={`w-full text-left px-3 py-1.5 flex items-center gap-2 transition-colors border-l-2 ${
        selected
          ? "bg-bg2 border-accent text-fg0"
          : isError
          ? "border-err/40 hover:bg-bg2 text-fg0"
          : "border-transparent hover:bg-bg2 text-fg0"
      }`}
    >
      <span className="text-fg2 mono text-xs w-20 shrink-0 tabular-nums">
        {formatTime(row.timestamp)}
      </span>
      <span
        className={`mono text-sm w-4 shrink-0 font-bold ${dirColor}`}
        title={row.direction === "c2s" ? "client → server" : "server → client"}
      >
        {arrow}
      </span>
      <span className="shrink-0 min-w-0 max-w-[150px]">
        <MethodBadge method={row.method ?? row.kind} />
      </span>
      <span className="flex-1 truncate text-fg2 text-xs mono">
        {row.rpc_id ? `#${row.rpc_id}` : ""}
      </span>
      {row.latency_ms != null && (
        <span
          className={`mono text-xs tabular-nums shrink-0 ${
            isError
              ? "text-err"
              : row.latency_ms > 500
              ? "text-warn"
              : "text-fg1"
          }`}
        >
          {formatDuration(row.latency_ms)}
        </span>
      )}
      {isError && (
        <span className="text-err text-xs shrink-0 px-1 rounded bg-err-soft">
          err
        </span>
      )}
    </button>
  );
}
