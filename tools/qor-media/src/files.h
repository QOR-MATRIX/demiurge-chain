// Files: what QOR Media needs to know about a file, for QML (ADR-094).
//
// A file's kind by its extension, as the launcher's index shelves it (src-tauri/src/library/mod.rs `kind_of`); the
// files of the same kind beside it, for next and previous in the viewer; whether a document is text the editor opens;
// and handing a file to the program the system opens it with, or showing it in its folder.

#pragma once

#include <QObject>
#include <QStringList>
#include <QUrl>
#include <QtQml/qqmlregistration.h>

namespace qormedia {

class Files : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    explicit Files(QObject *parent = nullptr) : QObject(parent) {}

    /// video, audio, picture, document or other.
    Q_INVOKABLE static QString kindOf(const QString &path);
    /// Whether QOR Media's editor opens this document: plain text, Markdown and code.
    Q_INVOKABLE static bool isText(const QString &path);
    /// Whether QOR Media's reader opens this document: a PDF.
    Q_INVOKABLE static bool isPdf(const QString &path);
    /// The language the editor highlights a text file as: markdown, code or plain.
    Q_INVOKABLE static QString textLanguage(const QString &path);
    /// The files of the same kind in the same folder, in the order a person reads names (2 before 10). `path` itself is
    /// listed as given; the others with the system's separators.
    Q_INVOKABLE static QStringList siblings(const QString &path);
    Q_INVOKABLE static QUrl urlOf(const QString &path);
    Q_INVOKABLE static QString nameOf(const QString &path);
    /// Open in the program the system chooses for this kind of file.
    Q_INVOKABLE static bool openElsewhere(const QString &path);
};

}  // namespace qormedia
