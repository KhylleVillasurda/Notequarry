// src/main.rs

slint::include_modules!();
mod db;

use env_logger;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

// Struct to hold current state
struct AppState {
    db: db::Database,
    current_entry_id: Option<i64>,
    current_entry_mode: Option<db::EntryMode>,
}

fn main() -> Result<(), slint::PlatformError> {
    // Initialize logger
    env_logger::init();

    info!("Starting NoteQuarry...");

    // Initialize database
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

    // Wrap app state
    let app_state = Rc::new(RefCell::new(AppState {
        db: database,
        current_entry_id: None,
        current_entry_mode: None,
    }));

    // Create UI
    let ui = MainWindow::new()?;

    // Load existing entries
    load_entries_to_ui(&ui, &app_state);

    // Set up all callbacks
    setup_callbacks(&ui, app_state.clone());

    info!("NoteQuarry UI started successfully!");

    ui.run()
}

/// Load existing entries from database into UI
fn load_entries_to_ui(ui: &MainWindow, app_state: &Rc<RefCell<AppState>>) {
    let state = app_state.borrow();

    match db::entries::get_all(state.db.connection()) {
        Ok(entries) => {
            info!("Loaded {} entries from database", entries.len());

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
            ui.set_entry_list(slint::VecModel::from_slice(&[]));
        }
    }
}

/// Set up all UI callbacks
fn setup_callbacks(ui: &MainWindow, app_state: Rc<RefCell<AppState>>) {
    let ui_weak = ui.as_weak();

    // New entry clicked - show mode selection dialog
    let state_clone = app_state.clone();
    ui.on_new_entry_clicked(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_mode_dialog(true);
        }
    });

    // Mode selected - create entry in database
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_mode_selected(move |data_str, _unused| {
        // Parse "MODE|TITLE" format from Slint
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

        let mut state = state_clone.borrow_mut();

        let mode = if mode_str == "BOOK" {
            db::EntryMode::Book
        } else {
            db::EntryMode::Note
        };

        let entry = db::Entry::new(title.to_string(), mode.clone(), generate_dummy_salt());

        match db::entries::create(state.db.connection(), &entry) {
            Ok(entry_id) => {
                info!("Entry created with ID: {}", entry_id);

                // Create initial content based on mode
                match mode {
                    db::EntryMode::Book => {
                        let page = db::Page::new(entry_id, 1, vec![], 0);
                        if let Err(e) = db::pages::create(state.db.connection(), &page) {
                            eprintln!("Failed to create initial page: {}", e);
                        }
                    }
                    db::EntryMode::Note => {
                        let note = db::Note::new(entry_id, vec![], false);
                        if let Err(e) = db::notes::create(state.db.connection(), &note) {
                            eprintln!("Failed to create initial note: {}", e);
                        }
                    }
                }

                // Reload entry list
                drop(state); // Release borrow
                if let Some(ui) = ui_weak.upgrade() {
                    load_entries_to_ui(&ui, &state_clone);
                }
            }
            Err(e) => {
                eprintln!("Failed to create entry: {}", e);
            }
        }
    });

    // Entry selected - open editor
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_entry_selected(move |index| {
        info!("Selected entry at index: {}", index);

        let mut state = state_clone.borrow_mut();

        match db::entries::get_all(state.db.connection()) {
            Ok(entries) => {
                if let Some(entry) = entries.get(index as usize) {
                    if let Some(entry_id) = entry.id {
                        state.current_entry_id = Some(entry_id);
                        state.current_entry_mode = Some(entry.mode.clone());

                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_current_entry_title(entry.title.clone().into());

                            match entry.mode {
                                db::EntryMode::Book => {
                                    // Load first page
                                    match db::pages::get_by_entry(state.db.connection(), entry_id) {
                                        Ok(pages) => {
                                            let total = pages.len() as i32;
                                            ui.set_total_pages(if total == 0 { 1 } else { total });
                                            ui.set_current_page(1);

                                            // TODO: Decrypt content in Week 2
                                            ui.set_current_content(
                                                "Content will be decrypted here...".into(),
                                            );
                                            ui.set_word_count(0);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to load pages: {}", e);
                                        }
                                    }
                                    ui.set_show_book_editor(true);
                                }
                                db::EntryMode::Note => {
                                    // Load note content
                                    match db::notes::get_by_entry(state.db.connection(), entry_id) {
                                        Ok(_note) => {
                                            // TODO: Decrypt content in Week 2
                                            ui.set_current_content(
                                                "Note content will be decrypted here...".into(),
                                            );
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to load note: {}", e);
                                        }
                                    }
                                    ui.set_show_note_editor(true);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get entries: {}", e);
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

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_book_editor(false);
            ui.set_show_note_editor(false);

            drop(state); // Release borrow
            load_entries_to_ui(&ui, &state_clone);
        }
    });

    // Save content
    let state_clone = app_state.clone();
    ui.on_save_content(move |content| {
        let state = state_clone.borrow();

        if let Some(entry_id) = state.current_entry_id {
            info!("Saving content for entry {}", entry_id);
            info!(
                "Content preview: {}...",
                &content.to_string()[..content.len().min(50)]
            );

            // TODO: Implement encryption and actual save in Week 2
            info!("Save functionality will be implemented with encryption in Week 2");
        }
    });

    // Delete entry
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_delete_entry_clicked(move |index| {
        info!("Delete entry at index: {}", index);

        let state = state_clone.borrow();

        match db::entries::get_all(state.db.connection()) {
            Ok(entries) => {
                if let Some(entry) = entries.get(index as usize) {
                    if let Some(entry_id) = entry.id {
                        match db::entries::delete(state.db.connection(), entry_id) {
                            Ok(_) => {
                                info!("Entry {} deleted successfully", entry_id);
                                drop(state); // Release borrow
                                if let Some(ui) = ui_weak.upgrade() {
                                    load_entries_to_ui(&ui, &state_clone);
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to delete entry: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get entries: {}", e);
            }
        }
    });

    // Search entries
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_search_entries(move |query| {
        if query.is_empty() {
            // Reload all entries
            if let Some(ui) = ui_weak.upgrade() {
                load_entries_to_ui(&ui, &state_clone);
            }
            return;
        }

        info!("Searching for: {}", query);
        let state = state_clone.borrow();

        match db::search::search_entries(state.db.connection(), &query.to_string()) {
            Ok(results) => {
                info!("Found {} results", results.len());

                let result_titles: Vec<slint::SharedString> = results
                    .iter()
                    .map(|result| format!("🔍 {}", result.title).into())
                    .collect();

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_entry_list(slint::VecModel::from_slice(&result_titles));
                }
            }
            Err(e) => {
                eprintln!("Search failed: {}", e);
            }
        }
    });

    // Page navigation
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_page_changed(move |new_page| {
        info!("Navigating to page {}", new_page);

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_current_page(new_page);
            // TODO: Load page content in Week 2
            ui.set_current_content(format!("Page {} content...", new_page).into());
        }
    });

    // Add new page
    let ui_weak = ui.as_weak();
    let state_clone = app_state.clone();
    ui.on_add_new_page(move || {
        let state = state_clone.borrow();

        if let Some(entry_id) = state.current_entry_id {
            info!("Adding new page to entry {}", entry_id);

            match db::pages::count_by_entry(state.db.connection(), entry_id) {
                Ok(count) => {
                    let new_page_number = (count + 1) as i32;
                    let new_page = db::Page::new(entry_id, new_page_number, vec![], 0);

                    match db::pages::create(state.db.connection(), &new_page) {
                        Ok(_) => {
                            info!("Created page {}", new_page_number);
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_total_pages(new_page_number);
                                ui.set_current_page(new_page_number);
                                ui.set_current_content("".into());
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
        info!("Insert image clicked (feature coming in future phases)");
        // TODO: Implement in Phase 3
    });

    // Add checkbox (placeholder)
    ui.on_add_checkbox(move || {
        info!("Add checkbox clicked");
        // TODO: Implement checkbox insertion
    });
}

/// Generate dummy salt (temporary until crypto module is ready)
fn generate_dummy_salt() -> Vec<u8> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    timestamp.to_le_bytes().to_vec()
}
