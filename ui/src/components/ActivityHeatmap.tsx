interface Props {
  /// Counts per hour for the last 24 hours (oldest first).
  hours: number[];
}

export function ActivityHeatmap({ hours }: Props) {
  const max = Math.max(1, ...hours);
  const total = hours.reduce((a, b) => a + b, 0);

  // Empty / nearly-empty case: short height, fully muted bars, helper text.
  if (total === 0) {
    return (
      <div className="flex flex-col gap-2">
        <div className="flex items-end gap-1 h-8">
          {hours.map((_, i) => (
            <div
              key={i}
              className="flex-1 rounded-sm bg-bg2"
              style={{ height: "20%" }}
            />
          ))}
        </div>
        <div className="text-fg2 text-xs">
          No traffic in the last 24 hours.
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-end gap-1 h-14">
      {hours.map((v, i) => {
        const intensity = Math.min(1, v / max);
        const bgPct = v === 0 ? 0 : 25 + intensity * 75;
        return (
          <div
            key={i}
            className="flex-1 rounded-sm transition-colors"
            style={{
              height: v === 0 ? "12%" : `${Math.max(20, intensity * 100)}%`,
              background:
                v === 0
                  ? "var(--bg-2)"
                  : `color-mix(in srgb, var(--accent) ${bgPct}%, var(--bg-2))`,
            }}
            title={`${i}:00 — ${v} message${v === 1 ? "" : "s"}`}
          />
        );
      })}
    </div>
  );
}
