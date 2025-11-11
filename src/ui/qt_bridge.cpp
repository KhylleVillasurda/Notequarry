// src/ui/qt_bridge.cpp
#include "qt_bridge.h"
#include "mainwindow.h"
#include <QApplication>
#include <QString>
#include <QStringList>

// Internal structure that holds Qt objects and callbacks
struct MainWindowHandle
{
    QApplication *app;
    MainWindow *window;

    // Callback storage
    PasswordSubmittedCallback password_cb;
    void *password_user_data;

    NewEntryClickedCallback new_entry_cb;
    void *new_entry_user_data;

    ModeSelectedCallback mode_selected_cb;
    void *mode_selected_user_data;

    EntrySelectedCallback entry_selected_cb;
    void *entry_selected_user_data;

    DeleteEntryCallback delete_entry_cb;
    void *delete_entry_user_data;

    SaveContentCallback save_content_cb;
    void *save_content_user_data;

    BackToListCallback back_to_list_cb;
    void *back_to_list_user_data;

    SearchEntriesCallback search_entries_cb;
    void *search_entries_user_data;

    PageChangedCallback page_changed_cb;
    void *page_changed_user_data;

    AddNewPageCallback add_new_page_cb;
    void *add_new_page_user_data;
};

// ==============================================
// Initialization and Lifecycle
// ==============================================

MainWindowHandle *qt_init(int argc, char **argv)
{
    MainWindowHandle *handle = new MainWindowHandle();

    handle->app = new QApplication(argc, argv);
    handle->window = new MainWindow();

    // Initialize all callbacks to nullptr
    handle->password_cb = nullptr;
    handle->password_user_data = nullptr;
    handle->new_entry_cb = nullptr;
    handle->new_entry_user_data = nullptr;
    handle->mode_selected_cb = nullptr;
    handle->mode_selected_user_data = nullptr;
    handle->entry_selected_cb = nullptr;
    handle->entry_selected_user_data = nullptr;
    handle->delete_entry_cb = nullptr;
    handle->delete_entry_user_data = nullptr;
    handle->save_content_cb = nullptr;
    handle->save_content_user_data = nullptr;
    handle->back_to_list_cb = nullptr;
    handle->back_to_list_user_data = nullptr;
    handle->search_entries_cb = nullptr;
    handle->search_entries_user_data = nullptr;
    handle->page_changed_cb = nullptr;
    handle->page_changed_user_data = nullptr;
    handle->add_new_page_cb = nullptr;
    handle->add_new_page_user_data = nullptr;

    handle->window->show();

    return handle;
}

int qt_exec(MainWindowHandle *handle)
{
    if (!handle || !handle->app)
        return -1;
    return handle->app->exec();
}

void qt_cleanup(MainWindowHandle *handle)
{
    if (handle)
    {
        if (handle->window)
            delete handle->window;
        if (handle->app)
            delete handle->app;
        delete handle;
    }
}

// ==============================================
// UI Update Functions
// ==============================================

void qt_set_entry_list(MainWindowHandle *handle, const char **entries, int count)
{
    if (!handle || !handle->window)
        return;

    QStringList list;
    for (int i = 0; i < count; i++)
    {
        list.append(QString::fromUtf8(entries[i]));
    }
    handle->window->setEntryList(list);
}

void qt_set_current_entry_title(MainWindowHandle *handle, const char *title)
{
    if (!handle || !handle->window)
        return;
    handle->window->setCurrentEntryTitle(QString::fromUtf8(title));
}

void qt_set_current_content(MainWindowHandle *handle, const char *content)
{
    if (!handle || !handle->window)
        return;
    handle->window->setCurrentContent(QString::fromUtf8(content));
}

void qt_set_current_page(MainWindowHandle *handle, int page)
{
    if (!handle || !handle->window)
        return;
    handle->window->setCurrentPage(page);
}

void qt_set_total_pages(MainWindowHandle *handle, int total)
{
    if (!handle || !handle->window)
        return;
    handle->window->setTotalPages(total);
}

void qt_set_word_count(MainWindowHandle *handle, int count)
{
    if (!handle || !handle->window)
        return;
    handle->window->setWordCount(count);
}

void qt_set_password_error(MainWindowHandle *handle, const char *error)
{
    if (!handle || !handle->window)
        return;
    handle->window->setPasswordError(QString::fromUtf8(error));
}

void qt_show_password_error(MainWindowHandle *handle, int show)
{
    if (!handle || !handle->window)
        return;
    handle->window->setShowPasswordError(show != 0);
}

void qt_show_book_editor(MainWindowHandle *handle)
{
    // This would require adding a method to MainWindow
    // For now, the view switching is handled internally
}

void qt_show_note_editor(MainWindowHandle *handle)
{
    // Same as above
}

void qt_show_list_view(MainWindowHandle *handle)
{
    // Same as above
}

// ==============================================
// Callback Registration
// ==============================================

void qt_register_password_submitted(MainWindowHandle *handle, PasswordSubmittedCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->password_cb = cb;
    handle->password_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::passwordSubmitted,
                     [handle](const QString &password)
                     {
                         if (handle->password_cb)
                         {
                             QByteArray utf8 = password.toUtf8();
                             handle->password_cb(utf8.constData(), handle->password_user_data);
                         }
                     });
}

void qt_register_new_entry_clicked(MainWindowHandle *handle, NewEntryClickedCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->new_entry_cb = cb;
    handle->new_entry_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::newEntryClicked,
                     [handle]()
                     {
                         if (handle->new_entry_cb)
                         {
                             handle->new_entry_cb(handle->new_entry_user_data);
                         }
                     });
}

void qt_register_mode_selected(MainWindowHandle *handle, ModeSelectedCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->mode_selected_cb = cb;
    handle->mode_selected_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::modeSelected,
                     [handle](const QString &data, const QString &)
                     {
                         if (handle->mode_selected_cb)
                         {
                             // Parse "MODE|TITLE" format
                             QStringList parts = data.split('|');
                             if (parts.size() >= 2)
                             {
                                 QByteArray mode = parts[0].toUtf8();
                                 QByteArray title = parts[1].toUtf8();
                                 handle->mode_selected_cb(mode.constData(), title.constData(),
                                                          handle->mode_selected_user_data);
                             }
                         }
                     });
}

void qt_register_entry_selected(MainWindowHandle *handle, EntrySelectedCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->entry_selected_cb = cb;
    handle->entry_selected_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::entrySelected,
                     [handle](int index)
                     {
                         if (handle->entry_selected_cb)
                         {
                             handle->entry_selected_cb(index, handle->entry_selected_user_data);
                         }
                     });
}

void qt_register_delete_entry(MainWindowHandle *handle, DeleteEntryCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->delete_entry_cb = cb;
    handle->delete_entry_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::deleteEntryClicked,
                     [handle](int index)
                     {
                         if (handle->delete_entry_cb)
                         {
                             handle->delete_entry_cb(index, handle->delete_entry_user_data);
                         }
                     });
}

void qt_register_save_content(MainWindowHandle *handle, SaveContentCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->save_content_cb = cb;
    handle->save_content_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::saveContent,
                     [handle](const QString &content)
                     {
                         if (handle->save_content_cb)
                         {
                             QByteArray utf8 = content.toUtf8();
                             handle->save_content_cb(utf8.constData(), handle->save_content_user_data);
                         }
                     });
}

void qt_register_back_to_list(MainWindowHandle *handle, BackToListCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->back_to_list_cb = cb;
    handle->back_to_list_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::backToList,
                     [handle]()
                     {
                         if (handle->back_to_list_cb)
                         {
                             handle->back_to_list_cb(handle->back_to_list_user_data);
                         }
                     });
}

void qt_register_search_entries(MainWindowHandle *handle, SearchEntriesCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->search_entries_cb = cb;
    handle->search_entries_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::searchEntries,
                     [handle](const QString &query)
                     {
                         if (handle->search_entries_cb)
                         {
                             QByteArray utf8 = query.toUtf8();
                             handle->search_entries_cb(utf8.constData(), handle->search_entries_user_data);
                         }
                     });
}

void qt_register_page_changed(MainWindowHandle *handle, PageChangedCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->page_changed_cb = cb;
    handle->page_changed_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::pageChanged,
                     [handle](int page)
                     {
                         if (handle->page_changed_cb)
                         {
                             handle->page_changed_cb(page, handle->page_changed_user_data);
                         }
                     });
}

void qt_register_add_new_page(MainWindowHandle *handle, AddNewPageCallback cb, void *user_data)
{
    if (!handle || !handle->window)
        return;

    handle->add_new_page_cb = cb;
    handle->add_new_page_user_data = user_data;

    QObject::connect(handle->window, &MainWindow::addNewPage,
                     [handle]()
                     {
                         if (handle->add_new_page_cb)
                         {
                             handle->add_new_page_cb(handle->add_new_page_user_data);
                         }
                     });
}