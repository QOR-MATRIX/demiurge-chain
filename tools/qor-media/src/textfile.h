// TextFile: the editor's file, read and saved faithfully (ADR-094 decision 3; docs/blueprints/library.md section 4).
//
// Read as UTF-8 (or UTF-16 when its byte-order mark says so), its line endings and byte-order mark remembered so a save
// writes them back as they were; refused when it is binary or larger than the editor holds. Saved through a temporary
// file renamed into place, so a failed save never leaves half a file.

#pragma once

#include <QObject>
#include <QQuickTextDocument>
#include <QtQml/qqmlregistration.h>

namespace qormedia {

class Highlighter;

class TextFile : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString path READ path NOTIFY changed)
    Q_PROPERTY(QString error READ error NOTIFY changed)
    Q_PROPERTY(QString language READ language NOTIFY changed)
    Q_PROPERTY(bool crlf READ crlf NOTIFY changed)

public:
    /// The largest file the editor opens: 16 MiB.
    static constexpr qint64 Limit = 16 * 1024 * 1024;

    explicit TextFile(QObject *parent = nullptr);

    QString path() const { return m_path; }
    QString error() const { return m_error; }
    QString language() const { return m_language; }
    bool crlf() const { return m_crlf; }

    /// The file's words, with its line breaks as `\n`; empty with `error` set when it cannot be opened here.
    Q_INVOKABLE QString load(const QString &path);
    /// Write `text` back to the file it came from, as it was encoded. True when it is saved. Refused when the file was
    /// not read as text, so a file the editor could not open is never overwritten by what was typed over nothing.
    Q_INVOKABLE bool save(const QString &text);
    /// Colour `document` for this file's language, in the theme's colours.
    Q_INVOKABLE void highlight(QQuickTextDocument *document, const QVariantMap &colors);

signals:
    void changed();

private:
    QString m_path;
    QString m_error;
    QString m_language;
    bool m_loaded = false;
    bool m_crlf = false;
    bool m_bom = false;
    bool m_utf16 = false;
    Highlighter *m_highlighter = nullptr;
};

}  // namespace qormedia
