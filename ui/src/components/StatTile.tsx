import { ReactNode } from "react";
import { Sparkline } from "./Sparkline";

type Trend = "up" | "down" | "flat";

interface Props {
  label: string;
  value: ReactNode;
  hint?: string;
  trend?: Trend;
  delta?: string;
  spark?: number[];
  tone?: "default" | "ok" | "warn" | "err";
  icon?: ReactNode;
}

export function StatTile({ label, value, hint, trend, delta, spark, tone = "default", icon }: Props) {
  const valueColor =
    tone === "err"
      ? "text-err"
      : tone === "warn"
      ? "text-warn"
      : tone === "ok"
      ? "text-accent"
      : "text-fg0";

  return (
    <div className="card p-4 flex flex-col gap-2 relative overflow-hidden">
      <div className="flex items-center gap-2 text-fg2 text-xs uppercase tracking-wider">
        {icon && <span className="opacity-80">{icon}</span>}
        <span>{label}</span>
      </div>
      <div className="flex items-baseline gap-2">
        <div className={`font-medium tracking-tight text-[26px] leading-none ${valueColor}`}>{value}</div>
        {delta && (
          <span
            className={`text-xs ${
              trend === "up" ? "text-err" : trend === "down" ? "text-accent" : "text-fg2"
            }`}
          >
            {trend === "up" ? "↑" : trend === "down" ? "↓" : "·"} {delta}
          </span>
        )}
      </div>
      {spark && spark.length > 0 && (
        <div className="-mt-1">
          <Sparkline values={spark} height={28} width={180} />
        </div>
      )}
      {hint && <div className="text-xs text-fg2 mt-auto">{hint}</div>}
    </div>
  );
}
