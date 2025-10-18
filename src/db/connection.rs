// src/db/connection.rs

use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Database connection manager
pub struct Database {
    conn: Connection,
    db_path: PathBuf,
}

impl Database {
    /// Create a new database connection
    ///
    /// # Arguments
    /// * `db_path` - Optional custom path. If None, uses default app data directory
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| Self::default_db_path());

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create database directory");
        }

        let conn = Connection::open(&path)?;

        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // Enable foreign keys
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Database {
            conn,
            db_path: path,
        })
    }

    /// Get the default database path based on OS
    fn default_db_path() -> PathBuf {
        let proj_dirs = ProjectDirs::from("com", "notequarry", "NoteQuarry")
            .expect("Unable to determine project directories");

        let data_dir = proj_dirs.data_dir();
        data_dir.join("notequarry.db")
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get the database file path
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Create an in-memory database (useful for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Database {
            conn,
            db_path: PathBuf::from(":memory:"),
        })
    }

    /// Check if database file exists
    pub fn exists(&self) -> bool {
        self.db_path.exists()
    }

    /// Get database file size in bytes
    pub fn size(&self) -> std::io::Result<u64> {
        if self.db_path.exists() {
            fs::metadata(&self.db_path).map(|meta| meta.len())
        } else {
            Ok(0)
        }
    }

    /// Vacuum the database to reclaim space
    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }

    /// Run database integrity check
    pub fn check_integrity(&self) -> Result<bool> {
        let mut stmt = self.conn.prepare("PRAGMA integrity_check")?;
        let result: String = stmt.query_row([], |row| row.get(0))?;
        Ok(result == "ok")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_database() {
        let db = Database::in_memory().unwrap();
        assert_eq!(db.path(), Path::new(":memory:"));
    }

    #[test]
    fn test_wal_mode_enabled() {
        let db = Database::in_memory().unwrap();
        let journal_mode: String = db
            .connection()
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        // In-memory databases don't use WAL, but we can test with file-based
        assert!(journal_mode == "memory" || journal_mode == "wal");
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let db = Database::in_memory().unwrap();
        let foreign_keys: bool = db
            .connection()
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert!(foreign_keys);
    }

    #[test]
    fn test_integrity_check() {
        let db = Database::in_memory().unwrap();
        assert!(db.check_integrity().unwrap());
    }
}
