interface Props {
  values: number[];
  width?: number;
  height?: number;
  fill?: boolean;
  color?: string;
}

/// Inline-SVG sparkline. Optional area-fill underneath for a nicer look.
export function Sparkline({ values, width = 120, height = 24, fill = true, color }: Props) {
  if (values.length === 0) {
    return <span className="text-fg2 mono">—</span>;
  }
  const max = Math.max(1, ...values);
  const stepX = width / Math.max(values.length - 1, 1);
  const points = values.map((v, i) => {
    const x = i * stepX;
    const y = height - (v / max) * (height - 2) - 1;
    return [x, y] as const;
  });

  const lineD = points
    .map((p, i) => `${i === 0 ? "M" : "L"} ${p[0].toFixed(1)} ${p[1].toFixed(1)}`)
    .join(" ");
  const areaD =
    fill && points.length > 1
      ? `${lineD} L ${points[points.length - 1][0].toFixed(1)} ${height} L 0 ${height} Z`
      : "";

  const stroke = color ?? "var(--accent)";

  return (
    <svg
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      role="img"
      aria-label={`activity sparkline, ${values.length} data points`}
    >
      {fill && areaD && <path d={areaD} fill={stroke} opacity={0.18} />}
      <path
        d={lineD}
        fill="none"
        stroke={stroke}
        strokeWidth={1.5}
        strokeLinejoin="round"
        strokeLinecap="round"
      />
    </svg>
  );
}
