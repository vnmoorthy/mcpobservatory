import { useEffect, useRef, useState } from "react";

export interface LiveEvent {
  event: string;
  data: unknown;
}

export function useLiveTail(enabled: boolean) {
  const [events, setEvents] = useState<LiveEvent[]>([]);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (!enabled) return;
    const url = `${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/ws/live`;
    const ws = new WebSocket(url);
    wsRef.current = ws;
    ws.onmessage = (msg) => {
      try {
        const ev = JSON.parse(msg.data) as LiveEvent;
        setEvents((prev) => {
          const next = prev.concat(ev);
          return next.length > 500 ? next.slice(-500) : next;
        });
      } catch {
        // ignore
      }
    };
    return () => {
      ws.close();
    };
  }, [enabled]);

  return events;
}
