// PdfDrawing: a PDF's pages drawn with their marks (ADR-094 decision 3, L9.5).
//
// Qt PDF's page image in QML draws a page without its annotations: highlights, notes, stamps, drawings and a form's
// filled fields do not show. Qt PDF's QPdfDocument draws them when asked, so the reader's pages are images drawn here,
// through an image provider the engine knows as "pdfpage": `image://pdfpage/<key>/<page>`, as wide as the image asks.
//
// A key names a PDF opened here, with its password when it has one; the password stays here and never reaches a URL.
// Closing a key lets go of its file at once, as saving needs. What a key opened is closed when the PdfDrawing that
// opened it goes, with its engine.

#pragma once

#include <QImage>
#include <QObject>
#include <QSet>
#include <QString>
#include <QtQml/qqmlregistration.h>

namespace qormedia {

class PdfDrawing : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    explicit PdfDrawing(QObject *parent = nullptr) : QObject(parent) {}
    ~PdfDrawing() override;

    /// The image provider's name: pages are `image://pdfpage/<key>/<page>`.
    static constexpr const char *Provider = "pdfpage";
    /// The widest page drawn, in pixels; a wider request is drawn this wide.
    static constexpr int WidthLimit = 8192;

    /// Open `path` for drawing, with `password` when it needs one. The key its pages' images name, or an empty string
    /// when it cannot be opened.
    Q_INVOKABLE QString open(const QString &path, const QString &password = QString());
    /// Let go of what `key` opened, and of its file.
    Q_INVOKABLE void close(const QString &key);

    /// Draw `page` of what `key` opened, `width` pixels wide (its printed width at 96 dots an inch when 0), with its
    /// marks: what the image provider does. A null image when the key or the page is unknown.
    static QImage draw(const QString &key, int page, int width);
    /// How many PDFs are open for drawing, in every engine: none once every reader has gone.
    static int openCount();

private:
    QSet<QString> m_opened;
};

}  // namespace qormedia
