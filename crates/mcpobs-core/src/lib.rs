//! mcpobs-core
//!
//! Protocol parsing, transport bridges, and the proxy loop. No I/O on storage
//! or HTTP server — those live in `mcpobs-store` and `mcpobs-server`.

pub mod observation;
pub mod protocol;
pub mod proxy;
pub mod session;
pub mod transport;

pub use observation::{Direction, Observation, ObservationKind, ObservationSink};
pub use protocol::jsonrpc::{JsonRpcId, JsonRpcMessage, ParsedMessage};
pub use session::{SessionId, SessionMeta};

/// MCP spec revision this build is pinned against.
pub const MCP_SPEC_REVISION: &str = "2025-06-18";
