import { useState } from "react";
import { toast } from "./Toast";

interface Props {
  value: unknown;
  query?: string;
  initialDepth?: number;
}

/// Syntax-highlighted, collapsible JSON tree. No external deps.
export function JsonTree({ value, query, initialDepth = 3 }: Props) {
  return (
    <div className="mono text-sm leading-relaxed">
      <Node value={value} path="$" depth={0} initialDepth={initialDepth} query={query?.toLowerCase()} />
    </div>
  );
}

function Node({
  value,
  path,
  depth,
  initialDepth,
  query,
}: {
  value: unknown;
  path: string;
  depth: number;
  initialDepth: number;
  query?: string;
}) {
  const [open, setOpen] = useState(depth < initialDepth);

  if (value === null) return <Token kind="null">null</Token>;
  if (typeof value === "boolean") return <Token kind="bool">{String(value)}</Token>;
  if (typeof value === "number") return <Token kind="number">{value}</Token>;
  if (typeof value === "string")
    return (
      <Token kind="string">
        <Highlight text={JSON.stringify(value)} q={query} />
      </Token>
    );

  if (Array.isArray(value)) {
    if (value.length === 0) return <Token kind="punct">[]</Token>;
    return (
      <span>
        <Toggle open={open} onClick={() => setOpen((v) => !v)} label={`Array(${value.length})`} />
        <Token kind="punct">[</Token>
        {open ? (
          <ul className="ml-4 border-l border-bg3 pl-3">
            {value.map((v, i) => (
              <li key={i} className="group flex items-start">
                <Index>{i}</Index>
                <span className="flex-1">
                  <Node
                    value={v}
                    path={`${path}[${i}]`}
                    depth={depth + 1}
                    initialDepth={initialDepth}
                    query={query}
                  />
                  {i < value.length - 1 && <Token kind="punct">,</Token>}
                </span>
                <CopyPathBtn path={`${path}[${i}]`} />
              </li>
            ))}
          </ul>
        ) : (
          <Token kind="punct">…</Token>
        )}
        <Token kind="punct">]</Token>
      </span>
    );
  }

  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.length === 0) return <Token kind="punct">{"{}"}</Token>;
    return (
      <span>
        <Toggle open={open} onClick={() => setOpen((v) => !v)} label={`Object(${entries.length})`} />
        <Token kind="punct">{"{"}</Token>
        {open ? (
          <ul className="ml-4 border-l border-bg3 pl-3">
            {entries.map(([k, v], i) => (
              <li key={k} className="group flex items-start">
                <Key name={k} query={query} />
                <Token kind="punct" className="mx-1">:</Token>
                <span className="flex-1">
                  <Node
                    value={v}
                    path={`${path}.${k}`}
                    depth={depth + 1}
                    initialDepth={initialDepth}
                    query={query}
                  />
                  {i < entries.length - 1 && <Token kind="punct">,</Token>}
                </span>
                <CopyPathBtn path={`${path}.${k}`} />
              </li>
            ))}
          </ul>
        ) : (
          <Token kind="punct">…</Token>
        )}
        <Token kind="punct">{"}"}</Token>
      </span>
    );
  }

  return <span>{String(value)}</span>;
}

function Token({
  kind,
  children,
  className = "",
}: {
  kind: "string" | "number" | "bool" | "null" | "punct";
  children: React.ReactNode;
  className?: string;
}) {
  const colorVar =
    kind === "string"
      ? "text-[var(--json-string)]"
      : kind === "number"
      ? "text-[var(--json-number)]"
      : kind === "bool"
      ? "text-[var(--json-bool)]"
      : kind === "null"
      ? "text-[var(--json-null)]"
      : "text-[var(--json-punct)]";
  return <span className={`${colorVar} ${className}`}>{children}</span>;
}

function Key({ name, query }: { name: string; query?: string }) {
  return (
    <span className="text-[var(--json-key)]">
      "<Highlight text={name} q={query} />"
    </span>
  );
}

function Index({ children }: { children: number }) {
  return <span className="text-fg2 mr-2 text-xs select-none">[{children}]</span>;
}

function Toggle({ open, onClick, label }: { open: boolean; onClick: () => void; label: string }) {
  return (
    <button
      onClick={onClick}
      title={label}
      className="text-fg2 hover:text-fg0 mr-1 select-none"
    >
      {open ? "▾" : "▸"}
    </button>
  );
}

function CopyPathBtn({ path }: { path: string }) {
  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        navigator.clipboard.writeText(path).catch(() => {});
        toast(`copied path: ${path}`, "ok");
      }}
      className="opacity-0 group-hover:opacity-100 ml-2 text-fg2 hover:text-accent text-xs"
      title="Copy JSON path"
    >
      ⧉
    </button>
  );
}

function Highlight({ text, q }: { text: string; q?: string }) {
  if (!q) return <>{text}</>;
  const lower = text.toLowerCase();
  const idx = lower.indexOf(q);
  if (idx < 0) return <>{text}</>;
  return (
    <>
      {text.slice(0, idx)}
      <mark className="bg-warn/40 text-fg0 rounded px-0.5">
        {text.slice(idx, idx + q.length)}
      </mark>
      {text.slice(idx + q.length)}
    </>
  );
}
