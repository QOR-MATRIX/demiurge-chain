// PdfDrawing: a PDF's pages drawn with their marks and their form's fields (ADR-094 decision 3, L9.5).
//
// Qt PDF's page image in QML draws a page without its annotations: highlights, notes, stamps and drawings do not show.
// Its QPdfDocument draws those when asked, but neither draws a form's fields, filled or not: PDFium draws fields only
// through a form environment (FPDF_FFLDraw), which Qt PDF never makes. So the reader's pages are drawn here, through an
// image provider the engine knows as "pdfpage": `image://pdfpage/<key>/<page>`, as wide as the image asks. A PDF without
// a form is drawn by Qt PDF's QPdfDocument, whose PDFium is its own and so never waits on QOR Media's; a PDF with one by
// QOR Media's own PDFium, through its form. `image://pdfpage/<key>/<page>/<hidden>` draws a page without some of its
// marks, those the reader is taking away (their places among the page's annotations, by commas); and
// `image://pdfpage/<key>/<page>/<hidden>/<filled>` with its fields as the reader last filled them (`fill`), `<filled>`
// naming that filling, so each is drawn afresh. Either is drawn by QOR Media's PDFium from a copy read for that drawing
// alone: nothing is written.
//
// A key names a PDF opened here, with its password when it has one; the password, and whatever is filled in, stay here
// and never reach a URL. Closing a key lets go of its file at once, as saving needs. What a key opened is closed when the
// PdfDrawing that opened it goes, with its engine.

#pragma once

#include <QImage>
#include <QObject>
#include <QSet>
#include <QString>
#include <QVariantList>
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
    /// Hold what the person has filled in what `key` opened (entries as PdfForms describes them), for the pages drawn with
    /// their fields filled; an empty list holds nothing.
    Q_INVOKABLE void fill(const QString &key, const QVariantList &filled);

    /// Draw `page` of what `key` opened, `width` pixels wide (its printed width at 96 dots an inch when 0), with its
    /// marks but those `hidden` names, and its fields as last filled when `filled`: what the image provider does. A null
    /// image when the key or the page is unknown.
    static QImage draw(const QString &key, int page, int width, const QList<int> &hidden = {}, bool filled = false);
    /// How many PDFs are open for drawing, in every engine: none once every reader has gone.
    static int openCount();

private:
    QSet<QString> m_opened;
};

}  // namespace qormedia
