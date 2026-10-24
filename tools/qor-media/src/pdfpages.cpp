#include "pdfpages.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QSaveFile>
#include <QSet>
#include <QUuid>

#include <algorithm>

#include <fpdf_edit.h>

#include "pdfium.h"
#include "pdfmarks.h"

namespace qormedia {

namespace {

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
    PdfiumLock locked;
    PdfSource source(path);
    if (!source.open())
        return false;
    PdfiumDocument document{FPDF_LoadCustomDocument(&source.access, nullptr)};
    return document.handle && FPDF_GetSecurityHandlerRevision(document.handle) == -1;
}

bool PdfPages::save(const QString &path, const QVariantList &layout, const QVariantList &added, const QVariantList &removed)
{
    m_error.clear();

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

    PdfiumLock locked;
    PdfSource source(path);
    if (!source.open())
        return fail(tr("This PDF could not be opened to save it."));
    PdfiumDocument document{FPDF_LoadCustomDocument(&source.access, nullptr)};
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

    // The marks first, on the pages as the file has them and before any is turned: that is where the reader made them.
    QString why;
    if (!writeMarks(document.handle, added, removed, &why))
        return fail(why);

    // Turn next, while every page is where the list found it.
    for (const Kept &k : std::as_const(kept)) {
        if (!k.turn)
            continue;
        PdfiumPage page{FPDF_LoadPage(document.handle, k.page)};
        if (!page.handle)
            return fail(tr("A page of this PDF could not be read."));
        FPDFPage_SetRotation(page.handle, (FPDFPage_GetRotation(page.handle) + k.turn) % 4);
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
    PdfSink sink(&out);
    if (!FPDF_SaveAsCopy(document.handle, &sink, FPDF_NO_INCREMENTAL)) {
        out.cancelWriting();
        return fail(tr("The PDF could not be written."));
    }
    // PDFium has read all it needs; the file is let go before it is replaced.
    document.close();
    source.file->close();
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
