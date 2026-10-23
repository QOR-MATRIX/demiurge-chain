// The reader (ADR-094 decision 3, L9.5): a PDF opens in QOR Media with every page, fitted to the width; the keys go
// through it and zoom it; find marks each match and goes to it; a page's text is selected and copied; a link goes to its
// page, and only the web or mail leave the document; the pages and the contents sit beside it; a locked PDF opens with
// its password and not without; a broken one says so; and every control is reached by Tab. Every PDF here is written by
// the test, byte by byte, so what each holds is known exactly.

#include <QClipboard>
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QJSValue>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QTest>
#include <QWheelEvent>
#include <QtQml/qqmlextensionplugin.h>

#include "medialink.h"
#include "testpdf.h"

Q_IMPORT_QML_PLUGIN(QOR_MediaPlugin)

using qormedia::MediaLink;
using namespace testpdf;

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
    // A three-page PDF of its own, for a test that changes it: "alpha one", "beta two", "gamma".
    QString fresh(const QString &name)
    {
        const QString path = dir.filePath(name);
        save(path, writePdf({{{"alpha one"}, {}}, {{"beta two"}, {}}, {{"gamma"}, {}}}));
        return path;
    }
    // The reader's pages as they show: each one's page in the file and its added turn.
    using Layout = QList<QPair<int, int>>;
    static Layout layoutOf(QQuickItem *reader)
    {
        Layout layout;
        for (const QVariant &entry : reader->property("layout").value<QJSValue>().toVariant().toList()) {
            const QVariantMap map = entry.toMap();
            layout.append({map.value("page").toInt(), map.value("turn").toInt()});
        }
        return layout;
    }
    // The copies a save shows while it writes, which it deletes once done.
    static QStringList copiesLeft()
    {
        return QDir(QDir::temp().filePath("qor-media")).entryList({"saving-*"}, QDir::Files);
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
        // Two jumps at once, the second before the view has drawn the first: the last one wins, and stays.
        QTest::keyClick(window, Qt::Key_End);
        QTest::keyClick(window, Qt::Key_Home);
        QTest::qWait(400);
        QCOMPARE(reader->property("page").toInt(), 0);
        QCOMPARE(pages->property("contentY").toReal(), top);
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
        // Clicked once both entries stand where they will stay: the view lays its rows out as it draws.
        const auto entry = [&](const char *title) {
            const QList<QQuickItem *> found = itemsUnder(contents, [&](QQuickItem *i) { return i->property("title").toString() == QLatin1String(title); });
            return found.size() == 1 ? found.first() : nullptr;
        };
        QTRY_VERIFY(entry("Start") && entry("End") && entry("End")->y() >= entry("Start")->y() + entry("Start")->height());
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, centreOf(entry("Start")).toPoint());
        QTRY_COMPARE(reader->property("page").toInt(), 0);
        // And by the keyboard: the arrows choose an entry, Return goes there.
        contents->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(reader->property("page").toInt(), 2);
        QTest::keyClick(window, Qt::Key_Up);
        QTest::keyClick(window, Qt::Key_Return);
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

    void pages_are_turned_deleted_moved_undone_and_saved()
    {
        const QString path = fresh("changes.pdf");
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QTRY_VERIFY(reader->property("editable").toBool());
        const QByteArray before = bytesOf(path);
        // The first page turned, the last deleted, the second moved before the first: each shows at once.
        QTest::keyClick(window, Qt::Key_R);
        QTRY_COMPARE(layoutOf(reader), (Layout{{0, 1}, {1, 0}, {2, 0}}));
        QVERIFY(reader->property("dirty").toBool());
        QTest::keyClick(window, Qt::Key_End);
        QTRY_COMPARE(reader->property("page").toInt(), 2);
        QTest::keyClick(window, Qt::Key_Delete);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 2);
        QCOMPARE(reader->property("page").toInt(), 1);
        QTest::keyClick(window, Qt::Key_Up, Qt::AltModifier);
        QTRY_COMPARE(layoutOf(reader), (Layout{{1, 0}, {0, 1}}));
        QCOMPARE(reader->property("page").toInt(), 0);
        // Undo and redo walk the changes; none of it has reached the file.
        QTest::keyClick(window, Qt::Key_Z, Qt::ControlModifier);
        QTRY_COMPARE(layoutOf(reader), (Layout{{0, 1}, {1, 0}}));
        QTest::keyClick(window, Qt::Key_Y, Qt::ControlModifier);
        QTRY_COMPARE(layoutOf(reader), (Layout{{1, 0}, {0, 1}}));
        QCOMPARE(bytesOf(path), before);
        // Saved: the file holds the pages as they showed, and the reader shows the file as it now is.
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        const Held held = readBack(path);
        QCOMPARE(held.text, (QStringList{"beta two", "alpha one"}));
        QCOMPARE(held.turns, (QList<int>{0, 1}));
        QTRY_COMPARE(layoutOf(reader), (Layout{{0, 0}, {1, 0}}));
        QCOMPARE(reader->property("undoStack").value<QJSValue>().property("length").toInt(), 0);
        // The copy shown while saving is gone.
        QCOMPARE(copiesLeft(), QStringList());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void the_last_page_stays()
    {
        const QString path = dir.filePath("one page.pdf");
        save(path, writePdf({{{"only"}, {}}}));
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTest::keyClick(window, Qt::Key_Delete);
        QTRY_VERIFY(!reader->property("notice").toString().isEmpty());
        QCOMPARE(reader->property("pageCount").toInt(), 1);
        QVERIFY(!reader->property("dirty").toBool());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void going_with_changes_unsaved_asks_first()
    {
        const QString path = fresh("unsaved.pdf");
        const QByteArray before = bytesOf(path);
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTest::keyClick(window, Qt::Key_R);
        QTRY_VERIFY(reader->property("dirty").toBool());
        // Esc asks; Esc again keeps the changes and the PDF open.
        QQuickItem *asking = named("asking");
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(asking->isVisible());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!asking->isVisible());
        QVERIFY(window->isVisible());
        QVERIFY(reader->property("dirty").toBool());
        // Asked again, "Don't save": the PDF goes, and the file is as it was.
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(asking->isVisible());
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, centreOf(named("askDiscard")).toPoint());
        QTRY_VERIFY(!window->isVisible());
        QCOMPARE(bytesOf(path), before);

        // Opened again and turned: "Save", the answer the question starts on, writes it before going.
        reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTest::keyClick(window, Qt::Key_R);
        QTRY_VERIFY(reader->property("dirty").toBool());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(named("asking")->isVisible());
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(!window->isVisible());
        QCOMPARE(readBack(path).turns, (QList<int>{1, 0, 0}));
        QCOMPARE(copiesLeft(), QStringList());
    }

    void a_save_that_fails_keeps_the_changes_and_the_file()
    {
        const QString path = fresh("read-only.pdf");
        const QByteArray before = bytesOf(path);
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTest::keyClick(window, Qt::Key_R);
        QTRY_VERIFY(reader->property("dirty").toBool());
        QVERIFY(QFile::setPermissions(path, QFile::ReadOwner | QFile::ReadUser | QFile::ReadGroup | QFile::ReadOther));
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("notice").toString().isEmpty());
        QFile::setPermissions(path, QFile::ReadOwner | QFile::WriteOwner | QFile::ReadUser | QFile::WriteUser);
        QVERIFY(reader->property("notice").toString() != QLatin1String("Saved"));
        QVERIFY(reader->property("dirty").toBool());
        QTRY_COMPARE(layoutOf(reader), (Layout{{0, 1}, {1, 0}, {2, 0}}));
        QCOMPARE(bytesOf(path), before);
        // Still here to be saved once the file can be written.
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        QCOMPARE(readBack(path).turns, (QList<int>{1, 0, 0}));
        QCOMPARE(copiesLeft(), QStringList());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_locked_or_protected_pdf_is_read_not_changed()
    {
        // Opened with its password: its pages do not change.
        QQuickItem *reader = read(dir.filePath("locked.pdf"));
        QVERIFY(reader);
        QTRY_VERIFY(named("password")->hasActiveFocus());
        type(QStringLiteral("open sesame"));
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 1);
        QVERIFY(!reader->property("editable").toBool());
        QVERIFY(!named("turnPage")->isVisible());
        QTest::keyClick(window, Qt::Key_R);
        QTRY_VERIFY(!reader->property("notice").toString().isEmpty());
        QVERIFY(!reader->property("dirty").toBool());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());

        // Protected without a password: it opens, and is read, not changed.
        const QString guarded = dir.filePath("guarded.pdf");
        save(guarded, writePdf({{{"guarded"}, {}}, {{"more"}, {}}}, {}, {}, true));
        reader = read(guarded);
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 2);
        QVERIFY(!reader->property("editable").toBool());
        QTest::keyClick(window, Qt::Key_Delete);
        QTRY_VERIFY(!reader->property("notice").toString().isEmpty());
        QCOMPARE(reader->property("pageCount").toInt(), 2);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void its_controls_are_reached_by_tab()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QTRY_VERIFY(reader->property("editable").toBool());
        // Back, pages, find, zoom out, zoom in, fit, turn, delete, full screen, open elsewhere, and the page's number.
        QStringList reached;
        for (int i = 0; i < 11; ++i) {
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
