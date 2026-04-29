import { ReactNode } from "react";

type Tone = "neutral" | "accent" | "info" | "warn" | "err" | "purple" | "client" | "server";

const toneClass: Record<Tone, string> = {
  neutral: "bg-bg2 text-fg1 border-border1",
  accent: "bg-accent-soft text-accent border-accent/30",
  info: "bg-info-soft text-info border-info/30",
  warn: "bg-warn-soft text-warn border-warn/30",
  err: "bg-err-soft text-err border-err/30",
  purple: "bg-purple-soft text-purple border-purple/30",
  client: "bg-accent-soft text-client border-client/30",
  server: "bg-info-soft text-server border-server/30",
};

interface Props {
  tone?: Tone;
  children: ReactNode;
  className?: string;
  title?: string;
}

export function Badge({ tone = "neutral", children, className = "", title }: Props) {
  return (
    <span
      title={title}
      className={`inline-flex items-center px-1.5 py-0.5 rounded border mono text-xs leading-none whitespace-nowrap ${toneClass[tone]} ${className}`}
    >
      {children}
    </span>
  );
}

export function MethodBadge({ method }: { method: string | null | undefined }) {
  if (!method) return <Badge tone="neutral">—</Badge>;
  let tone: Tone = "neutral";
  if (method === "initialize" || method === "notifications/initialized") tone = "info";
  else if (method === "tools/call") tone = "purple";
  else if (method.startsWith("notifications/")) tone = "warn";
  else if (method === "ping") tone = "neutral";
  else if (
    method.endsWith("/list") ||
    method.endsWith("/get") ||
    method.endsWith("/read")
  )
    tone = "accent";
  return (
    <Badge tone={tone} title={method}>
      {method}
    </Badge>
  );
}

export function KindBadge({ kind }: { kind: string }) {
  const tone: Tone =
    kind === "request"
      ? "info"
      : kind === "response"
      ? "accent"
      : kind === "error"
      ? "err"
      : kind === "notification"
      ? "warn"
      : kind === "unknown" || kind === "unparsed"
      ? "neutral"
      : "neutral";
  return <Badge tone={tone}>{kind}</Badge>;
}

export function DirectionBadge({ direction }: { direction: "c2s" | "s2c" }) {
  return (
    <Badge tone={direction === "c2s" ? "client" : "server"} title={direction === "c2s" ? "client → server" : "server → client"}>
      {direction === "c2s" ? "▶ c2s" : "◀ s2c"}
    </Badge>
  );
}
