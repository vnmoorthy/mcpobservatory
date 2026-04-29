//! High-level proxy entry points used by the CLI.

use crate::observation::ObservationSink;
use crate::transport::http::{HttpForwarder, HttpUpstream};
use crate::transport::sse::{SseSubscriber, SseUpstream};
use crate::transport::stdio::{StdioExitStatus, StdioProxy, StdioUpstream};
use anyhow::Result;

/// Run a stdio proxy bridging this process's stdin/stdout/stderr to the
/// configured upstream child.
pub async fn run_stdio_proxy(
    server_name: String,
    upstream: StdioUpstream,
    sink: ObservationSink,
) -> Result<StdioExitStatus> {
    let proxy = StdioProxy::new(server_name, upstream, sink);
    proxy
        .run(tokio::io::stdin(), tokio::io::stdout(), tokio::io::stderr())
        .await
}

/// Build an HTTP forwarder that the daemon mounts behind a listen path.
pub fn build_http_forwarder(
    server_name: String,
    upstream: HttpUpstream,
    sink: ObservationSink,
) -> Result<HttpForwarder> {
    HttpForwarder::new(server_name, upstream, sink)
}

/// Build an SSE subscriber. The daemon spawns one of these per configured
/// SSE upstream.
pub fn build_sse_subscriber(
    server_name: String,
    upstream: SseUpstream,
    sink: ObservationSink,
) -> Result<SseSubscriber> {
    SseSubscriber::new(server_name, upstream, sink)
}
