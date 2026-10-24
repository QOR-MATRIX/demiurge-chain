#include "pdfium.h"

namespace qormedia {

namespace {

QMutex &pdfium()
{
    static QMutex lock;
    return lock;
}

int writeBlock(FPDF_FILEWRITE *self, const void *data, unsigned long size)
{
    return static_cast<PdfSink *>(self)->out->write(static_cast<const char *>(data), qint64(size)) == qint64(size);
}

}  // namespace

PdfiumLock::PdfiumLock() : m_locked(&pdfium())
{
    static const bool ready = [] {
        FPDF_InitLibrary();
        return true;
    }();
    Q_UNUSED(ready);
}

PdfiumLock::~PdfiumLock() = default;

bool PdfSource::open()
{
    if ((!file->isOpen() && !file->open(QIODevice::ReadOnly)) || file->size() > qint64(0xFFFFFFFFu))
        return false;
    access.m_FileLen = static_cast<unsigned long>(file->size());
    access.m_Param = this;
    access.m_GetBlock = [](void *self, unsigned long position, unsigned char *buffer, unsigned long size) -> int {
        QFile *f = static_cast<PdfSource *>(self)->file;
        return f->seek(qint64(position)) && f->read(reinterpret_cast<char *>(buffer), qint64(size)) == qint64(size);
    };
    return true;
}

PdfSink::PdfSink(QSaveFile *into) : FPDF_FILEWRITE{}, out(into)
{
    version = 1;
    WriteBlock = writeBlock;
}

void PdfiumDocument::close()
{
    if (handle)
        FPDF_CloseDocument(handle);
    handle = nullptr;
}

PdfiumPage::~PdfiumPage()
{
    if (handle)
        FPDF_ClosePage(handle);
}

QTransform pageSpace(FPDF_PAGE page)
{
    const int w = qMax(1, qRound(FPDF_GetPageWidthF(page)));
    const int h = qMax(1, qRound(FPDF_GetPageHeightF(page)));
    // Three corners of the page as laid out, in the PDF's space: the map between them is the whole of it.
    double ox = 0, oy = 0, ax = 0, ay = 0, bx = 0, by = 0;
    FPDF_DeviceToPage(page, 0, 0, w, h, 0, 0, 0, &ox, &oy);
    FPDF_DeviceToPage(page, 0, 0, w, h, 0, w, 0, &ax, &ay);
    FPDF_DeviceToPage(page, 0, 0, w, h, 0, 0, h, &bx, &by);
    return QTransform((ax - ox) / w, (ay - oy) / w, (bx - ox) / h, (by - oy) / h, ox, oy);
}

}  // namespace qormedia
