// mainwindow.cpp
#include "mainwindow.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QScrollArea>
#include <QFrame>
#include <QStyle>
#include <QApplication>
#include <QRegularExpression>
#include <QKeyEvent>
#include <QMessageBox>
#include <QMenu>

// ============ MainWindow Implementation ============
MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent), m_stackedWidget(new QStackedWidget(this)), m_statusBar(nullptr), m_passwordDialog(nullptr), m_listViewWidget(nullptr), m_bookEditor(nullptr), m_noteEditor(nullptr), m_modeDialog(nullptr), m_currentPage(1), m_totalPages(1), m_wordCount(0)
{
    setupUI();
    setupStatusBar();
    applyDarkTheme();
    updateWindowTitle();

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

void MainWindow::setupMenuBar()
{
    QMenuBar *menuBar = new QMenuBar(this);
    setMenuBar(menuBar);

    // File Menu
    QMenu *fileMenu = menuBar->addMenu(tr("&File"));

    m_newEntryAction = new QAction(tr("&New Entry..."), this);
    m_newEntryAction->setShortcut(QKeySequence::New);
    connect(m_newEntryAction, &QAction::triggered, this, &MainWindow::onNewEntry);
    fileMenu->addAction(m_newEntryAction);

    m_saveAction = new QAction(tr("&Save"), this);
    m_saveAction->setShortcut(QKeySequence::Save);
    m_saveAction->setEnabled(false);
    connect(m_saveAction, &QAction::triggered, this, &MainWindow::onSaveContent);
    fileMenu->addAction(m_saveAction);

    fileMenu->addSeparator();

    QAction *exitAction = new QAction(tr("E&xit"), this);
    exitAction->setShortcut(QKeySequence::Quit);
    connect(exitAction, &QAction::triggered, this, &QMainWindow::close);
    fileMenu->addAction(exitAction);

    // Edit Menu
    QMenu *editMenu = menuBar->addMenu(tr("&Edit"));

    QAction *undoAction = editMenu->addAction(tr("&Undo"));
    undoAction->setShortcut(QKeySequence::Undo);

    QAction *redoAction = editMenu->addAction(tr("&Redo"));
    redoAction->setShortcut(QKeySequence::Redo);

    editMenu->addSeparator();

    QAction *cutAction = editMenu->addAction(tr("Cu&t"));
    cutAction->setShortcut(QKeySequence::Cut);

    QAction *copyAction = editMenu->addAction(tr("&Copy"));
    copyAction->setShortcut(QKeySequence::Copy);

    QAction *pasteAction = editMenu->addAction(tr("&Paste"));
    pasteAction->setShortcut(QKeySequence::Paste);

    // View Menu
    QMenu *viewMenu = menuBar->addMenu(tr("&View"));

    m_backAction = new QAction(tr("&Back to List"), this);
    m_backAction->setShortcut(QKeySequence(Qt::ALT | Qt::Key_Left));
    m_backAction->setEnabled(false);
    connect(m_backAction, &QAction::triggered, this, &MainWindow::onBackToList);
    viewMenu->addAction(m_backAction);

    // Help Menu
    QMenu *helpMenu = menuBar->addMenu(tr("&Help"));

    QAction *aboutAction = new QAction(tr("&About NoteQuarry"), this);
    connect(aboutAction, &QAction::triggered, this, [this]()
            { QMessageBox::about(this, tr("About NoteQuarry"),
                                 tr("NoteQuarry - Your Personal Journal\n\n"
                                    "A secure, encrypted journaling application.")); });
    helpMenu->addAction(aboutAction);
}


void MainWindow::setupStatusBar()
{
    m_statusBar = statusBar();
    m_statusBar->showMessage(tr("Ready"));
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
    headerWidget->setMinimumHeight(80);
    QHBoxLayout *headerLayout = new QHBoxLayout(headerWidget);
    headerLayout->setContentsMargins(20, 20, 20, 20);
    headerLayout->setSpacing(15);

    QLabel *titleLabel = new QLabel(tr("🌿 NoteQuarry"));
    titleLabel->setObjectName("appTitle");

    m_searchBox = new QLineEdit;
    m_searchBox->setPlaceholderText(tr("🔍 Search entries..."));
    m_searchBox->setMinimumWidth(250);
    m_searchBox->setMaximumWidth(400);
    m_searchBox->setClearButtonEnabled(true);
    connect(m_searchBox, &QLineEdit::textChanged, this, &MainWindow::onSearchTextChanged);

    m_newEntryButton = new QPushButton(tr("+ New Entry"));
    m_newEntryButton->setObjectName("primaryButton");
    m_newEntryButton->setMinimumWidth(120);
    connect(m_newEntryButton, &QPushButton::clicked, this, &MainWindow::onNewEntry);

    headerLayout->addWidget(titleLabel);
    headerLayout->addStretch();
    headerLayout->addWidget(m_searchBox);
    headerLayout->addWidget(m_newEntryButton);

    // Entry list
    QScrollArea *scrollArea = new QScrollArea;
    scrollArea->setWidgetResizable(true);
    scrollArea->setFrameShape(QFrame::NoFrame);
    scrollArea->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);

    QWidget *listContainer = new QWidget;
    QVBoxLayout *listLayout = new QVBoxLayout(listContainer);
    listLayout->setContentsMargins(30, 30, 30, 30);
    listLayout->setSpacing(12);

    m_entryListWidget = new QListWidget;
    m_entryListWidget->setObjectName("entryList");
    m_entryListWidget->setAlternatingRowColors(true);
    m_entryListWidget->setSelectionMode(QAbstractItemView::SingleSelection);
    m_entryListWidget->setContextMenuPolicy(Qt::CustomContextMenu);
    connect(m_entryListWidget, &QListWidget::itemClicked, this, &MainWindow::onEntryItemClicked);
    connect(m_entryListWidget, &QListWidget::itemDoubleClicked, this, &MainWindow::onEntryItemClicked);
    connect(m_entryListWidget, &QListWidget::customContextMenuRequested, this, [this](const QPoint &pos)
            {
        QListWidgetItem *item = m_entryListWidget->itemAt(pos);
        if (item) {
            QMenu contextMenu;
            QAction *deleteAction = contextMenu.addAction(tr("Delete Entry"));
            connect(deleteAction, &QAction::triggered, this, &MainWindow::onDeleteEntry);
            contextMenu.exec(m_entryListWidget->mapToGlobal(pos));
        } });

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
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial;
        }
        
        QMenuBar {
            background-color: #1e1e1e;
            color: #c5c5c5;
            border-bottom: 1px solid #2d5016;
        }
        
        QMenuBar::item {
            padding: 4px 12px;
            background-color: transparent;
        }
        
        QMenuBar::item:selected {
            background-color: #2d5016;
        }
        
        QMenu {
            background-color: #1e1e1e;
            color: #c5c5c5;
            border: 1px solid #2d5016;
        }
        
        QMenu::item:selected {
            background-color: #2d5016;
        }
        
        QToolBar {
            background-color: #1e1e1e;
            border-bottom: 1px solid #2d5016;
            spacing: 3px;
        }
        
        QStatusBar {
            background-color: #1e1e1e;
            color: #7a9b68;
            border-top: 1px solid #2d5016;
        }
        
        #headerBar {
            background: qlineargradient(x1:0, y1:0, x2:1, y2:1,
                stop:0 #1a3d14, stop:0.5 #2d5016, stop:1 #3d6b21);
        }
        
        #appTitle {
            font-size: 26px;
            font-weight: 700;
            color: #e8f5e3;
        }
        
        QLineEdit {
            background-color: #252525;
            border: 2px solid #2d5016;
            border-radius: 6px;
            padding: 8px 12px;
            color: #c5c5c5;
            font-size: 14px;
            selection-background-color: #2d5016;
        }
        
        QLineEdit:focus {
            border: 2px solid #5a8c3a;
        }
        
        QPushButton {
            background-color: #252525;
            border: 2px solid #2d5016;
            border-radius: 6px;
            padding: 8px 16px;
            color: #c5c5c5;
            font-size: 14px;
            font-weight: 500;
            min-height: 32px;
        }
        
        QPushButton:hover {
            background-color: #2d5016;
            border: 2px solid #5a8c3a;
        }
        
        QPushButton:pressed {
            background-color: #1a3010;
        }
        
        QPushButton:disabled {
            background-color: #1a1a1a;
            border: 2px solid #1a1a1a;
            color: #555555;
        }
        
        QPushButton#primaryButton {
            background-color: #2d5016;
            color: #e8f5e3;
            font-weight: 600;
            border: 2px solid #3d6b21;
        }
        
        QPushButton#primaryButton:hover {
            background-color: #3d6b21;
            border: 2px solid #5a8c3a;
        }
        
        #entryList {
            background-color: transparent;
            border: none;
            outline: none;
        }
        
        #entryList::item {
            background-color: #1e1e1e;
            border: 2px solid #2a2a2a;
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 8px;
            min-height: 60px;
        }
        
        #entryList::item:hover {
            background-color: #252525;
            border: 2px solid #3d6b21;
        }
        
        #entryList::item:selected {
            background-color: #2d5016;
            border: 2px solid #5a8c3a;
        }
        
        #entryList::item:alternate {
            background-color: #1a1a1a;
        }
        
        QTextEdit {
            background-color: #252525;
            border: 2px solid #2d5016;
            border-radius: 6px;
            padding: 12px;
            color: #c5c5c5;
            font-size: 14px;
            font-family: "Consolas", "Monaco", "Courier New", monospace;
            line-height: 1.6;
            selection-background-color: #2d5016;
        }
        
        QTextEdit:focus {
            border: 2px solid #5a8c3a;
        }
        
        QSpinBox {
            background-color: #252525;
            border: 2px solid #2d5016;
            border-radius: 6px;
            padding: 6px;
            color: #c5c5c5;
            font-size: 14px;
        }
        
        QSpinBox:focus {
            border: 2px solid #5a8c3a;
        }
        
        QSpinBox::up-button, QSpinBox::down-button {
            background-color: #2d5016;
            border: none;
            width: 20px;
        }
        
        QSpinBox::up-button:hover, QSpinBox::down-button:hover {
            background-color: #3d6b21;
        }
        
        QLabel {
            color: #c5c5c5;
        }
        
        QScrollBar:vertical {
            background-color: #1e1e1e;
            width: 14px;
            border-radius: 7px;
            margin: 2px;
        }
        
        QScrollBar::handle:vertical {
            background-color: #2d5016;
            border-radius: 7px;
            min-height: 30px;
        }
        
        QScrollBar::handle:vertical:hover {
            background-color: #3d6b21;
        }
        
        QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
            height: 0px;
        }
        
        QScrollBar:horizontal {
            background-color: #1e1e1e;
            height: 14px;
            border-radius: 7px;
            margin: 2px;
        }
        
        QScrollBar::handle:horizontal {
            background-color: #2d5016;
            border-radius: 7px;
            min-width: 30px;
        }
        
        QScrollBar::handle:horizontal:hover {
            background-color: #3d6b21;
        }
        
        QFrame[frameShape="4"] { /* HLine */
            background-color: #2d5016;
            max-height: 1px;
        }
    )";

    setStyleSheet(styleSheet);
}

void MainWindow::updateWindowTitle()
{
    if (m_stackedWidget->currentWidget() == m_listViewWidget)
    {
        setWindowTitle(tr("NoteQuarry - Your Personal Journal"));
    }
    else
    {
        setWindowTitle(tr("NoteQuarry - %1").arg(m_currentEntryTitle));
    }
}

void MainWindow::setEntryList(const QStringList &entries)
{
    m_entryList = entries;
    m_entryListWidget->clear();

    if (entries.isEmpty())
    {
        QWidget *emptyWidget = new QWidget;
        QVBoxLayout *layout = new QVBoxLayout(emptyWidget);
        layout->setAlignment(Qt::AlignCenter);
        layout->setContentsMargins(40, 60, 40, 60);  // ← ADD PADDING

        QLabel *icon = new QLabel("🌱");
        icon->setAlignment(Qt::AlignCenter);

        QLabel *text1 = new QLabel(tr("No entries yet"));
        text1->setAlignment(Qt::AlignCenter);
        text1->setStyleSheet("font-size: 20px; color: #7a9b68; font-weight: 600;");

        QLabel *text2 = new QLabel(tr("Click 'New Entry' to plant your first thought"));
        text2->setAlignment(Qt::AlignCenter);
        text2->setStyleSheet("font-size: 14px; color: #5a7a4a;");

        layout->addWidget(icon);
        layout->addWidget(text1);
        layout->addWidget(text2);

        QListWidgetItem *item = new QListWidgetItem(m_entryListWidget);
        item->setFlags(Qt::NoItemFlags);
        item->setSizeHint(QSize(0, 200));
        m_entryListWidget->setItemWidget(item, emptyWidget);
    }
    else
    {
        for (const QString &entry : entries)
        {
            QListWidgetItem *item = new QListWidgetItem(entry, m_entryListWidget);
            item->setSizeHint(QSize(0, 70));
            item->setFont(QFont(font().family(), 15));
        }
    }

    m_statusBar->showMessage(tr("%n entry(ies)", "", entries.size()));
}

void MainWindow::setCurrentEntryTitle(const QString &title)
{
    m_currentEntryTitle = title;
    m_bookEditor->setEntryTitle(title);
    m_noteEditor->setEntryTitle(title);
    updateWindowTitle();
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

void MainWindow::showListView()
{
    m_stackedWidget->setCurrentWidget(m_listViewWidget);
    updateWindowTitle();
}

void MainWindow::showBookEditor()
{
    m_stackedWidget->setCurrentWidget(m_bookEditor);
    updateWindowTitle();
}

void MainWindow::showNoteEditor()
{
    m_stackedWidget->setCurrentWidget(m_noteEditor);
    updateWindowTitle();
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
    if (index >= 0)
    {
        emit entrySelected(index);
    }
}

void MainWindow::onDeleteEntry()
{
    int index = m_entryListWidget->currentRow();
    if (index >= 0)
    {
        QMessageBox::StandardButton reply = QMessageBox::question(
            this,
            tr("Delete Entry"),
            tr("Are you sure you want to delete this entry?"),
            QMessageBox::Yes | QMessageBox::No);

        if (reply == QMessageBox::Yes)
        {
            emit deleteEntryClicked(index);
        }
    }
}

void MainWindow::onSaveContent()
{
    QString content = getCurrentContent();
    emit saveContent(content);
    m_statusBar->showMessage(tr("Entry saved"), 3000);
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
    showListView();
    emit backToList();
}

// ============ PasswordDialog Implementation ============
PasswordDialog::PasswordDialog(QWidget *parent)
    : QDialog(parent)
{
    setModal(true);
    setFixedSize(420, 320);
    setWindowFlags(Qt::Dialog | Qt::FramelessWindowHint);
  //setAttribute(Qt::WA_TranslucentBackground);

    QVBoxLayout *mainLayout = new QVBoxLayout(this);
    mainLayout->setSpacing(20);
    mainLayout->setContentsMargins(40, 40, 40, 40);

    // Title
    QLabel *titleLabel = new QLabel(tr("NoteQuarry"));
    titleLabel->setAlignment(Qt::AlignCenter);
    titleLabel->setStyleSheet("font-size: 24px; font-weight: 700; color: #a8d08d;");

    QLabel *subtitleLabel = new QLabel(tr("Enter your master password"));
    subtitleLabel->setAlignment(Qt::AlignCenter);
    subtitleLabel->setStyleSheet("font-size: 14px; color: #7a9b68;");

    // Separator
    QFrame *separator = new QFrame;
    separator->setFrameShape(QFrame::HLine);
    separator->setStyleSheet("background-color: #2d5016; max-height: 2px;");

    // Password input
    m_passwordInput = new QLineEdit;
    m_passwordInput->setEchoMode(QLineEdit::Password);
    m_passwordInput->setPlaceholderText(tr("Master password..."));
    m_passwordInput->setMinimumHeight(40);
    connect(m_passwordInput, &QLineEdit::returnPressed, this, &PasswordDialog::accept);

    // Error widget
    m_errorWidget = new QWidget;
    m_errorWidget->setVisible(false);
    m_errorWidget->setStyleSheet(
        "background-color: #3d1616; "
        "border: 2px solid #ff6b6b; "
        "border-radius: 6px;");
    QHBoxLayout *errorLayout = new QHBoxLayout(m_errorWidget);
    errorLayout->setContentsMargins(12, 12, 12, 12);

    QLabel *errorIcon = new QLabel("⚠️");
    errorIcon->setStyleSheet("font-size: 16px;");
    m_errorLabel = new QLabel;
    m_errorLabel->setStyleSheet("color: #ff6b6b; font-size: 13px;");
    m_errorLabel->setWordWrap(true);

    errorLayout->addWidget(errorIcon);
    errorLayout->addWidget(m_errorLabel, 1);

    // Buttons
    QHBoxLayout *buttonLayout = new QHBoxLayout;
    buttonLayout->setSpacing(10);

    QPushButton *closeButton = new QPushButton("✕");
    closeButton->setFixedSize(32, 32);
    closeButton->setStyleSheet(
        "QPushButton { "
        "  background-color: transparent; "
        "  color: #ff6b6b; "
        "  border: none; "
        "  border-radius: 16px; "
        "  font-size: 18px; "
        "  font-weight: bold; "
        "} "
        "QPushButton:hover { "
        "  background-color: #3d1616; "
        "}"
        );
    connect(closeButton, &QPushButton::clicked, []() {
        QApplication::quit();
    });

    // Add to a top bar layout
    QHBoxLayout *topBar = new QHBoxLayout;
    topBar->addStretch();
    topBar->addWidget(closeButton);

    m_cancelButton = new QPushButton(tr("Exit"));
    m_cancelButton->setStyleSheet(
        "QPushButton { "
        "  background-color: #3d1616; "
        "  color: #ff6b6b; "
        "  border: 2px solid #ff6b6b; "
        "  border-radius: 6px; "
        "  padding: 8px 16px; "
        "  font-weight: 600; "
        "} "
        "QPushButton:hover { "
        "  background-color: #4d2020; "
        "}"
        );
    connect(m_cancelButton, &QPushButton::clicked, [this]() {
        QApplication::quit();  // ← Exit the entire app
    });
    m_unlockButton = new QPushButton(tr("Unlock"));
    m_unlockButton->setObjectName("primaryButton");
    m_unlockButton->setMinimumWidth(100);

    connect(m_cancelButton, &QPushButton::clicked, this, &QDialog::reject);
    connect(m_unlockButton, &QPushButton::clicked, this, &PasswordDialog::accept);

    buttonLayout->addStretch();
    buttonLayout->addWidget(m_cancelButton);
    buttonLayout->addWidget(m_unlockButton);

    // Info label
    QLabel *infoLabel = new QLabel(tr("First time? Any password will create a new vault."));
    infoLabel->setAlignment(Qt::AlignCenter);
    infoLabel->setWordWrap(true);
    infoLabel->setStyleSheet("font-size: 12px; color: #5a7a4a;");

    mainLayout->addLayout(topBar);
    mainLayout->addWidget(titleLabel);
    mainLayout->addWidget(subtitleLabel);
    mainLayout->addWidget(separator);
    mainLayout->addSpacing(10);
    mainLayout->addWidget(m_passwordInput);
    mainLayout->addWidget(m_errorWidget);
    mainLayout->addSpacing(10);
    mainLayout->addLayout(buttonLayout);
    mainLayout->addWidget(infoLabel);
    mainLayout->addStretch();

    setStyleSheet(R"(
        QDialog {
            background-color: #1e1e1e;
            border: 2px solid #2d5016;
            border-radius: 12px;
        QLabel {
             background-color: transparent;
    }
        }
    )");

    m_passwordInput->setFocus();
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
    if (show)
    {
        adjustSize();
    }
}

void PasswordDialog::accept()
{
    QString password = m_passwordInput->text().trimmed();

    if (password.isEmpty())
    {
        setErrorMessage(tr("Password cannot be empty"));
        setShowError(true);
        return;  // ← Don't close dialog!
    }

    emit passwordSubmitted(password);
    QDialog::accept();  // ← Only close if password valid
}

void PasswordDialog::keyPressEvent(QKeyEvent *event)
{
    if (event->key() == Qt::Key_Escape)
    {
        reject();
    }
    else
    {
        QDialog::keyPressEvent(event);
    }
}

// ============ ModeSelectionDialog Implementation ============
ModeSelectionDialog::ModeSelectionDialog(QWidget *parent)
    : QDialog(parent)
{
    setWindowTitle(tr("Create New Entry"));
    setModal(true);
    setFixedSize(460, 340);
    setWindowFlags(windowFlags() & ~Qt::WindowContextHelpButtonHint);

    QVBoxLayout *mainLayout = new QVBoxLayout(this);
    mainLayout->setSpacing(20);
    mainLayout->setContentsMargins(30, 30, 30, 30);

    // Title
    QLabel *titleLabel = new QLabel(tr("Create New Entry"));
    titleLabel->setAlignment(Qt::AlignCenter);
    titleLabel->setStyleSheet("font-size: 22px; font-weight: 700; color: #a8d08d;");

    // Separator
    QFrame *separator = new QFrame;
    separator->setFrameShape(QFrame::HLine);
    separator->setStyleSheet("background-color: #2d5016; max-height: 2px;");

    // Title input
    QLabel *titlePrompt = new QLabel(tr("Entry Title:"));
    titlePrompt->setStyleSheet("font-size: 14px; color: #c5c5c5; font-weight: 600;");

    m_titleInput = new QLineEdit;
    m_titleInput->setPlaceholderText(tr("Enter title..."));
    m_titleInput->setMinimumHeight(40);

    // Mode selection
    QLabel *modePrompt = new QLabel(tr("Select Mode:"));
    modePrompt->setStyleSheet("font-size: 14px; color: #c5c5c5; font-weight: 600;");

    QHBoxLayout *modeLayout = new QHBoxLayout;
    modeLayout->setSpacing(20);

    // Book mode button
    m_bookButton = new QPushButton(tr("📚\n\nBook Mode\n\nPaginated writing"));
    m_bookButton->setFixedSize(170, 120);
    m_bookButton->setCursor(Qt::PointingHandCursor);
    m_bookButton->setStyleSheet(R"(
        QPushButton {
            background-color: #252525;
            border: 2px solid #2d5016;
            border-radius: 10px;
            font-size: 13px;
            font-weight: 600;
            color: #a8d08d;
            padding: 10px;
        }
        QPushButton:hover {
            background-color: #1a3010;
            border: 2px solid #5a8c3a;
        }
        QPushButton:pressed {
            background-color: #2d5016;
        }
    )");
    connect(m_bookButton, &QPushButton::clicked, this, &ModeSelectionDialog::onBookModeClicked);

    // Note mode button
    m_noteButton = new QPushButton(tr("📝\n\nNote Mode\n\nFreeform notes"));
    m_noteButton->setFixedSize(170, 120);
    m_noteButton->setCursor(Qt::PointingHandCursor);
    m_noteButton->setStyleSheet(R"(
        QPushButton {
            background-color: #252525;
            border: 2px solid #2d5016;
            border-radius: 10px;
            font-size: 13px;
            font-weight: 600;
            color: #a8d08d;
            padding: 10px;
        }
        QPushButton:hover {
            background-color: #1a3010;
            border: 2px solid #5a8c3a;
        }
        QPushButton:pressed {
            background-color: #2d5016;
        }
    )");
    connect(m_noteButton, &QPushButton::clicked, this, &ModeSelectionDialog::onNoteModeClicked);

    modeLayout->addWidget(m_bookButton);
    modeLayout->addWidget(m_noteButton);

    // Cancel button
    QHBoxLayout *buttonLayout = new QHBoxLayout;
    QPushButton *cancelButton = new QPushButton(tr("Cancel"));
    cancelButton->setMinimumWidth(80);
    connect(cancelButton, &QPushButton::clicked, [this]() {
        // Do nothing, or show a m  essage that password is required
        QMessageBox::information(this, tr("Password Required"),
                                 tr("A password is required to use NoteQuarry."));
    });

    buttonLayout->addStretch();
    buttonLayout->addWidget(cancelButton);

    mainLayout->addWidget(titleLabel);
    mainLayout->addWidget(separator);
    mainLayout->addSpacing(5);
    mainLayout->addWidget(titlePrompt);
    mainLayout->addWidget(m_titleInput);
    mainLayout->addSpacing(5);
    mainLayout->addWidget(modePrompt);
    mainLayout->addLayout(modeLayout);
    mainLayout->addStretch();
    mainLayout->addLayout(buttonLayout);

    setStyleSheet(R"(
        QDialog {
            background-color: #1e1e1e;
            border: 2px solid #2d5016;
            border-radius: 12px;
        }
    )");

    m_titleInput->setFocus();
}

void ModeSelectionDialog::onBookModeClicked()
{
    validateAndAccept();
    if (result() == QDialog::Accepted)
    {
        emit modeSelected("BOOK", m_titleInput->text());
    }
}

void ModeSelectionDialog::onNoteModeClicked()
{
    validateAndAccept();
    if (result() == QDialog::Accepted)
    {
        emit modeSelected("NOTE", m_titleInput->text());
    }
}

void ModeSelectionDialog::validateAndAccept()
{
    QString title = m_titleInput->text().trimmed();
    if (title.isEmpty())
    {
        QMessageBox::warning(this, tr("Empty Title"),
                             tr("Please enter a title for your entry."));
        m_titleInput->setFocus();
        return;
    }
    accept();
}

void ModeSelectionDialog::keyPressEvent(QKeyEvent *event)
{
    if (event->key() == Qt::Key_Escape)
    {
        reject();
    }
    else
    {
        QDialog::keyPressEvent(event);
    }
}

// ============ BookEditor Implementation ============
BookEditor::BookEditor(QWidget *parent)
    : QWidget(parent), m_currentPage(1), m_totalPages(1), m_wordCount(0)
{
    setupUI();
}

void BookEditor::setupUI()
{
    QVBoxLayout *mainLayout = new QVBoxLayout(this);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    // Header
    QWidget *headerWidget = new QWidget;
    headerWidget->setStyleSheet("background-color: #1e1e1e; border-bottom: 2px solid #2d5016;");
    headerWidget->setFixedHeight(70);
    QHBoxLayout *headerLayout = new QHBoxLayout(headerWidget);
    headerLayout->setContentsMargins(20, 15, 20, 15);
    headerLayout->setSpacing(15);

    m_backButton = new QPushButton(tr("← Back"));
    m_backButton->setMinimumWidth(80);
    connect(m_backButton, &QPushButton::clicked, this, &BookEditor::backClicked);

    m_titleLabel = new QLabel;
    m_titleLabel->setStyleSheet("font-size: 20px; font-weight: 700; color: #a8d08d;");

    m_saveButton = new QPushButton(tr("💾 Save"));
    m_saveButton->setObjectName("primaryButton");
    m_saveButton->setMinimumWidth(100);
    connect(m_saveButton, &QPushButton::clicked, [this]()
            { emit saveClicked(m_contentEditor->toPlainText()); });

    headerLayout->addWidget(m_backButton);
    headerLayout->addWidget(m_titleLabel);
    headerLayout->addStretch();
    headerLayout->addWidget(m_saveButton);

    // Page info bar
    QWidget *infoBar = new QWidget;
    infoBar->setStyleSheet("background-color: #1a1a1a; border-bottom: 1px solid #2d5016;");
    infoBar->setFixedHeight(45);
    QHBoxLayout *infoLayout = new QHBoxLayout(infoBar);
    infoLayout->setContentsMargins(20, 10, 20, 10);
    infoLayout->setSpacing(20);

    m_pageInfoLabel = new QLabel;
    m_pageInfoLabel->setStyleSheet("font-size: 14px; color: #c5c5c5; font-weight: 500;");

    m_wordCountLabel = new QLabel;
    m_wordCountLabel->setStyleSheet("font-size: 14px; font-weight: 500;");

    infoLayout->addWidget(m_pageInfoLabel);
    infoLayout->addStretch();
    infoLayout->addWidget(m_wordCountLabel);

    // Content editor
    QScrollArea *scrollArea = new QScrollArea;
    scrollArea->setWidgetResizable(true);
    scrollArea->setFrameShape(QFrame::NoFrame);
    scrollArea->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);

    QWidget *editorContainer = new QWidget;
    QVBoxLayout *editorLayout = new QVBoxLayout(editorContainer);
    editorLayout->setContentsMargins(40, 30, 40, 30);

    m_contentEditor = new QTextEdit;
    m_contentEditor->setMinimumHeight(500);
    m_contentEditor->setAcceptRichText(false);
    m_contentEditor->setTabStopDistance(40);
    connect(m_contentEditor, &QTextEdit::textChanged, this, &BookEditor::onContentChanged);

    editorLayout->addWidget(m_contentEditor);
    scrollArea->setWidget(editorContainer);

    // Toolbar
    QWidget *toolbar = new QWidget;
    toolbar->setStyleSheet("background-color: #1e1e1e; border-top: 1px solid #2d5016;");
    toolbar->setFixedHeight(55);
    QHBoxLayout *toolbarLayout = new QHBoxLayout(toolbar);
    toolbarLayout->setContentsMargins(20, 10, 20, 10);
    toolbarLayout->setSpacing(10);

    m_imageButton = new QPushButton(tr("🖼️ Insert Image"));
    connect(m_imageButton, &QPushButton::clicked, this, &BookEditor::insertImage);

    toolbarLayout->addWidget(m_imageButton);
    toolbarLayout->addStretch();

    // Navigation footer
    QWidget *footer = new QWidget;
    footer->setStyleSheet("background-color: #0d1f0a; border-top: 2px solid #2d5016;");
    footer->setFixedHeight(70);
    QHBoxLayout *footerLayout = new QHBoxLayout(footer);
    footerLayout->setContentsMargins(20, 15, 20, 15);
    footerLayout->setSpacing(15);
    footerLayout->setAlignment(Qt::AlignCenter);

    m_prevButton = new QPushButton(tr("◀ Previous"));
    m_prevButton->setMinimumWidth(100);
    connect(m_prevButton, &QPushButton::clicked, this, &BookEditor::previousPage);

    m_pageSpinBox = new QSpinBox;
    m_pageSpinBox->setMinimum(1);
    m_pageSpinBox->setFixedWidth(80);
    m_pageSpinBox->setAlignment(Qt::AlignCenter);
    m_pageSpinBox->setButtonSymbols(QAbstractSpinBox::NoButtons);
    connect(m_pageSpinBox, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &BookEditor::onPageSpinBoxChanged);

    m_nextButton = new QPushButton(tr("Next ▶"));
    m_nextButton->setMinimumWidth(100);
    connect(m_nextButton, &QPushButton::clicked, this, &BookEditor::nextPage);

    m_addPageButton = new QPushButton(tr("+ New Page"));
    m_addPageButton->setObjectName("primaryButton");
    m_addPageButton->setMinimumWidth(120);
    connect(m_addPageButton, &QPushButton::clicked, this, &BookEditor::addPage);

    footerLayout->addWidget(m_prevButton);
    footerLayout->addWidget(m_pageSpinBox);
    footerLayout->addWidget(m_nextButton);
    footerLayout->addSpacing(30);
    footerLayout->addWidget(m_addPageButton);

    mainLayout->addWidget(headerWidget);
    mainLayout->addWidget(infoBar);
    mainLayout->addWidget(scrollArea);
    mainLayout->addWidget(toolbar);
    mainLayout->addWidget(footer);

    updateNavigationButtons();
    updatePageInfo();
    updateWordCount();
}

void BookEditor::setEntryTitle(const QString &title)
{
    m_titleLabel->setText(title);
}

void BookEditor::setContent(const QString &content)
{
    m_contentEditor->blockSignals(true);
    m_contentEditor->setPlainText(content);
    m_contentEditor->blockSignals(false);
    onContentChanged();
}

void BookEditor::setCurrentPage(int page)
{
    m_currentPage = page;
    m_pageSpinBox->blockSignals(true);
    m_pageSpinBox->setValue(page);
    m_pageSpinBox->blockSignals(false);
    updateNavigationButtons();
    updatePageInfo();
}

void BookEditor::setTotalPages(int total)
{
    m_totalPages = total;
    m_pageSpinBox->setMaximum(total);
    updateNavigationButtons();
    updatePageInfo();
}

void BookEditor::setWordCount(int count)
{
    m_wordCount = count;
    updateWordCount();
}

QString BookEditor::getContent() const
{
    return m_contentEditor->toPlainText();
}

int BookEditor::getCurrentPage() const
{
    return m_currentPage;
}

void BookEditor::onContentChanged()
{
    emit contentChanged(m_contentEditor->toPlainText());
}

void BookEditor::onPageSpinBoxChanged(int value)
{
    if (value != m_currentPage)
    {
        m_currentPage = value;
        emit pageChanged(value);
    }
}

void BookEditor::updateNavigationButtons()
{
    m_prevButton->setEnabled(m_currentPage > 1);
    m_nextButton->setEnabled(m_currentPage < m_totalPages);
}

void BookEditor::updatePageInfo()
{
    m_pageInfoLabel->setText(tr("Page %1 of %2").arg(m_currentPage).arg(m_totalPages));
}

void BookEditor::updateWordCount()
{
    m_wordCountLabel->setText(tr("Words: %1 / 800").arg(m_wordCount));
    if (m_wordCount > 800)
    {
        m_wordCountLabel->setStyleSheet("color: #ff6b6b; font-size: 14px; font-weight: 600;");
    }
    else
    {
        m_wordCountLabel->setStyleSheet("color: #7a9b68; font-size: 14px; font-weight: 500;");
    }
}

// ============ NoteEditor Implementation ============
NoteEditor::NoteEditor(QWidget *parent)
    : QWidget(parent)
{
    setupUI();
}

void NoteEditor::setupUI()
{
    QVBoxLayout *mainLayout = new QVBoxLayout(this);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    // Header
    QWidget *headerWidget = new QWidget;
    headerWidget->setStyleSheet("background-color: #1e1e1e; border-bottom: 2px solid #2d5016;");
    headerWidget->setFixedHeight(70);
    QHBoxLayout *headerLayout = new QHBoxLayout(headerWidget);
    headerLayout->setContentsMargins(20, 15, 20, 15);
    headerLayout->setSpacing(15);

    m_backButton = new QPushButton(tr("← Back"));
    m_backButton->setMinimumWidth(80);
    connect(m_backButton, &QPushButton::clicked, this, &NoteEditor::backClicked);

    m_titleLabel = new QLabel;
    m_titleLabel->setStyleSheet("font-size: 20px; font-weight: 700; color: #a8d08d;");

    m_saveButton = new QPushButton(tr("💾 Save"));
    m_saveButton->setObjectName("primaryButton");
    m_saveButton->setMinimumWidth(100);
    connect(m_saveButton, &QPushButton::clicked, [this]()
            { emit saveClicked(m_contentEditor->toPlainText()); });

    headerLayout->addWidget(m_backButton);
    headerLayout->addWidget(m_titleLabel);
    headerLayout->addStretch();
    headerLayout->addWidget(m_saveButton);

    // Toolbar
    QWidget *toolbar = new QWidget;
    toolbar->setStyleSheet("background-color: #1a1a1a; border-bottom: 1px solid #2d5016;");
    toolbar->setFixedHeight(55);
    QHBoxLayout *toolbarLayout = new QHBoxLayout(toolbar);
    toolbarLayout->setContentsMargins(20, 10, 20, 10);
    toolbarLayout->setSpacing(10);

    m_checkboxButton = new QPushButton(tr("☑ Add Checkbox"));
    connect(m_checkboxButton, &QPushButton::clicked, this, &NoteEditor::onAddCheckboxClicked);

    m_imageButton = new QPushButton(tr("🖼️ Insert Image"));
    connect(m_imageButton, &QPushButton::clicked, this, &NoteEditor::insertImage);

    toolbarLayout->addWidget(m_checkboxButton);
    toolbarLayout->addWidget(m_imageButton);
    toolbarLayout->addStretch();

    // Content editor
    QScrollArea *scrollArea = new QScrollArea;
    scrollArea->setWidgetResizable(true);
    scrollArea->setFrameShape(QFrame::NoFrame);
    scrollArea->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);

    QWidget *editorContainer = new QWidget;
    QVBoxLayout *editorLayout = new QVBoxLayout(editorContainer);
    editorLayout->setContentsMargins(40, 30, 40, 30);

    m_contentEditor = new QTextEdit;
    m_contentEditor->setMinimumHeight(500);
    m_contentEditor->setAcceptRichText(false);
    m_contentEditor->setTabStopDistance(40);
    connect(m_contentEditor, &QTextEdit::textChanged, this, &NoteEditor::onContentChanged);

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
    m_contentEditor->blockSignals(true);
    m_contentEditor->setPlainText(content);
    m_contentEditor->blockSignals(false);
}

QString NoteEditor::getContent() const
{
    return m_contentEditor->toPlainText();
}

void NoteEditor::onAddCheckboxClicked()
{
    QTextCursor cursor = m_contentEditor->textCursor();
    cursor.insertText("☐ ");
    emit addCheckbox();
}

void NoteEditor::onContentChanged()
{
    emit contentChanged(m_contentEditor->toPlainText());
}
