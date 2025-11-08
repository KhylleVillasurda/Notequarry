// mainwindow.cpp
#include "mainwindow.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QScrollArea>
#include <QFrame>
#include <QStyle>
#include <QApplication>
#include <QRegularExpression>

// ============ MainWindow Implementation ============
MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent), m_stackedWidget(new QStackedWidget(this)), m_passwordDialog(nullptr), m_listViewWidget(nullptr), m_bookEditor(nullptr), m_noteEditor(nullptr), m_modeDialog(nullptr), m_currentPage(1), m_totalPages(1), m_wordCount(0)
{
    setupUI();
    applyDarkTheme();

    // Show password dialog on startup
    m_passwordDialog = new PasswordDialog(this);
    connect(m_passwordDialog, &PasswordDialog::passwordSubmitted,
            this, &MainWindow::passwordSubmitted);
    m_passwordDialog->exec();
}

MainWindow::~MainWindow()
{
}

void MainWindow::setupUI()
{
    setWindowTitle("NoteQuarry - Your Personal Journal");
    resize(1200, 800);

    setCentralWidget(m_stackedWidget);

    // Setup list view
    setupListView();
    m_stackedWidget->addWidget(m_listViewWidget);

    // Setup book editor
    m_bookEditor = new BookEditor(this);
    m_stackedWidget->addWidget(m_bookEditor);
    connect(m_bookEditor, &BookEditor::backClicked, this, &MainWindow::onBackToList);
    connect(m_bookEditor, &BookEditor::saveClicked, this, &MainWindow::saveContent);
    connect(m_bookEditor, &BookEditor::previousPage, this, &MainWindow::onPreviousPage);
    connect(m_bookEditor, &BookEditor::nextPage, this, &MainWindow::onNextPage);
    connect(m_bookEditor, &BookEditor::addPage, this, &MainWindow::onAddPage);
    connect(m_bookEditor, &BookEditor::insertImage, this, &MainWindow::insertImage);
    connect(m_bookEditor, &BookEditor::contentChanged, [this](const QString &text)
            {
        m_wordCount = text.split(QRegularExpression("\\s+"), Qt::SkipEmptyParts).count();
        m_bookEditor->setWordCount(m_wordCount); });

    // Setup note editor
    m_noteEditor = new NoteEditor(this);
    m_stackedWidget->addWidget(m_noteEditor);
    connect(m_noteEditor, &NoteEditor::backClicked, this, &MainWindow::onBackToList);
    connect(m_noteEditor, &NoteEditor::saveClicked, this, &MainWindow::saveContent);
    connect(m_noteEditor, &NoteEditor::addCheckbox, this, &MainWindow::addCheckbox);
    connect(m_noteEditor, &NoteEditor::insertImage, this, &MainWindow::insertImage);

    // Show list view by default
    m_stackedWidget->setCurrentWidget(m_listViewWidget);
}

void MainWindow::setupListView()
{
    m_listViewWidget = new QWidget;
    QVBoxLayout *mainLayout = new QVBoxLayout(m_listViewWidget);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    // Header bar
    QWidget *headerWidget = new QWidget;
    headerWidget->setObjectName("headerBar");
    headerWidget->setFixedHeight(80);
    QHBoxLayout *headerLayout = new QHBoxLayout(headerWidget);
    headerLayout->setContentsMargins(20, 20, 20, 20);

    QLabel *titleLabel = new QLabel("🌿 NoteQuarry");
    titleLabel->setObjectName("appTitle");
    titleLabel->setStyleSheet("font-size: 24px; font-weight: bold; color: #e8f5e3;");

    m_searchBox = new QLineEdit;
    m_searchBox->setPlaceholderText("🔍 Search entries...");
    m_searchBox->setFixedWidth(300);
    connect(m_searchBox, &QLineEdit::textChanged, this, &MainWindow::onSearchTextChanged);

    QPushButton *clearSearchBtn = new QPushButton("✕");
    clearSearchBtn->setFixedWidth(40);
    connect(clearSearchBtn, &QPushButton::clicked, this, &MainWindow::onClearSearch);

    m_newEntryButton = new QPushButton("+ New Entry");
    m_newEntryButton->setObjectName("primaryButton");
    connect(m_newEntryButton, &QPushButton::clicked, this, &MainWindow::onNewEntry);

    headerLayout->addWidget(titleLabel);
    headerLayout->addStretch();
    headerLayout->addWidget(m_searchBox);
    headerLayout->addWidget(clearSearchBtn);
    headerLayout->addWidget(m_newEntryButton);

    // Entry list
    QScrollArea *scrollArea = new QScrollArea;
    scrollArea->setWidgetResizable(true);
    scrollArea->setFrameShape(QFrame::NoFrame);

    QWidget *listContainer = new QWidget;
    QVBoxLayout *listLayout = new QVBoxLayout(listContainer);
    listLayout->setContentsMargins(30, 30, 30, 30);
    listLayout->setSpacing(15);

    m_entryListWidget = new QListWidget;
    m_entryListWidget->setObjectName("entryList");
    connect(m_entryListWidget, &QListWidget::itemClicked, this, &MainWindow::onEntryItemClicked);

    listLayout->addWidget(m_entryListWidget);
    scrollArea->setWidget(listContainer);

    mainLayout->addWidget(headerWidget);
    mainLayout->addWidget(scrollArea);
}

void MainWindow::applyDarkTheme()
{
    QString styleSheet = R"(
        QMainWindow, QWidget {
            background-color: #121212;
            color: #c5c5c5;
        }
        
        #headerBar {
            background: qlineargradient(x1:0, y1:0, x2:1, y2:1,
                stop:0 #1a3d14, stop:0.5 #2d5016, stop:1 #3d6b21);
        }
        
        #appTitle {
            font-size: 24px;
            font-weight: bold;
            color: #e8f5e3;
        }
        
        QLineEdit {
            background-color: #252525;
            border: 1px solid #2d5016;
            border-radius: 4px;
            padding: 8px;
            color: #c5c5c5;
            font-size: 14px;
        }
        
        QLineEdit:focus {
            border: 1px solid #5a8c3a;
        }
        
        QPushButton {
            background-color: #252525;
            border: 1px solid #2d5016;
            border-radius: 4px;
            padding: 8px 16px;
            color: #c5c5c5;
            font-size: 14px;
        }
        
        QPushButton:hover {
            background-color: #2d5016;
            border: 1px solid #5a8c3a;
        }
        
        QPushButton:pressed {
            background-color: #1a3010;
        }
        
        QPushButton#primaryButton {
            background-color: #2d5016;
            color: #a8d08d;
            font-weight: bold;
        }
        
        QPushButton#primaryButton:hover {
            background-color: #3d6b21;
        }
        
        #entryList {
            background-color: transparent;
            border: none;
        }
        
        #entryList::item {
            background-color: #1e1e1e;
            border: 1px solid #2a2a2a;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 10px;
            min-height: 40px;
        }
        
        #entryList::item:hover {
            background-color: #252525;
            border: 1px solid #3d6b21;
        }
        
        #entryList::item:selected {
            background-color: #2d5016;
            border: 1px solid #5a8c3a;
        }
        
        QTextEdit {
            background-color: #1e1e1e;
            border: 1px solid #2d5016;
            border-radius: 4px;
            padding: 10px;
            color: #c5c5c5;
            font-size: 14px;
        }
        
        QSpinBox {
            background-color: #252525;
            border: 1px solid #2d5016;
            border-radius: 4px;
            padding: 5px;
            color: #c5c5c5;
        }
        
        QLabel {
            color: #c5c5c5;
        }
        
        QScrollBar:vertical {
            background-color: #1e1e1e;
            width: 12px;
            border-radius: 6px;
        }
        
        QScrollBar::handle:vertical {
            background-color: #2d5016;
            border-radius: 6px;
            min-height: 20px;
        }
        
        QScrollBar::handle:vertical:hover {
            background-color: #3d6b21;
        }
    )";

    setStyleSheet(styleSheet);
}

void MainWindow::setEntryList(const QStringList &entries)
{
    m_entryList = entries;
    m_entryListWidget->clear();

    if (entries.isEmpty())
    {
        QListWidgetItem *item = new QListWidgetItem(m_entryListWidget);
        item->setText("🌱 No entries yet\nClick 'New Entry' to plant your first thought");
        item->setTextAlignment(Qt::AlignCenter);
        item->setFlags(Qt::NoItemFlags);
    }
    else
    {
        for (const QString &entry : entries)
        {
            QListWidgetItem *item = new QListWidgetItem(entry, m_entryListWidget);
            item->setSizeHint(QSize(0, 80));
        }
    }
}

void MainWindow::setCurrentEntryTitle(const QString &title)
{
    m_currentEntryTitle = title;
    m_bookEditor->setEntryTitle(title);
    m_noteEditor->setEntryTitle(title);
}

void MainWindow::setCurrentContent(const QString &content)
{
    m_bookEditor->setContent(content);
    m_noteEditor->setContent(content);
}

void MainWindow::setCurrentPage(int page)
{
    m_currentPage = page;
    m_bookEditor->setCurrentPage(page);
}

void MainWindow::setTotalPages(int total)
{
    m_totalPages = total;
    m_bookEditor->setTotalPages(total);
}

void MainWindow::setWordCount(int count)
{
    m_wordCount = count;
    m_bookEditor->setWordCount(count);
}

void MainWindow::setPasswordError(const QString &error)
{
    if (m_passwordDialog)
    {
        m_passwordDialog->setErrorMessage(error);
    }
}

void MainWindow::setShowPasswordError(bool show)
{
    if (m_passwordDialog)
    {
        m_passwordDialog->setShowError(show);
    }
}

QString MainWindow::getCurrentContent() const
{
    if (m_stackedWidget->currentWidget() == m_bookEditor)
    {
        return m_bookEditor->getContent();
    }
    else if (m_stackedWidget->currentWidget() == m_noteEditor)
    {
        return m_noteEditor->getContent();
    }
    return QString();
}

int MainWindow::getCurrentPage() const
{
    return m_bookEditor->getCurrentPage();
}

void MainWindow::onNewEntry()
{
    if (!m_modeDialog)
    {
        m_modeDialog = new ModeSelectionDialog(this);
        connect(m_modeDialog, &ModeSelectionDialog::modeSelected,
                this, &MainWindow::onModeDialogAccepted);
    }
    m_modeDialog->exec();
}

void MainWindow::onModeDialogAccepted(const QString &mode, const QString &title)
{
    QString data = mode + "|" + title;
    emit modeSelected(data, "");
}

void MainWindow::onEntryItemClicked(QListWidgetItem *item)
{
    int index = m_entryListWidget->row(item);
    emit entrySelected(index);
}

void MainWindow::onDeleteEntry()
{
    int index = m_entryListWidget->currentRow();
    if (index >= 0)
    {
        emit deleteEntryClicked(index);
    }
}

void MainWindow::onSaveContent()
{
    QString content = getCurrentContent();
    emit saveContent(content);
}

void MainWindow::onSearchTextChanged(const QString &text)
{
    emit searchEntries(text);
}

void MainWindow::onClearSearch()
{
    m_searchBox->clear();
    emit clearSearch();
}

void MainWindow::onPreviousPage()
{
    if (m_currentPage > 1)
    {
        emit pageChanged(m_currentPage - 1);
    }
}

void MainWindow::onNextPage()
{
    if (m_currentPage < m_totalPages)
    {
        emit pageChanged(m_currentPage + 1);
    }
}

void MainWindow::onAddPage()
{
    emit addNewPage();
}

void MainWindow::onBackToList()
{
    m_stackedWidget->setCurrentWidget(m_listViewWidget);
    emit backToList();
}

// ============ PasswordDialog Implementation ============
PasswordDialog::PasswordDialog(QWidget *parent)
    : QDialog(parent)
{
    setWindowTitle("Unlock NoteQuarry");
    setModal(true);
    setFixedSize(400, 280);

    QVBoxLayout *mainLayout = new QVBoxLayout(this);
    mainLayout->setSpacing(20);
    mainLayout->setContentsMargins(30, 30, 30, 30);

    // Title
    QLabel *titleLabel = new QLabel("🔒 Unlock NoteQuarry");
    titleLabel->setAlignment(Qt::AlignCenter);
    titleLabel->setStyleSheet("font-size: 22px; font-weight: bold; color: #a8d08d;");

    QLabel *subtitleLabel = new QLabel("Enter your master password");
    subtitleLabel->setAlignment(Qt::AlignCenter);
    subtitleLabel->setStyleSheet("font-size: 13px; color: #7a9b68;");

    // Separator
    QFrame *separator = new QFrame;
    separator->setFrameShape(QFrame::HLine);
    separator->setStyleSheet("background-color: #2d5016;");
    separator->setFixedHeight(1);

    // Password input
    m_passwordInput = new QLineEdit;
    m_passwordInput->setEchoMode(QLineEdit::Password);
    m_passwordInput->setPlaceholderText("Master password...");
    connect(m_passwordInput, &QLineEdit::returnPressed, this, &PasswordDialog::accept);

    // Error widget
    m_errorWidget = new QWidget;
    m_errorWidget->setVisible(false);
    m_errorWidget->setStyleSheet("background-color: #3d1616; border: 1px solid #ff6b6b; border-radius: 4px;");
    QHBoxLayout *errorLayout = new QHBoxLayout(m_errorWidget);
    errorLayout->setContentsMargins(8, 8, 8, 8);

    QLabel *errorIcon = new QLabel("⚠️");
    m_errorLabel = new QLabel;
    m_errorLabel->setStyleSheet("color: #ff6b6b; font-size: 12px;");

    errorLayout->addWidget(errorIcon);
    errorLayout->addWidget(m_errorLabel);
    errorLayout->addStretch();

    // Unlock button
    m_unlockButton = new QPushButton("Unlock");
    m_unlockButton->setObjectName("primaryButton");
    connect(m_unlockButton, &QPushButton::clicked, this, &PasswordDialog::accept);

    // Info label
    QLabel *infoLabel = new QLabel("First time? Any password will create a new vault.");
    infoLabel->setAlignment(Qt::AlignCenter);
    infoLabel->setWordWrap(true);
    infoLabel->setStyleSheet("font-size: 11px; color: #5a7a4a;");

    mainLayout->addWidget(titleLabel);
    mainLayout->addWidget(subtitleLabel);
    mainLayout->addWidget(separator);
    mainLayout->addWidget(m_passwordInput);
    mainLayout->addWidget(m_errorWidget);
    mainLayout->addWidget(m_unlockButton);
    mainLayout->addWidget(infoLabel);
    mainLayout->addStretch();

    setStyleSheet(R"(
        QDialog {
            background-color: #1e1e1e;
            border: 1px solid #2d5016;
            border-radius: 12px;
        }
    )");
}

QString PasswordDialog::getPassword() const
{
    return m_passwordInput->text();
}

void PasswordDialog::setErrorMessage(const QString &message)
{
    m_errorLabel->setText(message);
}

void PasswordDialog::setShowError(bool show)
{
    m_errorWidget->setVisible(show);
}

void PasswordDialog::accept()
{
    emit passwordSubmitted(m_passwordInput->text());
    QDialog::accept();
}

// ============ ModeSelectionDialog Implementation ============
ModeSelectionDialog::ModeSelectionDialog(QWidget *parent)
    : QDialog(parent)
{
    setWindowTitle("Create New Entry");
    setModal(true);
    setFixedSize(400, 300);

    QVBoxLayout *mainLayout = new QVBoxLayout(this);
    mainLayout->setSpacing(15);
    mainLayout->setContentsMargins(20, 20, 20, 20);

    // Title
    QLabel *titleLabel = new QLabel("Create New Entry");
    titleLabel->setAlignment(Qt::AlignCenter);
    titleLabel->setStyleSheet("font-size: 20px; font-weight: bold; color: #a8d08d;");

    // Separator
    QFrame *separator = new QFrame;
    separator->setFrameShape(QFrame::HLine);
    separator->setStyleSheet("background-color: #2d5016;");
    separator->setFixedHeight(1);

    // Title input
    QLabel *titlePrompt = new QLabel("Entry Title:");
    titlePrompt->setStyleSheet("font-size: 14px; color: #c5c5c5;");

    m_titleInput = new QLineEdit;
    m_titleInput->setPlaceholderText("Enter title...");

    // Mode selection
    QLabel *modePrompt = new QLabel("Select Mode:");
    modePrompt->setStyleSheet("font-size: 14px; color: #c5c5c5;");

    QHBoxLayout *modeLayout = new QHBoxLayout;
    modeLayout->setSpacing(15);

    // Book mode button
    QPushButton *bookButton = new QPushButton("📚\nBook Mode\nPaginated writing");
    bookButton->setFixedSize(150, 100);
    bookButton->setStyleSheet(R"(
        QPushButton {
            background-color: #252525;
            border: 2px solid #2d5016;
            border-radius: 8px;
            font-size: 14px;
            font-weight: 600;
            color: #a8d08d;
        }
        QPushButton:hover {
            background-color: #1a3010;
            border: 2px solid #5a8c3a;
        }
    )");
    connect(bookButton, &QPushButton::clicked, this, &ModeSelectionDialog::onBookModeClicked);

    // Note mode button
    QPushButton *noteButton = new QPushButton("📝\nNote Mode\nFreeform notes");
    noteButton->setFixedSize(150, 100);
    noteButton->setStyleSheet(R"(
        QPushButton {
            background-color: #252525;
            border: 2px solid #2d5016;
            border-radius: 8px;
            font-size: 14px;
            font-weight: 600;
            color: #a8d08d;
        }
        QPushButton:hover {
            background-color: #1a3010;
            border: 2px solid #5a8c3a;
        }
    )");
    connect(noteButton, &QPushButton::clicked, this, &ModeSelectionDialog::onNoteModeClicked);

    modeLayout->addWidget(bookButton);
    modeLayout->addWidget(noteButton);

    // Cancel button
    QPushButton *cancelButton = new QPushButton("Cancel");
    connect(cancelButton, &QPushButton::clicked, this, &QDialog::reject);

    mainLayout->addWidget(titleLabel);
    mainLayout->addWidget(separator);
    mainLayout->addWidget(titlePrompt);
    mainLayout->addWidget(m_titleInput);
    mainLayout->addWidget(modePrompt);
    mainLayout->addLayout(modeLayout);
    mainLayout->addStretch();
    mainLayout->addWidget(cancelButton, 0, Qt::AlignRight);

    setStyleSheet(R"(
        QDialog {
            background-color: #1e1e1e;
            border: 1px solid #2d5016;
            border-radius: 12px;
        }
    )");
}

void ModeSelectionDialog::onBookModeClicked()
{
    emit modeSelected("BOOK", m_titleInput->text());
    accept();
}

void ModeSelectionDialog::onNoteModeClicked()
{
    emit modeSelected("NOTE", m_titleInput->text());
    accept();
}

// ============ BookEditor Implementation ============
BookEditor::BookEditor(QWidget *parent)
    : QWidget(parent), m_currentPage(1), m_totalPages(1)
{
    QVBoxLayout *mainLayout = new QVBoxLayout(this);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    // Header
    QWidget *headerWidget = new QWidget;
    headerWidget->setStyleSheet("background-color: #1e1e1e;");
    headerWidget->setFixedHeight(60);
    QHBoxLayout *headerLayout = new QHBoxLayout(headerWidget);
    headerLayout->setContentsMargins(15, 15, 15, 15);

    QPushButton *backButton = new QPushButton("← Back");
    connect(backButton, &QPushButton::clicked, this, &BookEditor::backClicked);

    m_titleLabel = new QLabel;
    m_titleLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #a8d08d;");

    QPushButton *saveButton = new QPushButton("Save");
    saveButton->setObjectName("primaryButton");
    connect(saveButton, &QPushButton::clicked, [this]()
            { emit saveClicked(m_contentEditor->toPlainText()); });

    headerLayout->addWidget(backButton);
    headerLayout->addWidget(m_titleLabel);
    headerLayout->addStretch();
    headerLayout->addWidget(saveButton);

    // Page info bar
    QWidget *infoBar = new QWidget;
    infoBar->setStyleSheet("background-color: #1a1a1a;");
    infoBar->setFixedHeight(40);
    QHBoxLayout *infoLayout = new QHBoxLayout(infoBar);
    infoLayout->setContentsMargins(10, 10, 10, 10);

    m_pageInfoLabel = new QLabel;
    m_wordCountLabel = new QLabel;

    infoLayout->addWidget(m_pageInfoLabel);
    infoLayout->addStretch();
    infoLayout->addWidget(m_wordCountLabel);

    // Content editor
    QScrollArea *scrollArea = new QScrollArea;
    scrollArea->setWidgetResizable(true);
    scrollArea->setFrameShape(QFrame::NoFrame);

    QWidget *editorContainer = new QWidget;
    QVBoxLayout *editorLayout = new QVBoxLayout(editorContainer);
    editorLayout->setContentsMargins(30, 30, 30, 30);

    m_contentEditor = new QTextEdit;
    m_contentEditor->setMinimumHeight(500);
    connect(m_contentEditor, &QTextEdit::textChanged, [this]()
            { emit contentChanged(m_contentEditor->toPlainText()); });

    editorLayout->addWidget(m_contentEditor);
    scrollArea->setWidget(editorContainer);

    // Toolbar
    QWidget *toolbar = new QWidget;
    toolbar->setStyleSheet("background-color: #1e1e1e;");
    toolbar->setFixedHeight(50);
    QHBoxLayout *toolbarLayout = new QHBoxLayout(toolbar);
    toolbarLayout->setContentsMargins(10, 10, 10, 10);

    QPushButton *imageButton = new QPushButton("🖼️ Insert Image");
    connect(imageButton, &QPushButton::clicked, this, &BookEditor::insertImage);

    toolbarLayout->addWidget(imageButton);
    toolbarLayout->addStretch();

    // Navigation footer
    QWidget *footer = new QWidget;
    footer->setStyleSheet("background-color: #0d1f0a;");
    footer->setFixedHeight(60);
    QHBoxLayout *footerLayout = new QHBoxLayout(footer);
    footerLayout->setContentsMargins(15, 15, 15, 15);
    footerLayout->setAlignment(Qt::AlignCenter);

    m_prevButton = new QPushButton("◀ Previous");
    connect(m_prevButton, &QPushButton::clicked, this, &BookEditor::previousPage);

    m_pageSpinBox = new QSpinBox;
    m_pageSpinBox->setMinimum(1);
    m_pageSpinBox->setFixedWidth(80);
    m_pageSpinBox->setAlignment(Qt::AlignCenter);
    connect(m_pageSpinBox, QOverload<int>::of(&QSpinBox::valueChanged), [this](int value)
            {
        if (value != m_currentPage) {
            m_currentPage = value;
            emit pageChanged(value);
        } });

    m_nextButton = new QPushButton("Next ▶");
    connect(m_nextButton, &QPushButton::clicked, this, &BookEditor::nextPage);

    m_addPageButton = new QPushButton("+ New Page");
    connect(m_addPageButton, &QPushButton::clicked, this, &BookEditor::addPage);

    footerLayout->addWidget(m_prevButton);
    footerLayout->addWidget(m_pageSpinBox);
    footerLayout->addWidget(m_nextButton);
    footerLayout->addSpacing(20);
    footerLayout->addWidget(m_addPageButton);

    mainLayout->addWidget(headerWidget);
    mainLayout->addWidget(infoBar);
    mainLayout->addWidget(scrollArea);
    mainLayout->addWidget(toolbar);
    mainLayout->addWidget(footer);
}

void BookEditor::setEntryTitle(const QString &title)
{
    m_titleLabel->setText(title);
}

void BookEditor::setContent(const QString &content)
{
    m_contentEditor->setPlainText(content);
}

void BookEditor::setCurrentPage(int page)
{
    m_currentPage = page;
    m_pageSpinBox->setValue(page);
    m_pageInfoLabel->setText(QString("Page %1 of %2").arg(page).arg(m_totalPages));
    m_prevButton->setEnabled(page > 1);
    m_nextButton->setEnabled(page < m_totalPages);
}

void BookEditor::setTotalPages(int total)
{
    m_totalPages = total;
    m_pageSpinBox->setMaximum(total);
    m_pageInfoLabel->setText(QString("Page %1 of %2").arg(m_currentPage).arg(total));
    m_nextButton->setEnabled(m_currentPage < total);
}

void BookEditor::setWordCount(int count)
{
    m_wordCountLabel->setText(QString("Words: %1 / 800").arg(count));
    if (count > 800)
    {
        m_wordCountLabel->setStyleSheet("color: #ff6b6b;");
    }
    else
    {
        m_wordCountLabel->setStyleSheet("color: #7a9b68;");
    }
}

QString BookEditor::getContent() const
{
    return m_contentEditor->toPlainText();
}

int BookEditor::getCurrentPage() const
{
    return m_currentPage;
}

// ============ NoteEditor Implementation ============
NoteEditor::NoteEditor(QWidget *parent)
    : QWidget(parent)
{
    QVBoxLayout *mainLayout = new QVBoxLayout(this);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    // Header
    QWidget *headerWidget = new QWidget;
    headerWidget->setStyleSheet("background-color: #1e1e1e;");
    headerWidget->setFixedHeight(60);
    QHBoxLayout *headerLayout = new QHBoxLayout(headerWidget);
    headerLayout->setContentsMargins(15, 15, 15, 15);

    QPushButton *backButton = new QPushButton("← Back");
    connect(backButton, &QPushButton::clicked, this, &NoteEditor::backClicked);

    m_titleLabel = new QLabel;
    m_titleLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #a8d08d;");

    QPushButton *saveButton = new QPushButton("Save");
    saveButton->setObjectName("primaryButton");
    connect(saveButton, &QPushButton::clicked, [this]()
            { emit saveClicked(m_contentEditor->toPlainText()); });

    headerLayout->addWidget(backButton);
    headerLayout->addWidget(m_titleLabel);
    headerLayout->addStretch();
    headerLayout->addWidget(saveButton);

    // Toolbar
    QWidget *toolbar = new QWidget;
    toolbar->setStyleSheet("background-color: #1e1e1e;");
    toolbar->setFixedHeight(50);
    QHBoxLayout *toolbarLayout = new QHBoxLayout(toolbar);
    toolbarLayout->setContentsMargins(10, 10, 10, 10);

    QPushButton *checkboxButton = new QPushButton("☑ Add Checkbox");
    connect(checkboxButton, &QPushButton::clicked, this, &NoteEditor::addCheckbox);

    QPushButton *imageButton = new QPushButton("🖼️ Insert Image");
    connect(imageButton, &QPushButton::clicked, this, &NoteEditor::insertImage);

    toolbarLayout->addWidget(checkboxButton);
    toolbarLayout->addWidget(imageButton);
    toolbarLayout->addStretch();

    // Content editor
    QScrollArea *scrollArea = new QScrollArea;
    scrollArea->setWidgetResizable(true);
    scrollArea->setFrameShape(QFrame::NoFrame);

    QWidget *editorContainer = new QWidget;
    QVBoxLayout *editorLayout = new QVBoxLayout(editorContainer);
    editorLayout->setContentsMargins(30, 30, 30, 30);

    m_contentEditor = new QTextEdit;
    m_contentEditor->setMinimumHeight(500);

    editorLayout->addWidget(m_contentEditor);
    scrollArea->setWidget(editorContainer);

    mainLayout->addWidget(headerWidget);
    mainLayout->addWidget(toolbar);
    mainLayout->addWidget(scrollArea);
}

void NoteEditor::setEntryTitle(const QString &title)
{
    m_titleLabel->setText(title);
}

void NoteEditor::setContent(const QString &content)
{
    m_contentEditor->setPlainText(content);
}

QString NoteEditor::getContent() const
{
    return m_contentEditor->toPlainText();
}