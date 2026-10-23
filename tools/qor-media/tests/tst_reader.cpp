// The reader (ADR-094 decision 3, L9.5): a PDF opens in QOR Media with every page, fitted to the width; the keys go
// through it and zoom it; find marks each match and goes to it; a page's text is selected and copied; a link goes to its
// page, and only the web or mail leave the document; the pages and the contents sit beside it; a locked PDF opens with
// its password and not without; a broken one says so; and every control is reached by Tab. Every PDF here is written by
// the test, byte by byte, so what each holds is known exactly.

#include <QClipboard>
#include <QCryptographicHash>
#include <QFile>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QTest>
#include <QWheelEvent>
#include <QtQml/qqmlextensionplugin.h>

#include "medialink.h"

Q_IMPORT_QML_PLUGIN(QOR_MediaPlugin)

using qormedia::MediaLink;

namespace {

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
const QByteArray kPad = QByteArray::fromHex("28BF4E5E4E758A4164004E56FFFA01082E2E00B6D0683E802F0CA9FE6453697A");

QByteArray rc4(const QByteArray &key, const QByteArray &data)
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

QByteArray md5(const QByteArray &data)
{
    return QCryptographicHash::hash(data, QCryptographicHash::Md5);
}

QByteArray padded(const QByteArray &password)
{
    return (password + kPad).left(32);
}

// A PDF of `pages`, with an outline of `outline` when it has one, locked with `password` when there is one: the standard
// security handler's revision 2 (40-bit RC4), the oldest every reader opens, which is all a test of asking needs.
QByteArray writePdf(const QList<Page> &pages, const QList<Entry> &outline = {}, const QByteArray &password = {})
{
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
    const int lockObj = password.isEmpty() ? 0 : next++;

    const QByteArray id = md5("qor-media reader test");
    QByteArray key;
    QByteArray owner, user;
    if (!password.isEmpty()) {
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

void save(const QString &path, const QByteArray &bytes)
{
    QFile f(path);
    QVERIFY(f.open(QIODevice::WriteOnly));
    f.write(bytes);
}

}  // namespace

class TestReader : public QObject
{
    Q_OBJECT

    QTemporaryDir dir;
    QQmlApplicationEngine *engine = nullptr;
    MediaLink *link = nullptr;
    QQuickWindow *window = nullptr;
    QList<QQmlError> warnings;

    // Open `path` as the launcher would, and wait for the reader.
    QQuickItem *read(const QString &path)
    {
        QMetaObject::invokeMethod(link, "openRequested", Q_ARG(QString, path), Q_ARG(QString, QStringLiteral("document")), Q_ARG(double, 0));
        if (!QTest::qWaitFor([&] { return window->isVisible(); }))
            return nullptr;
        if (!QTest::qWaitForWindowActive(window))
            return nullptr;
        QQuickItem *reader = nullptr;
        if (!QTest::qWaitFor([&] { return (reader = window->findChild<QQuickItem *>(QStringLiteral("reader"))) != nullptr; }))
            return nullptr;
        return reader;
    }
    // Type `text` into whatever has the keyboard. QTest types whole strings only into widgets.
    void type(const QString &text)
    {
        for (const QChar c : text)
            QTest::keyClick(window, c.toLatin1());
    }
    QQuickItem *named(const char *name)
    {
        return window->findChild<QQuickItem *>(QString::fromLatin1(name));
    }
    // The items under `root` that `match`, by the visual tree: what a Repeater or a view makes has no QObject parent,
    // so findChild never sees it.
    template<typename Match>
    QList<QQuickItem *> itemsUnder(QQuickItem *root, Match match)
    {
        QList<QQuickItem *> found;
        for (QQuickItem *child : root->childItems()) {
            if (match(child))
                found << child;
            found << itemsUnder(child, match);
        }
        return found;
    }
    QList<QQuickItem *> itemsNamed(const QString &name)
    {
        return itemsUnder(window->contentItem(), [&](QQuickItem *i) { return i->objectName() == name; });
    }
    QPointF centreOf(QQuickItem *item)
    {
        return item->mapToScene(QPointF(item->width() / 2, item->height() / 2));
    }

private slots:
    void initTestCase()
    {
        QVERIFY(dir.isValid());
        save(dir.filePath("three.pdf"), writePdf({{{"alpha one"}, {{QRectF(72, 600, 200, 40), 2, {}}, {QRectF(72, 500, 200, 40), -1, "https://qorsync.dev/"}}},
                                                  {{"beta two"}, {}},
                                                  {{"gamma gamma", "the end"}, {}}},
                                                 {{"Start", 0}, {"End", 2}}));
        save(dir.filePath("locked.pdf"), writePdf({{{"secret words"}, {}}}, {}, "open sesame"));
        save(dir.filePath("locked too.pdf"), writePdf({{{"other words"}, {}}}, {}, "open sesame"));
        save(dir.filePath("broken.pdf"), "%PDF-1.7\n");
    }

    void init()
    {
        warnings.clear();
        link = new MediaLink(this);
        engine = new QQmlApplicationEngine(this);
        connect(engine, &QQmlEngine::warnings, this, [this](const QList<QQmlError> &w) { warnings << w; });
        engine->setInitialProperties({{"link", QVariant::fromValue(link)}, {"launched", true}, {"startPath", QString()}});
        engine->loadFromModule("QOR.Media", "Main");
        QCOMPARE(engine->rootObjects().size(), 1);
        window = qobject_cast<QQuickWindow *>(engine->rootObjects().first());
        QVERIFY(window);
    }

    void cleanup()
    {
        for (const QQmlError &w : std::as_const(warnings))
            qWarning().noquote() << w.toString();
        const bool clean = warnings.isEmpty();
        delete engine;
        engine = nullptr;
        delete link;
        link = nullptr;
        QVERIFY2(clean, "QML warned");
    }

    void a_pdf_opens_in_the_reader_with_every_page_fitted_to_the_width()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QCOMPARE(reader->property("page").toInt(), 0);
        QCOMPARE(reader->property("fitting").toString(), QStringLiteral("width"));
        QQuickItem *pages = named("pages");
        QVERIFY(pages);
        // The widest page fills the view but for the margin either side.
        const qreal margin = reader->property("margin").toReal();
        QTRY_VERIFY(qAbs(reader->property("zoom").toReal() * 612 - (pages->width() - 2 * margin)) < 1);
        // It is not the page that says QOR Media cannot open it.
        QVERIFY(!named("openElsewhere") || !named("openElsewhere")->isVisible());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void the_keys_go_through_the_pages()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QQuickItem *pages = named("pages");
        QTest::keyClick(window, Qt::Key_End);
        QTRY_COMPARE(reader->property("page").toInt(), 2);
        QTest::keyClick(window, Qt::Key_Home);
        QTRY_COMPARE(reader->property("page").toInt(), 0);
        const qreal top = pages->property("contentY").toReal();
        QTest::keyClick(window, Qt::Key_PageDown);
        QTRY_VERIFY(pages->property("contentY").toReal() > top + pages->height() / 2);
        QTest::keyClick(window, Qt::Key_PageUp);
        QTRY_COMPARE(pages->property("contentY").toReal(), top);
        // A page by its number.
        QTest::keyClick(window, Qt::Key_G, Qt::ControlModifier);
        QTRY_VERIFY(named("pageField")->hasActiveFocus());
        type(QStringLiteral("2"));
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(reader->property("page").toInt(), 1);
        // The keyboard is back on the pages, not left in the field.
        QVERIFY(reader->hasActiveFocus());
        QVERIFY(!named("pageField")->hasActiveFocus());
        QCOMPARE(named("pageField")->property("text").toString(), QStringLiteral("2"));
        QTest::keyClick(window, Qt::Key_End);
        QTRY_COMPARE(reader->property("page").toInt(), 2);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void zoom_stays_in_bounds_and_fitting_returns()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QQuickItem *pages = named("pages");
        const qreal margin = reader->property("margin").toReal();
        QTRY_VERIFY(qAbs(reader->property("zoom").toReal() * 612 - (pages->width() - 2 * margin)) < 1);
        const qreal fitted = reader->property("zoom").toReal();
        QTest::keyClick(window, Qt::Key_Plus);
        QTRY_VERIFY(qAbs(reader->property("zoom").toReal() - fitted * 1.25) < 0.001);
        QCOMPARE(reader->property("fitting").toString(), QString());
        for (int i = 0; i < 40; ++i)
            QTest::keyClick(window, Qt::Key_Minus);
        QCOMPARE(reader->property("zoom").toReal(), reader->property("minZoom").toReal());
        for (int i = 0; i < 40; ++i)
            QTest::keyClick(window, Qt::Key_Plus);
        QCOMPARE(reader->property("zoom").toReal(), reader->property("maxZoom").toReal());
        // 0 fits the width again, then the whole page, which is smaller.
        QTest::keyClick(window, Qt::Key_0);
        QTRY_COMPARE(reader->property("fitting").toString(), QStringLiteral("width"));
        QTRY_VERIFY(qAbs(reader->property("zoom").toReal() - fitted) < 0.001);
        QTest::keyClick(window, Qt::Key_0);
        QTRY_COMPARE(reader->property("fitting").toString(), QStringLiteral("page"));
        const qreal whole = reader->property("zoom").toReal();
        QVERIFY(whole < fitted);
        QVERIFY(792 * whole <= pages->height());
        // Ctrl and the wheel over a page zoom it in.
        const QPointF at = centreOf(pages);
        QWheelEvent wheel(at, window->mapToGlobal(at), QPoint(), QPoint(0, 120), Qt::NoButton, Qt::ControlModifier, Qt::NoScrollPhase, false);
        QGuiApplication::sendEvent(window, &wheel);
        QTRY_VERIFY(reader->property("zoom").toReal() > whole);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void find_marks_every_match_and_goes_to_each()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QTest::keyClick(window, Qt::Key_F, Qt::ControlModifier);
        QQuickItem *query = named("readerFind");
        QTRY_VERIFY(query->hasActiveFocus());
        type(QStringLiteral("gamma"));
        QTRY_COMPARE(reader->property("matchCount").toInt(), 2);
        QTRY_COMPARE(reader->property("match").toInt(), 0);
        QTRY_COMPARE(reader->property("page").toInt(), 2);
        QCOMPARE(named("readerMatches")->property("text").toString(), QStringLiteral("1 of 2"));
        QTest::keyClick(window, Qt::Key_F3);
        QTRY_COMPARE(reader->property("match").toInt(), 1);
        QTest::keyClick(window, Qt::Key_F3);
        QTRY_COMPARE(reader->property("match").toInt(), 0);
        QTest::keyClick(window, Qt::Key_F3, Qt::ShiftModifier);
        QTRY_COMPARE(reader->property("match").toInt(), 1);
        // A word on no page.
        QTest::keyClick(window, Qt::Key_A, Qt::ControlModifier);
        type(QStringLiteral("zeta"));
        QTRY_COMPARE(named("readerMatches")->property("text").toString(), QStringLiteral("Not found"));
        // Esc closes find and leaves the PDF open; the next goes back to the Library.
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!query->isVisible());
        QVERIFY(window->isVisible());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_pages_text_is_selected_and_copied()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QGuiApplication::clipboard()->clear();
        QTest::keyClick(window, Qt::Key_A, Qt::ControlModifier);
        QTRY_VERIFY(reader->property("selectedText").toString().contains(QStringLiteral("alpha one")));
        QTest::keyClick(window, Qt::Key_C, Qt::ControlModifier);
        QTRY_VERIFY(QGuiApplication::clipboard()->text().contains(QStringLiteral("alpha one")));
        // Another page's text, and the first page's selection gone.
        QTest::keyClick(window, Qt::Key_End);
        QTRY_COMPARE(reader->property("page").toInt(), 2);
        QTest::keyClick(window, Qt::Key_A, Qt::ControlModifier);
        QTRY_VERIFY(reader->property("selectedText").toString().contains(QStringLiteral("the end")));
        QVERIFY(!reader->property("selectedText").toString().contains(QStringLiteral("alpha")));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_link_goes_to_its_page_and_only_the_web_or_mail_leave()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QList<QQuickItem *> links;
        QTRY_COMPARE((links = itemsNamed(QStringLiteral("pdfLink"))).size(), 2);
        QQuickItem *inside = links.first()->property("page").toInt() == 2 ? links.first() : links.last();
        QCOMPARE(inside->property("page").toInt(), 2);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, centreOf(inside).toPoint());
        QTRY_COMPARE(reader->property("page").toInt(), 2);

        // Where a link may lead: never a file or a program, whatever the PDF says.
        const auto followable = [&](const QString &url) {
            QVariant result;
            QMetaObject::invokeMethod(reader, "followable", Q_RETURN_ARG(QVariant, result), Q_ARG(QVariant, url));
            return result.toBool();
        };
        QVERIFY(followable("https://qorsync.dev/"));
        QVERIFY(followable("HTTP://example.org"));
        QVERIFY(followable("mailto:someone@example.org"));
        QVERIFY(!followable("file:///C:/Windows/System32/notepad.exe"));
        QVERIFY(!followable("C:/Windows/System32/cmd.exe"));
        QVERIFY(!followable("javascript:alert(1)"));
        QVERIFY(!followable("\\\\server\\share\\run.exe"));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void the_pages_and_contents_sit_beside_it()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QTRY_VERIFY(reader->property("hasContents").toBool());
        QTest::keyClick(window, Qt::Key_F9);
        QTRY_VERIFY(reader->property("panel").toBool());
        QQuickItem *thumbnails = named("thumbnails");
        QTRY_VERIFY(thumbnails->isVisible());
        QTRY_COMPARE(thumbnails->property("count").toInt(), 3);
        // The keyboard goes through the small pages, and Return opens one.
        thumbnails->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(reader->property("page").toInt(), 2);
        // The contents: the outline the PDF holds, each entry going to its page.
        QList<QQuickItem *> tabs;
        QTRY_COMPARE((tabs = itemsNamed(QStringLiteral("contentsTab"))).size(), 1);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, centreOf(tabs.first()).toPoint());
        QQuickItem *contents = named("contents");
        QTRY_VERIFY(contents->isVisible());
        QTRY_COMPARE(contents->property("rows").toInt(), 2);
        QList<QQuickItem *> start;
        QTRY_COMPARE((start = itemsUnder(contents, [](QQuickItem *i) { return i->property("title").toString() == QLatin1String("Start"); })).size(), 1);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, centreOf(start.first()).toPoint());
        QTRY_COMPARE(reader->property("page").toInt(), 0);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_locked_pdf_opens_with_its_password_and_not_without()
    {
        QQuickItem *reader = read(dir.filePath("locked.pdf"));
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("locked").toBool());
        QCOMPARE(reader->property("pageCount").toInt(), 0);
        QQuickItem *password = named("password");
        QTRY_VERIFY(password->hasActiveFocus());
        type(QStringLiteral("nope"));
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(reader->property("wrongPassword").toBool());
        QVERIFY(reader->property("locked").toBool());
        QCOMPARE(password->property("text").toString(), QString());
        // The same wrong one again is still wrong.
        reader->setProperty("wrongPassword", false);
        type(QStringLiteral("nope"));
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(reader->property("wrongPassword").toBool());
        type(QStringLiteral("open sesame"));
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 1);
        QVERIFY(!reader->property("locked").toBool());
        QCOMPARE(password->property("text").toString(), QString());
        // Its words were sealed: reading them proves it was opened, not just let through.
        QTRY_VERIFY(reader->hasActiveFocus());
        QTest::keyClick(window, Qt::Key_A, Qt::ControlModifier);
        QTRY_VERIFY(reader->property("selectedText").toString().contains(QStringLiteral("secret words")));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_password_is_kept_only_while_its_pdf_is_open()
    {
        QQuickItem *reader = read(dir.filePath("locked.pdf"));
        QVERIFY(reader);
        QTRY_VERIFY(named("password")->hasActiveFocus());
        type(QStringLiteral("open sesame"));
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 1);
        // Another PDF with the same password, opened over it: it asks again, and nothing of the first is left.
        QMetaObject::invokeMethod(link, "openRequested", Q_ARG(QString, dir.filePath("locked too.pdf")), Q_ARG(QString, QStringLiteral("document")), Q_ARG(double, 0));
        QTRY_COMPARE(window->property("path").toString(), dir.filePath("locked too.pdf"));
        QTRY_VERIFY(reader->property("locked").toBool());
        QCOMPARE(reader->property("pageCount").toInt(), 0);
        QVERIFY(!reader->property("wrongPassword").toBool());
        QCOMPARE(reader->property("selectedText").toString(), QString());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_broken_pdf_says_it_cannot_be_read()
    {
        QQuickItem *reader = read(dir.filePath("broken.pdf"));
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("unreadable").toBool());
        QQuickItem *button = named("openElsewhere");
        QVERIFY(button);
        QTRY_VERIFY(button->isVisible());
        QVERIFY(!named("pages")->isVisible());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void its_controls_are_reached_by_tab()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        // Back, pages, find, zoom out, zoom in, fit, full screen, open elsewhere, and the page's number.
        QStringList reached;
        for (int i = 0; i < 9; ++i) {
            QTest::keyClick(window, Qt::Key_Tab);
            QQuickItem *focused = window->activeFocusItem();
            QVERIFY(focused);
            if (focused->objectName() != QLatin1String("pageField"))
                QVERIFY2(focused->property("visualFocus").toBool(), qPrintable(focused->objectName()));
            reached << (focused->objectName().isEmpty() ? QString::fromLatin1(focused->metaObject()->className()) : focused->objectName());
        }
        QVERIFY2(reached.contains(QStringLiteral("pageField")), qPrintable(reached.join(", ")));
        QTest::keyClick(window, Qt::Key_Escape);  // out of the page's number
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }
};

QTEST_MAIN(TestReader)
#include "tst_reader.moc"
