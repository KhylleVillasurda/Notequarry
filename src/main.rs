slint::include_modules!();
mod crypto;
mod db;

use chrono::{Local, TimeZone};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
//use env_logger;

// Struct to hold current state
struct AppState {
    db: db::Database,
    current_entry_id: Option<i64>,
    current_entry_mode: Option<db::EntryMode>,
    current_page_id: Option<i64>,
    displayed_entry_ids: Vec<i64>,
    master_key: Option<crypto::MasterKey>,
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::init();
    info!("Starting NoteQuarry...");

    let database = match db::init(None) {
        Ok(db) => {
            info!("Database initialized successfully at: {:?}", db.path());
            db
        }
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    let app_state = Rc::new(RefCell::new(AppState {
        db: database,
        current_entry_id: None,
        current_entry_mode: None,
        current_page_id: None,
        displayed_entry_ids: Vec::new(),
        master_key: None,
    }));

    let ui = MainWindow::new()?;
    ui.set_show_password_dialog(true);
    load_entries_to_ui(&ui, &app_state);
    setup_callbacks(&ui, app_state.clone());

    info!("NoteQuarry UI started successfully!");
    ui.run()
}

fn load_entries_to_ui(ui: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let mut state = app_state.borrow_mut();

    match db::entries::get_all(state.db.connection()) {
        Ok(entries) => {
            info!("Loaded {} entries from database", entries.len());
            state.displayed_entry_ids = entries.iter().filter_map(|e| e.id).collect();

            let entry_titles: Vec<slint::SharedString> = entries
                .iter()
                .map(|entry| {
                    let icon = match entry.mode {
                        db::EntryMode::Book => "📚",
                        db::EntryMode::Note => "📝",
                    };
                    format!("{} {}", icon, entry.title).into()
                })
                .collect();

            ui.set_entry_list(slint::VecModel::from_slice(&entry_titles));
        }
        Err(e) => {
            eprintln!("Failed to load entries: {}", e);
            state.displayed_entry_ids.clear();
            ui.set_entry_list(slint::VecModel::from_slice(&[]));
        }
    }
}
/*
fn format_date(timestamp: i64) -> String {
    let datetime = Local.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%b %d, %Y").to_string()
}
*/
fn count_words(text: &str) -> i32 {
    text.split_whitespace().count() as i32
}

fn setup_callbacks(ui: &MainWindow, app_state: Rc<RefCell<AppState>>) {
    // Password submitted
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();

    ui.on_password_submitted(move |password| {
        info!("Password submitted, deriving key...");
        let password_str = password.to_string();

        if password_str.is_empty() {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_password_error("Password cannot be empty".into());
                ui.set_show_password_error(true);
            }
            return;
        }

        let mut state = state_clone.borrow_mut();

        // Get or create persistent salt
        let salt = match db::settings::get(state.db.connection(), "master_salt") {
            Ok(Some(salt_hex)) => {
                // Decode existing salt from hex
                info!("Using existing salt");
                hex::decode(&salt_hex).unwrap_or_else(|_| crypto::generate_salt())
            }
            _ => {
                // Generate new salt and store it
                info!("Generating new salt");
                let new_salt = crypto::generate_salt();
                let salt_hex = hex::encode(&new_salt);
                let _ = db::settings::set(state.db.connection(), "master_salt", &salt_hex);
                new_salt
            }
        };

        match crypto::derive_key(&password_str, &salt) {
            Ok(master_key) => {
                info!("Master key derived successfully!");
                state.master_key = Some(master_key);
                drop(state);

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_show_password_dialog(false);
                    ui.set_show_password_error(false);
                    load_entries_to_ui(&ui, &state_clone);
                }
            }
            Err(e) => {
                eprintln!("Key derivation failed: {}", e);
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_password_error(format!("Key derivation failed: {}", e).into());
                    ui.set_show_password_error(true);
                }
            }
        }
    });

    // New entry clicked
    let ui_weak = ui.as_weak();
    ui.on_new_entry_clicked(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_mode_dialog(true);
        }
    });

    // Mode selected
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_mode_selected(move |data_str, _unused| {
        let data = data_str.to_string();
        let parts: Vec<&str> = data.split('|').collect();

        if parts.len() != 2 {
            eprintln!("Invalid mode selection data");
            return;
        }

        let mode_str = parts[0];
        let title = parts[1];
        info!("Creating entry: {} (mode: {})", title, mode_str);

        if title.is_empty() {
            eprintln!("Cannot create entry with empty title");
            return;
        }

        let master_key = {
            let state = state_clone.borrow();
            match &state.master_key {
                Some(k) => k.clone(),
                None => {
                    eprintln!("No master key available!");
                    return;
                }
            }
        };

        let state = state_clone.borrow_mut();
        let mode = if mode_str == "BOOK" {
            db::EntryMode::Book
        } else {
            db::EntryMode::Note
        };

        let entry = db::Entry::new(title.to_string(), mode.clone(), generate_dummy_salt());

        match db::entries::create(state.db.connection(), &entry) {
            Ok(entry_id) => {
                info!("Entry created with ID: {}", entry_id);

                // Encrypt empty content for initial entry
                let empty_encrypted = match crypto::encrypt("", &master_key) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        eprintln!("Failed to encrypt empty content: {}", e);
                        vec![]
                    }
                };

                match mode {
                    db::EntryMode::Book => {
                        let page = db::Page::new(entry_id, 1, empty_encrypted, 0);
                        if let Err(e) = db::pages::create(state.db.connection(), &page) {
                            eprintln!("Failed to create initial page: {}", e);
                        }
                    }
                    db::EntryMode::Note => {
                        let note = db::Note::new(entry_id, empty_encrypted, false);
                        if let Err(e) = db::notes::create(state.db.connection(), &note) {
                            eprintln!("Failed to create initial note: {}", e);
                        }
                    }
                }

                if let Err(e) = db::search::update_fts_content(state.db.connection(), entry_id, "")
                {
                    eprintln!("Failed to update search index: {}", e);
                }

                drop(state);
                if let Some(ui) = ui_weak.upgrade() {
                    load_entries_to_ui(&ui, &state_clone);
                }
            }
            Err(e) => {
                eprintln!("Failed to create entry: {}", e);
            }
        }
    });

    // Entry selected
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_entry_selected(move |index| {
        info!("Selected entry at index: {}", index);

        let master_key = {
            let state = state_clone.borrow();
            match &state.master_key {
                Some(key) => key.clone(),
                None => {
                    eprintln!("No master key available!");
                    return;
                }
            }
        };

        let entry_id = {
            let state = state_clone.borrow();
            match state.displayed_entry_ids.get(index as usize) {
                Some(&id) => id,
                None => {
                    eprintln!("Invalid entry index: {}", index);
                    return;
                }
            }
        };

        info!("Mapped index {} to entry ID {}", index, entry_id);

        let mut state = state_clone.borrow_mut();

        match db::entries::get_by_id(state.db.connection(), entry_id) {
            Ok(entry) => {
                state.current_entry_id = Some(entry_id);
                state.current_entry_mode = Some(entry.mode.clone());

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_current_entry_title(entry.title.clone().into());

                    match entry.mode {
                        db::EntryMode::Book => {
                            match db::pages::get_by_entry(state.db.connection(), entry_id) {
                                Ok(pages) => {
                                    let total = pages.len() as i32;
                                    ui.set_total_pages(if total == 0 { 1 } else { total });
                                    ui.set_current_page(1);

                                    if let Some(first_page) = pages.first() {
                                        state.current_page_id = first_page.id;
                                        match crypto::decrypt(
                                            &first_page.content_encrypted,
                                            &master_key,
                                        ) {
                                            Ok(plaintext) => {
                                                let word_count = count_words(&plaintext);
                                                ui.set_current_content(plaintext.into());
                                                ui.set_word_count(word_count);
                                            }
                                            Err(e) => {
                                                eprintln!("Decryption failed: {}", e);
                                                ui.set_current_content(
                                                    "[Decryption failed - wrong password?]".into(),
                                                );
                                                ui.set_word_count(0);
                                            }
                                        }
                                    } else {
                                        ui.set_current_content("".into());
                                        ui.set_word_count(0);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to load pages: {}", e);
                                }
                            }
                            ui.set_show_book_editor(true);
                        }
                        db::EntryMode::Note => {
                            match db::notes::get_by_entry(state.db.connection(), entry_id) {
                                Ok(note) => {
                                    match crypto::decrypt(&note.content_encrypted, &master_key) {
                                        Ok(plaintext) => {
                                            ui.set_current_content(plaintext.into());
                                        }
                                        Err(e) => {
                                            eprintln!("Decryption failed: {}", e);
                                            ui.set_current_content(
                                                "[Decryption failed - wrong password?]".into(),
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to load note: {}", e);
                                    ui.set_current_content("".into());
                                }
                            }
                            ui.set_show_note_editor(true);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get entry by ID {}: {}", entry_id, e);
            }
        }
    });

    // Back to list
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_back_to_list(move || {
        info!("Back to entry list");
        let mut state = state_clone.borrow_mut();
        state.current_entry_id = None;
        state.current_entry_mode = None;
        state.current_page_id = None;
        drop(state);

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_book_editor(false);
            ui.set_show_note_editor(false);
            load_entries_to_ui(&ui, &state_clone);
        }
    });

    // Save content
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_save_content(move |content| {
        let content_str = content.to_string();

        let (master_key, entry_id, entry_mode, page_id) = {
            let state = state_clone.borrow();
            let key = match &state.master_key {
                Some(k) => k.clone(),
                None => {
                    eprintln!("No master key available!");
                    return;
                }
            };
            (
                key,
                state.current_entry_id,
                state.current_entry_mode.clone(),
                state.current_page_id,
            )
        };

        if let Some(eid) = entry_id {
            info!("Saving content for entry {}", eid);

            let encrypted_bytes = match crypto::encrypt(&content_str, &master_key) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("Encryption failed: {}", e);
                    return;
                }
            };

            let state = state_clone.borrow_mut();

            match entry_mode {
                Some(db::EntryMode::Book) => {
                    if let Some(pid) = page_id {
                        let word_count = count_words(&content_str);
                        match db::pages::get_by_id(state.db.connection(), pid) {
                            Ok(mut page) => {
                                page.content_encrypted = encrypted_bytes;
                                page.word_count = word_count;

                                match db::pages::update(state.db.connection(), &page) {
                                    Ok(_) => {
                                        info!("Page saved and encrypted successfully");
                                        if let Some(ui) = ui_weak.upgrade() {
                                            ui.set_word_count(word_count);
                                        }

                                        let all_content = get_all_page_contents_decrypted(
                                            state.db.connection(),
                                            eid,
                                            &master_key,
                                        );
                                        let _ = db::search::update_fts_content(
                                            state.db.connection(),
                                            eid,
                                            &all_content,
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to save page: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to get page: {}", e);
                            }
                        }
                    }
                }
                Some(db::EntryMode::Note) => {
                    match db::notes::get_by_entry(state.db.connection(), eid) {
                        Ok(mut note) => {
                            note.content_encrypted = encrypted_bytes;
                            match db::notes::update(state.db.connection(), &note) {
                                Ok(_) => {
                                    info!("Note saved and encrypted successfully");
                                    let _ = db::search::update_fts_content(
                                        state.db.connection(),
                                        eid,
                                        &content_str,
                                    );
                                }
                                Err(e) => {
                                    eprintln!("Failed to save note: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get note: {}", e);
                        }
                    }
                }
                None => {
                    eprintln!("No entry mode set");
                }
            }
        }
    });

    // Delete entry
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_delete_entry_clicked(move |index| {
        info!("Delete entry at index: {}", index);

        let entry_id = {
            let state = state_clone.borrow();
            match state.displayed_entry_ids.get(index as usize) {
                Some(&id) => id,
                None => {
                    eprintln!("Invalid entry index for delete: {}", index);
                    return;
                }
            }
        };

        info!("Deleting entry ID {}", entry_id);

        let state = state_clone.borrow();
        match db::entries::delete(state.db.connection(), entry_id) {
            Ok(_) => {
                info!("Entry {} deleted successfully", entry_id);
                drop(state);
                if let Some(ui) = ui_weak.upgrade() {
                    load_entries_to_ui(&ui, &state_clone);
                }
            }
            Err(e) => {
                eprintln!("Failed to delete entry: {}", e);
            }
        }
    });

    // Search entries
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_search_entries(move |query| {
        let query_str = query.to_string().trim().to_string();

        if query_str.is_empty() {
            if let Some(ui) = ui_weak.upgrade() {
                load_entries_to_ui(&ui, &state_clone);
            }
            return;
        }

        info!("Searching for: {}", query_str);
        let mut state = state_clone.borrow_mut();
        let escaped_query = escape_fts5_query(&query_str);

        match db::search::search_entries(state.db.connection(), &escaped_query) {
            Ok(results) => {
                info!("Found {} results", results.len());
                let mut matched_entries = Vec::new();
                let mut matched_ids = Vec::new();

                for result in results {
                    if let Ok(entry) =
                        db::entries::get_by_id(state.db.connection(), result.entry_id)
                    {
                        if let Some(id) = entry.id {
                            matched_ids.push(id);
                            matched_entries.push(entry);
                        }
                    }
                }

                state.displayed_entry_ids = matched_ids;
                let result_titles: Vec<slint::SharedString> = matched_entries
                    .iter()
                    .map(|entry| {
                        let icon = match entry.mode {
                            db::EntryMode::Book => "📚",
                            db::EntryMode::Note => "📝",
                        };
                        format!("🔍 {} {}", icon, entry.title).into()
                    })
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_entry_list(slint::VecModel::from_slice(&result_titles));
                }
            }
            Err(e) => {
                eprintln!("Search failed: {}", e);
                state.displayed_entry_ids.clear();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_entry_list(slint::VecModel::from_slice(&[]));
                }
            }
        }
    });

    // Page navigation
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_page_changed(move |new_page| {
        info!("=== PAGE NAVIGATION ===");
        info!("Navigating to page {}", new_page);

        let current_content = if let Some(ui) = ui_weak.upgrade() {
            let content = ui.get_current_content().to_string();
            let current_pg = ui.get_current_page();
            info!("Current page {} has {} chars", current_pg, content.len());
            Some((content, current_pg))
        } else {
            None
        };

        if let Some((content, current_pg)) = current_content {
            if current_pg != new_page && !content.is_empty() {
                let (master_key, page_id) = {
                    let state = state_clone.borrow();
                    let key = match &state.master_key {
                        Some(k) => k.clone(),
                        None => return,
                    };
                    (key, state.current_page_id)
                };

                if let Some(pid) = page_id {
                    if let Ok(encrypted) = crypto::encrypt(&content, &master_key) {
                        let state = state_clone.borrow_mut();
                        if let Ok(mut page) = db::pages::get_by_id(state.db.connection(), pid) {
                            page.content_encrypted = encrypted;
                            page.word_count = count_words(&content);
                            let _ = db::pages::update(state.db.connection(), &page);
                            info!("✓ Auto-saved page {} before navigation", current_pg);
                        }
                    }
                }
            }
        }

        let (master_key, entry_id) = {
            let state = state_clone.borrow();
            let key = match &state.master_key {
                Some(k) => k.clone(),
                None => {
                    eprintln!("No master key available!");
                    return;
                }
            };
            (key, state.current_entry_id)
        };

        if let Some(eid) = entry_id {
            let mut state = state_clone.borrow_mut();

            match db::pages::get_by_entry(state.db.connection(), eid) {
                Ok(pages) => {
                    if let Some(page) = pages.get((new_page - 1) as usize) {
                        state.current_page_id = page.id;

                        match crypto::decrypt(&page.content_encrypted, &master_key) {
                            Ok(plaintext) => {
                                let word_count = count_words(&plaintext);
                                info!(
                                    "✓ Loaded page {} content: {} chars",
                                    new_page,
                                    plaintext.len()
                                );

                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_current_page(new_page);
                                    ui.set_current_content(plaintext.into());
                                    ui.set_word_count(word_count);
                                }
                            }
                            Err(e) => {
                                eprintln!("Decryption failed: {}", e);
                                if let Some(ui) = ui_weak.upgrade() {
                                    ui.set_current_content("[Decryption failed]".into());
                                    ui.set_word_count(0);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load pages: {}", e);
                }
            }
        }
    });

    // Add new page
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_add_new_page(move || {
        let (entry_id, master_key) = {
            let state = state_clone.borrow();
            let key = match &state.master_key {
                Some(k) => k.clone(),
                None => {
                    eprintln!("No master key available!");
                    return;
                }
            };
            (state.current_entry_id, key)
        };

        if let Some(eid) = entry_id {
            info!("Adding new page to entry {}", eid);

            // Encrypt empty content for new page
            let empty_encrypted = match crypto::encrypt("", &master_key) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("Failed to encrypt empty content: {}", e);
                    return;
                }
            };

            let mut state = state_clone.borrow_mut();

            match db::pages::count_by_entry(state.db.connection(), eid) {
                Ok(count) => {
                    let new_page_number = (count + 1) as i32;
                    let new_page = db::Page::new(eid, new_page_number, empty_encrypted, 0);

                    match db::pages::create(state.db.connection(), &new_page) {
                        Ok(page_id) => {
                            info!("Created page {}", new_page_number);
                            state.current_page_id = Some(page_id);

                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_total_pages(new_page_number);
                                ui.set_current_page(new_page_number);
                                ui.set_current_content("".into());
                                ui.set_word_count(0);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to create page: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to count pages: {}", e);
                }
            }
        }
    });

    // Insert image (placeholder)
    ui.on_insert_image(move || {
        info!("Insert image clicked (feature coming in Week 3)");
    });

    // Add checkbox (placeholder)
    ui.on_add_checkbox(move || {
        info!("Add checkbox clicked (feature coming in Week 3)");
    });

    // Clear search
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_clear_search(move || {
        info!("Clearing search, showing all entries");
        if let Some(ui) = ui_weak.upgrade() {
            load_entries_to_ui(&ui, &state_clone);
        }
    });
} // This is the closing brace for setup_callbacks - make sure it's properly aligned

fn generate_dummy_salt() -> Vec<u8> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    timestamp.to_le_bytes().to_vec()
}
/*
fn get_all_page_contents(conn: &rusqlite::Connection, entry_id: i64) -> String {
    match db::pages::get_by_entry(conn, entry_id) {
        Ok(pages) => pages
            .iter()
            .map(|page| String::from_utf8_lossy(&page.content_encrypted).to_string())
            .collect::<Vec<_>>()
            .join(" "),
        Err(_) => String::new(),
    }
}
*/
fn get_all_page_contents_decrypted(
    conn: &rusqlite::Connection,
    entry_id: i64,
    key: &crypto::MasterKey,
) -> String {
    match db::pages::get_by_entry(conn, entry_id) {
        Ok(pages) => pages
            .iter()
            .filter_map(|page| crypto::decrypt(&page.content_encrypted, key).ok())
            .collect::<Vec<_>>()
            .join(" "),
        Err(_) => String::new(),
    }
}

fn escape_fts5_query(query: &str) -> String {
    let escaped = query.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}
