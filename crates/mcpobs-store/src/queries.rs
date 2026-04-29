//! Read-side queries that the server uses to render the UI. Every query
//! goes through sqlx parameter binding — no string interpolation into SQL.

use crate::schema::Store;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRow {
    pub name: String,
    pub transport: String,
    pub config_json: String,
    pub sessions_today: i64,
    pub errors_today: i64,
    pub p50_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: String,
    pub server_name: String,
    pub transport: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub message_count: i64,
    pub error_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRow {
    pub id: i64,
    pub session_id: String,
    pub server_name: String,
    pub direction: String,
    pub kind: String,
    pub method: Option<String>,
    pub rpc_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub payload_size_bytes: i64,
    pub payload_json: String,
    pub parse_error: Option<String>,
    pub metadata_json: String,
    pub correlated_message_id: Option<i64>,
    pub latency_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPair {
    pub a: MessageRow,
    pub b: MessageRow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceTreeNode {
    pub message: MessageRow,
    pub children: Vec<TraceTreeNode>,
}

fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

fn start_of_today_ms() -> i64 {
    let now = Utc::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    start.timestamp_millis()
}

fn ts_from_ms(ms: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp_millis(ms).unwrap_or_else(Utc::now)
}

pub async fn list_servers_with_latency(store: &Store) -> Result<Vec<ServerRow>> {
    let mut servers = list_servers(store).await?;
    let today_ms = start_of_today_ms();
    for s in servers.iter_mut() {
        let lats: Vec<i64> = sqlx::query_scalar(
            r#"SELECT latency_ms FROM messages
               WHERE server_name = ?1 AND timestamp_ms >= ?2 AND latency_ms IS NOT NULL
               ORDER BY latency_ms ASC"#,
        )
        .bind(&s.name)
        .bind(today_ms)
        .fetch_all(store.pool())
        .await
        .unwrap_or_default();
        if !lats.is_empty() {
            let p50 = lats[lats.len() / 2];
            let p99_idx = ((lats.len() as f64) * 0.99).floor() as usize;
            let p99 = lats[p99_idx.min(lats.len() - 1)];
            s.p50_latency_ms = Some(p50);
            s.p99_latency_ms = Some(p99);
        }
    }
    Ok(servers)
}

pub async fn server_sparkline(
    store: &Store,
    name: &str,
    bucket_count: i64,
    bucket_seconds: i64,
) -> Result<Vec<i64>> {
    let now_ms = Utc::now().timestamp_millis();
    let bucket_ms = bucket_seconds * 1000;
    let window = bucket_count * bucket_ms;
    let start = now_ms - window;
    let raw: Vec<(i64,)> = sqlx::query_as(
        r#"SELECT (timestamp_ms - ?1) / ?2 AS bucket
           FROM messages
           WHERE server_name = ?3 AND timestamp_ms >= ?1 AND timestamp_ms < ?4"#,
    )
    .bind(start)
    .bind(bucket_ms)
    .bind(name)
    .bind(now_ms)
    .fetch_all(store.pool())
    .await?;

    let mut buckets = vec![0i64; bucket_count as usize];
    for (b,) in raw {
        if (0..bucket_count).contains(&b) {
            buckets[b as usize] += 1;
        }
    }
    Ok(buckets)
}

pub async fn list_servers(store: &Store) -> Result<Vec<ServerRow>> {
    let today_ms = start_of_today_ms();
    let rows = sqlx::query(
        r#"
        SELECT
            s.name, s.transport, s.config_json,
            COALESCE((
                SELECT COUNT(*) FROM sessions ses
                WHERE ses.server_name = s.name AND ses.started_at_ms >= ?1
            ), 0) AS sessions_today,
            COALESCE((
                SELECT COUNT(*) FROM messages m
                WHERE m.server_name = s.name AND m.timestamp_ms >= ?1 AND m.kind = 'error'
            ), 0) AS errors_today
        FROM servers s
        ORDER BY s.name
        "#,
    )
    .bind(today_ms)
    .fetch_all(store.pool())
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ServerRow {
            name: r.get("name"),
            transport: r.get("transport"),
            config_json: r.get("config_json"),
            sessions_today: r.get("sessions_today"),
            errors_today: r.get("errors_today"),
            p50_latency_ms: None,
            p99_latency_ms: None,
        })
        .collect())
}

pub async fn upsert_server(
    store: &Store,
    name: &str,
    transport: &str,
    config_json: &str,
) -> Result<()> {
    let now = now_ms();
    sqlx::query(
        r#"
        INSERT INTO servers(name, transport, config_json, created_at_ms, updated_at_ms)
        VALUES (?1, ?2, ?3, ?4, ?4)
        ON CONFLICT(name) DO UPDATE SET
            transport = excluded.transport,
            config_json = excluded.config_json,
            updated_at_ms = excluded.updated_at_ms
        "#,
    )
    .bind(name)
    .bind(transport)
    .bind(config_json)
    .bind(now)
    .execute(store.pool())
    .await?;
    Ok(())
}

pub async fn delete_server(store: &Store, name: &str) -> Result<u64> {
    let res = sqlx::query("DELETE FROM servers WHERE name = ?1")
        .bind(name)
        .execute(store.pool())
        .await?;
    Ok(res.rows_affected())
}

pub async fn list_sessions(
    store: &Store,
    server_name: Option<&str>,
    limit: i64,
) -> Result<Vec<SessionRow>> {
    let rows = if let Some(server) = server_name {
        sqlx::query(
            r#"
            SELECT
                s.id, s.server_name, s.transport, s.started_at_ms, s.ended_at_ms,
                (SELECT COUNT(*) FROM messages m WHERE m.session_id = s.id) AS message_count,
                (SELECT COUNT(*) FROM messages m WHERE m.session_id = s.id AND m.kind = 'error') AS error_count
            FROM sessions s
            WHERE s.server_name = ?1
            ORDER BY s.started_at_ms DESC
            LIMIT ?2
            "#,
        )
        .bind(server)
        .bind(limit)
        .fetch_all(store.pool())
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                s.id, s.server_name, s.transport, s.started_at_ms, s.ended_at_ms,
                (SELECT COUNT(*) FROM messages m WHERE m.session_id = s.id) AS message_count,
                (SELECT COUNT(*) FROM messages m WHERE m.session_id = s.id AND m.kind = 'error') AS error_count
            FROM sessions s
            ORDER BY s.started_at_ms DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(store.pool())
        .await?
    };

    Ok(rows
        .into_iter()
        .map(|r| {
            let started: i64 = r.get("started_at_ms");
            let ended: Option<i64> = r.get("ended_at_ms");
            SessionRow {
                id: r.get("id"),
                server_name: r.get("server_name"),
                transport: r.get("transport"),
                started_at: ts_from_ms(started),
                ended_at: ended.map(ts_from_ms),
                message_count: r.get("message_count"),
                error_count: r.get("error_count"),
            }
        })
        .collect())
}

pub async fn messages_since_id(
    store: &Store,
    after_id: i64,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    let rows = sqlx::query("SELECT * FROM messages WHERE id > ?1 ORDER BY id ASC LIMIT ?2")
        .bind(after_id)
        .bind(limit)
        .fetch_all(store.pool())
        .await?;
    Ok(rows.into_iter().map(map_message).collect())
}

pub async fn list_session_messages(
    store: &Store,
    session_id: &str,
    limit: i64,
    after_id: Option<i64>,
) -> Result<Vec<MessageRow>> {
    let after = after_id.unwrap_or(0);
    let rows = sqlx::query(
        r#"
        SELECT * FROM messages
        WHERE session_id = ?1 AND id > ?2
        ORDER BY id ASC
        LIMIT ?3
        "#,
    )
    .bind(session_id)
    .bind(after)
    .bind(limit)
    .fetch_all(store.pool())
    .await?;
    Ok(rows.into_iter().map(map_message).collect())
}

pub async fn get_message(store: &Store, id: i64) -> Result<Option<MessageRow>> {
    let row = sqlx::query("SELECT * FROM messages WHERE id = ?1")
        .bind(id)
        .fetch_optional(store.pool())
        .await?;
    Ok(row.map(map_message))
}

pub async fn search_messages(
    store: &Store,
    method: Option<&str>,
    since_ms: Option<i64>,
    until_ms: Option<i64>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    let since = since_ms.unwrap_or(0);
    let until = until_ms.unwrap_or(i64::MAX);
    let rows = if let Some(m) = method {
        sqlx::query(
            r#"
            SELECT * FROM messages
            WHERE method = ?1 AND timestamp_ms >= ?2 AND timestamp_ms <= ?3
            ORDER BY timestamp_ms DESC
            LIMIT ?4
            "#,
        )
        .bind(m)
        .bind(since)
        .bind(until)
        .bind(limit)
        .fetch_all(store.pool())
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT * FROM messages
            WHERE timestamp_ms >= ?1 AND timestamp_ms <= ?2
            ORDER BY timestamp_ms DESC
            LIMIT ?3
            "#,
        )
        .bind(since)
        .bind(until)
        .bind(limit)
        .fetch_all(store.pool())
        .await?
    };
    Ok(rows.into_iter().map(map_message).collect())
}

/// Build a trace tree rooted at `root_id`. The tree includes the root, any
/// notifications correlated by `correlated_message_id`, and the
/// response/error if present.
pub async fn get_trace_tree(store: &Store, root_id: i64) -> Result<Option<TraceTreeNode>> {
    let root = match get_message(store, root_id).await? {
        Some(r) => r,
        None => return Ok(None),
    };

    let children =
        sqlx::query("SELECT * FROM messages WHERE correlated_message_id = ?1 ORDER BY id ASC")
            .bind(root_id)
            .fetch_all(store.pool())
            .await?
            .into_iter()
            .map(map_message)
            .map(|m| TraceTreeNode {
                message: m,
                children: vec![],
            })
            .collect();

    Ok(Some(TraceTreeNode {
        message: root,
        children,
    }))
}

/// Delete messages and sessions older than `cutoff_ms`. Returns the number
/// of messages deleted.
pub async fn prune_older_than(store: &Store, cutoff_ms: i64) -> Result<u64> {
    let mut tx = store.pool().begin().await?;
    let msg_res = sqlx::query("DELETE FROM messages WHERE timestamp_ms < ?1")
        .bind(cutoff_ms)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE id NOT IN (SELECT DISTINCT session_id FROM messages)")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(msg_res.rows_affected())
}

fn map_message(r: sqlx::sqlite::SqliteRow) -> MessageRow {
    let ts: i64 = r.get("timestamp_ms");
    MessageRow {
        id: r.get("id"),
        session_id: r.get("session_id"),
        server_name: r.get("server_name"),
        direction: r.get("direction"),
        kind: r.get("kind"),
        method: r.get("method"),
        rpc_id: r.get("rpc_id"),
        timestamp: ts_from_ms(ts),
        payload_size_bytes: r.get("payload_size_bytes"),
        payload_json: r.get("payload_json"),
        parse_error: r.get("parse_error"),
        metadata_json: r.get("metadata_json"),
        correlated_message_id: r.get("correlated_message_id"),
        latency_ms: r.get("latency_ms"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::open;

    async fn setup() -> (Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.db");
        let store = open(&path).await.unwrap();
        (store, dir)
    }

    async fn insert_session(store: &Store, id: &str, server: &str, started_ms: i64) {
        sqlx::query(
            "INSERT INTO sessions(id, server_name, transport, started_at_ms) VALUES (?1, ?2, 'stdio', ?3)",
        )
        .bind(id)
        .bind(server)
        .bind(started_ms)
        .execute(store.pool())
        .await
        .unwrap();
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_message(
        store: &Store,
        session: &str,
        server: &str,
        direction: &str,
        kind: &str,
        method: Option<&str>,
        rpc_id: Option<&str>,
        ts_ms: i64,
    ) -> i64 {
        let res = sqlx::query(
            r#"
            INSERT INTO messages
            (session_id, server_name, direction, kind, method, rpc_id, timestamp_ms,
             payload_size_bytes, payload_json, parse_error, metadata_json,
             correlated_message_id, latency_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, '{}', NULL, '{}', NULL, NULL)
            "#,
        )
        .bind(session)
        .bind(server)
        .bind(direction)
        .bind(kind)
        .bind(method)
        .bind(rpc_id)
        .bind(ts_ms)
        .execute(store.pool())
        .await
        .unwrap();
        res.last_insert_rowid()
    }

    #[tokio::test]
    async fn upsert_and_list_servers() {
        let (store, _dir) = setup().await;
        upsert_server(&store, "fs", "stdio", "{}").await.unwrap();
        upsert_server(&store, "gh", "stdio", "{}").await.unwrap();
        let servers = list_servers(&store).await.unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "fs");
    }

    #[tokio::test]
    async fn sessions_filtered_by_server() {
        let (store, _dir) = setup().await;
        upsert_server(&store, "fs", "stdio", "{}").await.unwrap();
        upsert_server(&store, "gh", "stdio", "{}").await.unwrap();
        insert_session(&store, "s1", "fs", 100).await;
        insert_session(&store, "s2", "gh", 200).await;
        insert_session(&store, "s3", "fs", 300).await;

        let fs_sessions = list_sessions(&store, Some("fs"), 100).await.unwrap();
        assert_eq!(fs_sessions.len(), 2);

        let all = list_sessions(&store, None, 100).await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn message_roundtrip_and_search() {
        let (store, _dir) = setup().await;
        upsert_server(&store, "fs", "stdio", "{}").await.unwrap();
        insert_session(&store, "s1", "fs", 100).await;
        let id = insert_message(
            &store,
            "s1",
            "fs",
            "c2s",
            "request",
            Some("tools/call"),
            Some("1"),
            500,
        )
        .await;

        let m = get_message(&store, id).await.unwrap().unwrap();
        assert_eq!(m.method.as_deref(), Some("tools/call"));

        let results = search_messages(&store, Some("tools/call"), None, None, 10)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn prune_drops_old_messages() {
        let (store, _dir) = setup().await;
        upsert_server(&store, "fs", "stdio", "{}").await.unwrap();
        insert_session(&store, "old", "fs", 100).await;
        insert_session(&store, "new", "fs", 10_000).await;
        insert_message(
            &store,
            "old",
            "fs",
            "c2s",
            "request",
            Some("a"),
            Some("1"),
            100,
        )
        .await;
        insert_message(
            &store,
            "new",
            "fs",
            "c2s",
            "request",
            Some("a"),
            Some("2"),
            10_000,
        )
        .await;

        let deleted = prune_older_than(&store, 5_000).await.unwrap();
        assert_eq!(deleted, 1);

        let remaining: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(remaining.0, 1);
    }

    #[tokio::test]
    async fn sparkline_buckets_count_messages() {
        let (store, _dir) = setup().await;
        upsert_server(&store, "fs", "stdio", "{}").await.unwrap();
        insert_session(&store, "s", "fs", 0).await;
        let now_ms = Utc::now().timestamp_millis();
        // 5 messages within the last 60 seconds.
        for i in 0..5 {
            insert_message(
                &store,
                "s",
                "fs",
                "c2s",
                "request",
                Some("tools/list"),
                Some(&i.to_string()),
                now_ms - 1000 - (i * 10),
            )
            .await;
        }
        let buckets = server_sparkline(&store, "fs", 60, 60).await.unwrap();
        assert_eq!(buckets.len(), 60);
        assert!(buckets.iter().sum::<i64>() == 5);
    }

    #[tokio::test]
    async fn list_servers_with_latency_computes_percentiles() {
        let (store, _dir) = setup().await;
        upsert_server(&store, "fs", "stdio", "{}").await.unwrap();
        insert_session(&store, "s", "fs", 0).await;
        let now_ms = Utc::now().timestamp_millis();
        for lat in [1, 2, 3, 4, 5, 10, 100] {
            sqlx::query(
                r#"INSERT INTO messages
                   (session_id, server_name, direction, kind, method, rpc_id, timestamp_ms,
                    payload_size_bytes, payload_json, metadata_json, latency_ms)
                   VALUES ('s', 'fs', 's2c', 'response', NULL, '1', ?1, 0, '{}', '{}', ?2)"#,
            )
            .bind(now_ms)
            .bind(lat as i64)
            .execute(store.pool())
            .await
            .unwrap();
        }
        let rows = list_servers_with_latency(&store).await.unwrap();
        let fs = rows.iter().find(|r| r.name == "fs").unwrap();
        assert!(fs.p50_latency_ms.is_some());
        assert!(fs.p99_latency_ms.is_some());
        assert!(fs.p99_latency_ms.unwrap() >= fs.p50_latency_ms.unwrap());
    }

    #[tokio::test]
    async fn messages_since_id_returns_only_newer() {
        let (store, _dir) = setup().await;
        upsert_server(&store, "fs", "stdio", "{}").await.unwrap();
        insert_session(&store, "s", "fs", 0).await;
        let id1 = insert_message(
            &store,
            "s",
            "fs",
            "c2s",
            "request",
            Some("a"),
            Some("1"),
            100,
        )
        .await;
        let id2 = insert_message(
            &store,
            "s",
            "fs",
            "c2s",
            "request",
            Some("b"),
            Some("2"),
            200,
        )
        .await;
        let rows = messages_since_id(&store, id1, 100).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, id2);
    }

    #[tokio::test]
    async fn ten_thousand_inserts_and_query_under_100ms() {
        let (store, _dir) = setup().await;
        upsert_server(&store, "fs", "stdio", "{}").await.unwrap();
        insert_session(&store, "big", "fs", 0).await;

        // Bulk insert via a single transaction so we're not measuring per-row commits.
        let mut tx = store.pool().begin().await.unwrap();
        for i in 0..10_000 {
            sqlx::query(
                r#"INSERT INTO messages
                   (session_id, server_name, direction, kind, method, rpc_id, timestamp_ms,
                    payload_size_bytes, payload_json, metadata_json)
                   VALUES (?1, 'fs', 'c2s', 'request', 'tools/call', ?2, ?3, 0, '{}', '{}')"#,
            )
            .bind("big")
            .bind(i.to_string())
            .bind(i)
            .execute(&mut *tx)
            .await
            .unwrap();
        }
        tx.commit().await.unwrap();

        let started = std::time::Instant::now();
        let results = search_messages(&store, Some("tools/call"), None, None, 100)
            .await
            .unwrap();
        let elapsed = started.elapsed();
        assert_eq!(results.len(), 100);
        assert!(
            elapsed.as_millis() < 100,
            "search took {}ms, expected <100ms",
            elapsed.as_millis()
        );
    }
}
