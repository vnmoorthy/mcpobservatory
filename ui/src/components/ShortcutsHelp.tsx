import { X } from "lucide-react";

const SHORTCUTS: { keys: string[]; label: string }[] = [
  { keys: ["⌘", "K"], label: "Open command palette" },
  { keys: ["?"], label: "Toggle this help" },
  { keys: ["j"], label: "Next message in list" },
  { keys: ["k"], label: "Previous message in list" },
  { keys: ["Esc"], label: "Close palette / dialog" },
  { keys: ["⌘", "F"], label: "Search inside JSON" },
  { keys: ["g", "d"], label: "Go to dashboard" },
  { keys: ["g", "s"], label: "Go to search" },
];

export function ShortcutsHelp({ onClose }: { onClose: () => void }) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="card w-full max-w-md p-5 shadow-lg"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="flex items-center justify-between mb-4">
          <h2 className="text-fg0 font-medium">Keyboard shortcuts</h2>
          <button onClick={onClose} className="btn btn-ghost p-1">
            <X className="w-4 h-4" />
          </button>
        </header>
        <ul className="flex flex-col gap-2">
          {SHORTCUTS.map((s) => (
            <li key={s.label} className="flex items-center justify-between text-sm">
              <span className="text-fg1">{s.label}</span>
              <span className="flex items-center gap-1">
                {s.keys.map((k) => (
                  <span key={k} className="kbd">
                    {k}
                  </span>
                ))}
              </span>
            </li>
          ))}
        </ul>
        <p className="text-fg2 text-xs mt-4">Esc to close.</p>
      </div>
    </div>
  );
}
