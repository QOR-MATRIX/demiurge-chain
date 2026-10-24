// QOR Media's own PDFium, shared by what changes a PDF (PdfPages, PdfMarks, PdfForms) and what draws its pages
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
#include <fpdf_annot.h>
#include <fpdf_formfill.h>
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

/// A document's form, through which PDFium draws its fields (FPDF_FFLDraw: nothing else does) and fills them; let go of
/// when it goes, which is before its document closes. It runs none of the form's scripts: this PDFium has no JavaScript.
class PdfiumForm
{
public:
    explicit PdfiumForm(FPDF_DOCUMENT document);
    ~PdfiumForm();
    PdfiumForm(const PdfiumForm &) = delete;
    PdfiumForm &operator=(const PdfiumForm &) = delete;

    FPDF_FORMHANDLE handle() const { return m_handle; }

private:
    // PDFium keeps a pointer to it for as long as the form lives.
    FPDF_FORMFILLINFO m_info{};
    FPDF_FORMHANDLE m_handle = nullptr;
};

/// A page, closed when it goes. With a form, the form is told it is loaded, as it must be before it draws or fills the
/// page's fields, and told again before it closes.
class PdfiumPage
{
public:
    explicit PdfiumPage(FPDF_PAGE page, FPDF_FORMHANDLE form = nullptr);
    ~PdfiumPage();
    PdfiumPage(const PdfiumPage &) = delete;
    PdfiumPage &operator=(const PdfiumPage &) = delete;

    FPDF_PAGE handle = nullptr;

private:
    FPDF_FORMHANDLE m_form = nullptr;
};

/// An annotation of a page, let go of when it goes.
struct PdfiumAnnot
{
    FPDF_ANNOTATION handle = nullptr;
    ~PdfiumAnnot();
};

/// From a page as Qt PDF lays it out (in points, from its top left, turned as the page's own /Rotate turns it: the space
/// of a selection's geometry, and of every mark the reader makes) to the PDF's own space for that page (from its bottom
/// left, y up, before its turn). Qt PDF maps through FPDF_PageToDevice at the page's size in whole points, so this does.
QTransform pageSpace(FPDF_PAGE page);

}  // namespace qormedia
