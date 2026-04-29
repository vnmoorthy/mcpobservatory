interface Props {
  title: string;
  hint?: string;
  command?: string;
}

export function EmptyState({ title, hint, command }: Props) {
  return (
    <div className="flex flex-col items-start gap-3 p-6 border border-border1 rounded bg-bg1 text-fg1">
      <div className="text-fg0 text-lg">{title}</div>
      {hint && <div>{hint}</div>}
      {command && (
        <pre className="bg-bg0 border border-border1 rounded px-3 py-2 mono text-fg0 select-all">
          {command}
        </pre>
      )}
    </div>
  );
}
