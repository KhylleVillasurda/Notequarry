// src/ui/qt_bridge.h
// C Bridge between Rust and Qt
#ifndef QT_BRIDGE_H
#define QT_BRIDGE_H

#ifdef __cplusplus
extern "C"
{
#endif

    // Opaque pointer to MainWindow (Rust doesn't need to know the internals)
    typedef struct MainWindowHandle MainWindowHandle;

    // ==============================================
    // Initialization and Lifecycle
    // ==============================================

    /// Initialize Qt application and create main window
    /// Returns: Handle to the main window
    MainWindowHandle *qt_init(int argc, char **argv);

    /// Run the Qt event loop (blocking call)
    /// Returns: Exit code
    int qt_exec(MainWindowHandle *handle);

    /// Cleanup and destroy the window
    void qt_cleanup(MainWindowHandle *handle);

    // ==============================================
    // UI Update Functions (Called from Rust)
    // ==============================================

    /// Set the entry list in the UI
    void qt_set_entry_list(MainWindowHandle *handle, const char **entries, int count);

    /// Set current entry title
    void qt_set_current_entry_title(MainWindowHandle *handle, const char *title);

    /// Set current content in editor
    void qt_set_current_content(MainWindowHandle *handle, const char *content);

    /// Set current page number
    void qt_set_current_page(MainWindowHandle *handle, int page);

    /// Set total pages
    void qt_set_total_pages(MainWindowHandle *handle, int total);

    /// Set word count
    void qt_set_word_count(MainWindowHandle *handle, int count);

    /// Set password error message
    void qt_set_password_error(MainWindowHandle *handle, const char *error);

    /// Show/hide password error
    void qt_show_password_error(MainWindowHandle *handle, int show);

    /// Switch to book editor view
    void qt_show_book_editor(MainWindowHandle *handle);

    /// Switch to note editor view
    void qt_show_note_editor(MainWindowHandle *handle);

    /// Switch back to list view
    void qt_show_list_view(MainWindowHandle *handle);

    // ==============================================
    // Callback Registration (Rust provides callbacks)
    // ==============================================

    /// Callback function types
    typedef void (*PasswordSubmittedCallback)(const char *password, void *user_data);
    typedef void (*NewEntryClickedCallback)(void *user_data);
    typedef void (*ModeSelectedCallback)(const char *mode, const char *title, void *user_data);
    typedef void (*EntrySelectedCallback)(int index, void *user_data);
    typedef void (*DeleteEntryCallback)(int index, void *user_data);
    typedef void (*SaveContentCallback)(const char *content, void *user_data);
    typedef void (*BackToListCallback)(void *user_data);
    typedef void (*SearchEntriesCallback)(const char *query, void *user_data);
    typedef void (*PageChangedCallback)(int page, void *user_data);
    typedef void (*AddNewPageCallback)(void *user_data);

    /// Register callbacks that Qt will call when events occur
    void qt_register_password_submitted(MainWindowHandle *handle, PasswordSubmittedCallback cb, void *user_data);
    void qt_register_new_entry_clicked(MainWindowHandle *handle, NewEntryClickedCallback cb, void *user_data);
    void qt_register_mode_selected(MainWindowHandle *handle, ModeSelectedCallback cb, void *user_data);
    void qt_register_entry_selected(MainWindowHandle *handle, EntrySelectedCallback cb, void *user_data);
    void qt_register_delete_entry(MainWindowHandle *handle, DeleteEntryCallback cb, void *user_data);
    void qt_register_save_content(MainWindowHandle *handle, SaveContentCallback cb, void *user_data);
    void qt_register_back_to_list(MainWindowHandle *handle, BackToListCallback cb, void *user_data);
    void qt_register_search_entries(MainWindowHandle *handle, SearchEntriesCallback cb, void *user_data);
    void qt_register_page_changed(MainWindowHandle *handle, PageChangedCallback cb, void *user_data);
    void qt_register_add_new_page(MainWindowHandle *handle, AddNewPageCallback cb, void *user_data);

#ifdef __cplusplus
}
#endif

#endif // QT_BRIDGE_H