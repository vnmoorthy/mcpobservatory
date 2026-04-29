import { ReactNode, useEffect, useRef } from "react";

export interface MenuItem {
  label: string;
  hint?: string;
  onSelect: () => void;
  disabled?: boolean;
  danger?: boolean;
  icon?: ReactNode;
}

interface Props {
  items: MenuItem[];
  x: number;
  y: number;
  onClose: () => void;
}

export function ContextMenu({ items, x, y, onClose }: Props) {
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    function onDoc(e: MouseEvent) {
      if (!ref.current?.contains(e.target as Node)) onClose();
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("mousedown", onDoc);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      document.removeEventListener("keydown", onKey);
    };
  }, [onClose]);

  // Clamp to viewport
  const left = Math.min(x, window.innerWidth - 220);
  const top = Math.min(y, window.innerHeight - items.length * 32 - 16);

  return (
    <div
      ref={ref}
      className="fixed z-50 card shadow-lg py-1 min-w-[200px]"
      style={{ left, top }}
    >
      {items.map((it, i) => (
        <button
          key={i}
          disabled={it.disabled}
          onClick={() => {
            it.onSelect();
            onClose();
          }}
          className={`w-full text-left px-3 py-1.5 text-sm flex items-center gap-2 hover:bg-bg2 disabled:opacity-40 disabled:hover:bg-transparent ${
            it.danger ? "text-err hover:bg-err-soft" : "text-fg1 hover:text-fg0"
          }`}
        >
          {it.icon && <span className="text-fg2">{it.icon}</span>}
          <span className="flex-1">{it.label}</span>
          {it.hint && <span className="kbd">{it.hint}</span>}
        </button>
      ))}
    </div>
  );
}
