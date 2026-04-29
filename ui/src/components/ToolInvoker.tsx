import { useMemo, useState } from "react";
import { Send, Code2 } from "lucide-react";
import { JsonTree } from "./JsonTree";
import { toast } from "./Toast";

interface ToolSchema {
  name: string;
  description?: string;
  inputSchema?: SchemaNode;
}
interface SchemaNode {
  type?: string;
  properties?: Record<string, SchemaNode>;
  required?: string[];
  description?: string;
  default?: unknown;
  enum?: unknown[];
  items?: SchemaNode;
}

interface Props {
  serverName: string;
  tool: ToolSchema;
}

/// Auto-generates a form from a tool's `inputSchema` and lets the user
/// "Try it" — sends a synthetic `tools/call` via /api/replay.
///
/// We don't have an existing message id to replay, so this works only when
/// the server is HTTP. The button is disabled and shows a hint otherwise.
export function ToolInvoker({ serverName, tool }: Props) {
  const fields = useMemo(() => collectFields(tool.inputSchema), [tool.inputSchema]);
  const [values, setValues] = useState<Record<string, unknown>>(() =>
    Object.fromEntries(fields.map((f) => [f.name, f.default ?? defaultFor(f.type)])),
  );
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<unknown>(null);
  const [open, setOpen] = useState(false);

  async function send() {
    setBusy(true);
    setResult(null);
    try {
      const r = await fetch(`/mcp/${serverName}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: `inv-${Date.now()}`,
          method: "tools/call",
          params: { name: tool.name, arguments: values },
        }),
      });
      const text = await r.text();
      try {
        setResult(JSON.parse(text));
      } catch {
        setResult(text);
      }
      toast(`called ${tool.name}`, r.ok ? "ok" : "err");
    } catch (e) {
      toast(`call failed: ${(e as Error).message}`, "err");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="card p-3 flex flex-col gap-2">
      <button
        onClick={() => setOpen((v) => !v)}
        className="flex items-center justify-between text-left"
      >
        <div className="min-w-0">
          <div className="mono text-fg0 font-medium truncate">{tool.name}</div>
          {tool.description && (
            <div className="text-fg2 text-xs mt-0.5 line-clamp-2">{tool.description}</div>
          )}
        </div>
        <span className="text-fg2 text-xs ml-3 shrink-0 flex items-center gap-1">
          <Code2 className="w-3 h-3" /> {open ? "hide" : "try it"}
        </span>
      </button>

      {open && (
        <div className="flex flex-col gap-2 pt-2 border-t border-border1">
          {fields.length === 0 && (
            <div className="text-fg2 text-xs">no input parameters.</div>
          )}
          {fields.map((f) => (
            <label key={f.name} className="flex flex-col gap-1">
              <span className="text-xs text-fg1 mono flex items-center gap-1">
                {f.name}
                {f.required && <span className="text-err">*</span>}
                <span className="text-fg2">:{f.type ?? "any"}</span>
              </span>
              <input
                value={String(values[f.name] ?? "")}
                onChange={(e) =>
                  setValues((v) => ({ ...v, [f.name]: coerce(e.target.value, f.type) }))
                }
                placeholder={f.description}
                className="mono text-xs bg-bg0 border border-border1 rounded-md px-2 py-1 outline-none focus:border-accent"
              />
            </label>
          ))}
          <button
            onClick={send}
            disabled={busy}
            className="btn btn-primary self-start mt-1"
          >
            <Send className="w-3 h-3" />
            {busy ? "sending…" : "Send"}
          </button>
          {result != null && (
            <div className="border border-border1 rounded-md p-2 bg-bg0 mt-2 max-h-72 overflow-auto">
              <JsonTree value={result} initialDepth={3} />
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface Field {
  name: string;
  type: string | undefined;
  required: boolean;
  description?: string;
  default?: unknown;
}

function collectFields(schema?: SchemaNode): Field[] {
  if (!schema || schema.type !== "object" || !schema.properties) return [];
  const required = new Set(schema.required ?? []);
  return Object.entries(schema.properties).map(([name, node]) => ({
    name,
    type: node.type,
    required: required.has(name),
    description: node.description,
    default: node.default,
  }));
}

function defaultFor(type: string | undefined): unknown {
  if (type === "number" || type === "integer") return 0;
  if (type === "boolean") return false;
  if (type === "array") return [];
  if (type === "object") return {};
  return "";
}

function coerce(v: string, type: string | undefined): unknown {
  if (type === "number" || type === "integer") {
    const n = Number(v);
    return Number.isFinite(n) ? n : v;
  }
  if (type === "boolean") return v === "true";
  if (type === "array" || type === "object") {
    try {
      return JSON.parse(v);
    } catch {
      return v;
    }
  }
  return v;
}
