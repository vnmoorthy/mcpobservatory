import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
  LayoutDashboard,
  Search as SearchIcon,
  Settings as SettingsIcon,
  Server as ServerIcon,
  Clock,
} from "lucide-react";
import { api } from "../lib/api";

interface Item {
  label: string;
  hint: string;
  icon: React.ReactNode;
  onSelect: () => void;
}

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const [highlight, setHighlight] = useState(0);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const nav = useNavigate();

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
      } else if (e.key === "Escape") {
        setOpen(false);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    if (open) inputRef.current?.focus();
    else {
      setQ("");
      setHighlight(0);
    }
  }, [open]);

  const servers = useQuery({
    queryKey: ["servers"],
    queryFn: api.servers,
    enabled: open,
  });
  const sessions = useQuery({
    queryKey: ["sessions", "all"],
    queryFn: () => api.sessions(undefined, 50),
    enabled: open,
  });

  const items: Item[] = useMemo(() => {
    const xs: Item[] = [
      {
        label: "Dashboard",
        hint: "/",
        icon: <LayoutDashboard className="w-3.5 h-3.5" />,
        onSelect: () => nav("/"),
      },
      {
        label: "Search messages",
        hint: "/search",
        icon: <SearchIcon className="w-3.5 h-3.5" />,
        onSelect: () => nav("/search"),
      },
      {
        label: "Settings",
        hint: "/settings",
        icon: <SettingsIcon className="w-3.5 h-3.5" />,
        onSelect: () => nav("/settings"),
      },
    ];
    for (const s of servers.data ?? []) {
      xs.push({
        label: `Server: ${s.name}`,
        hint: s.transport,
        icon: <ServerIcon className="w-3.5 h-3.5" />,
        onSelect: () => nav(`/servers/${s.name}`),
      });
    }
    for (const s of sessions.data ?? []) {
      xs.push({
        label: `Session ${s.id.slice(0, 8)} · ${s.server_name}`,
        hint: new Date(s.started_at).toLocaleTimeString(),
        icon: <Clock className="w-3.5 h-3.5" />,
        onSelect: () => nav(`/sessions/${s.id}`),
      });
    }
    if (!q) return xs.slice(0, 14);
    const ql = q.toLowerCase();
    return xs.filter((i) => i.label.toLowerCase().includes(ql)).slice(0, 14);
  }, [servers.data, sessions.data, q, nav]);

  useEffect(() => {
    setHighlight(0);
  }, [q]);

  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/50 backdrop-blur-sm"
      onClick={() => setOpen(false)}
    >
      <div
        className="card w-full max-w-xl shadow-lg overflow-hidden"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown") {
            e.preventDefault();
            setHighlight((h) => Math.min(items.length - 1, h + 1));
          } else if (e.key === "ArrowUp") {
            e.preventDefault();
            setHighlight((h) => Math.max(0, h - 1));
          } else if (e.key === "Enter") {
            e.preventDefault();
            items[highlight]?.onSelect();
            setOpen(false);
          }
        }}
      >
        <div className="flex items-center px-4 py-3 border-b border-border1">
          <SearchIcon className="w-4 h-4 text-fg2" />
          <input
            ref={inputRef}
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Find a server, session, or page…"
            className="flex-1 bg-transparent ml-3 outline-none text-fg0 text-sm placeholder:text-fg2"
          />
          <span className="kbd">Esc</span>
        </div>
        <ul className="max-h-[50vh] overflow-y-auto py-1">
          {items.length === 0 && (
            <li className="px-4 py-6 text-fg2 text-sm text-center">No matches.</li>
          )}
          {items.map((it, i) => (
            <li key={i}>
              <button
                className={`w-full text-left px-4 py-2 flex items-center justify-between gap-3 ${
                  i === highlight ? "bg-bg2 text-fg0" : "text-fg1 hover:bg-bg2"
                }`}
                onMouseEnter={() => setHighlight(i)}
                onClick={() => {
                  it.onSelect();
                  setOpen(false);
                }}
              >
                <span className="flex items-center gap-3 min-w-0">
                  <span className="text-fg2">{it.icon}</span>
                  <span className="truncate">{it.label}</span>
                </span>
                <span className="text-fg2 text-xs mono shrink-0">{it.hint}</span>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
