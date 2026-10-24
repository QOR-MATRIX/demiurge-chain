// PdfMarks: marks made on a PDF's pages, and the PDF's own marks read and taken away, through PDFium's own interface
// (ADR-094 decision 3, ADR-114 decision 1, L9.5).
//
// The reader holds a mark it makes as a map until it is saved:
//   kind     "highlight", "underline" or "strikeout", over words; "note"; "ink", a line drawn by hand; or "signature", a
//            picture of the person's signature, drawn or typed, which certifies nothing (ADR-114 decision 1)
//   page     the page's index in the file as it is now
//   colour   "#rrggbb", one of `colours`
//   rects    for words: each run's rectangle, as a selection's geometry gives them (rectsOf)
//   x, y     a note's corner; with width and height, the box a signature sits in
//   text     what a note says
//   strokes  ink's lines, each a list of points; a drawn signature's, from 0 to 1 across and down its box
//   width    ink's line, in points
//   line     a drawn signature's line, as a share of its box's height
//   shapes   a typed signature's outlines, from 0 to 1 across and down its box, filled
// Every point is in the page's points as Qt PDF lays it out (pageSpace, pdfium.h), the space the reader works in.
//
// Each is written as the PDF annotation it is (Highlight, Underline, StrikeOut, Text, Ink, Stamp) with an appearance
// drawn here, so another reader shows it as QOR Media does, and with what other editors read to change it: the words'
// quadrilaterals, ink's list of strokes, a note's text.
//
// The PDF's own marks are read as maps too (`read`), and one is taken away by its page and its place among the page's
// annotations, with the pop-up that shows a note's text. Links, form fields and pop-ups are not marks.

#pragma once

#include <QObject>
#include <QString>
#include <QVariantList>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

#include <fpdfview.h>

namespace qormedia {

class PdfMarks : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(QVariantList colours READ colours CONSTANT)
    Q_PROPERTY(int noteSize READ noteSize CONSTANT)
    Q_PROPERTY(qreal highlightOpacity READ highlightOpacity CONSTANT)

public:
    explicit PdfMarks(QObject *parent = nullptr) : QObject(parent) {}

    /// The colours a mark can be: each {name, colour}. They are what the PDF holds, a highlighter's and a pen's, the
    /// same in every theme, so they are written here and not taken from the look.
    static QVariantList colours();
    /// A note's side, in points.
    static constexpr int NoteSize = 20;
    int noteSize() const { return NoteSize; }
    /// How much of a highlight's colour lies over the words it marks.
    static constexpr qreal HighlightOpacity = 0.4;
    qreal highlightOpacity() const { return HighlightOpacity; }
    /// The face a typed signature is drawn in.
    static QString signatureFace();

    /// The marks `path` holds on `page`, or on every page when it is -1, opened with `password` when it has one: each
    /// {page, index (its place among the page's annotations), kind, rect, text, author}. Hidden ones are left out. Empty
    /// when it cannot be read. Reading a page parses all it draws, so the reader asks for a page as it shows it.
    Q_INVOKABLE static QVariantList read(const QString &path, const QString &password = QString(), int page = -1);
    /// A selection's geometry (QPdfSelection's polygons) as the rectangles of its runs of words.
    Q_INVOKABLE static QVariantList rectsOf(const QVariant &geometry);
    /// `name` typed in the signature's face: {aspect, shapes}, its outlines from 0 to 1 across and down the box they
    /// fill, and how wide that box is for each unit it is high. Empty when there is nothing to draw.
    Q_INVOKABLE static QVariantMap typed(const QString &name);
};

/// Take `removed` away from `document` and write `added` on it, both as PdfMarks describes them. The caller has the
/// document open under PdfiumLock. Every entry is checked before anything changes; false, with `why`, when one is not a
/// mark this can write, or not one of the PDF's own.
bool writeMarks(FPDF_DOCUMENT document, const QVariantList &added, const QVariantList &removed, QString *why);

}  // namespace qormedia
