//! mcpobs-store
//!
//! SQLite persistence for observations. Single writer, many readers, WAL
//! mode. The writer task drains an mpsc::Receiver<Observation> from the
//! proxy.

pub mod queries;
pub mod redact;
pub mod schema;
pub mod writer;

pub use queries::{DiffPair, MessageRow, ServerRow, SessionRow, TraceTreeNode};
pub use redact::{redact_value, RedactionConfig};
pub use schema::{open, Store};
pub use writer::{spawn_writer, spawn_writer_with_sink, LiveSink, NullLiveSink, WriterHandle};
