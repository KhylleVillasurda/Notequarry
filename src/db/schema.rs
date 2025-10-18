// src/db/schema.rs

use log::{info, warn};
use rusqlite::{Connection, Result};

/// Current schema version
const CURRENT_VERSION: i32 = 1;

/// Initialize database schema
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Get current version
    let version = get_schema_version(conn)?;

    if version == 0 {
        info!("Creating new database schema...");
        create_initial_schema(conn)?;
        set_schema_version(conn, CURRENT_VERSION)?;
        info!("Database schema created successfully");
    } else if version < CURRENT_VERSION {
        info!(
            "Migrating database from version {} to {}",
            version, CURRENT_VERSION
        );
        migrate_schema(conn, version)?;
        info!("Database migration completed");
    } else if version > CURRENT_VERSION {
        warn!(
            "Database version {} is newer than application version {}",
            version, CURRENT_VERSION
        );
    }

    Ok(())
}

/// Get current schema version
fn get_schema_version(conn: &Connection) -> Result<i32> {
    match conn.query_row("PRAGMA user_version", [], |row| row.get(0)) {
        Ok(version) => Ok(version),
        Err(_) => Ok(0),
    }
}

/// Set schema version
fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(&format!("PRAGMA user_version = {}", version), [])?;
    Ok(())
}

/// Create initial database schema (Version 1)
fn create_initial_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        BEGIN;

        -- Core entry table
        CREATE TABLE entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            mode TEXT NOT NULL CHECK(mode IN ('BOOK', 'NOTE')),
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            tags TEXT,
            encryption_key_salt BLOB NOT NULL,
            is_encrypted INTEGER DEFAULT 1
        );

        -- Book mode pages
        CREATE TABLE pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL,
            page_number INTEGER NOT NULL,
            content_encrypted BLOB NOT NULL,
            word_count INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            UNIQUE(entry_id, page_number)
        );

        -- Note mode content
        CREATE TABLE notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL UNIQUE,
            content_encrypted BLOB NOT NULL,
            has_checkboxes INTEGER DEFAULT 0,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
        );

        -- Checkboxes for Note mode
        CREATE TABLE checkboxes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note_id INTEGER NOT NULL,
            text TEXT NOT NULL,
            is_checked INTEGER DEFAULT 0,
            position INTEGER NOT NULL,
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );

        -- Image metadata
        CREATE TABLE images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL,
            page_id INTEGER,
            file_path TEXT NOT NULL,
            thumbnail_path TEXT,
            position_in_content INTEGER NOT NULL,
            width INTEGER,
            height INTEGER,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
        );

        -- Full-text search (FTS5)
        CREATE VIRTUAL TABLE entries_fts USING fts5(
            entry_id UNINDEXED,
            title,
            content,
            tokenize='porter unicode61'
        );

        -- Sync metadata
        CREATE TABLE sync_metadata (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL UNIQUE,
            cloud_provider TEXT NOT NULL,
            cloud_bundle_id TEXT,
            last_synced INTEGER,
            sync_status TEXT CHECK(sync_status IN ('SYNCED', 'PENDING', 'CONFLICT', 'ERROR')),
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
        );

        -- User settings
        CREATE TABLE user_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- Indexes for performance
        CREATE INDEX idx_entries_created ON entries(created_at DESC);
        CREATE INDEX idx_entries_mode ON entries(mode);
        CREATE INDEX idx_entries_updated ON entries(updated_at DESC);
        CREATE INDEX idx_pages_entry ON pages(entry_id, page_number);
        CREATE INDEX idx_images_entry ON images(entry_id);
        CREATE INDEX idx_sync_status ON sync_metadata(sync_status);
        CREATE INDEX idx_checkboxes_note ON checkboxes(note_id, position);

        -- Triggers for FTS5 sync
        CREATE TRIGGER entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(entry_id, title, content) 
            VALUES (new.id, new.title, '');
        END;

        CREATE TRIGGER entries_ad AFTER DELETE ON entries BEGIN
            DELETE FROM entries_fts WHERE entry_id = old.id;
        END;

        CREATE TRIGGER entries_au AFTER UPDATE ON entries BEGIN
            UPDATE entries_fts 
            SET title = new.title 
            WHERE entry_id = new.id;
        END;

        COMMIT;
        "#,
    )?;

    Ok(())
}

/// Migrate schema from old version to new version
fn migrate_schema(conn: &Connection, from_version: i32) -> Result<()> {
    match from_version {
        1 => {
            // Future migrations will go here
            Ok(())
        }
        _ => {
            warn!("No migration path from version {}", from_version);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::Database;

    #[test]
    fn test_schema_initialization() {
        let db = Database::in_memory().unwrap();
        let result = initialize_schema(db.connection());
        assert!(result.is_ok());
    }

    #[test]
    fn test_schema_version() {
        let db = Database::in_memory().unwrap();
        initialize_schema(db.connection()).unwrap();

        let version = get_schema_version(db.connection()).unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }

    #[test]
    fn test_tables_created() {
        let db = Database::in_memory().unwrap();
        initialize_schema(db.connection()).unwrap();

        let tables = vec![
            "entries",
            "pages",
            "notes",
            "checkboxes",
            "images",
            "sync_metadata",
            "user_settings",
        ];

        for table in tables {
            let count: i32 = db
                .connection()
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'",
                        table
                    ),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table {} not created", table);
        }
    }
}
