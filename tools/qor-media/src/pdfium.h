// QOR Media's own PDFium, shared by what changes a PDF (PdfPages, PdfMarks) and what draws a page with marks taken away
// (PdfDrawing). It is a copy of its own, beside the one inside Qt PDF, and shares nothing with it.
//
// PDFium is not safe to use from two threads at once, and pages are drawn on the image provider's threads while the
// reader saves on the application's, so every use of it holds `PdfiumLock` from first call to last.

#pragma once

#include <QFile>
#include <QMutex>
#include <QPointF>
#include <QSaveFile>
#include <QString>
#include <QTransform>

#include <fpdfview.h>
#include <fpdf_save.h>

namespace qormedia {

/// Held around every use of QOR Media's PDFium; sets it up on first use.
class PdfiumLock
{
public:
    PdfiumLock();
    ~PdfiumLock();
    PdfiumLock(const PdfiumLock &) = delete;
    PdfiumLock &operator=(const PdfiumLock &) = delete;

private:
    QMutexLocker<QMutex> m_locked;
};

/// PDFium reading a file a block at a time through a QFile, which opens any path Windows does: one of its own, or one
/// already open (`over`), read from wherever it stands.
struct PdfSource
{
    QFile own;
    QFile *file = &own;
    FPDF_FILEACCESS access{};

    explicit PdfSource(const QString &path) : own(path) {}
    explicit PdfSource(QFile *over) : file(over) {}

    /// False when the file cannot be read, or is larger than PDFium's reading interface counts (4 GiB).
    bool open();
};

/// What PDFium writes, into a QSaveFile, which becomes the file only when it is complete.
struct PdfSink : FPDF_FILEWRITE
{
    explicit PdfSink(QSaveFile *into);
    QSaveFile *out;
};

/// A document, closed when it goes.
struct PdfiumDocument
{
    FPDF_DOCUMENT handle = nullptr;
    ~PdfiumDocument() { close(); }
    void close();
};

/// A page, closed when it goes.
struct PdfiumPage
{
    FPDF_PAGE handle = nullptr;
    ~PdfiumPage();
};

/// From a page as Qt PDF lays it out (in points, from its top left, turned as the page's own /Rotate turns it: the space
/// of a selection's geometry, and of every mark the reader makes) to the PDF's own space for that page (from its bottom
/// left, y up, before its turn). Qt PDF maps through FPDF_PageToDevice at the page's size in whole points, so this does.
QTransform pageSpace(FPDF_PAGE page);

}  // namespace qormedia
