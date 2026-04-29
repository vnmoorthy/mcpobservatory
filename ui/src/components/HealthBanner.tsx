import { CheckCircle2, AlertTriangle, AlertOctagon } from "lucide-react";

interface Props {
  errors: number;
  servers: number;
  sessions: number;
}

export function HealthBanner({ errors, servers, sessions }: Props) {
  let tone: "ok" | "warn" | "err" = "ok";
  let title = "All systems normal";
  let icon = <CheckCircle2 className="w-5 h-5" />;
  let detail = `${servers} server${servers === 1 ? "" : "s"} configured · ${sessions} session${sessions === 1 ? "" : "s"} today`;

  if (servers === 0) {
    tone = "warn";
    title = "No upstreams configured yet";
    icon = <AlertTriangle className="w-5 h-5" />;
    detail = "Add an upstream MCP server to start capturing traffic.";
  } else if (errors > 0) {
    tone = "err";
    title = `${errors} error${errors === 1 ? "" : "s"} captured today`;
    icon = <AlertOctagon className="w-5 h-5" />;
    detail = `Across ${servers} server${servers === 1 ? "" : "s"}.`;
  }

  const cls =
    tone === "err"
      ? "bg-err-soft border-err/40 text-err"
      : tone === "warn"
      ? "bg-warn-soft border-warn/40 text-warn"
      : "bg-accent-soft border-accent/30 text-accent";

  return (
    <div
      className={`card p-4 flex items-center gap-3 border ${cls}`}
      style={{ background: "var(--bg-1)" }}
    >
      <div
        className={`w-9 h-9 rounded-md flex items-center justify-center shrink-0 ${
          tone === "err"
            ? "bg-err-soft text-err"
            : tone === "warn"
            ? "bg-warn-soft text-warn"
            : "bg-accent-soft text-accent"
        }`}
      >
        {icon}
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-fg0 font-medium text-sm">{title}</div>
        <div className="text-fg1 text-xs mt-0.5">{detail}</div>
      </div>
    </div>
  );
}
