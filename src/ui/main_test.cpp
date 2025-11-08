// src/ui/main_test.cpp
// Standalone Qt application for testing UI without Rust
#include "mainwindow.h"
#include <QApplication>
#include <QDebug>
#include <QTimer>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);

    qDebug() << "Starting NoteQuarry Qt Test...";

    MainWindow window;
    window.show();

    // Test: Connect signals to debug output
    QObject::connect(&window, &MainWindow::passwordSubmitted, [](const QString &pwd)
                     { qDebug() << "Password submitted:" << (pwd.isEmpty() ? "<empty>" : "<hidden>"); });

    QObject::connect(&window, &MainWindow::newEntryClicked, []()
                     { qDebug() << "New entry clicked"; });

    QObject::connect(&window, &MainWindow::entrySelected, [](int index)
                     { qDebug() << "Entry selected:" << index; });

    QObject::connect(&window, &MainWindow::saveContent, [](const QString &content)
                     { qDebug() << "Save content:" << content.left(50) << "..."; });

    // Test: Populate with dummy data
    QStringList dummyEntries = {
        "📚 My First Book Entry",
        "📝 Quick Notes",
        "📚 Another Book",
        "📝 Todo List"};

    // Set dummy entries after password dialog (simulate)
    (100, [&window, dummyEntries]()
     {
        window.setEntryList(dummyEntries);
        qDebug() << "Loaded" << dummyEntries.size() << "dummy entries"; });

    return app.exec();
}