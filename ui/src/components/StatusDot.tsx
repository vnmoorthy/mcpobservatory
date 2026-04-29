interface Props {
  state: "ok" | "warn" | "err" | "idle";
  pulse?: boolean;
  className?: string;
  title?: string;
}

const colorMap = {
  ok: "text-accent",
  warn: "text-warn",
  err: "text-err",
  idle: "text-fg2",
};

export function StatusDot({ state, pulse = false, className = "", title }: Props) {
  return (
    <span
      className={`relative inline-flex items-center justify-center w-2 h-2 rounded-full bg-current ${colorMap[state]} ${pulse ? "pulse-dot" : ""} ${className}`}
      title={title}
    />
  );
}
