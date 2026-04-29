//! Live tail bus. The writer fan-outs each persisted message to any
//! connected `/ws/live` subscriber.

use mcpobs_store::writer::LiveSink;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEvent {
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct LiveBus {
    tx: broadcast::Sender<LiveEvent>,
}

impl LiveBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LiveEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, ev: LiveEvent) {
        let _ = self.tx.send(ev);
    }
}

impl LiveSink for LiveBus {
    fn publish(&self, payload: serde_json::Value) {
        let event = payload
            .get("event")
            .and_then(|v| v.as_str())
            .unwrap_or("message")
            .to_string();
        let data = payload.get("data").cloned().unwrap_or(payload);
        self.publish(LiveEvent { event, data });
    }
}
