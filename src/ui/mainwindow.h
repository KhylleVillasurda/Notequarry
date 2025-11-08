// mainwindow.h
#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QStackedWidget>
#include <QListWidget>
#include <QTextEdit>
#include <QLineEdit>
#include <QPushButton>
#include <QLabel>
#include <QDialog>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QSpinBox>
#include <memory>

// Forward declarations
class PasswordDialog;
class ModeSelectionDialog;
class BookEditor;
class NoteEditor;

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

    // Property setters/getters
    void setEntryList(const QStringList &entries);
    void setCurrentEntryTitle(const QString &title);
    void setCurrentContent(const QString &content);
    void setCurrentPage(int page);
    void setTotalPages(int total);
    void setWordCount(int count);
    void setPasswordError(const QString &error);
    void setShowPasswordError(bool show);

    QString getCurrentContent() const;
    int getCurrentPage() const;

signals:
    // Main callbacks
    void passwordSubmitted(const QString &password);
    void newEntryClicked();
    void modeSelected(const QString &data, const QString &unused);
    void entrySelected(int index);
    void deleteEntryClicked(int index);
    void saveContent(const QString &content);
    void backToList();
    void searchEntries(const QString &query);
    void clearSearch();
    void pageChanged(int newPage);
    void addNewPage();
    void insertImage();
    void addCheckbox();

private slots:
    // REMOVED: void onPasswordSubmit();  <- This was never implemented!
    void onNewEntry();
    void onModeDialogAccepted(const QString &mode, const QString &title);
    void onEntryItemClicked(QListWidgetItem *item);
    void onDeleteEntry();
    void onSaveContent();
    void onSearchTextChanged(const QString &text);
    void onClearSearch();
    void onPreviousPage();
    void onNextPage();
    void onAddPage();
    void onBackToList();

private:
    void setupUI();
    void setupPasswordDialog();
    void setupListView();
    void applyDarkTheme();

    // UI Components
    QStackedWidget *m_stackedWidget;

    // Password Dialog
    PasswordDialog *m_passwordDialog;

    // List View
    QWidget *m_listViewWidget;
    QListWidget *m_entryListWidget;
    QLineEdit *m_searchBox;
    QPushButton *m_newEntryButton;

    // Editors
    BookEditor *m_bookEditor;
    NoteEditor *m_noteEditor;

    // Mode Selection Dialog
    ModeSelectionDialog *m_modeDialog;

    // State
    QStringList m_entryList;
    QString m_currentEntryTitle;
    int m_currentPage;
    int m_totalPages;
    int m_wordCount;
};

// ============ Password Dialog ============
class PasswordDialog : public QDialog
{
    Q_OBJECT

public:
    explicit PasswordDialog(QWidget *parent = nullptr);
    QString getPassword() const;
    void setErrorMessage(const QString &message);
    void setShowError(bool show);

signals:
    void passwordSubmitted(const QString &password);

private:
    void accept() override;

    QLineEdit *m_passwordInput;
    QLabel *m_errorLabel;
    QWidget *m_errorWidget;
    QPushButton *m_unlockButton;
};

// ============ Mode Selection Dialog ============
class ModeSelectionDialog : public QDialog
{
    Q_OBJECT

public:
    explicit ModeSelectionDialog(QWidget *parent = nullptr);

signals:
    void modeSelected(const QString &mode, const QString &title);

private slots:
    void onBookModeClicked();
    void onNoteModeClicked();

private:
    QLineEdit *m_titleInput;
};

// ============ Book Editor ============
class BookEditor : public QWidget
{
    Q_OBJECT

public:
    explicit BookEditor(QWidget *parent = nullptr);

    void setEntryTitle(const QString &title);
    void setContent(const QString &content);
    void setCurrentPage(int page);
    void setTotalPages(int total);
    void setWordCount(int count);

    QString getContent() const;
    int getCurrentPage() const;

signals:
    void backClicked();
    void saveClicked(const QString &content);
    void previousPage();
    void nextPage();
    void addPage();
    void insertImage();
    void contentChanged(const QString &text);
    void pageChanged(int newPage);

private:
    QLabel *m_titleLabel;
    QTextEdit *m_contentEditor;
    QLabel *m_pageInfoLabel;
    QLabel *m_wordCountLabel;
    QSpinBox *m_pageSpinBox;
    QPushButton *m_prevButton;
    QPushButton *m_nextButton;
    QPushButton *m_addPageButton;

    int m_currentPage;
    int m_totalPages;
};

// ============ Note Editor ============
class NoteEditor : public QWidget
{
    Q_OBJECT

public:
    explicit NoteEditor(QWidget *parent = nullptr);

    void setEntryTitle(const QString &title);
    void setContent(const QString &content);
    QString getContent() const;

signals:
    void backClicked();
    void saveClicked(const QString &content);
    void addCheckbox();
    void insertImage();

private:
    QLabel *m_titleLabel;
    QTextEdit *m_contentEditor;
};

#endif // MAINWINDOW_H