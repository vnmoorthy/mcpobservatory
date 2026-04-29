//! stdio transport.
//!
//! The proxy:
//! 1. Reads JSON-RPC lines from its own stdin.
//! 2. Spawns the upstream as a child with piped stdin/stdout/stderr.
//! 3. Forwards every input line to upstream stdin and emits a c2s observation.
//! 4. Forwards every upstream stdout line to its own stdout and emits a s2c observation.
//! 5. Pipes upstream stderr unchanged so the client sees it.
//! 6. On client EOF: SIGTERM upstream, wait 5s, SIGKILL if still alive.
//! 7. On upstream exit: log status, EOF on own stdout, exit cleanly.

use crate::observation::{Direction, Observation, ObservationKind, ObservationSink};
use crate::protocol::jsonrpc::ParsedMessage;
use crate::session::{SessionId, SessionMeta, TransportKind};
use crate::transport::MAX_LINE_BYTES;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct StdioUpstream {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
}

pub struct StdioProxy {
    upstream: StdioUpstream,
    server_name: String,
    sink: ObservationSink,
}

impl StdioProxy {
    pub fn new(server_name: String, upstream: StdioUpstream, sink: ObservationSink) -> Self {
        Self {
            upstream,
            server_name,
            sink,
        }
    }

    /// Run the proxy until either side closes. Generic over reader/writer so
    /// tests can drive it with in-memory pipes.
    pub async fn run<R, W, E>(
        self,
        client_in: R,
        client_out: W,
        client_err: E,
    ) -> Result<StdioExitStatus>
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
        E: AsyncWrite + Unpin + Send + 'static,
    {
        let session = SessionMeta {
            id: SessionId::new(),
            server_name: self.server_name.clone(),
            transport: TransportKind::Stdio,
            started_at: Utc::now(),
            client_hint: None,
        };

        let mut cmd = Command::new(&self.upstream.command);
        cmd.args(&self.upstream.args)
            .envs(&self.upstream.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(cwd) = &self.upstream.cwd {
            cmd.current_dir(cwd);
        }
        let mut child = cmd
            .spawn()
            .with_context(|| format!("spawn upstream `{}`", self.upstream.command))?;

        let upstream_stdin = child
            .stdin
            .take()
            .context("upstream stdin missing after spawn")?;
        let upstream_stdout = child
            .stdout
            .take()
            .context("upstream stdout missing after spawn")?;
        let upstream_stderr = child
            .stderr
            .take()
            .context("upstream stderr missing after spawn")?;

        // c2s pump
        let mut c2s = tokio::spawn(pump(
            BufReader::new(client_in),
            upstream_stdin,
            session.clone(),
            self.sink.clone(),
            Direction::C2s,
        ));

        // s2c pump
        let mut s2c = tokio::spawn(pump(
            BufReader::new(upstream_stdout),
            client_out,
            session.clone(),
            self.sink.clone(),
            Direction::S2c,
        ));

        // stderr passthrough — no observation, just bytes.
        let stderr = tokio::spawn(passthrough(upstream_stderr, client_err));

        let child = Arc::new(Mutex::new(child));
        let child_for_kill = child.clone();

        // Whichever pump finishes first dictates teardown. When c2s drains
        // (client EOF), the upstream's stdin is closed by `writer.shutdown`
        // inside the pump, so the upstream should exit naturally and s2c
        // will then see EOF on the upstream's stdout. When s2c drains
        // (upstream exited), there is no point reading further from the
        // client — abort c2s.
        let exit = tokio::select! {
            r = &mut c2s => {
                tracing::debug!(direction = "c2s", "pump finished");
                let _ = r;
                let _ = tokio::time::timeout(Duration::from_secs(5), &mut s2c).await;
                shutdown_upstream(&child_for_kill).await
            }
            r = &mut s2c => {
                tracing::debug!(direction = "s2c", "pump finished");
                let _ = r;
                c2s.abort();
                shutdown_upstream(&child_for_kill).await
            }
        };

        let _ = tokio::time::timeout(Duration::from_millis(200), stderr).await;

        Ok(StdioExitStatus {
            session_id: session.id,
            upstream_exit_code: exit,
        })
    }
}

async fn pump<R, W>(
    mut reader: R,
    mut writer: W,
    session: SessionMeta,
    sink: ObservationSink,
    direction: Direction,
) -> Result<()>
where
    R: AsyncBufReadExt + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = Vec::with_capacity(4096);
    loop {
        // If the previous iteration grew the buffer pathologically, drop
        // the allocation back to a sane size between lines.
        if buf.capacity() > MAX_LINE_BYTES * 2 {
            buf = Vec::with_capacity(4096);
        } else {
            buf.clear();
        }
        let n = reader.read_until(b'\n', &mut buf).await?;
        if n == 0 {
            // EOF
            let _ = writer.shutdown().await;
            return Ok(());
        }

        // Forward verbatim before parsing — the proxy must be transparent.
        // If the write fails, the other side is gone, so we exit.
        if let Err(e) = writer.write_all(&buf).await {
            tracing::debug!(error = %e, "downstream write failed, exiting pump");
            return Ok(());
        }
        let _ = writer.flush().await;

        // Strip trailing newline(s) for parse.
        let trimmed_end = buf
            .iter()
            .rposition(|b| *b != b'\n' && *b != b'\r')
            .map(|i| i + 1)
            .unwrap_or(0);

        // If the line is huge, skip parse but still emit a stub
        // observation so the user sees that something happened. This
        // bounds memory while keeping the proxy transparent on the wire.
        if trimmed_end > MAX_LINE_BYTES {
            let obs = Observation {
                session_id: session.id.clone(),
                server_name: session.server_name.clone(),
                direction,
                kind: ObservationKind::Unparsed,
                method: None,
                rpc_id: None,
                timestamp: Utc::now(),
                payload_size_bytes: trimmed_end as u64,
                payload_json: serde_json::Value::Null,
                parse_error: Some(format!(
                    "line exceeds MAX_LINE_BYTES ({} > {}); not parsed",
                    trimmed_end, MAX_LINE_BYTES
                )),
                metadata: serde_json::json!({"oversize": true}),
            };
            if sink.try_send(obs).is_err() {
                tracing::warn!("oversize observation dropped");
            }
            continue;
        }

        let to_parse = buf[..trimmed_end].to_vec();
        if to_parse.is_empty() {
            continue;
        }
        let size = to_parse.len() as u64;
        let parsed = ParsedMessage::parse(to_parse);
        let kind = ObservationKind::from(&parsed);
        let method = parsed.method().map(|s| s.to_string());
        let rpc_id = parsed.id().map(|i| i.as_string());
        let payload_json = parsed
            .parsed
            .as_ref()
            .and_then(|m| serde_json::to_value(m).ok())
            .unwrap_or(serde_json::Value::Null);

        let obs = Observation {
            session_id: session.id.clone(),
            server_name: session.server_name.clone(),
            direction,
            kind,
            method,
            rpc_id,
            timestamp: Utc::now(),
            payload_size_bytes: size,
            payload_json,
            parse_error: parsed.parse_error.clone(),
            metadata: serde_json::json!({}),
        };

        if let Err(_dropped) = sink.try_send(obs) {
            tracing::warn!("observation channel full; dropping");
        }
    }
}

async fn passthrough<R, W>(mut reader: R, mut writer: W) -> Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    use tokio::io::AsyncReadExt;
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            let _ = writer.shutdown().await;
            return Ok(());
        }
        if writer.write_all(&buf[..n]).await.is_err() {
            return Ok(());
        }
        let _ = writer.flush().await;
    }
}

async fn shutdown_upstream(child: &Arc<Mutex<Child>>) -> Option<i32> {
    let mut guard = child.lock().await;
    let pid = guard.id();
    if let Some(pid) = pid {
        send_sigterm(pid);
    }

    match tokio::time::timeout(Duration::from_secs(5), guard.wait()).await {
        Ok(Ok(status)) => status.code(),
        Ok(Err(_)) => None,
        Err(_) => {
            let _ = guard.kill().await;
            guard.wait().await.ok().and_then(|s| s.code())
        }
    }
}

#[cfg(unix)]
fn send_sigterm(pid: u32) {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
}

#[cfg(not(unix))]
fn send_sigterm(_pid: u32) {
    // Windows: tokio's `Child::kill` (TerminateProcess) is the only option.
    // We rely on the timeout branch in shutdown_upstream for that.
}

#[derive(Debug, Clone)]
pub struct StdioExitStatus {
    pub session_id: SessionId,
    pub upstream_exit_code: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn echo_upstream_round_trip() {
        // Use `cat` as a trivial echo upstream on Unix.
        if cfg!(windows) {
            return;
        }
        let upstream = StdioUpstream {
            command: "cat".into(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
        };
        let (sink, mut rx) = ObservationSink::new(64);
        let proxy = StdioProxy::new("echo".into(), upstream, sink);

        let (client_in_r, mut client_in_w) = tokio::io::duplex(1024);
        let (client_out_r, client_out_w) = tokio::io::duplex(1024);
        let (_client_err_r, client_err_w) = tokio::io::duplex(1024);

        let h = tokio::spawn(async move {
            proxy
                .run(client_in_r, client_out_w, client_err_w)
                .await
                .unwrap()
        });

        let line = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
        client_in_w.write_all(line).await.unwrap();
        client_in_w.flush().await.unwrap();
        drop(client_in_w);

        let mut out = Vec::new();
        let mut reader = client_out_r;
        reader.read_to_end(&mut out).await.unwrap();

        assert_eq!(&out[..], &line[..]);

        let mut got = Vec::new();
        while let Ok(obs) = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
            match obs {
                Some(o) => got.push(o),
                None => break,
            }
        }
        assert!(got
            .iter()
            .any(|o| o.direction == Direction::C2s && o.method.as_deref() == Some("ping")));
        assert!(got.iter().any(|o| o.direction == Direction::S2c));

        let _ = h.await;
    }

    #[tokio::test]
    async fn oversize_line_is_marked_unparsed_but_forwarded() {
        if cfg!(windows) {
            return;
        }
        let upstream = StdioUpstream {
            command: "cat".into(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
        };
        let (sink, mut rx) = ObservationSink::new(64);
        let proxy = StdioProxy::new("echo".into(), upstream, sink);

        let (client_in_r, mut client_in_w) = tokio::io::duplex(32 * 1024 * 1024);
        let (client_out_r, client_out_w) = tokio::io::duplex(32 * 1024 * 1024);
        let (_client_err_r, client_err_w) = tokio::io::duplex(1024);

        let h = tokio::spawn(async move {
            proxy
                .run(client_in_r, client_out_w, client_err_w)
                .await
                .unwrap()
        });

        // 17MB of 'a' followed by newline — exceeds MAX_LINE_BYTES (16MB).
        let big = vec![b'a'; super::super::MAX_LINE_BYTES + 1];
        client_in_w.write_all(&big).await.unwrap();
        client_in_w.write_all(b"\n").await.unwrap();
        drop(client_in_w);

        // The huge line is still forwarded (transparency requirement).
        let mut out = Vec::new();
        let mut reader = client_out_r;
        let _ = reader.read_to_end(&mut out).await;
        assert!(out.len() > super::super::MAX_LINE_BYTES);

        // The observation should be Unparsed with parse_error mentioning size.
        let mut saw_oversize = false;
        while let Ok(Some(obs)) = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
        {
            if obs
                .parse_error
                .as_deref()
                .map(|e| e.contains("MAX_LINE_BYTES"))
                .unwrap_or(false)
            {
                saw_oversize = true;
                assert_eq!(obs.kind, ObservationKind::Unparsed);
            }
        }
        assert!(saw_oversize, "expected an oversize observation");

        let _ = h.await;
    }

    #[tokio::test]
    async fn malformed_line_is_forwarded() {
        if cfg!(windows) {
            return;
        }
        let upstream = StdioUpstream {
            command: "cat".into(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
        };
        let (sink, mut rx) = ObservationSink::new(64);
        let proxy = StdioProxy::new("echo".into(), upstream, sink);

        let (client_in_r, mut client_in_w) = tokio::io::duplex(1024);
        let (client_out_r, client_out_w) = tokio::io::duplex(1024);
        let (_client_err_r, client_err_w) = tokio::io::duplex(1024);

        let h = tokio::spawn(async move {
            proxy
                .run(client_in_r, client_out_w, client_err_w)
                .await
                .unwrap()
        });

        let bad = b"{not json\n";
        client_in_w.write_all(bad).await.unwrap();
        drop(client_in_w);

        let mut out = Vec::new();
        let mut reader = client_out_r;
        let _ = reader.read_to_end(&mut out).await;
        assert_eq!(&out[..bad.len()], &bad[..]);

        let mut saw_parse_err = false;
        while let Ok(Some(obs)) = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
        {
            if obs.parse_error.is_some() {
                saw_parse_err = true;
            }
        }
        assert!(
            saw_parse_err,
            "expected at least one observation with parse_error"
        );

        let _ = h.await;
    }
}
