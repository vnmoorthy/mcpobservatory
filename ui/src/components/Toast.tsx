import { createContext, ReactNode, useCallback, useContext, useEffect, useState } from "react";

type ToastTone = "neutral" | "ok" | "warn" | "err";

interface Toast {
  id: number;
  text: string;
  tone: ToastTone;
}

interface Ctx {
  push: (text: string, tone?: ToastTone) => void;
}

const ToastCtx = createContext<Ctx>({ push: () => {} });

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const push = useCallback((text: string, tone: ToastTone = "neutral") => {
    const id = Date.now() + Math.random();
    setToasts((t) => [...t, { id, text, tone }]);
    setTimeout(() => {
      setToasts((t) => t.filter((x) => x.id !== id));
    }, 3500);
  }, []);

  return (
    <ToastCtx.Provider value={{ push }}>
      {children}
      <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={`mono text-sm border rounded px-3 py-2 shadow ${
              t.tone === "ok"
                ? "bg-accent-soft text-accent border-accent/30"
                : t.tone === "err"
                ? "bg-err-soft text-err border-err/30"
                : t.tone === "warn"
                ? "bg-warn-soft text-warn border-warn/30"
                : "bg-bg2 text-fg0 border-border1"
            }`}
          >
            {t.text}
          </div>
        ))}
      </div>
    </ToastCtx.Provider>
  );
}

export function useToast() {
  return useContext(ToastCtx);
}

/// Lets non-React code (like a context menu helper) trigger a toast.
let globalPush: ((text: string, tone?: ToastTone) => void) | null = null;
export function ToastBridge() {
  const { push } = useToast();
  useEffect(() => {
    globalPush = push;
    return () => {
      globalPush = null;
    };
  }, [push]);
  return null;
}
export function toast(text: string, tone?: ToastTone) {
  globalPush?.(text, tone);
}
