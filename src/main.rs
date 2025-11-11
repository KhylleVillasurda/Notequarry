// main.rs - Qt integration version
mod crypto;
mod db;
mod qt_ffi;

use log::info;
use std::cell::RefCell;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

// Struct to hold current state
struct AppState {
    db: db::Database,
    current_entry_id: Option<i64>,
    current_entry_mode: Option<db::EntryMode>,
    current_page_id: Option<i64>,
    displayed_entry_ids: Vec<i64>,
    master_key: Option<crypto::MasterKey>,
    qt_handle: *mut qt_ffi::MainWindowHandle,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Starting NoteQuarry (Qt version)...");

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

    // Initialize Qt
    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect();
    let c_args: Vec<*mut c_char> = args.iter()
        .map(|arg| arg.as_ptr() as *mut c_char)
        .collect();

    let qt_handle = unsafe {
        qt_ffi::qt_init(c_args.len() as i32, c_args.as_ptr() as *mut *mut c_char)
    };

    let app_state = Box::into_raw(Box::new(RefCell::new(AppState {
        db: database,
        current_entry_id: None,
        current_entry_mode: None,
        current_page_id: None,
        displayed_entry_ids: Vec::new(),
        master_key: None,
        qt_handle,
    })));

    // Register all callbacks
    setup_callbacks(app_state);

    // Load initial entries
    unsafe {
        load_entries_to_ui(&(*app_state).borrow());
    }

    // Run Qt event loop (blocking)
    let _exit_code = unsafe { qt_ffi::qt_exec(qt_handle) };

    // Cleanup
    unsafe {
        qt_ffi::qt_cleanup(qt_handle);
        let _ = Box::from_raw(app_state); // Reclaim memory
    }

    Ok(())
}

fn setup_callbacks(app_state: *mut RefCell<AppState>) {
    let state_ptr = app_state as *mut std::ffi::c_void;
    
    let qt_handle = unsafe { (*app_state).borrow().qt_handle };

    // Password submitted
    unsafe {
        qt_ffi::qt_register_password_submitted(
            qt_handle,
            Some(on_password_submitted),
            state_ptr,
        );
    }

    // New entry clicked
    unsafe {
        qt_ffi::qt_register_new_entry_clicked(
            qt_handle,
            Some(on_new_entry_clicked),
            state_ptr,
        );
    }

    // Mode selected
    unsafe {
        qt_ffi::qt_register_mode_selected(
            qt_handle,
            Some(on_mode_selected),
            state_ptr,
        );
    }

    // Entry selected
    unsafe {
        qt_ffi::qt_register_entry_selected(
            qt_handle,
            Some(on_entry_selected),
            state_ptr,
        );
    }

    // Delete entry
    unsafe {
        qt_ffi::qt_register_delete_entry(
            qt_handle,
            Some(on_delete_entry),
            state_ptr,
        );
    }

    // Save content
    unsafe {
        qt_ffi::qt_register_save_content(
            qt_handle,
            Some(on_save_content),
            state_ptr,
        );
    }

    // Back to list
    unsafe {
        qt_ffi::qt_register_back_to_list(
            qt_handle,
            Some(on_back_to_list),
            state_ptr,
        );
    }

    // Search entries
    unsafe {
        qt_ffi::qt_register_search_entries(
            qt_handle,
            Some(on_search_entries),
            state_ptr,
        );
    }

    // Page changed
    unsafe {
        qt_ffi::qt_register_page_changed(
            qt_handle,
            Some(on_page_changed),
            state_ptr,
        );
    }

    // Add new page
    unsafe {
        qt_ffi::qt_register_add_new_page(
            qt_handle,
            Some(on_add_new_page),
            state_ptr,
        );
    }
}

// ============ Callback Implementations ============

extern "C" fn on_password_submitted(password: *const c_char, user_data: *mut std::ffi::c_void) {
    let app_state = user_data as *mut RefCell<AppState>;
    let password_str = unsafe { CStr::from_ptr(password).to_str().unwrap() };
    
    info!("Password submitted, deriving key...");
    
    let mut state = unsafe { &mut *app_state }.borrow_mut();
    
    // Get or create persistent salt
    let salt = match db::settings::get(state.db.connection(), "master_salt") {
        Ok(Some(salt_hex)) => {
            info!("Using existing salt");
            hex::decode(&salt_hex).unwrap_or_else(|_| crypto::generate_salt())
        }
        _ => {
            info!("Generating new salt");
            let new_salt = crypto::generate_salt();
            let salt_hex = hex::encode(&new_salt);
            let _ = db::settings::set(state.db.connection(), "master_salt", &salt_hex);
            new_salt
        }
    };
    
    match crypto::derive_key(password_str, &salt) {
        Ok(master_key) => {
            info!("Master key derived successfully!");
            state.master_key = Some(master_key);
            
            // Load entries after successful password
            drop(state);
            unsafe {
                load_entries_to_ui(&(*app_state).borrow());
            }
        }
        Err(e) => {
            eprintln!("Key derivation failed: {}", e);
            let error_msg = CString::new(format!("Key derivation failed: {}", e)).unwrap();
            unsafe {
                qt_ffi::qt_set_password_error(state.qt_handle, error_msg.as_ptr());
                qt_ffi::qt_show_password_error(state.qt_handle, 1);
            }
        }
    }
}

extern "C" fn on_new_entry_clicked(user_data: *mut std::ffi::c_void) {
    let _app_state = user_data as *mut RefCell<AppState>;
    info!("New entry clicked");
    // Mode dialog will handle this
}

extern "C" fn on_mode_selected(mode: *const c_char, title: *const c_char, user_data: *mut std::ffi::c_void) {
    let app_state = user_data as *mut RefCell<AppState>;
    let mode_str = unsafe { CStr::from_ptr(mode).to_str().unwrap() };
    let title_str = unsafe { CStr::from_ptr(title).to_str().unwrap() };
    
    info!("Creating entry: {} (mode: {})", title_str, mode_str);
    
    let mut state = unsafe { &mut *app_state }.borrow_mut();
    
    let master_key = match &state.master_key {
        Some(k) => k.clone(),
        None => {
            eprintln!("No master key available!");
            return;
        }
    };
    
    let entry_mode = if mode_str == "BOOK" {
        db::EntryMode::Book
    } else {
        db::EntryMode::Note
    };
    
    let entry = db::Entry::new(title_str.to_string(), entry_mode.clone(), generate_dummy_salt());
    
    match db::entries::create(state.db.connection(), &entry) {
        Ok(entry_id) => {
            info!("Entry created with ID: {}", entry_id);
            
            let empty_encrypted = match crypto::encrypt("", &master_key) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("Failed to encrypt empty content: {}", e);
                    vec![]
                }
            };
            
            match entry_mode {
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
            
            let _ = db::search::update_fts_content(state.db.connection(), entry_id, "");
            
            drop(state);
            unsafe {
                load_entries_to_ui(&(*app_state).borrow());
            }
        }
        Err(e) => {
            eprintln!("Failed to create entry: {}", e);
        }
    }
}

extern "C" fn on_entry_selected(index: i32, user_data: *mut std::ffi::c_void) {
    let app_state = user_data as *mut RefCell<AppState>;
    info!("Selected entry at index: {}", index);
    
    let state_ref = unsafe { &mut *app_state };
    let state = state_ref.borrow();
    
    let master_key = match &state.master_key {
        Some(key) => key.clone(),
        None => {
            eprintln!("No master key available!");
            return;
        }
    };
    
    let entry_id = match state.displayed_entry_ids.get(index as usize) {
        Some(&id) => id,
        None => {
            eprintln!("Invalid entry index: {}", index);
            return;
        }
    };
    
    drop(state);
    
    let mut state = state_ref.borrow_mut();
    
    match db::entries::get_by_id(state.db.connection(), entry_id) {
        Ok(entry) => {
            state.current_entry_id = Some(entry_id);
            state.current_entry_mode = Some(entry.mode.clone());
            
            let title_cstr = CString::new(entry.title.clone()).unwrap();
            unsafe {
                qt_ffi::qt_set_current_entry_title(state.qt_handle, title_cstr.as_ptr());
            }
            
            match entry.mode {
                db::EntryMode::Book => {
                    if let Ok(pages) = db::pages::get_by_entry(state.db.connection(), entry_id) {
                        let total = pages.len() as i32;
                        unsafe {
                            qt_ffi::qt_set_total_pages(state.qt_handle, if total == 0 { 1 } else { total });
                            qt_ffi::qt_set_current_page(state.qt_handle, 1);
                        }
                        
                        if let Some(first_page) = pages.first() {
                            state.current_page_id = first_page.id;
                            if let Ok(plaintext) = crypto::decrypt(&first_page.content_encrypted, &master_key) {
                                let content_cstr = CString::new(plaintext.clone()).unwrap();
                                let word_count = count_words(&plaintext);
                                unsafe {
                                    qt_ffi::qt_set_current_content(state.qt_handle, content_cstr.as_ptr());
                                    qt_ffi::qt_set_word_count(state.qt_handle, word_count);
                                }
                            }
                        }
                    }
                }
                db::EntryMode::Note => {
                    if let Ok(note) = db::notes::get_by_entry(state.db.connection(), entry_id) {
                        if let Ok(plaintext) = crypto::decrypt(&note.content_encrypted, &master_key) {
                            let content_cstr = CString::new(plaintext).unwrap();
                            unsafe {
                                qt_ffi::qt_set_current_content(state.qt_handle, content_cstr.as_ptr());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get entry: {}", e);
        }
    }
}

extern "C" fn on_delete_entry(index: i32, user_data: *mut std::ffi::c_void) {
    let app_state = user_data as *mut RefCell<AppState>;
    info!("Delete entry at index: {}", index);
    
    let state_ref = unsafe { &mut *app_state };
    let state = state_ref.borrow();
    
    let entry_id = match state.displayed_entry_ids.get(index as usize) {
        Some(&id) => id,
        None => {
            return;
        }
    };
    
    match db::entries::delete(state.db.connection(), entry_id) {
        Ok(_) => {
            info!("Entry {} deleted successfully", entry_id);
            drop(state);
            unsafe {
                load_entries_to_ui(&(*app_state).borrow());
            }
        }
        Err(e) => {
            eprintln!("Failed to delete entry: {}", e);
        }
    }
}

extern "C" fn on_save_content(content: *const c_char, user_data: *mut std::ffi::c_void) {
    let _app_state = user_data as *mut RefCell<AppState>;
    let _content_str = unsafe { CStr::from_ptr(content).to_str().unwrap() };
    
    info!("Saving content...");
    
    // Implementation similar to your original save_content callback
    // ... (truncated for brevity, follows same pattern as original)
}

extern "C" fn on_back_to_list(user_data: *mut std::ffi::c_void) {
    let app_state = user_data as *mut RefCell<AppState>;
    info!("Back to list");
    
    let mut state = unsafe { &mut *app_state }.borrow_mut();
    state.current_entry_id = None;
    state.current_entry_mode = None;
    state.current_page_id = None;
}

extern "C" fn on_search_entries(query: *const c_char, user_data: *mut std::ffi::c_void) {
    let _app_state = user_data as *mut RefCell<AppState>;
    let _query_str = unsafe { CStr::from_ptr(query).to_str().unwrap() };
    
    info!("Searching...");
    
    // Implementation follows your original search logic
    // ... (truncated for brevity)
}

extern "C" fn on_page_changed(_page: i32, _user_data: *mut std::ffi::c_void) {
    info!("Page changed");
    // Implementation follows your original page navigation
}

extern "C" fn on_add_new_page(_user_data: *mut std::ffi::c_void) {
    info!("Add new page");
    // Implementation follows your original add page logic
}

// ============ Helper Functions ============

fn load_entries_to_ui(state: &AppState) {
    match db::entries::get_all(state.db.connection()) {
        Ok(entries) => {
            info!("Loaded {} entries from database", entries.len());
            
            let entry_strings: Vec<CString> = entries
                .iter()
                .map(|entry| {
                    let icon = match entry.mode {
                        db::EntryMode::Book => "📚",
                        db::EntryMode::Note => "📝",
                    };
                    CString::new(format!("{} {}", icon, entry.title)).unwrap()
                })
                .collect();
            
            let c_strings: Vec<*const c_char> = entry_strings
                .iter()
                .map(|s| s.as_ptr())
                .collect();
            
            unsafe {
                qt_ffi::qt_set_entry_list(
                    state.qt_handle,
                    c_strings.as_ptr(),
                    c_strings.len() as i32,
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to load entries: {}", e);
        }
    }
}

fn count_words(text: &str) -> i32 {
    text.split_whitespace().count() as i32
}

fn generate_dummy_salt() -> Vec<u8> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    timestamp.to_le_bytes().to_vec()
}