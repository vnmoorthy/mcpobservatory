import { useQuery } from "@tanstack/react-query";
import { Link, useSearchParams } from "react-router-dom";
import { ArrowLeftRight } from "lucide-react";
import { api } from "../lib/api";
import { PageHeader } from "../components/PageHeader";

export function DiffView() {
  const [params] = useSearchParams();
  const a = Number(params.get("a"));
  const b = Number(params.get("b"));

  const diff = useQuery({
    queryKey: ["diff", a, b],
    queryFn: () => api.diff(a, b),
    enabled: !!a && !!b,
  });

  if (!a || !b) {
    return (
      <div className="p-6 text-fg2">
        missing <code className="mono">a</code> or <code className="mono">b</code> query param.
      </div>
    );
  }
  if (diff.isError) {
    return (
      <div className="p-6">
        <div className="card p-4 border-err/40 bg-err-soft text-err inline-flex items-center gap-2">
          <span className="font-medium">Diff failed:</span>
          <span className="mono text-sm">
            {diff.error instanceof Error ? diff.error.message : "unknown error"}
          </span>
        </div>
      </div>
    );
  }
  if (!diff.data) return <div className="p-6 text-fg2">…</div>;

  // Build aligned line lists from change tags. The `similar` crate returns
  // changes in order; we expand each into the appropriate column.
  type Cell = { text: string; tag: "equal" | "delete" | "insert"; n?: number };
  const left: Cell[] = [];
  const right: Cell[] = [];
  let nA = 0;
  let nB = 0;
  for (const c of diff.data.changes) {
    const lines = c.text.split("\n");
    if (lines[lines.length - 1] === "") lines.pop();
    for (const line of lines) {
      if (c.tag === "equal") {
        nA++;
        nB++;
        left.push({ text: line, tag: "equal", n: nA });
        right.push({ text: line, tag: "equal", n: nB });
      } else if (c.tag === "delete") {
        nA++;
        left.push({ text: line, tag: "delete", n: nA });
        right.push({ text: "", tag: "equal" });
      } else if (c.tag === "insert") {
        nB++;
        left.push({ text: "", tag: "equal" });
        right.push({ text: line, tag: "insert", n: nB });
      }
    }
  }

  return (
    <div>
      <PageHeader
        crumbs={[{ label: "Home", to: "/" }, { label: "Diff" }]}
        title={
          <span className="flex items-center gap-3">
            <Link to={`/sessions/${diff.data.a.session_id}`} className="link mono">#{a}</Link>
            <ArrowLeftRight className="w-4 h-4 text-fg2" />
            <Link to={`/sessions/${diff.data.b.session_id}`} className="link mono">#{b}</Link>
          </span>
        }
        subtitle={`${diff.data.a.method ?? "—"} ↔ ${diff.data.b.method ?? "—"}`}
      />
      <div className="p-6">
        <div className="card overflow-hidden">
          <div className="grid grid-cols-2 border-b border-border1 mono text-xs">
            <div className="px-3 py-2 border-r border-border1 text-fg2">
              a · {diff.data.a.server_name} ·{" "}
              <span className="text-err">- removed</span>
            </div>
            <div className="px-3 py-2 text-fg2">
              b · {diff.data.b.server_name} ·{" "}
              <span className="text-accent">+ added</span>
            </div>
          </div>
          <div className="grid grid-cols-2 mono text-xs">
            <DiffColumn cells={left} side="left" />
            <DiffColumn cells={right} side="right" />
          </div>
        </div>
      </div>
    </div>
  );
}

function DiffColumn({ cells, side }: { cells: { text: string; tag: "equal" | "delete" | "insert"; n?: number }[]; side: "left" | "right" }) {
  return (
    <div className={side === "left" ? "border-r border-border1" : ""}>
      {cells.map((c, i) => (
        <div
          key={i}
          className={`flex items-start gap-2 px-3 py-0.5 leading-5 ${
            c.tag === "delete"
              ? "bg-err-soft"
              : c.tag === "insert"
              ? "bg-accent-soft"
              : ""
          }`}
        >
          <span className="text-fg2 select-none w-8 text-right tabular-nums shrink-0">
            {c.n ?? ""}
          </span>
          <span
            className={`shrink-0 ${
              c.tag === "delete"
                ? "text-err"
                : c.tag === "insert"
                ? "text-accent"
                : "text-fg2"
            }`}
          >
            {c.tag === "delete" ? "-" : c.tag === "insert" ? "+" : " "}
          </span>
          <span
            className={`whitespace-pre flex-1 ${
              c.tag === "delete"
                ? "text-fg0"
                : c.tag === "insert"
                ? "text-fg0"
                : "text-fg1"
            }`}
          >
            {c.text || " "}
          </span>
        </div>
      ))}
    </div>
  );
}
