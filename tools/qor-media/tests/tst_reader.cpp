// The reader (ADR-094 decision 3, L9.5): a PDF opens in QOR Media with every page, fitted to the width; the keys go
// through it and zoom it; find marks each match and goes to it; a page's text is selected and copied; a link goes to its
// page, and only the web or mail leave the document; the pages and the contents sit beside it; a locked PDF opens with
// its password and not without; a broken one says so; and every control is reached by Tab. It is marked up: words
// highlighted, underlined and struck through, notes, ink and a picture of a signature, each shown at once, undone,
// redone and saved with the pages, and the PDF's own marks read and taken away. Its form is filled: words typed where the
// field is, a box ticked, a button of a group chosen, a list's option chosen, Tab from field to field and Esc out of
// them, each drawn at once, undone and saved; a signature field signed with a picture; a locked PDF's form read, not
// filled; and words half typed asked about before going. Every PDF here is written by the test, byte by byte, so what
// each holds is known exactly.

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

#include "activewindow.h"
#include "medialink.h"
#include "pdfdrawing.h"
#include "pdfforms.h"
#include "pdfmarks.h"
#include "testpdf.h"

Q_IMPORT_QML_PLUGIN(QOR_MediaPlugin)

using qormedia::MediaLink;
using qormedia::PdfDrawing;
using qormedia::PdfForms;
using qormedia::PdfMarks;
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
        // Each wait that fails says so. The window is asked to take the keyboard (activewindow.h); not becoming active
        // even so is the desktop's doing, not the reader's (HANDOFF §5).
        if (!QTest::qWaitFor([&] { return window->isVisible(); })) {
            qWarning("The window did not show.");
            return nullptr;
        }
        if (!activewindow::activate(window)) {
            qWarning("The window showed but did not become active: another window kept the keyboard.");
            return nullptr;
        }
        QQuickItem *reader = nullptr;
        if (!QTest::qWaitFor([&] { return (reader = window->findChild<QQuickItem *>(QStringLiteral("reader"))) != nullptr; })) {
            qWarning("The window showed, but not the reader.");
            return nullptr;
        }
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
    // One of the reader's lists of marks: those made and not saved, those of the PDF's own taken away, and its own.
    static QVariantList marksOf(QQuickItem *reader, const char *which)
    {
        // A list made in QML is a script value; one from C++ is a list.
        const QVariant list = reader->property(which);
        return list.metaType() == QMetaType::fromType<QJSValue>() ? list.value<QJSValue>().toVariant().toList() : list.toList();
    }
    // The PDF's own marks the reader has read on a page of its file.
    static QVariantList heldOn(QQuickItem *reader, int page)
    {
        QVariant held;
        QMetaObject::invokeMethod(reader, "heldOn", Q_RETURN_ARG(QVariant, held), Q_ARG(QVariant, page));
        return held.metaType() == QMetaType::fromType<QJSValue>() ? held.value<QJSValue>().toVariant().toList() : held.toList();
    }
    // How many pixels of the pages, as the window shows them, `test` holds for. Only the pages: the colours a mark can be
    // are in the bar above them too.
    template<typename Test>
    int shownWhere(Test test)
    {
        const QImage shown = window->grabWindow();
        QQuickItem *pages = named("pages");
        const qreal ratio = shown.devicePixelRatio();
        const QRect area = QRectF(pages->mapToScene(QPointF(0, 0)) * ratio, pages->size() * ratio).toAlignedRect().intersected(shown.rect());
        int n = 0;
        for (int y = area.top(); y <= area.bottom(); ++y)
            for (int x = area.left(); x <= area.right(); ++x)
                n += test(QColor(shown.pixel(x, y)));
        return n;
    }
    static bool yellowed(const QColor &p)
    {
        return p.red() > 230 && p.green() > 200 && p.blue() > 110 && p.blue() < 200;
    }
    static bool reddish(const QColor &p)
    {
        return p.red() > 200 && p.green() < 60 && p.blue() < 60;
    }
    // Press what `item` is, as a pointer would.
    bool click(QQuickItem *item)
    {
        if (!item || !item->isVisible())
            return false;
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, centreOf(item).toPoint());
        return true;
    }
    // Drag across the window from `from` by each of `steps`, as a pointer would.
    void drag(QPoint from, const QList<QPoint> &steps)
    {
        QTest::mousePress(window, Qt::LeftButton, Qt::NoModifier, from);
        QPoint at = from;
        for (const QPoint &step : steps) {
            for (int i = 1; i <= 4; ++i)
                QTest::mouseMove(window, at + step * i / 4);
            at += step;
        }
        QTest::mouseRelease(window, Qt::LeftButton, Qt::NoModifier, at);
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
    // The one item named so that shows, wherever it is: on a page, or over everything, as a list's options are.
    QQuickItem *shown(const QString &name)
    {
        const QList<QQuickItem *> showing = itemsUnder(window->contentItem(), [&](QQuickItem *i) { return i->objectName() == name && i->isVisible(); });
        return showing.size() == 1 ? showing.first() : nullptr;
    }
    QPointF centreOf(QQuickItem *item)
    {
        return item->mapToScene(QPointF(item->width() / 2, item->height() / 2));
    }

    // A form of one page, each field a widget at the place named: words, a check box, a group of three, a drop-down
    // list, words read only, and a signature field.
    enum At { Name, Agree, Small, Medium, Large, Colour, Fixed, Sign };
    QString form(const QString &name, const QByteArray &password = {})
    {
        Page page{{"Form"}, {}};
        page.fields = {Field{Field::Text, "name", QRectF(100, 600, 200, 24), "Ada Lovelace"},
                       Field{Field::Check, "agree", QRectF(100, 560, 20, 20), "Off", {"Yes"}},
                       Field{Field::Radio, "size", QRectF(100, 520, 20, 20), "S", {"S", "M", "L"}, {QRectF(140, 520, 20, 20), QRectF(180, 520, 20, 20)}},
                       Field{Field::Combo, "colour", QRectF(100, 470, 120, 24), "Red", {"Red", "Green", "Blue"}},
                       Field{Field::Text, "fixed", QRectF(100, 420, 200, 24), "Locked", {}, {}, 1},
                       Field{Field::Signature, "sign", QRectF(300, 100, 200, 50)}};
        const QString path = dir.filePath(name);
        save(path, writePdf({page}, {}, password));
        return path;
    }
    // A value the reader holds, as plain data: a map made in QML is a script value.
    static QVariant plainOf(const QVariant &v)
    {
        return v.metaType() == QMetaType::fromType<QJSValue>() ? v.value<QJSValue>().toVariant() : v;
    }
    // The field on the page at a place, as the reader shows it.
    QQuickItem *fieldAt(int index)
    {
        for (QQuickItem *item : itemsNamed(QStringLiteral("formField")))
            if (plainOf(item->property("field")).toMap().value("index").toInt() == index)
                return item;
        return nullptr;
    }
    // Whether a field shows the focus ring.
    bool ringOn(QQuickItem *field)
    {
        const QList<QQuickItem *> rings = itemsUnder(field, [](QQuickItem *i) { return i->objectName() == QLatin1String("focusRing"); });
        return !rings.isEmpty() && rings.first()->isVisible();
    }
    // What the reader holds filled, by field name.
    static QVariantMap fillsOf(QQuickItem *reader)
    {
        return plainOf(reader->property("fills")).toMap();
    }
    static QVariantMap focusOf(QQuickItem *reader)
    {
        return plainOf(reader->property("fieldFocus")).toMap();
    }
    // How many pixels under an item, as the window shows it, are dark: a box ticked, a button chosen.
    int darkUnder(QQuickItem *item)
    {
        const QImage shown = window->grabWindow();
        const qreal ratio = shown.devicePixelRatio();
        const QRect area = QRectF(item->mapToScene(QPointF(0, 0)) * ratio, item->size() * ratio).toAlignedRect().intersected(shown.rect());
        int n = 0;
        for (int y = area.top(); y <= area.bottom(); ++y)
            for (int x = area.left(); x <= area.right(); ++x)
                n += QColor(shown.pixel(x, y)).lightness() < 80;
        return n;
    }
    // The fields' values read back from the file, by field name.
    static QMap<QString, QVariantMap> filedIn(const QString &path)
    {
        QMap<QString, QVariantMap> filed;
        for (const QVariant &w : PdfForms::read(path))
            filed.insert(w.toMap().value("name").toString() + (w.toMap().value("kind") == QStringLiteral("radio") ? w.toMap().value("state").toString() : QString()), w.toMap());
        return filed;
    }

private slots:
    void initTestCase()
    {
        activewindow::mayTakeTheKeyboard();
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
        activewindow::holdTheKeyboard(window);
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
        // Its page is drawn too, which needs the password as much as reading it does.
        QList<QQuickItem *> images;
        QTRY_VERIFY(!(images = itemsNamed(QStringLiteral("pageImage"))).isEmpty() && images.first()->property("status").toInt() == 1);
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
        // Nor marked: no button for it, and a note asked for by its key says so and is not made.
        QVERIFY(!named("markUp")->isVisible());
        reader->setProperty("notice", QString());
        QTest::keyClick(window, Qt::Key_N);
        QTRY_VERIFY(!reader->property("notice").toString().isEmpty());
        QVERIFY(!named("noteEditor")->isVisible());
        QCOMPARE(marksOf(reader, "added").size(), 0);
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

    void a_pdfs_own_marks_show_and_its_file_is_let_go_after()
    {
        // A red box near the top of the page, a mark with its own appearance: Qt PDF's page image would leave it out.
        const QString path = dir.filePath("marked.pdf");
        save(path, writePdf({{{"words"}, {}, {QRectF(100, 560, 200, 100)}}}));
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 1);
        QList<QQuickItem *> images;
        QTRY_VERIFY(!(images = itemsNamed(QStringLiteral("pageImage"))).isEmpty() && images.first()->property("status").toInt() == 1);
        QVERIFY(images.first()->property("source").toString().startsWith(QLatin1String("image://pdfpage/")));
        const QImage shown = window->grabWindow();
        int red = 0;
        for (int y = 0; y < shown.height(); ++y)
            for (int x = 0; x < shown.width(); ++x) {
                const QRgb p = shown.pixel(x, y);
                red += qRed(p) > 200 && qGreen(p) < 60 && qBlue(p) < 60;
            }
        QVERIFY2(red > 500, qPrintable(QString::number(red)));
        // Back to the Library: the reader lets go of what it drew from (tst_pdfpages holds that closing frees the file).
        QCOMPARE(PdfDrawing::openCount(), 1);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
        QTRY_COMPARE(PdfDrawing::openCount(), 0);
    }

    void words_are_marked_undone_redone_and_saved()
    {
        const QString path = fresh("marked words.pdf");
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        const QByteArray before = bytesOf(path);
        // M shows the marks; the page's words are selected and H highlights them, in yellow, and lets go of them.
        QTest::keyClick(window, Qt::Key_M);
        QQuickItem *bar = named("markupBar");
        QTRY_VERIFY(bar->isVisible());
        QTest::keyClick(window, Qt::Key_A, Qt::ControlModifier);
        QTRY_VERIFY(reader->property("selectedText").toString().contains(QStringLiteral("alpha one")));
        QTest::keyClick(window, Qt::Key_H);
        QTRY_COMPARE(marksOf(reader, "added").size(), 1);
        const QVariantMap highlight = marksOf(reader, "added").first().toMap();
        QCOMPARE(highlight.value("kind").toString(), QStringLiteral("highlight"));
        QCOMPARE(highlight.value("page").toInt(), 0);
        QCOMPARE(highlight.value("colour").toString(), PdfMarks::colours().first().toMap().value("colour").toString());
        QVERIFY(!highlight.value("rects").toList().isEmpty());
        QVERIFY(reader->property("dirty").toBool());
        QCOMPARE(reader->property("selectedText").toString(), QString());
        // Shown at once, before it is saved.
        QTRY_VERIFY(shownWhere(yellowed) > 300);
        // With no words selected, U says so and marks nothing.
        QTest::keyClick(window, Qt::Key_U);
        QTRY_VERIFY(!reader->property("notice").toString().isEmpty());
        QCOMPARE(marksOf(reader, "added").size(), 1);
        // Underlined by its button, in the green chosen first.
        QTest::keyClick(window, Qt::Key_A, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("selectedText").toString().isEmpty());
        const QList<QQuickItem *> green = itemsUnder(bar, [](QQuickItem *i) { return i->objectName() == QLatin1String("swatchGreen"); });
        QCOMPARE(green.size(), 1);
        QVERIFY(click(green.first()));
        QVERIFY(click(named("markUnderline")));
        QTRY_COMPARE(marksOf(reader, "added").size(), 2);
        QCOMPARE(marksOf(reader, "added").last().toMap().value("kind").toString(), QStringLiteral("underline"));
        QCOMPARE(marksOf(reader, "added").last().toMap().value("colour").toString(), PdfMarks::colours()[1].toMap().value("colour").toString());
        // Undone and redone with the pages' own changes; nothing has reached the file.
        QTest::keyClick(window, Qt::Key_R);
        QTRY_COMPARE(layoutOf(reader), (Layout{{0, 1}, {1, 0}, {2, 0}}));
        QTest::keyClick(window, Qt::Key_Z, Qt::ControlModifier);
        QTRY_COMPARE(layoutOf(reader), (Layout{{0, 0}, {1, 0}, {2, 0}}));
        QCOMPARE(marksOf(reader, "added").size(), 2);
        QTest::keyClick(window, Qt::Key_Z, Qt::ControlModifier);
        QTRY_COMPARE(marksOf(reader, "added").size(), 1);
        QTest::keyClick(window, Qt::Key_Y, Qt::ControlModifier);
        QTRY_COMPARE(marksOf(reader, "added").size(), 2);
        QCOMPARE(bytesOf(path), before);
        // Saved: the file holds both, and the reader shows them as the PDF's own.
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 2);
        QCOMPARE(annots[0].subtype, FPDF_ANNOT_HIGHLIGHT);
        QCOMPARE(annots[1].subtype, FPDF_ANNOT_UNDERLINE);
        QCOMPARE(annots[0].page, 0);
        QCOMPARE(marksOf(reader, "added").size(), 0);
        QTRY_COMPARE(heldOn(reader, 0).size(), 2);
        QTRY_VERIFY(shownWhere(yellowed) > 300);
        QCOMPARE(copiesLeft(), QStringList());
        // Esc puts the marks away, then goes back.
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!bar->isVisible());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_note_is_written_saved_and_read_by_a_tap()
    {
        const QString path = fresh("noted.pdf");
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        // N writes a note at the top of the page in view; Ctrl+Return keeps it.
        QTest::keyClick(window, Qt::Key_N);
        QQuickItem *editor = named("noteEditor");
        QTRY_VERIFY(editor->isVisible());
        QTRY_VERIFY(named("noteText")->hasActiveFocus());
        type(QStringLiteral("call back"));
        QTest::keyClick(window, Qt::Key_Return, Qt::ControlModifier);
        QTRY_VERIFY(!editor->isVisible());
        QTRY_COMPARE(marksOf(reader, "added").size(), 1);
        const QVariantMap note = marksOf(reader, "added").first().toMap();
        QCOMPARE(note.value("kind").toString(), QStringLiteral("note"));
        QCOMPARE(note.value("text").toString(), QStringLiteral("call back"));
        // Another, left with Esc: nothing is made.
        QTest::keyClick(window, Qt::Key_N);
        QTRY_VERIFY(named("noteText")->hasActiveFocus());
        type(QStringLiteral("never mind"));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!editor->isVisible());
        QCOMPARE(marksOf(reader, "added").size(), 1);
        // Saved, and then the PDF's own: a tap shows its words, to read, not to change.
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 1);
        QCOMPARE(annots[0].subtype, FPDF_ANNOT_TEXT);
        QCOMPARE(annots[0].contents, QStringLiteral("call back"));
        QTRY_COMPARE(heldOn(reader, 0).size(), 1);
        QList<QQuickItem *> own;
        QTRY_COMPARE((own = itemsNamed(QStringLiteral("ownMark"))).size(), 1);
        QVERIFY(click(own.first()));
        QTRY_VERIFY(editor->isVisible());
        QCOMPARE(named("noteText")->property("text").toString(), QStringLiteral("call back"));
        QVERIFY(named("noteText")->property("readOnly").toBool());
        // A tap on the page beside it puts it away.
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, centreOf(named("pages")).toPoint());
        QTRY_VERIFY(!editor->isVisible());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void ink_is_drawn_by_hand_and_saved()
    {
        const QString path = fresh("inked.pdf");
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        // D draws, and shows the marks.
        QTest::keyClick(window, Qt::Key_D);
        QTRY_COMPARE(reader->property("tool").toString(), QStringLiteral("ink"));
        QVERIFY(reader->property("marking").toBool());
        const QPointF middle = centreOf(named("pages"));
        drag(middle.toPoint(), {QPoint(60, 20), QPoint(60, -20)});
        QTRY_COMPARE(marksOf(reader, "added").size(), 1);
        const QVariantMap ink = marksOf(reader, "added").first().toMap();
        QCOMPARE(ink.value("kind").toString(), QStringLiteral("ink"));
        QCOMPARE(ink.value("strokes").toList().size(), 1);
        QVERIFY(ink.value("strokes").toList().first().toList().size() >= 3);
        // Drawing, the page did not scroll and no words were selected.
        QCOMPARE(reader->property("selectedText").toString(), QString());
        // D again stops; Ctrl+S saves it as ink.
        QTest::keyClick(window, Qt::Key_D);
        QTRY_COMPARE(reader->property("tool").toString(), QString());
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 1);
        QCOMPARE(annots[0].subtype, FPDF_ANNOT_INK);
        QCOMPARE(annots[0].strokes, 1);
        QTest::keyClick(window, Qt::Key_Escape);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_signature_is_typed_placed_moved_and_saved()
    {
        const QString path = fresh("signed.pdf");
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTest::keyClick(window, Qt::Key_M);
        QTRY_VERIFY(named("markupBar")->isVisible());
        QVERIFY(click(named("markSign")));
        QQuickItem *pad = named("signaturePad");
        QTRY_VERIFY(pad->isVisible());
        QVERIFY(click(named("signType")));
        QTRY_VERIFY(named("signatureName")->hasActiveFocus());
        type(QStringLiteral("Ada"));
        QTRY_VERIFY(named("signPlace")->isEnabled());
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(!pad->isVisible());
        QTRY_COMPARE(marksOf(reader, "added").size(), 1);
        QVariantMap signature = marksOf(reader, "added").first().toMap();
        QCOMPARE(signature.value("kind").toString(), QStringLiteral("signature"));
        QVERIFY(!signature.value("shapes").toList().isEmpty());
        QVERIFY(signature.value("width").toDouble() <= 180);
        // Placed, it says it certifies nothing, and is chosen: the arrows move it, a point or ten.
        QVERIFY(reader->property("notice").toString().contains(QStringLiteral("does not certify")));
        const double x = signature.value("x").toDouble(), y = signature.value("y").toDouble(), width = signature.value("width").toDouble();
        QTest::keyClick(window, Qt::Key_Right);
        QTest::keyClick(window, Qt::Key_Down, Qt::ShiftModifier);
        QTRY_COMPARE(marksOf(reader, "added").first().toMap().value("x").toDouble(), x + 1);
        QCOMPARE(marksOf(reader, "added").first().toMap().value("y").toDouble(), y + 10);
        // ] makes it larger, about its middle.
        QTest::keyClick(window, Qt::Key_BracketRight);
        QTRY_VERIFY(marksOf(reader, "added").first().toMap().value("width").toDouble() > width);
        // Dragged by the pointer, it moves as far, in one step undone at once.
        signature = marksOf(reader, "added").first().toMap();
        QList<QQuickItem *> placed;
        const auto shownMarks = [&] {
            return itemsUnder(window->contentItem(), [](QQuickItem *i) { return i->objectName() == QLatin1String("mark") && i->isVisible(); });
        };
        QTRY_COMPARE((placed = shownMarks()).size(), 1);
        const QRectF box(signature.value("x").toDouble(), signature.value("y").toDouble(), signature.value("width").toDouble(),
                         signature.value("height").toDouble());
        const qreal zoom = reader->property("zoom").toReal();
        const QPointF on = placed.first()->mapToScene(box.center() * zoom);
        drag(on.toPoint(), {QPoint(30, 0)});
        QTRY_VERIFY2(std::abs(marksOf(reader, "added").first().toMap().value("x").toDouble() - (box.x() + 30 / zoom)) < 0.5 / zoom + 0.01,
                     qPrintable(QStringLiteral("%1, not %2").arg(marksOf(reader, "added").first().toMap().value("x").toDouble()).arg(box.x() + 30 / zoom)));
        QTest::keyClick(window, Qt::Key_Z, Qt::ControlModifier);
        QTRY_COMPARE(marksOf(reader, "added").first().toMap().value("x").toDouble(), box.x());
        // Let go of past its last move, it goes where it was let go.
        QTest::mousePress(window, Qt::LeftButton, Qt::NoModifier, on.toPoint());
        QTest::mouseMove(window, on.toPoint() + QPoint(15, 0));
        QTest::mouseMove(window, on.toPoint() + QPoint(20, 0));
        QTest::mouseRelease(window, Qt::LeftButton, Qt::NoModifier, on.toPoint() + QPoint(40, 0));
        QTRY_VERIFY2(std::abs(marksOf(reader, "added").first().toMap().value("x").toDouble() - (box.x() + 40 / zoom)) < 0.5 / zoom + 0.01,
                     qPrintable(QStringLiteral("%1, not %2").arg(marksOf(reader, "added").first().toMap().value("x").toDouble()).arg(box.x() + 40 / zoom)));
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 1);
        QCOMPARE(annots[0].subtype, FPDF_ANNOT_STAMP);
        QCOMPARE(annots[0].contents, QStringLiteral("Signature"));
        QTest::keyClick(window, Qt::Key_Escape);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_signature_is_drawn_on_the_pad()
    {
        QQuickItem *reader = read(fresh("drawn signature.pdf"));
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTest::keyClick(window, Qt::Key_M);
        QTRY_VERIFY(click(named("markSign")));
        QTRY_VERIFY(named("signaturePad")->isVisible());
        // Nothing drawn, nothing to place.
        QVERIFY(!named("signPlace")->isEnabled());
        QQuickItem *area = named("signatureArea");
        QTRY_VERIFY(area->isVisible());
        drag(centreOf(area).toPoint() - QPoint(120, 0), {QPoint(60, -30), QPoint(60, 50), QPoint(80, -40)});
        QTRY_VERIFY(named("signPlace")->isEnabled());
        QVERIFY(click(named("signPlace")));
        QTRY_COMPARE(marksOf(reader, "added").size(), 1);
        const QVariantMap signature = marksOf(reader, "added").first().toMap();
        QCOMPARE(signature.value("strokes").toList().size(), 1);
        QVERIFY(signature.value("line").toDouble() > 0 && signature.value("line").toDouble() < 1);
        // Its box keeps the drawing's own shape: 200 pixels across and 50 down, with its line, so near four to one.
        const double aspect = signature.value("width").toDouble() / signature.value("height").toDouble();
        QVERIFY2(aspect > 3.4 && aspect < 4.3, qPrintable(QString::number(aspect)));
        for (const QVariant &p : signature.value("strokes").toList().first().toList()) {
            const QPointF at = p.toPointF();
            QVERIFY(at.x() >= 0 && at.x() <= 1 && at.y() >= 0 && at.y() <= 1);
        }
        QTest::keyClick(window, Qt::Key_Escape);  // the mark chosen
        QTest::keyClick(window, Qt::Key_Escape);  // the marks
        QTest::keyClick(window, Qt::Key_Escape);  // asks, as it is unsaved
        QTRY_VERIFY(named("asking")->isVisible());
        QVERIFY(click(named("askDiscard")));
        QTRY_VERIFY(!window->isVisible());
    }

    void the_pdfs_own_marks_are_taken_away_at_once_and_on_saving()
    {
        // A red box near the top of the page, a mark the PDF holds.
        const QString path = dir.filePath("boxed.pdf");
        save(path, writePdf({{{"words"}, {}, {QRectF(100, 560, 200, 100)}}}));
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTRY_VERIFY(shownWhere(reddish) > 500);
        QTRY_COMPARE(heldOn(reader, 0).size(), 1);
        // Not chosen by a tap until the marks show.
        QList<QQuickItem *> own;
        QTRY_COMPARE((own = itemsNamed(QStringLiteral("ownMark"))).size(), 1);
        QVERIFY(!own.first()->isEnabled());
        QTest::keyClick(window, Qt::Key_M);
        QTRY_VERIFY(own.first()->isEnabled());
        QVERIFY(click(own.first()));
        QTRY_VERIFY(!reader->property("chosen").isNull() && reader->property("chosen").value<QJSValue>().isObject());
        QVERIFY(named("markRemove")->isVisible());
        // Delete takes it away at once, not the page; drawn without it.
        QTest::keyClick(window, Qt::Key_Delete);
        QTRY_COMPARE(marksOf(reader, "removed").size(), 1);
        QCOMPARE(reader->property("pageCount").toInt(), 1);
        QVERIFY(reader->property("dirty").toBool());
        QTRY_COMPARE(shownWhere(reddish), 0);
        QList<QQuickItem *> images = itemsNamed(QStringLiteral("pageImage"));
        QVERIFY(images.first()->property("source").toString().endsWith(QLatin1String("/0/0")));
        // Undone, it is back.
        QTest::keyClick(window, Qt::Key_Z, Qt::ControlModifier);
        QTRY_COMPARE(marksOf(reader, "removed").size(), 0);
        QTRY_VERIFY(shownWhere(reddish) > 500);
        QTest::keyClick(window, Qt::Key_Y, Qt::ControlModifier);
        QTRY_COMPARE(marksOf(reader, "removed").size(), 1);
        // Saved, it is gone from the file.
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        QCOMPARE(annotsOf(path).size(), 0);
        QTRY_COMPARE(heldOn(reader, 0).size(), 0);
        QTRY_COMPARE(shownWhere(reddish), 0);
        QTest::keyClick(window, Qt::Key_Escape);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_form_is_filled_where_it_is_shown_at_once_undone_and_saved()
    {
        const QString path = form(QStringLiteral("form.pdf"));
        const QByteArray before = bytesOf(path);
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        // Its fields over their widgets, and a word that it is a form.
        QTRY_COMPARE(itemsNamed(QStringLiteral("formField")).size(), 8);
        QTRY_VERIFY(reader->property("notice").toString().contains(QStringLiteral("form")));
        // Typed where the field is: the editor over it, holding its words.
        QQuickItem *name = fieldAt(Name);
        QVERIFY(click(name));
        QQuickItem *editor = nullptr;
        QTRY_VERIFY((editor = shown(QStringLiteral("fieldEditor"))) && editor->hasActiveFocus());
        QCOMPARE(editor->property("text").toString(), QStringLiteral("Ada Lovelace"));
        QTest::keyClick(window, Qt::Key_A, Qt::ControlModifier);
        type(QStringLiteral("Grace"));
        // Not kept yet, and not to be lost: going now would ask.
        QTRY_VERIFY(reader->property("dirty").toBool());
        QVERIFY(fillsOf(reader).isEmpty());
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(fillsOf(reader).value("name").toMap().value("text").toString(), QStringLiteral("Grace"));
        QTRY_VERIFY(!shown(QStringLiteral("fieldEditor")));
        QTRY_VERIFY(name->hasActiveFocus());
        // The page drawn again by PDFium, filled as it will be saved.
        QList<QQuickItem *> images;
        QTRY_VERIFY(!(images = itemsNamed(QStringLiteral("pageImage"))).isEmpty()
                    && images.first()->property("source").toString().contains(QStringLiteral("/0//")));
        // A box ticked and a button of the group chosen, each drawn at once.
        QQuickItem *agree = fieldAt(Agree);
        QVERIFY(darkUnder(agree) < 20);
        QVERIFY(click(agree));
        QTRY_VERIFY(fillsOf(reader).value("agree").toMap().value("checked").toBool());
        QTRY_VERIFY(darkUnder(agree) > 60);
        // Tapped, it has the keyboard without the focus ring, which shows for the keys alone.
        QTRY_VERIFY(agree->hasActiveFocus());
        QVERIFY(!ringOn(agree));
        QVERIFY(darkUnder(fieldAt(Small)) > 60);
        QVERIFY(click(fieldAt(Large)));
        QTRY_COMPARE(fillsOf(reader).value("size").toMap().value("index").toInt(), int(Large));
        QTRY_VERIFY(darkUnder(fieldAt(Large)) > 60);
        QTRY_VERIFY(darkUnder(fieldAt(Small)) < 20);
        // An option chosen from under the list, by the keys.
        QVERIFY(click(fieldAt(Colour)));
        QQuickItem *options = nullptr;
        QTRY_VERIFY((options = shown(QStringLiteral("fieldOptionList"))) && options->hasActiveFocus());
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Down);
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(fillsOf(reader).value("colour").toMap().value("chosen").toList().size(), 1);
        QCOMPARE(fillsOf(reader).value("colour").toMap().value("chosen").toList().first().toInt(), 2);
        QTRY_VERIFY(!shown(QStringLiteral("fieldOptionList")));
        // Undone and redone with the rest; nothing has reached the file.
        QTest::keyClick(window, Qt::Key_Z, Qt::ControlModifier);
        QTRY_VERIFY(!fillsOf(reader).contains("colour"));
        QTest::keyClick(window, Qt::Key_Y, Qt::ControlModifier);
        QTRY_VERIFY(fillsOf(reader).contains("colour"));
        QCOMPARE(bytesOf(path), before);
        // A box ticked back as the file has it leaves nothing to save for it.
        QVERIFY(click(agree));
        QTRY_VERIFY(!fillsOf(reader).contains("agree"));
        QVERIFY(click(agree));
        QTRY_VERIFY(fillsOf(reader).contains("agree"));
        // The field read only says so, and is not typed in. The whole page in view first, to reach it.
        QTest::keyClick(window, Qt::Key_0);
        QTRY_COMPARE(reader->property("fitting").toString(), QStringLiteral("page"));
        QTRY_VERIFY(fieldAt(Fixed) && centreOf(fieldAt(Fixed)).y() < window->height() - 40);
        reader->setProperty("notice", QString());
        QVERIFY(click(fieldAt(Fixed)));
        QTRY_VERIFY(reader->property("notice").toString().contains(QStringLiteral("read only")));
        QVERIFY(!shown(QStringLiteral("fieldEditor")));
        // Saved: the file holds them, read back as its own.
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        const QMap<QString, QVariantMap> filed = filedIn(path);
        QCOMPARE(filed["name"].value("value").toString(), QStringLiteral("Grace"));
        QVERIFY(filed["agree"].value("checked").toBool());
        QVERIFY(filed["sizeL"].value("checked").toBool());
        QVERIFY(!filed["sizeS"].value("checked").toBool());
        QCOMPARE(filed["colour"].value("value").toString(), QStringLiteral("Blue"));
        QCOMPARE(filed["fixed"].value("value").toString(), QStringLiteral("Locked"));
        QVERIFY(fillsOf(reader).isEmpty());
        QTRY_VERIFY(fieldAt(Agree) && darkUnder(fieldAt(Agree)) > 60);
        QCOMPARE(copiesLeft(), QStringList());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void tab_goes_through_the_fields_and_esc_leaves_them()
    {
        const QString path = form(QStringLiteral("tabbed.pdf"));
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTRY_COMPARE(itemsNamed(QStringLiteral("formField")).size(), 8);
        // Tab from the pages goes into the first field, to type in it.
        QTest::keyClick(window, Qt::Key_Tab);
        QTRY_COMPARE(focusOf(reader).value("index").toInt(), int(Name));
        QCOMPARE(focusOf(reader).value("mode").toString(), QStringLiteral("typing"));
        QQuickItem *editor = nullptr;
        QTRY_VERIFY((editor = shown(QStringLiteral("fieldEditor"))) && editor->hasActiveFocus());
        // Esc leaves the words as they were and the keyboard on the field; Esc again lets go of it, and stays.
        type(QStringLiteral(" Jr"));
        QTRY_VERIFY(reader->property("dirty").toBool());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!shown(QStringLiteral("fieldEditor")));
        QVERIFY(fillsOf(reader).isEmpty());
        QVERIFY(!reader->property("dirty").toBool());
        QTRY_VERIFY(fieldAt(Name)->hasActiveFocus());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(focusOf(reader).isEmpty());
        QVERIFY(window->isVisible());
        // In again: Tab keeps the words and goes on to the box, which Space ticks.
        QTest::keyClick(window, Qt::Key_Tab);
        QTRY_VERIFY((editor = shown(QStringLiteral("fieldEditor"))) && editor->hasActiveFocus());
        QTest::keyClick(window, Qt::Key_End);
        type(QStringLiteral("!"));
        QTest::keyClick(window, Qt::Key_Tab);
        QTRY_COMPARE(fillsOf(reader).value("name").toMap().value("text").toString(), QStringLiteral("Ada Lovelace!"));
        QTRY_COMPARE(focusOf(reader).value("index").toInt(), int(Agree));
        QTRY_VERIFY(fieldAt(Agree)->hasActiveFocus());
        QVERIFY(ringOn(fieldAt(Agree)));
        QTest::keyClick(window, Qt::Key_Space);
        QTRY_VERIFY(fillsOf(reader).value("agree").toMap().value("checked").toBool());
        // On through the group's buttons and the list, passing the field read only; past the last, back to the pages.
        for (const int next : {Small, Medium, Large, Colour}) {
            QTest::keyClick(window, Qt::Key_Tab);
            QTRY_COMPARE(focusOf(reader).value("index").toInt(), next);
        }
        QTest::keyClick(window, Qt::Key_Tab);
        QTRY_VERIFY(focusOf(reader).isEmpty());
        // Shift+Tab goes back, from the box to the words, to type in them.
        QTest::keyClick(window, Qt::Key_Tab);
        QTRY_COMPARE(focusOf(reader).value("mode").toString(), QStringLiteral("typing"));
        QTest::keyClick(window, Qt::Key_Tab);
        QTRY_COMPARE(focusOf(reader).value("index").toInt(), int(Agree));
        QTest::keyClick(window, Qt::Key_Tab, Qt::ShiftModifier);
        QTRY_COMPARE(focusOf(reader).value("index").toInt(), int(Name));
        QTRY_COMPARE(focusOf(reader).value("mode").toString(), QStringLiteral("typing"));
        // Ctrl+S with words typed and not kept keeps them, and saves them.
        QTRY_VERIFY((editor = shown(QStringLiteral("fieldEditor"))) && editor->hasActiveFocus());
        type(QStringLiteral("?"));
        QTest::keyClick(window, Qt::Key_S, Qt::ControlModifier);
        QTRY_VERIFY(!reader->property("dirty").toBool());
        QCOMPARE(filedIn(path)["name"].value("value").toString(), QStringLiteral("Ada Lovelace!?"));
        QVERIFY(filedIn(path)["agree"].value("checked").toBool());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_locked_pdfs_form_is_read_not_filled()
    {
        QQuickItem *reader = read(form(QStringLiteral("locked form.pdf"), "open sesame"));
        QVERIFY(reader);
        QTRY_VERIFY(named("password")->hasActiveFocus());
        type(QStringLiteral("open sesame"));
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 1);
        QTRY_COMPARE(itemsNamed(QStringLiteral("formField")).size(), 8);
        // Its words drawn, with its password; a tap says it is read, not filled; Tab does not go in.
        QTRY_VERIFY(darkUnder(fieldAt(Name)) > 30);
        reader->setProperty("notice", QString());
        QVERIFY(click(fieldAt(Name)));
        QTRY_VERIFY(reader->property("notice").toString().contains(QStringLiteral("not filled")));
        QVERIFY(!shown(QStringLiteral("fieldEditor")));
        QTest::keyClick(window, Qt::Key_Tab);
        QVERIFY(focusOf(reader).isEmpty());
        QVERIFY(!reader->property("dirty").toBool());
        QVERIFY(fillsOf(reader).isEmpty());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_form_kept_in_xfa_says_why_it_is_not_filled()
    {
        // The form's own default look traded for an XFA entry of the same length, so every offset holds.
        const QString path = form(QStringLiteral("xfa form.pdf"));
        QByteArray bytes = bytesOf(path);
        const QByteArray look = "/DA <" + QByteArray("/Helv 0 Tf 0 g").toHex() + ">";
        QCOMPARE(bytes.count(look), 1);
        bytes.replace(look, QByteArray("/XFA []").leftJustified(look.size(), ' '));
        save(path, bytes);
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTRY_COMPARE(itemsNamed(QStringLiteral("formField")).size(), 8);
        // Said as it opens, and again by a tap; Tab does not go in.
        QTRY_VERIFY(reader->property("notice").toString().contains(QStringLiteral("XFA")));
        reader->setProperty("notice", QString());
        QVERIFY(click(fieldAt(Name)));
        QTRY_VERIFY(reader->property("notice").toString().contains(QStringLiteral("XFA")));
        QVERIFY(!shown(QStringLiteral("fieldEditor")));
        QTest::keyClick(window, Qt::Key_Tab);
        QVERIFY(focusOf(reader).isEmpty());
        QVERIFY(!reader->property("dirty").toBool());
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void a_signature_field_takes_a_picture_of_a_signature()
    {
        QQuickItem *reader = read(form(QStringLiteral("sign here.pdf")));
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTRY_COMPARE(itemsNamed(QStringLiteral("formField")).size(), 8);
        // The whole page in view, to reach the field at its foot.
        QTest::keyClick(window, Qt::Key_0);
        QTRY_VERIFY(fieldAt(Sign) && centreOf(fieldAt(Sign)).y() < window->height() - 40);
        QVERIFY(click(fieldAt(Sign)));
        QTRY_VERIFY(named("signaturePad")->isVisible());
        QVERIFY(click(named("signType")));
        QTRY_VERIFY(named("signatureName")->hasActiveFocus());
        type(QStringLiteral("Ada"));
        QTRY_VERIFY(named("signPlace")->isEnabled());
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_COMPARE(marksOf(reader, "added").size(), 1);
        // In the field's middle, as large as it takes: 300 to 500 across and 642 to 692 down the page as laid out.
        const QVariantMap signature = marksOf(reader, "added").first().toMap();
        QCOMPARE(signature.value("kind").toString(), QStringLiteral("signature"));
        const QRectF box(signature.value("x").toDouble(), signature.value("y").toDouble(), signature.value("width").toDouble(),
                         signature.value("height").toDouble());
        QVERIFY2(qAbs(box.center().x() - 400) < 0.01 && qAbs(box.center().y() - 667) < 0.01,
                 qPrintable(QStringLiteral("%1, %2").arg(box.center().x()).arg(box.center().y())));
        QVERIFY(qAbs(box.width() - 200) < 0.01 || qAbs(box.height() - 50) < 0.01);
        QVERIFY(box.width() <= 200.01 && box.height() <= 50.01);
        QVERIFY(reader->property("notice").toString().contains(QStringLiteral("in its field")));
        QTest::keyClick(window, Qt::Key_Escape);  // the mark chosen
        QTest::keyClick(window, Qt::Key_Escape);  // asks, as it is unsaved
        QTRY_VERIFY(named("asking")->isVisible());
        QVERIFY(click(named("askDiscard")));
        QTRY_VERIFY(!window->isVisible());
    }

    void words_half_typed_are_asked_about_before_going()
    {
        const QString path = form(QStringLiteral("half typed.pdf"));
        const QByteArray before = bytesOf(path);
        QQuickItem *reader = read(path);
        QVERIFY(reader);
        QTRY_VERIFY(reader->property("editable").toBool());
        QTRY_COMPARE(itemsNamed(QStringLiteral("formField")).size(), 8);
        QVERIFY(click(fieldAt(Name)));
        QQuickItem *editor = nullptr;
        QTRY_VERIFY((editor = shown(QStringLiteral("fieldEditor"))) && editor->hasActiveFocus());
        type(QStringLiteral("xyz"));
        // The bar's way back: the words are kept, and going asks first.
        const QList<QQuickItem *> back = itemsUnder(window->contentItem(), [](QQuickItem *i) {
            return i->property("iconName").toString() == QLatin1String("back") && i->isVisible();
        });
        QCOMPARE(back.size(), 1);
        QVERIFY(click(back.first()));
        QTRY_VERIFY(named("asking")->isVisible());
        QVERIFY(fillsOf(reader).contains("name"));
        QVERIFY(click(named("askDiscard")));
        QTRY_VERIFY(!window->isVisible());
        QCOMPARE(bytesOf(path), before);
    }

    void its_controls_are_reached_by_tab()
    {
        QQuickItem *reader = read(dir.filePath("three.pdf"));
        QVERIFY(reader);
        QTRY_COMPARE(reader->property("pageCount").toInt(), 3);
        QTRY_VERIFY(reader->property("editable").toBool());
        // Back, pages, find, zoom out, zoom in, fit, mark up, turn, delete, full screen, open elsewhere, and the page's
        // number.
        QStringList reached;
        for (int i = 0; i < 12; ++i) {
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
