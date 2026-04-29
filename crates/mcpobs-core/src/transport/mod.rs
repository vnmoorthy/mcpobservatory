pub mod http;
pub mod sse;
pub mod stdio;

/// Maximum line length we will accept on a stdio transport. Lines longer
/// than this are still forwarded verbatim, but the parser short-circuits to
/// "unparsed" rather than allocating arbitrary memory.
pub const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;
