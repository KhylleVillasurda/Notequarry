// src/db/mod.rs

pub mod connection;
pub mod queries;
pub mod schema;

// Re-export commonly used items
pub use connection::Database;
pub use queries::entries::settings;
pub use queries::{Entry, EntryMode, Note, Page, entries, notes, pages, search};
pub use schema::initialize_schema;

use log::info;
use rusqlite::Result;

/// Initialize the complete database system
pub fn init(db_path: Option<std::path::PathBuf>) -> Result<Database> {
    info!("Initializing NoteQuarry database...");

    let db = Database::new(db_path)?;
    initialize_schema(db.connection())?;

    info!("Database initialized successfully");

    Ok(db)
}

/// Initialize an in-memory database (for testing)
pub fn init_memory() -> Result<Database> {
    let db = Database::in_memory()?;
    initialize_schema(db.connection())?;
    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_memory() {
        let db = init_memory().unwrap();
        assert_eq!(db.path(), std::path::Path::new(":memory:"));
    }

    #[test]
    fn test_database_integrity() {
        let db = init_memory().unwrap();
        assert!(db.check_integrity().unwrap());
    }
}
