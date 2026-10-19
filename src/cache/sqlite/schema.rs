pub(super) const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS cache_meta (
    id INTEGER PRIMARY KEY CHECK (id = 1), bytes INTEGER NOT NULL DEFAULT 0,
    optional_bytes INTEGER NOT NULL DEFAULT 0,
    last_clean INTEGER NOT NULL DEFAULT 0
);
INSERT OR IGNORE INTO cache_meta(id) VALUES(1);
CREATE TABLE IF NOT EXISTS blob_gc (digest TEXT PRIMARY KEY);
CREATE TABLE IF NOT EXISTS entries (
    namespace TEXT NOT NULL, key TEXT NOT NULL, base_key TEXT NOT NULL,
    metadata BLOB NOT NULL, body BLOB, blob TEXT, body_hash TEXT NOT NULL DEFAULT '', bytes INTEGER NOT NULL CHECK(bytes>=0),
    created INTEGER NOT NULL, last_access INTEGER NOT NULL, expires INTEGER,
    PRIMARY KEY(namespace, key),
    CHECK ((body IS NULL) != (blob IS NULL))
);
CREATE INDEX IF NOT EXISTS entries_lookup ON entries(namespace, base_key);
CREATE INDEX IF NOT EXISTS entries_lru ON entries(last_access,namespace,key);
CREATE INDEX IF NOT EXISTS entries_namespace_lru ON entries(namespace,last_access,key);
CREATE INDEX IF NOT EXISTS entries_age ON entries(created);
CREATE INDEX IF NOT EXISTS entries_expiry ON entries(expires);
CREATE INDEX IF NOT EXISTS entries_blob ON entries(blob);
CREATE TRIGGER IF NOT EXISTS entries_insert AFTER INSERT ON entries BEGIN
    UPDATE cache_meta SET bytes = bytes + NEW.bytes, optional_bytes = optional_bytes + CASE WHEN NEW.namespace != 'capture' THEN NEW.bytes ELSE 0 END WHERE id = 1;
END;
CREATE TRIGGER IF NOT EXISTS entries_delete AFTER DELETE ON entries BEGIN
    UPDATE cache_meta SET bytes = bytes - OLD.bytes, optional_bytes = optional_bytes - CASE WHEN OLD.namespace != 'capture' THEN OLD.bytes ELSE 0 END WHERE id = 1;
END;
CREATE TRIGGER IF NOT EXISTS entries_update AFTER UPDATE OF bytes ON entries BEGIN
    UPDATE cache_meta SET bytes = bytes + NEW.bytes - OLD.bytes, optional_bytes = optional_bytes + CASE WHEN NEW.namespace != 'capture' THEN NEW.bytes - OLD.bytes ELSE 0 END WHERE id = 1;
END;
PRAGMA user_version = 1;
"#;
