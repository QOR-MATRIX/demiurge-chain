#include "pdfpages.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QSaveFile>
#include <QSet>
#include <QUuid>

#include <algorithm>

#include <fpdf_edit.h>
#include <fpdf_save.h>
#include <fpdfview.h>

namespace qormedia {

namespace {

// PDFium is set up on first use and kept: QOR Media's own copy, beside the one inside Qt PDF, which shares nothing with it.
void ensurePdfium()
{
    static const bool ready = [] {
        FPDF_InitLibrary();
        return true;
    }();
    Q_UNUSED(ready);
}

// PDFium reads the file a block at a time through a QFile, which opens any path Windows does.
struct Source
{
    QFile file;
    FPDF_FILEACCESS access{};

    explicit Source(const QString &path) : file(path) {}

    // False when the file cannot be read, or is larger than PDFium's reading interface counts (4 GiB).
    bool open()
    {
        if (!file.open(QIODevice::ReadOnly) || file.size() > qint64(0xFFFFFFFFu))
            return false;
        access.m_FileLen = static_cast<unsigned long>(file.size());
        access.m_Param = this;
        access.m_GetBlock = [](void *self, unsigned long position, unsigned char *buffer, unsigned long size) -> int {
            QFile &f = static_cast<Source *>(self)->file;
            return f.seek(qint64(position)) && f.read(reinterpret_cast<char *>(buffer), qint64(size)) == qint64(size);
        };
        return true;
    }
};

// What PDFium writes goes into a QSaveFile, which becomes the file only when it is complete.
struct Sink : FPDF_FILEWRITE
{
    QSaveFile *out = nullptr;
};

int writeBlock(FPDF_FILEWRITE *self, const void *data, unsigned long size)
{
    return static_cast<Sink *>(self)->out->write(static_cast<const char *>(data), qint64(size)) == qint64(size);
}

struct Document
{
    FPDF_DOCUMENT handle = nullptr;
    ~Document() { close(); }
    void close()
    {
        if (handle)
            FPDF_CloseDocument(handle);
        handle = nullptr;
    }
};

const QString kCopyPrefix = QStringLiteral("saving-");

}  // namespace

bool PdfPages::fail(const QString &why)
{
    m_error = why;
    emit changed();
    return false;
}

bool PdfPages::changeable(const QString &path)
{
    ensurePdfium();
    Source source(path);
    if (!source.open())
        return false;
    Document document{FPDF_LoadCustomDocument(&source.access, nullptr)};
    return document.handle && FPDF_GetSecurityHandlerRevision(document.handle) == -1;
}

bool PdfPages::save(const QString &path, const QVariantList &layout)
{
    m_error.clear();
    ensurePdfium();

    // What is kept, in order, and how far each is turned. The list is checked whole before the file is touched.
    struct Kept
    {
        int page;
        int turn;
    };
    QList<Kept> kept;
    QSet<int> keeping;
    for (const QVariant &entry : layout) {
        const QVariantMap map = entry.toMap();
        bool pageOk = false, turnOk = false;
        const int page = map.value(QStringLiteral("page")).toInt(&pageOk);
        const int turn = map.value(QStringLiteral("turn")).toInt(&turnOk);
        if (!pageOk || !turnOk || page < 0 || turn < 0 || turn > 3 || keeping.contains(page))
            return fail(tr("The pages to save were not a list QOR Media can write."));
        keeping.insert(page);
        kept.append({page, turn});
    }
    if (kept.isEmpty())
        return fail(tr("A PDF keeps at least one page."));

    Source source(path);
    if (!source.open())
        return fail(tr("This PDF could not be opened to save it."));
    Document document{FPDF_LoadCustomDocument(&source.access, nullptr)};
    if (!document.handle)
        return fail(FPDF_GetLastError() == FPDF_ERR_PASSWORD ? tr("A locked PDF's pages cannot be changed here yet.")
                                                             : tr("This PDF could not be read to save it."));
    // Saving through PDFium could drop a PDF's protection, so a protected one is left as it is.
    if (FPDF_GetSecurityHandlerRevision(document.handle) != -1)
        return fail(tr("This PDF is protected, so its pages cannot be changed here yet."));
    const int count = FPDF_GetPageCount(document.handle);
    for (const Kept &k : std::as_const(kept))
        if (k.page >= count)
            return fail(tr("The pages to save were not a list QOR Media can write."));

    // Turn first, while every page is where the list found it.
    for (const Kept &k : std::as_const(kept)) {
        if (!k.turn)
            continue;
        FPDF_PAGE page = FPDF_LoadPage(document.handle, k.page);
        if (!page)
            return fail(tr("A page of this PDF could not be read."));
        FPDFPage_SetRotation(page, (FPDFPage_GetRotation(page) + k.turn) % 4);
        FPDF_ClosePage(page);
    }
    // Then delete what was left out, from the end, so the rest keep their places.
    for (int i = count - 1; i >= 0; --i)
        if (!keeping.contains(i))
            FPDFPage_Delete(document.handle, i);
    // And put the rest in the list's order: each page's place among those kept, all moved together to the front.
    QList<int> survivors(keeping.cbegin(), keeping.cend());
    std::sort(survivors.begin(), survivors.end());
    QList<int> order;
    for (const Kept &k : std::as_const(kept))
        order.append(int(survivors.indexOf(k.page)));
    if (!std::is_sorted(order.cbegin(), order.cend())
        && !FPDF_MovePages(document.handle, order.constData(), static_cast<unsigned long>(order.size()), 0))
        return fail(tr("The pages could not be put in order."));

    QSaveFile out(path);
    if (!out.open(QIODevice::WriteOnly))
        return fail(tr("This PDF cannot be written: %1").arg(out.errorString()));
    Sink sink;
    sink.version = 1;
    sink.WriteBlock = writeBlock;
    sink.out = &out;
    if (!FPDF_SaveAsCopy(document.handle, &sink, FPDF_NO_INCREMENTAL)) {
        out.cancelWriting();
        return fail(tr("The PDF could not be written."));
    }
    // PDFium has read all it needs; the file is let go before it is replaced.
    document.close();
    source.file.close();
    if (!out.commit())
        return fail(tr("This PDF cannot be written: %1").arg(out.errorString()));
    emit changed();
    return true;
}

QUrl PdfPages::copyForSaving(const QString &path)
{
    QDir dir(QDir::temp().filePath(QStringLiteral("qor-media")));
    if (!dir.mkpath(QStringLiteral(".")))
        return {};
    // A copy an earlier save left behind, which nothing shows any more, goes now; one still shown stays.
    for (const QFileInfo &old : dir.entryInfoList({kCopyPrefix + QStringLiteral("*.pdf")}, QDir::Files))
        discard(QUrl::fromLocalFile(old.absoluteFilePath()));
    const QString copy = dir.filePath(kCopyPrefix + QUuid::createUuid().toString(QUuid::WithoutBraces) + QStringLiteral(".pdf"));
    if (!QFile::copy(path, copy))
        return {};
    return QUrl::fromLocalFile(copy);
}

bool PdfPages::discard(const QUrl &copy)
{
    // Only a copy this made: one of its names, in its folder.
    const QFileInfo info(copy.toLocalFile());
    if (!copy.isLocalFile() || !info.fileName().startsWith(kCopyPrefix)
        || info.absolutePath() != QFileInfo(QDir::temp().filePath(QStringLiteral("qor-media"))).absoluteFilePath())
        return false;
    // A copy takes its file's attributes, and Windows will not delete a read-only file.
    QFile::setPermissions(info.absoluteFilePath(), QFile::ReadOwner | QFile::WriteOwner | QFile::ReadUser | QFile::WriteUser);
    return QFile::remove(info.absoluteFilePath()) || !info.exists();
}

}  // namespace qormedia
