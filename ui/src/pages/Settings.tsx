import { useQuery } from "@tanstack/react-query";
import { Cpu, Shield, Database, Box } from "lucide-react";
import { api } from "../lib/api";
import { PageHeader } from "../components/PageHeader";
import { Badge } from "../components/Badge";

export function SettingsPage() {
  const settings = useQuery({ queryKey: ["settings"], queryFn: api.settings });

  return (
    <div>
      <PageHeader
        crumbs={[{ label: "Home", to: "/" }, { label: "Settings" }]}
        title="Settings"
        subtitle="Daemon configuration. Read-only in v0 — edit ~/.mcpobs/config.toml to change."
      />

      <div className="p-6 grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-[1200px]">
        {settings.isPending && (
          <div className="text-fg2 text-sm">loading…</div>
        )}
        {settings.data && (
          <>
            <Group title="Daemon" icon={<Cpu className="w-4 h-4" />}>
              <Row label="version" value={<span className="mono">{settings.data.version}</span>} />
              <Row
                label="MCP spec revision"
                value={<Badge tone="info">{settings.data.mcp_spec_revision}</Badge>}
              />
              <Row label="listen" value={<span className="mono">{settings.data.listen}</span>} />
              <Row label="retention days" value={<span className="mono">{settings.data.retention_days}</span>} />
            </Group>

            <Group title="Security" icon={<Shield className="w-4 h-4" />}>
              <div className="text-fg1 text-sm leading-relaxed">
                Bound to <span className="mono">127.0.0.1</span> by default — accessible only on this machine.
                Replay is gated by an Origin check and a method safe-list.
                Default-on redaction strips <span className="mono">password</span>, <span className="mono">token</span>, <span className="mono">secret</span>, <span className="mono">api_key</span>, and <span className="mono">Authorization</span>.
              </div>
            </Group>

            <Group title={`Upstreams (${settings.data.upstreams.length})`} icon={<Box className="w-4 h-4" />}>
              {settings.data.upstreams.length === 0 && (
                <div className="text-fg2 text-sm">none configured</div>
              )}
              {settings.data.upstreams.map((u) => (
                <Row
                  key={u.name}
                  label={u.name}
                  value={<Badge tone="neutral">{u.transport}</Badge>}
                />
              ))}
            </Group>

            <Group title="Storage" icon={<Database className="w-4 h-4" />}>
              <div className="text-fg1 text-sm leading-relaxed">
                SQLite WAL mode at <span className="mono">~/.mcpobs/traces.db</span>. Run{" "}
                <span className="mono">mcpobs prune --older-than {settings.data.retention_days}</span> to compact.
              </div>
            </Group>

            <Group title="Allowed origins" icon={<Shield className="w-4 h-4" />}>
              <ul className="flex flex-col gap-1 mono text-xs">
                {settings.data.allowed_origins.map((o) => (
                  <li key={o} className="text-fg1">
                    {o}
                  </li>
                ))}
              </ul>
            </Group>
          </>
        )}
      </div>
    </div>
  );
}

function Group({
  title,
  icon,
  children,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section className="card overflow-hidden">
      <div className="border-b border-border1 px-4 py-3 flex items-center gap-2">
        <span className="text-fg2">{icon}</span>
        <h3 className="text-fg0 font-medium text-sm">{title}</h3>
      </div>
      <div className="p-4 flex flex-col gap-2">{children}</div>
    </section>
  );
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 text-sm">
      <span className="text-fg2">{label}</span>
      <span className="text-fg0">{value}</span>
    </div>
  );
}
