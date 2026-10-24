// PdfPages: a PDF's pages and marks changed through PDFium's own interface (ADR-094 decision 3, L9.5).
//
// The reader shows a change at once: which of the file's pages are kept, in what order, each turned by quarter turns,
// and the marks made on them or taken away (PdfMarks). This writes it: the file is read through PDFium, the marks
// written and taken away, its pages turned, deleted and moved as the reader's list says, and written back through a
// temporary file renamed into place, so a failed save never leaves half a file. A PDF with a password or other
// protection is refused, so saving never strips it.
//
// Qt PDF keeps the file it shows open, and Windows will not replace an open file, so while saving the reader shows a
// copy (`copyForSaving`), and lets it go after (`discard`).

#pragma once

#include <QObject>
#include <QUrl>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>

namespace qormedia {

class PdfPages : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString error READ error NOTIFY changed)

public:
    explicit PdfPages(QObject *parent = nullptr) : QObject(parent) {}

    QString error() const { return m_error; }

    /// Whether the file at `path` is a PDF whose pages this can change: it opens, with no password, and carries no
    /// protection of its own.
    Q_INVOKABLE static bool changeable(const QString &path);

    /// Write `path` again with its pages as `layout` says: a list of {page, turn}, `page` a page's index in the file as
    /// it is now, `turn` the quarter turns clockwise to add to it (0 to 3). Each page appears once at most; a page left
    /// out is deleted; at least one stays. `added` are marks to write and `removed` the PDF's own marks to take away, as
    /// PdfMarks describes them, on the pages as the file has them now. True when saved; otherwise `error` says why and
    /// the file is as it was.
    Q_INVOKABLE bool save(const QString &path, const QVariantList &layout, const QVariantList &added = {},
                          const QVariantList &removed = {});

    /// A copy of `path` beside the system's temporary files, for the reader to show while `path` is written; an empty
    /// URL when it cannot be made.
    Q_INVOKABLE QUrl copyForSaving(const QString &path);
    /// Delete a copy made by copyForSaving, once nothing shows it. False when it is still there.
    Q_INVOKABLE bool discard(const QUrl &copy);

signals:
    void changed();

private:
    bool fail(const QString &why);
    QString m_error;
};

}  // namespace qormedia
