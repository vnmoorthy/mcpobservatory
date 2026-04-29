-- mcpobs storage schema, revision 0001.
--
-- We use TEXT for payloads so the JSON1 functions work, and a denormalised
-- payload_size_bytes so per-session totals do not require touching the
-- payload column.

CREATE TABLE IF NOT EXISTS servers (
    name             TEXT PRIMARY KEY,
    transport        TEXT NOT NULL,
    config_json      TEXT NOT NULL,
    created_at_ms    INTEGER NOT NULL,
    updated_at_ms    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id               TEXT PRIMARY KEY,
    server_name      TEXT NOT NULL,
    transport        TEXT NOT NULL,
    started_at_ms    INTEGER NOT NULL,
    ended_at_ms      INTEGER,
    client_hint      TEXT,
    FOREIGN KEY (server_name) REFERENCES servers(name)
);

CREATE INDEX IF NOT EXISTS sessions_started_idx ON sessions(started_at_ms DESC);
CREATE INDEX IF NOT EXISTS sessions_server_idx  ON sessions(server_name, started_at_ms DESC);

CREATE TABLE IF NOT EXISTS messages (
    id                       INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id               TEXT NOT NULL,
    server_name              TEXT NOT NULL,
    direction                TEXT NOT NULL CHECK (direction IN ('c2s','s2c')),
    kind                     TEXT NOT NULL,
    method                   TEXT,
    rpc_id                   TEXT,
    timestamp_ms             INTEGER NOT NULL,
    payload_size_bytes       INTEGER NOT NULL,
    payload_json             TEXT NOT NULL,
    parse_error              TEXT,
    metadata_json            TEXT NOT NULL DEFAULT '{}',
    correlated_message_id    INTEGER,
    latency_ms               INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE INDEX IF NOT EXISTS messages_session_idx   ON messages(session_id, id);
CREATE INDEX IF NOT EXISTS messages_timestamp_idx ON messages(timestamp_ms DESC);
CREATE INDEX IF NOT EXISTS messages_method_idx    ON messages(method, timestamp_ms DESC);
CREATE INDEX IF NOT EXISTS messages_rpc_id_idx    ON messages(session_id, rpc_id);

CREATE TABLE IF NOT EXISTS schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT OR REPLACE INTO schema_meta(key, value) VALUES ('schema_version', '1');
