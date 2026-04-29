import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useSearchParams } from "react-router-dom";
import { Repeat, AlertTriangle } from "lucide-react";
import { api } from "../lib/api";
import { PageHeader } from "../components/PageHeader";
import { JsonTree } from "../components/JsonTree";
import { Badge, MethodBadge } from "../components/Badge";
import { toast } from "../components/Toast";

export function ReplayPanel() {
  const [params] = useSearchParams();
  const of = Number(params.get("of"));
  const [confirmed, setConfirmed] = useState(false);
  const [result, setResult] = useState<unknown>(null);
  const [pending, setPending] = useState(false);
  const [overrideText, setOverrideText] = useState("");

  const original = useQuery({
    queryKey: ["message", of],
    queryFn: () => api.message(of),
    enabled: !!of,
  });

  const isSafe =
    original.data?.method != null &&
    /\/(list|get|read)$|^ping$/.test(original.data.method);

  async function send() {
    setPending(true);
    setResult(null);
    try {
      let overrideParams: unknown = undefined;
      if (overrideText.trim()) {
        try {
          overrideParams = JSON.parse(overrideText);
        } catch (e) {
          toast(`override params: ${(e as Error).message}`, "err");
          setPending(false);
          return;
        }
      }
      const r = await api.replay(of, !isSafe ? confirmed : true, overrideParams);
      setResult(r);
      toast(r.status === "ok" ? "replay sent" : `replay: ${r.status}`, r.status === "ok" ? "ok" : "warn");
    } catch (e) {
      toast(`replay failed: ${(e as Error).message}`, "err");
    } finally {
      setPending(false);
    }
  }

  if (!of) {
    return (
      <div className="p-6 text-fg2">
        missing <code className="mono">of</code> query param.
      </div>
    );
  }

  return (
    <div>
      <PageHeader
        crumbs={[{ label: "Home", to: "/" }, { label: "Replay" }]}
        title={
          <span className="flex items-center gap-2">
            <Repeat className="w-5 h-5 text-accent" />
            <span>Replay</span>
            <span className="mono text-fg2 text-base">#{of}</span>
          </span>
        }
        subtitle="Re-issue a captured request against the upstream."
      />
      <div className="p-6 grid grid-cols-1 lg:grid-cols-2 gap-4">
        <section className="card p-4 flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <h3 className="text-fg0 font-medium text-sm">Original</h3>
            {original.data && <MethodBadge method={original.data.method} />}
          </div>
          {original.isPending ? (
            <div className="text-fg2 text-sm">loading…</div>
          ) : original.data ? (
            <div className="border border-border1 rounded-md p-3 bg-bg0 max-h-[420px] overflow-auto">
              <JsonTree
                value={(() => {
                  try {
                    return JSON.parse(original.data.payload_json);
                  } catch {
                    return original.data.payload_json;
                  }
                })()}
              />
            </div>
          ) : (
            <div className="text-err text-sm">message not found</div>
          )}

          <div className="flex flex-col gap-2 pt-2 border-t border-border1">
            <label className="text-xs text-fg1">
              Override <span className="mono">params</span> (optional JSON)
            </label>
            <textarea
              value={overrideText}
              onChange={(e) => setOverrideText(e.target.value)}
              rows={4}
              placeholder='e.g. {"path":"/tmp/other.txt"}'
              className="mono text-xs bg-bg0 border border-border1 rounded-md p-2 outline-none focus:border-accent"
            />
            {!isSafe && (
              <label className="flex items-center gap-2 text-warn text-sm">
                <AlertTriangle className="w-3.5 h-3.5" />
                <input
                  type="checkbox"
                  checked={confirmed}
                  onChange={(e) => setConfirmed(e.target.checked)}
                />
                I understand this method may have side effects.
              </label>
            )}
            <button
              onClick={send}
              disabled={pending || (!isSafe && !confirmed)}
              className="btn btn-primary self-start"
            >
              <Repeat className="w-3.5 h-3.5" />
              {pending ? "sending…" : "Replay"}
            </button>
          </div>
        </section>

        <section className="card p-4 flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <h3 className="text-fg0 font-medium text-sm">Replay result</h3>
            {result != null && (
              <Badge
                tone={
                  (result as { status?: string }).status === "ok"
                    ? "accent"
                    : (result as { status?: string }).status === "needs_confirmation"
                    ? "warn"
                    : "err"
                }
              >
                {(result as { status?: string }).status ?? "—"}
              </Badge>
            )}
          </div>
          {result == null ? (
            <div className="text-fg2 text-sm py-12 text-center">
              run a replay to see the response here.
            </div>
          ) : (
            <div className="border border-border1 rounded-md p-3 bg-bg0 max-h-[420px] overflow-auto">
              <JsonTree value={result} />
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
