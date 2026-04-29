import { Link, NavLink, Outlet } from "react-router-dom";
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Activity,
  Search as SearchIcon,
  Settings as SettingsIcon,
  LayoutDashboard,
  Server as ServerIcon,
  Moon,
  Sun,
  Command,
  HelpCircle,
} from "lucide-react";
import { api } from "../lib/api";
import { CommandPalette } from "./CommandPalette";
import { ShortcutsHelp } from "./ShortcutsHelp";
import { StatusDot } from "./StatusDot";

export function Layout() {
  const [theme, setTheme] = useState<string>(
    () => localStorage.getItem("mcpobs.theme") ?? "dark",
  );
  const [showHelp, setShowHelp] = useState(false);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("mcpobs.theme", theme);
  }, [theme]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement)?.tagName;
      if (["INPUT", "TEXTAREA", "SELECT"].includes(tag)) return;
      if (e.key === "?" && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        setShowHelp((v) => !v);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const health = useQuery({
    queryKey: ["health"],
    queryFn: async () => {
      const r = await fetch("/api/health");
      return r.ok;
    },
    refetchInterval: 5000,
  });

  const servers = useQuery({ queryKey: ["servers"], queryFn: api.servers });

  return (
    <div className="min-h-screen flex flex-col">
      <header className="h-14 border-b border-border1 bg-bg0/85 backdrop-blur-md flex items-center px-3 sm:px-5 gap-1.5 sm:gap-2 sticky top-0 z-20">
        <Link to="/" className="flex items-center gap-2 mr-1 sm:mr-2 shrink-0">
          <span className="w-7 h-7 rounded-md flex items-center justify-center bg-gradient-to-br from-accent to-info">
            <Activity className="w-4 h-4 text-bg0" strokeWidth={2.5} />
          </span>
          <span className="font-medium tracking-tight text-fg0">mcpobs</span>
          <span className="text-fg2 text-xs hidden lg:inline">observatory</span>
        </Link>

        <nav className="flex items-center gap-0.5">
          <NavItem to="/" end icon={<LayoutDashboard className="w-3.5 h-3.5" />} label="Dashboard" hideLabelOnMobile />
          <NavItem to="/search" icon={<SearchIcon className="w-3.5 h-3.5" />} label="Search" hideLabelOnMobile />
          <NavItem to="/settings" icon={<SettingsIcon className="w-3.5 h-3.5" />} label="Settings" hideLabelOnMobile />
        </nav>

        <div className="flex-1" />

        <button
          onClick={() =>
            window.dispatchEvent(
              new KeyboardEvent("keydown", { key: "k", metaKey: true }),
            )
          }
          className="btn btn-ghost text-xs hidden md:inline-flex"
          title="Open command palette (⌘K)"
        >
          <Command className="w-3 h-3" />
          <span>Search…</span>
          <span className="kbd ml-2">⌘K</span>
        </button>

        <button
          onClick={() =>
            window.dispatchEvent(
              new KeyboardEvent("keydown", { key: "k", metaKey: true }),
            )
          }
          className="btn btn-ghost p-1.5 md:hidden"
          title="Open command palette"
          aria-label="Open command palette"
        >
          <Command className="w-4 h-4" />
        </button>

        <button
          onClick={() => setShowHelp(true)}
          className="btn btn-ghost p-1.5 hidden sm:inline-flex"
          title="Keyboard shortcuts (?)"
          aria-label="Help"
        >
          <HelpCircle className="w-4 h-4" />
        </button>

        <div
          className="flex items-center gap-1.5 px-1.5 sm:px-2 py-1 rounded-md border border-border1 bg-bg1 text-xs"
          title={health.data ? "daemon reachable" : "daemon unreachable"}
        >
          <StatusDot state={health.data ? "ok" : "err"} pulse={health.data} />
          <span className="text-fg1 hidden sm:inline">
            {health.data ? "live" : "offline"}
          </span>
        </div>

        <button
          onClick={() => setTheme((t) => (t === "dark" ? "light" : "dark"))}
          className="btn btn-ghost p-1.5"
          aria-label="Toggle theme"
        >
          {theme === "dark" ? <Moon className="w-4 h-4" /> : <Sun className="w-4 h-4" />}
        </button>
      </header>

      <div className="flex-1 flex">
        <aside className="w-64 border-r border-border1 bg-bg0 hidden lg:flex flex-col">
          <div className="px-4 py-3 border-b border-border1 flex items-center justify-between">
            <span className="text-fg2 text-xs uppercase tracking-wider font-medium">
              Servers
            </span>
            <span className="text-fg2 text-xs">{servers.data?.length ?? 0}</span>
          </div>
          <nav className="flex-1 overflow-y-auto px-2 py-2">
            {servers.isPending && (
              <div className="text-fg2 text-xs px-2 py-2">loading…</div>
            )}
            {servers.data?.length === 0 && (
              <div className="text-fg2 text-xs px-2 py-2 leading-relaxed">
                No upstreams configured.
                <br />
                Run <code className="mono text-fg1">mcpobs add</code> in your terminal.
              </div>
            )}
            <ul className="flex flex-col gap-0.5">
              {servers.data?.map((s) => (
                <li key={s.name}>
                  <NavLink
                    to={`/servers/${s.name}`}
                    className={({ isActive }) =>
                      `flex items-center justify-between gap-2 px-2 py-1.5 rounded-md transition-colors ${
                        isActive
                          ? "bg-bg2 text-fg0"
                          : "text-fg1 hover:bg-bg2 hover:text-fg0"
                      }`
                    }
                  >
                    <span className="flex items-center gap-2 min-w-0">
                      <ServerIcon className="w-3.5 h-3.5 text-fg2 shrink-0" />
                      <span className="truncate text-sm">{s.name}</span>
                    </span>
                    <span className="flex items-center gap-1.5 shrink-0">
                      {s.errors_today > 0 && (
                        <span className="text-err text-xs font-medium">
                          {s.errors_today}
                        </span>
                      )}
                      <StatusDot
                        state={
                          s.errors_today > 0
                            ? "err"
                            : s.sessions_today > 0
                            ? "ok"
                            : "idle"
                        }
                      />
                    </span>
                  </NavLink>
                </li>
              ))}
            </ul>
          </nav>
          <div className="px-3 py-2 border-t border-border1 text-xs text-fg2 flex flex-col gap-1">
            <div className="flex items-center justify-between">
              <span>spec</span>
              <span className="mono text-fg1">2025-06-18</span>
            </div>
            <a
              href="https://github.com/vnmoorthy/mcpobservatory"
              target="_blank"
              rel="noreferrer noopener"
              className="hover:text-fg0 inline-flex items-center justify-between"
            >
              <span>source</span>
              <span className="text-fg2">↗</span>
            </a>
            <a
              href="https://github.com/vnmoorthy/mcpobservatory/issues/new/choose"
              target="_blank"
              rel="noreferrer noopener"
              className="hover:text-fg0 inline-flex items-center justify-between"
            >
              <span>report issue</span>
              <span className="text-fg2">↗</span>
            </a>
          </div>
        </aside>

        <main className="flex-1 overflow-auto bg-bg0">
          <Outlet />
        </main>
      </div>

      <CommandPalette />
      {showHelp && <ShortcutsHelp onClose={() => setShowHelp(false)} />}
    </div>
  );
}

function NavItem({
  to,
  icon,
  label,
  end,
  hideLabelOnMobile,
}: {
  to: string;
  icon: React.ReactNode;
  label: string;
  end?: boolean;
  hideLabelOnMobile?: boolean;
}) {
  return (
    <NavLink
      to={to}
      end={end}
      title={label}
      className={({ isActive }) =>
        `flex items-center gap-1.5 px-2 sm:px-2.5 py-1.5 rounded-md text-sm transition-colors ${
          isActive ? "bg-bg2 text-fg0" : "text-fg1 hover:bg-bg2 hover:text-fg0"
        }`
      }
    >
      {icon}
      <span className={hideLabelOnMobile ? "hidden sm:inline" : ""}>
        {label}
      </span>
    </NavLink>
  );
}
