import { useMemo } from "react";
import type { MessageRow } from "../lib/api";

interface Props {
  messages: MessageRow[];
  selectedId?: number | null;
  onSelect?: (id: number) => void;
  height?: number;
}

const ROW_HEIGHT = 14;
const ROW_GAP = 4;
const ROW_CAP = 12;

/// Horizontal timing strip. Each row is a paired request, the bar starts at
/// the request timestamp and extends across the latency window. Notifications
/// and orphan messages get a thin tick. Up to ROW_CAP lanes are shown.
export function Waterfall({ messages, selectedId, onSelect }: Props) {
  const lanes = useMemo(() => buildLanes(messages), [messages]);

  if (messages.length === 0) return null;

  const t0 = new Date(messages[0].timestamp).getTime();
  const tEnd = Math.max(
    t0 + 1000,
    ...messages.map(
      (m) => new Date(m.timestamp).getTime() + (m.latency_ms ?? 0),
    ),
  );
  const span = Math.max(1, tEnd - t0);
  const visibleLanes = lanes.slice(0, ROW_CAP);
  const overflow = lanes.length - visibleLanes.length;
  const innerHeight =
    visibleLanes.length * (ROW_HEIGHT + ROW_GAP) - ROW_GAP;

  return (
    <div className="card p-3" style={{ background: "var(--bg-1)" }}>
      <div className="flex items-baseline justify-between text-xs uppercase tracking-wider text-fg2 mb-2">
        <span>Timing</span>
        <span className="mono normal-case tracking-normal">
          {formatSpan(span)} total · {messages.length} msg
        </span>
      </div>
      <div
        className="relative"
        style={{ height: Math.max(innerHeight, 28) }}
      >
        <div className="absolute inset-0 flex justify-between pointer-events-none">
          {[0, 1, 2, 3, 4].map((i) => (
            <div
              key={i}
              className="w-px h-full bg-border1"
              style={{ opacity: i === 0 || i === 4 ? 0.4 : 0.2 }}
            />
          ))}
        </div>

        {visibleLanes.map((lane, i) => {
          const top = i * (ROW_HEIGHT + ROW_GAP);
          if (lane.kind === "tick") {
            const t = new Date(lane.message.timestamp).getTime();
            const xPct = ((t - t0) / span) * 100;
            return (
              <button
                key={lane.message.id}
                onClick={() => onSelect?.(lane.message.id)}
                title={`${labelFor(lane.message)} @ ${new Date(lane.message.timestamp).toLocaleTimeString()}`}
                className="absolute rounded-sm transition-opacity hover:opacity-100"
                style={{
                  left: `${xPct}%`,
                  top,
                  width: 3,
                  height: ROW_HEIGHT,
                  background: colorFor(lane.message),
                  opacity: selectedId === lane.message.id ? 1 : 0.8,
                  outline:
                    selectedId === lane.message.id
                      ? `2px solid var(--accent)`
                      : "none",
                }}
              />
            );
          }
          const ts = new Date(lane.start.timestamp).getTime();
          const teMs =
            lane.end != null ? new Date(lane.end.timestamp).getTime() : ts + 1;
          const xPct = ((ts - t0) / span) * 100;
          const widthPct = Math.max(0.6, ((teMs - ts) / span) * 100);
          const isSelected =
            selectedId === lane.start.id ||
            (lane.end != null && selectedId === lane.end.id);
          const tone =
            lane.end?.kind === "error"
              ? "var(--error)"
              : lane.start.direction === "c2s"
              ? "var(--client-direction)"
              : "var(--server-direction)";
          return (
            <button
              key={`bar-${lane.start.id}`}
              onClick={() => onSelect?.(lane.end?.id ?? lane.start.id)}
              title={`${labelFor(lane.start)} → ${
                lane.end != null
                  ? `${lane.end.kind}${
                      lane.end.latency_ms != null
                        ? ` ${lane.end.latency_ms}ms`
                        : ""
                    }`
                  : "no response yet"
              }`}
              className="absolute rounded-sm transition-all hover:brightness-125"
              style={{
                left: `${xPct}%`,
                top,
                width: `${widthPct}%`,
                minWidth: 4,
                height: ROW_HEIGHT,
                background: tone,
                opacity: isSelected ? 1 : 0.9,
                boxShadow: isSelected ? `0 0 0 2px var(--accent)` : "none",
              }}
            />
          );
        })}
      </div>
      <div className="flex justify-between text-xs text-fg2 mt-2 mono">
        <span>0ms</span>
        {overflow > 0 && <span className="text-fg2">+{overflow} more</span>}
        <span>{formatSpan(span)}</span>
      </div>
    </div>
  );
}

type Lane =
  | { kind: "bar"; start: MessageRow; end: MessageRow | null }
  | { kind: "tick"; message: MessageRow };

function buildLanes(messages: MessageRow[]): Lane[] {
  const paired = new Set<number>();
  const lanes: Lane[] = [];
  const byId = new Map<number, MessageRow>();
  for (const m of messages) byId.set(m.id, m);

  for (const m of messages) {
    if (paired.has(m.id)) continue;
    if (m.kind === "request" && m.correlated_message_id != null) {
      const end = byId.get(m.correlated_message_id) ?? null;
      lanes.push({ kind: "bar", start: m, end });
      paired.add(m.id);
      if (end) paired.add(end.id);
    } else if (
      (m.kind === "response" || m.kind === "error") &&
      m.correlated_message_id != null &&
      byId.has(m.correlated_message_id)
    ) {
      // its request will pick it up
    } else if (m.kind === "request") {
      lanes.push({ kind: "bar", start: m, end: null });
      paired.add(m.id);
    } else {
      lanes.push({ kind: "tick", message: m });
      paired.add(m.id);
    }
  }
  return lanes;
}

function colorFor(m: MessageRow): string {
  if (m.kind === "error") return "var(--error)";
  if (m.kind === "notification") return "var(--warning)";
  return m.direction === "c2s"
    ? "var(--client-direction)"
    : "var(--server-direction)";
}

function labelFor(m: MessageRow): string {
  return m.method ?? m.kind;
}

function formatSpan(ms: number): string {
  if (ms < 1000) return `${Math.round(ms)}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}
