// src/db/queries.rs

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Result, params};

/// Entry mode enum
#[derive(Debug, Clone, PartialEq)]
pub enum EntryMode {
    Book,
    Note,
}

impl EntryMode {
    pub fn as_str(&self) -> &str {
        match self {
            EntryMode::Book => "BOOK",
            EntryMode::Note => "NOTE",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "BOOK" => Some(EntryMode::Book),
            "NOTE" => Some(EntryMode::Note),
            _ => None,
        }
    }
}

/// Entry struct
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: Option<i64>,
    pub title: String,
    pub mode: EntryMode,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Option<String>,
    pub encryption_key_salt: Vec<u8>,
    pub is_encrypted: bool,
}

impl Entry {
    pub fn new(title: String, mode: EntryMode, salt: Vec<u8>) -> Self {
        let now = Utc::now().timestamp();
        Entry {
            id: None,
            title,
            mode,
            created_at: now,
            updated_at: now,
            tags: None,
            encryption_key_salt: salt,
            is_encrypted: true,
        }
    }
}

/// Page struct for Book mode
#[derive(Debug, Clone)]
pub struct Page {
    pub id: Option<i64>,
    pub entry_id: i64,
    pub page_number: i32,
    pub content_encrypted: Vec<u8>,
    pub word_count: i32,
    pub created_at: i64,
}

impl Page {
    pub fn new(entry_id: i64, page_number: i32, content: Vec<u8>, word_count: i32) -> Self {
        Page {
            id: None,
            entry_id,
            page_number,
            content_encrypted: content,
            word_count,
            created_at: Utc::now().timestamp(),
        }
    }
}

/// Note struct for Note mode
#[derive(Debug, Clone)]
pub struct Note {
    pub id: Option<i64>,
    pub entry_id: i64,
    pub content_encrypted: Vec<u8>,
    pub has_checkboxes: bool,
}

impl Note {
    pub fn new(entry_id: i64, content: Vec<u8>, has_checkboxes: bool) -> Self {
        Note {
            id: None,
            entry_id,
            content_encrypted: content,
            has_checkboxes,
        }
    }
}

/// Entry queries
pub mod entries {
    use super::*;

    /// Create a new entry
    pub fn create(conn: &Connection, entry: &Entry) -> Result<i64> {
        conn.execute(
            "INSERT INTO entries (title, mode, created_at, updated_at, tags, encryption_key_salt, is_encrypted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &entry.title,
                entry.mode.as_str(),
                entry.created_at,
                entry.updated_at,
                &entry.tags,
                &entry.encryption_key_salt,
                entry.is_encrypted as i32,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub mod settings {
        use super::*;

        /// Get a setting value
        pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
            conn.query_row(
                "SELECT value FROM user_settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
        }

        /// Set a setting value
        pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
            conn.execute(
                "INSERT OR REPLACE INTO user_settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
            Ok(())
        }
    }

    /// Get entry by ID
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Entry> {
        conn.query_row(
            "SELECT id, title, mode, created_at, updated_at, tags, encryption_key_salt, is_encrypted
             FROM entries WHERE id = ?1",
            params![id],
            |row| {
                Ok(Entry {
                    id: Some(row.get(0)?),
                    title: row.get(1)?,
                    mode: EntryMode::from_str(&row.get::<_, String>(2)?).unwrap(),
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    tags: row.get(5)?,
                    encryption_key_salt: row.get(6)?,
                    is_encrypted: row.get::<_, i32>(7)? != 0,
                })
            },
        )
    }

    /// Get all entries (sorted by creation date, newest first)
    pub fn get_all(conn: &Connection) -> Result<Vec<Entry>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, mode, created_at, updated_at, tags, encryption_key_salt, is_encrypted
             FROM entries ORDER BY created_at DESC"
        )?;

        let entries = stmt.query_map([], |row| {
            Ok(Entry {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                mode: EntryMode::from_str(&row.get::<_, String>(2)?).unwrap(),
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                tags: row.get(5)?,
                encryption_key_salt: row.get(6)?,
                is_encrypted: row.get::<_, i32>(7)? != 0,
            })
        })?;

        entries.collect()
    }

    /// Update entry
    pub fn update(conn: &Connection, entry: &Entry) -> Result<()> {
        let id = entry.id.expect("Entry must have an ID to update");
        let now = Utc::now().timestamp();

        conn.execute(
            "UPDATE entries SET title = ?1, updated_at = ?2, tags = ?3 WHERE id = ?4",
            params![&entry.title, now, &entry.tags, id],
        )?;
        Ok(())
    }

    /// Delete entry (cascades to pages/notes/images)
    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Get entries by mode
    pub fn get_by_mode(conn: &Connection, mode: EntryMode) -> Result<Vec<Entry>> {
        let mut stmt = conn.prepare(
            "SELECT id, title, mode, created_at, updated_at, tags, encryption_key_salt, is_encrypted
             FROM entries WHERE mode = ?1 ORDER BY created_at DESC"
        )?;

        let entries = stmt.query_map(params![mode.as_str()], |row| {
            Ok(Entry {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                mode: EntryMode::from_str(&row.get::<_, String>(2)?).unwrap(),
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                tags: row.get(5)?,
                encryption_key_salt: row.get(6)?,
                is_encrypted: row.get::<_, i32>(7)? != 0,
            })
        })?;

        entries.collect()
    }

    /// Count all entries
    pub fn count(conn: &Connection) -> Result<i64> {
        conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
    }
}

/// Page queries (Book mode)
pub mod pages {
    use super::*;

    /// Create a new page
    pub fn create(conn: &Connection, page: &Page) -> Result<i64> {
        conn.execute(
            "INSERT INTO pages (entry_id, page_number, content_encrypted, word_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                page.entry_id,
                page.page_number,
                &page.content_encrypted,
                page.word_count,
                page.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get page by ID
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Page> {
        conn.query_row(
            "SELECT id, entry_id, page_number, content_encrypted, word_count, created_at
             FROM pages WHERE id = ?1",
            params![id],
            |row| {
                Ok(Page {
                    id: Some(row.get(0)?),
                    entry_id: row.get(1)?,
                    page_number: row.get(2)?,
                    content_encrypted: row.get(3)?,
                    word_count: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
    }

    /// Get all pages for an entry
    pub fn get_by_entry(conn: &Connection, entry_id: i64) -> Result<Vec<Page>> {
        let mut stmt = conn.prepare(
            "SELECT id, entry_id, page_number, content_encrypted, word_count, created_at
             FROM pages WHERE entry_id = ?1 ORDER BY page_number ASC",
        )?;

        let pages = stmt.query_map(params![entry_id], |row| {
            Ok(Page {
                id: Some(row.get(0)?),
                entry_id: row.get(1)?,
                page_number: row.get(2)?,
                content_encrypted: row.get(3)?,
                word_count: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        pages.collect()
    }

    /// Update page content
    pub fn update(conn: &Connection, page: &Page) -> Result<()> {
        let id = page.id.expect("Page must have an ID to update");

        conn.execute(
            "UPDATE pages SET content_encrypted = ?1, word_count = ?2 WHERE id = ?3",
            params![&page.content_encrypted, page.word_count, id],
        )?;
        Ok(())
    }

    /// Delete page
    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM pages WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Count pages for an entry
    pub fn count_by_entry(conn: &Connection, entry_id: i64) -> Result<i64> {
        conn.query_row(
            "SELECT COUNT(*) FROM pages WHERE entry_id = ?1",
            params![entry_id],
            |row| row.get(0),
        )
    }
}

/// Note queries (Note mode)
pub mod notes {
    use super::*;

    /// Create a new note
    pub fn create(conn: &Connection, note: &Note) -> Result<i64> {
        conn.execute(
            "INSERT INTO notes (entry_id, content_encrypted, has_checkboxes)
             VALUES (?1, ?2, ?3)",
            params![
                note.entry_id,
                &note.content_encrypted,
                note.has_checkboxes as i32,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get note by entry ID
    pub fn get_by_entry(conn: &Connection, entry_id: i64) -> Result<Note> {
        conn.query_row(
            "SELECT id, entry_id, content_encrypted, has_checkboxes
             FROM notes WHERE entry_id = ?1",
            params![entry_id],
            |row| {
                Ok(Note {
                    id: Some(row.get(0)?),
                    entry_id: row.get(1)?,
                    content_encrypted: row.get(2)?,
                    has_checkboxes: row.get::<_, i32>(3)? != 0,
                })
            },
        )
    }

    /// Update note content
    pub fn update(conn: &Connection, note: &Note) -> Result<()> {
        let id = note.id.expect("Note must have an ID to update");

        conn.execute(
            "UPDATE notes SET content_encrypted = ?1, has_checkboxes = ?2 WHERE id = ?3",
            params![&note.content_encrypted, note.has_checkboxes as i32, id],
        )?;
        Ok(())
    }

    /// Delete note
    pub fn delete(conn: &Connection, entry_id: i64) -> Result<()> {
        conn.execute("DELETE FROM notes WHERE entry_id = ?1", params![entry_id])?;
        Ok(())
    }
}

/// Search queries using FTS5
pub mod search {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct SearchResult {
        pub entry_id: i64,
        pub title: String,
        pub snippet: String,
    }

    /// Search entries by query
    pub fn search_entries(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
        let mut stmt = conn.prepare(
            "SELECT e.id, e.title, snippet(entries_fts, 2, '<b>', '</b>', '...', 32) as snippet
             FROM entries_fts
             JOIN entries e ON entries_fts.entry_id = e.id
             WHERE entries_fts MATCH ?1
             ORDER BY rank",
        )?;

        let results = stmt.query_map(params![query], |row| {
            Ok(SearchResult {
                entry_id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
            })
        })?;

        results.collect()
    }

    /// Update FTS5 index for an entry (called after content update)
    pub fn update_fts_content(conn: &Connection, entry_id: i64, content: &str) -> Result<()> {
        conn.execute(
            "UPDATE entries_fts SET content = ?1 WHERE entry_id = ?2",
            params![content, entry_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connection::Database, schema::initialize_schema};

    fn setup_test_db() -> Database {
        let db = Database::in_memory().unwrap();
        initialize_schema(db.connection()).unwrap();
        db
    }

    #[test]
    fn test_create_and_get_entry() {
        let db = setup_test_db();
        let mut entry = Entry::new("Test Entry".to_string(), EntryMode::Book, vec![1, 2, 3, 4]);

        let id = entries::create(db.connection(), &entry).unwrap();
        entry.id = Some(id);

        let retrieved = entries::get_by_id(db.connection(), id).unwrap();
        assert_eq!(retrieved.title, "Test Entry");
        assert_eq!(retrieved.mode, EntryMode::Book);
    }

    #[test]
    fn test_update_entry() {
        let db = setup_test_db();
        let mut entry = Entry::new(
            "Original Title".to_string(),
            EntryMode::Note,
            vec![1, 2, 3, 4],
        );

        let id = entries::create(db.connection(), &entry).unwrap();
        entry.id = Some(id);
        entry.title = "Updated Title".to_string();

        entries::update(db.connection(), &entry).unwrap();

        let retrieved = entries::get_by_id(db.connection(), id).unwrap();
        assert_eq!(retrieved.title, "Updated Title");
    }

    #[test]
    fn test_delete_entry() {
        let db = setup_test_db();
        let entry = Entry::new("Delete Me".to_string(), EntryMode::Book, vec![1, 2, 3, 4]);

        let id = entries::create(db.connection(), &entry).unwrap();
        entries::delete(db.connection(), id).unwrap();

        let result = entries::get_by_id(db.connection(), id);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_entries() {
        let db = setup_test_db();

        entries::create(
            db.connection(),
            &Entry::new("Entry 1".to_string(), EntryMode::Book, vec![1]),
        )
        .unwrap();

        entries::create(
            db.connection(),
            &Entry::new("Entry 2".to_string(), EntryMode::Note, vec![2]),
        )
        .unwrap();

        let all = entries::get_all(db.connection()).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_create_and_get_page() {
        let db = setup_test_db();
        let entry = Entry::new("Book Entry".to_string(), EntryMode::Book, vec![1]);
        let entry_id = entries::create(db.connection(), &entry).unwrap();

        let mut page = Page::new(entry_id, 1, vec![10, 20, 30], 500);
        let page_id = pages::create(db.connection(), &page).unwrap();
        page.id = Some(page_id);

        let retrieved = pages::get_by_id(db.connection(), page_id).unwrap();
        assert_eq!(retrieved.page_number, 1);
        assert_eq!(retrieved.word_count, 500);
    }

    #[test]
    fn test_get_pages_by_entry() {
        let db = setup_test_db();
        let entry = Entry::new("Book Entry".to_string(), EntryMode::Book, vec![1]);
        let entry_id = entries::create(db.connection(), &entry).unwrap();

        pages::create(db.connection(), &Page::new(entry_id, 1, vec![1], 100)).unwrap();
        pages::create(db.connection(), &Page::new(entry_id, 2, vec![2], 200)).unwrap();
        pages::create(db.connection(), &Page::new(entry_id, 3, vec![3], 300)).unwrap();

        let all_pages = pages::get_by_entry(db.connection(), entry_id).unwrap();
        assert_eq!(all_pages.len(), 3);
        assert_eq!(all_pages[0].page_number, 1);
        assert_eq!(all_pages[2].page_number, 3);
    }

    #[test]
    fn test_create_and_get_note() {
        let db = setup_test_db();
        let entry = Entry::new("Note Entry".to_string(), EntryMode::Note, vec![1]);
        let entry_id = entries::create(db.connection(), &entry).unwrap();

        let note = Note::new(entry_id, vec![10, 20, 30], true);
        notes::create(db.connection(), &note).unwrap();

        let retrieved = notes::get_by_entry(db.connection(), entry_id).unwrap();
        assert_eq!(retrieved.has_checkboxes, true);
    }

    #[test]
    fn test_cascade_delete() {
        let db = setup_test_db();
        let entry = Entry::new("Book Entry".to_string(), EntryMode::Book, vec![1]);
        let entry_id = entries::create(db.connection(), &entry).unwrap();

        pages::create(db.connection(), &Page::new(entry_id, 1, vec![1], 100)).unwrap();
        pages::create(db.connection(), &Page::new(entry_id, 2, vec![2], 200)).unwrap();

        // Delete entry should cascade to pages
        entries::delete(db.connection(), entry_id).unwrap();

        let page_count = pages::count_by_entry(db.connection(), entry_id).unwrap();
        assert_eq!(page_count, 0);
    }

    #[test]
    fn test_fts_search() {
        let db = setup_test_db();

        let entry1 = Entry::new("Rust Programming".to_string(), EntryMode::Book, vec![1]);
        let id1 = entries::create(db.connection(), &entry1).unwrap();
        search::update_fts_content(
            db.connection(),
            id1,
            "Learning about Rust programming language",
        )
        .unwrap();

        let entry2 = Entry::new("Python Guide".to_string(), EntryMode::Note, vec![2]);
        let id2 = entries::create(db.connection(), &entry2).unwrap();
        search::update_fts_content(db.connection(), id2, "Python is great for scripting").unwrap();

        let results = search::search_entries(db.connection(), "Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }

    #[test]
    fn test_entry_count() {
        let db = setup_test_db();

        entries::create(
            db.connection(),
            &Entry::new("E1".to_string(), EntryMode::Book, vec![1]),
        )
        .unwrap();
        entries::create(
            db.connection(),
            &Entry::new("E2".to_string(), EntryMode::Note, vec![2]),
        )
        .unwrap();

        let count = entries::count(db.connection()).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_entries_by_mode() {
        let db = setup_test_db();

        entries::create(
            db.connection(),
            &Entry::new("Book 1".to_string(), EntryMode::Book, vec![1]),
        )
        .unwrap();
        entries::create(
            db.connection(),
            &Entry::new("Note 1".to_string(), EntryMode::Note, vec![2]),
        )
        .unwrap();
        entries::create(
            db.connection(),
            &Entry::new("Book 2".to_string(), EntryMode::Book, vec![3]),
        )
        .unwrap();

        let books = entries::get_by_mode(db.connection(), EntryMode::Book).unwrap();
        assert_eq!(books.len(), 2);

        let notes = entries::get_by_mode(db.connection(), EntryMode::Note).unwrap();
        assert_eq!(notes.len(), 1);
    }
}
