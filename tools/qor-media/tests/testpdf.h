// A test's own PDFs, written byte by byte so what each holds is known exactly: pages of Helvetica text, links to a
// page or a URI, an outline, and, when asked, the standard security handler's lock (revision 2, 40-bit RC4), the
// oldest every reader opens. And what a saved PDF holds, read back through PDFium, which must be set up first
// (PdfPages does it on first use). Shared by QOR Media's suites (tst_reader, tst_pdfpages).

#pragma once

#include <QByteArray>
#include <QCryptographicHash>
#include <QFile>
#include <QList>
#include <QRectF>
#include <QString>
#include <QTest>

#include <utility>

#include <fpdf_edit.h>
#include <fpdf_text.h>
#include <fpdfview.h>

namespace testpdf {


// A page of a PDF: lines of text in Helvetica, and links to another page (by its index) or to a URI.
struct Link
{
    QRectF rect;  // in points, from the page's bottom left, as a PDF writes it
    int page = -1;
    QByteArray uri;
};
struct Page
{
    QList<QByteArray> lines;
    QList<Link> links;
};
struct Entry
{
    QByteArray title;
    int page;
};

// The padding of the standard security handler (PDF 1.7, 7.6.3.3).
inline const QByteArray kPad = QByteArray::fromHex("28BF4E5E4E758A4164004E56FFFA01082E2E00B6D0683E802F0CA9FE6453697A");

inline QByteArray rc4(const QByteArray &key, const QByteArray &data)
{
    unsigned char s[256];
    for (int i = 0; i < 256; ++i)
        s[i] = static_cast<unsigned char>(i);
    for (int i = 0, j = 0; i < 256; ++i) {
        j = (j + s[i] + static_cast<unsigned char>(key[i % key.size()])) & 255;
        std::swap(s[i], s[j]);
    }
    QByteArray out(data.size(), '\0');
    for (int k = 0, i = 0, j = 0; k < data.size(); ++k) {
        i = (i + 1) & 255;
        j = (j + s[i]) & 255;
        std::swap(s[i], s[j]);
        out[k] = static_cast<char>(data[k] ^ s[(s[i] + s[j]) & 255]);
    }
    return out;
}

inline QByteArray md5(const QByteArray &data)
{
    return QCryptographicHash::hash(data, QCryptographicHash::Md5);
}

inline QByteArray padded(const QByteArray &password)
{
    return (password + kPad).left(32);
}

// A PDF of `pages`, with an outline of `outline` when it has one, locked with `password` when there is one: the standard
// security handler's revision 2 (40-bit RC4), the oldest every reader opens, which is all a test of asking needs.
// `protect` locks it with an empty password: it opens without asking, yet is protected.
inline QByteArray writePdf(const QList<Page> &pages, const QList<Entry> &outline = {}, const QByteArray &password = {},
                           bool protect = false)
{
    const bool locking = protect || !password.isEmpty();
    const int count = int(pages.size());
    // Objects: 1 catalog, 2 page tree, 3 font, then for each page its page and its contents, then the links, the
    // outline and its entries, and the lock.
    int next = 4;
    QList<int> pageObj, contentObj;
    for (int i = 0; i < count; ++i) {
        pageObj << next++;
        contentObj << next++;
    }
    QList<QList<int>> linkObj(count);
    for (int i = 0; i < count; ++i)
        for (qsizetype l = 0; l < pages[i].links.size(); ++l)
            linkObj[i] << next++;
    const int outlineObj = outline.isEmpty() ? 0 : next++;
    QList<int> entryObj;
    for (qsizetype e = 0; e < outline.size(); ++e)
        entryObj << next++;
    const int lockObj = locking ? next++ : 0;

    const QByteArray id = md5("qor-media reader test");
    QByteArray key;
    QByteArray owner, user;
    if (locking) {
        owner = rc4(md5(padded("owner")).left(5), padded(password));
        const QByteArray p = QByteArray::fromHex("FCFFFFFF");  // /P -4, little-endian
        key = md5(padded(password) + owner + p + id).left(5);
        user = rc4(key, kPad);
    }
    const auto sealed = [&](int obj, const QByteArray &data) {
        if (key.isEmpty())
            return data;
        QByteArray salt = key;
        salt.append(char(obj & 0xFF)).append(char((obj >> 8) & 0xFF)).append(char((obj >> 16) & 0xFF)).append('\0').append('\0');
        return rc4(md5(salt).left(10), data);
    };
    const auto dest = [&](int page) { return QByteArray("[%1 0 R /XYZ 0 792 0]").replace("%1", QByteArray::number(pageObj[page])); };

    QByteArray pdf = "%PDF-1.4\n%\xE2\xE3\xCF\xD3\n";
    QList<qsizetype> offsets(next, 0);
    const auto object = [&](int obj, const QByteArray &body) {
        offsets[obj] = pdf.size();
        pdf += QByteArray::number(obj) + " 0 obj\n" + body + "\nendobj\n";
    };

    QByteArray kids;
    for (int p : pageObj)
        kids += QByteArray::number(p) + " 0 R ";
    object(1, "<< /Type /Catalog /Pages 2 0 R" + (outlineObj ? " /Outlines " + QByteArray::number(outlineObj) + " 0 R" : QByteArray()) + " >>");
    object(2, "<< /Type /Pages /Kids [" + kids + "] /Count " + QByteArray::number(count) + " >>");
    object(3, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>");
    for (int i = 0; i < count; ++i) {
        QByteArray annots;
        for (int a : linkObj[i])
            annots += QByteArray::number(a) + " 0 R ";
        object(pageObj[i], "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents "
                               + QByteArray::number(contentObj[i]) + " 0 R" + (annots.isEmpty() ? QByteArray() : " /Annots [" + annots + "]") + " >>");
        QByteArray text = "BT /F1 24 Tf 72 700 Td";
        for (qsizetype l = 0; l < pages[i].lines.size(); ++l)
            text += (l ? " 0 -36 Td (" : " (") + pages[i].lines[l] + ") Tj";
        text += " ET";
        const QByteArray stream = sealed(contentObj[i], text);
        object(contentObj[i], "<< /Length " + QByteArray::number(stream.size()) + " >>\nstream\n" + stream + "\nendstream");
    }
    for (int i = 0; i < count; ++i)
        for (qsizetype l = 0; l < pages[i].links.size(); ++l) {
            const Link &link = pages[i].links[l];
            const QRectF &r = link.rect;
            const QByteArray rect = "[" + QByteArray::number(r.left()) + " " + QByteArray::number(r.top()) + " "
                + QByteArray::number(r.right()) + " " + QByteArray::number(r.bottom()) + "]";
            const QByteArray target = link.page >= 0 ? "/Dest " + dest(link.page) : "/A << /S /URI /URI (" + link.uri + ") >>";
            object(linkObj[i][l], "<< /Type /Annot /Subtype /Link /Rect " + rect + " /Border [0 0 0] " + target + " >>");
        }
    if (outlineObj) {
        object(outlineObj, "<< /Type /Outlines /First " + QByteArray::number(entryObj.first()) + " 0 R /Last "
                               + QByteArray::number(entryObj.last()) + " 0 R /Count " + QByteArray::number(outline.size()) + " >>");
        for (qsizetype e = 0; e < outline.size(); ++e) {
            QByteArray body = "<< /Title (" + outline[e].title + ") /Parent " + QByteArray::number(outlineObj) + " 0 R /Dest " + dest(outline[e].page);
            if (e > 0)
                body += " /Prev " + QByteArray::number(entryObj[e - 1]) + " 0 R";
            if (e + 1 < outline.size())
                body += " /Next " + QByteArray::number(entryObj[e + 1]) + " 0 R";
            object(entryObj[e], body + " >>");
        }
    }
    if (lockObj)
        object(lockObj, "<< /Filter /Standard /V 1 /R 2 /O <" + owner.toHex() + "> /U <" + user.toHex() + "> /P -4 >>");

    const qsizetype xref = pdf.size();
    pdf += "xref\n0 " + QByteArray::number(next) + "\n0000000000 65535 f \n";
    for (int obj = 1; obj < next; ++obj)
        pdf += QByteArray::number(offsets[obj]).rightJustified(10, '0') + " 00000 n \n";
    pdf += "trailer\n<< /Size " + QByteArray::number(next) + " /Root 1 0 R /ID [<" + id.toHex() + "> <" + id.toHex() + ">]"
        + (lockObj ? " /Encrypt " + QByteArray::number(lockObj) + " 0 R" : QByteArray()) + " >>\nstartxref\n"
        + QByteArray::number(xref) + "\n%%EOF\n";
    return pdf;
}

inline void save(const QString &path, const QByteArray &bytes)
{
    QFile f(path);
    QVERIFY(f.open(QIODevice::WriteOnly));
    f.write(bytes);
}


inline QByteArray bytesOf(const QString &path)
{
    QFile f(path);
    return f.open(QIODevice::ReadOnly) ? f.readAll() : QByteArray();
}

// What a PDF holds, read back through PDFium: each page's text and its turn in quarter turns.
struct Held
{
    bool read = false;
    QStringList text;
    QList<int> turns;
};

inline Held readBack(const QString &path)
{
    Held held;
    const QByteArray bytes = bytesOf(path);
    FPDF_DOCUMENT document = FPDF_LoadMemDocument64(bytes.constData(), size_t(bytes.size()), nullptr);
    if (!document)
        return held;
    for (int i = 0; i < FPDF_GetPageCount(document); ++i) {
        FPDF_PAGE page = FPDF_LoadPage(document, i);
        FPDF_TEXTPAGE text = FPDFText_LoadPage(page);
        const int chars = FPDFText_CountChars(text);
        QList<unsigned short> buffer(chars + 1, 0);
        FPDFText_GetText(text, 0, chars, buffer.data());
        held.text << QString::fromUtf16(reinterpret_cast<const char16_t *>(buffer.constData()), chars).simplified();
        held.turns << FPDFPage_GetRotation(page);
        FPDFText_ClosePage(text);
        FPDF_ClosePage(page);
    }
    FPDF_CloseDocument(document);
    held.read = true;
    return held;
}

}  // namespace testpdf
