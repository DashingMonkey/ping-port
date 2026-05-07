/// Database schema definitions for PingPort
/// Create collections table
pub const CREATE_COLLECTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT,
    parent_id TEXT,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES collections(id) ON DELETE CASCADE
)
"#;

/// Create requests table
pub const CREATE_REQUESTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS requests (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL,
    name TEXT NOT NULL,
    method TEXT NOT NULL DEFAULT 'GET',
    url TEXT,
    params TEXT,
    headers TEXT,
    body TEXT,
    auth TEXT,
    pre_request_script TEXT,
    test_script TEXT,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
)
"#;

/// Create environments table
pub const CREATE_ENVIRONMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS environments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    variables TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)
"#;
