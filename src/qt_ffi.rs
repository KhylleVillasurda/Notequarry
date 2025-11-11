// src/qt_ffi.rs
// Rust FFI bindings to Qt C bridge

use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
pub struct MainWindowHandle {
    _private: [u8; 0],
}

// Callback types
pub type PasswordSubmittedCallback = extern "C" fn(*const c_char, *mut c_void);
pub type NewEntryClickedCallback = extern "C" fn(*mut c_void);
pub type ModeSelectedCallback = extern "C" fn(*const c_char, *const c_char, *mut c_void);
pub type EntrySelectedCallback = extern "C" fn(c_int, *mut c_void);
pub type DeleteEntryCallback = extern "C" fn(c_int, *mut c_void);
pub type SaveContentCallback = extern "C" fn(*const c_char, *mut c_void);
pub type BackToListCallback = extern "C" fn(*mut c_void);
pub type SearchEntriesCallback = extern "C" fn(*const c_char, *mut c_void);
pub type PageChangedCallback = extern "C" fn(c_int, *mut c_void);
pub type AddNewPageCallback = extern "C" fn(*mut c_void);

#[link(name = "notequarry_ui")]
extern "C" {
    // Lifecycle
    pub fn qt_init(argc: c_int, argv: *mut *mut c_char) -> *mut MainWindowHandle;
    pub fn qt_exec(handle: *mut MainWindowHandle) -> c_int;
    pub fn qt_cleanup(handle: *mut MainWindowHandle);

    // UI Updates
    pub fn qt_set_entry_list(handle: *mut MainWindowHandle, entries: *const *const c_char, count: c_int);
    pub fn qt_set_current_entry_title(handle: *mut MainWindowHandle, title: *const c_char);
    pub fn qt_set_current_content(handle: *mut MainWindowHandle, content: *const c_char);
    pub fn qt_set_current_page(handle: *mut MainWindowHandle, page: c_int);
    pub fn qt_set_total_pages(handle: *mut MainWindowHandle, total: c_int);
    pub fn qt_set_word_count(handle: *mut MainWindowHandle, count: c_int);
    pub fn qt_set_password_error(handle: *mut MainWindowHandle, error: *const c_char);
    pub fn qt_show_password_error(handle: *mut MainWindowHandle, show: c_int);

    // Callback Registration
    pub fn qt_register_password_submitted(
        handle: *mut MainWindowHandle,
        cb: Option<PasswordSubmittedCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_new_entry_clicked(
        handle: *mut MainWindowHandle,
        cb: Option<NewEntryClickedCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_mode_selected(
        handle: *mut MainWindowHandle,
        cb: Option<ModeSelectedCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_entry_selected(
        handle: *mut MainWindowHandle,
        cb: Option<EntrySelectedCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_delete_entry(
        handle: *mut MainWindowHandle,
        cb: Option<DeleteEntryCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_save_content(
        handle: *mut MainWindowHandle,
        cb: Option<SaveContentCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_back_to_list(
        handle: *mut MainWindowHandle,
        cb: Option<BackToListCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_search_entries(
        handle: *mut MainWindowHandle,
        cb: Option<SearchEntriesCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_page_changed(
        handle: *mut MainWindowHandle,
        cb: Option<PageChangedCallback>,
        user_data: *mut c_void,
    );
    
    pub fn qt_register_add_new_page(
        handle: *mut MainWindowHandle,
        cb: Option<AddNewPageCallback>,
        user_data: *mut c_void,
    );
}