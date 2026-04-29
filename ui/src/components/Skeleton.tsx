interface RowProps {
  rows?: number;
  className?: string;
}

export function SkeletonRows({ rows = 6, className = "" }: RowProps) {
  return (
    <div className={`flex flex-col gap-1 ${className}`}>
      {Array.from({ length: rows }).map((_, i) => (
        <div
          key={i}
          className="h-6 bg-bg1 rounded animate-pulse"
          style={{ width: `${50 + Math.random() * 40}%` }}
        />
      ))}
    </div>
  );
}

export function SkeletonBlock({ height = 80 }: { height?: number }) {
  return (
    <div
      className="bg-bg1 rounded border border-border1 animate-pulse"
      style={{ height }}
    />
  );
}
