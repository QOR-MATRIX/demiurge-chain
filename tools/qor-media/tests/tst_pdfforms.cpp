// A PDF's form (ADR-094 decision 3, L9.5). Its fields drawn as the reader draws its pages, which Qt PDF never does: their
// words, a box ticked, a group's button chosen, a list's option, and a field with no appearance of its own. Its widgets
// read as the reader reads them, on a page as it is and turned. Filled through PDFium as a person fills them, saved
// with the pages and the marks in one write, then read back and drawn; drawn filled before it is saved, writing nothing;
// and what cannot be filled leaving the file's bytes as they were. Every PDF is written by the test, byte by byte, so
// what each holds is known exactly.

#include <QBuffer>
#include <QPainter>
#include <QPdfDocument>
#include <QPdfDocumentRenderOptions>
#include <QTemporaryDir>
#include <QTest>

#include "pdfdrawing.h"
#include "pdfforms.h"
#include "pdfpages.h"
#include "testpdf.h"

using qormedia::PdfDrawing;
using qormedia::PdfForms;
using qormedia::PdfPages;
using namespace testpdf;

namespace {

// Where each widget of the form below is among its page's annotations: a red box first, a mark and not a field, then
// the fields in turn, a radio group's three buttons one after another. The hidden one is never read.
enum At { Box, Name, Agree, Small, Medium, Large, Colour, Days, Notes, Code, Pin, Fixed, Go, Sign, Bare, Hidden };

Page formPage()
{
    Page page{{"Form"}, {}, {QRectF(500, 700, 40, 40)}, {}, {}};
    page.fields = {
        Field{Field::Text, "name", QRectF(100, 600, 200, 24), "Ada Lovelace"},
        Field{Field::Check, "agree", QRectF(100, 560, 20, 20), "Off", {"Yes"}},
        Field{Field::Radio, "size", QRectF(100, 520, 20, 20), "S", {"S", "M", "L"}, {QRectF(140, 520, 20, 20), QRectF(180, 520, 20, 20)}},
        Field{Field::Combo, "colour", QRectF(100, 470, 120, 24), "Red", {"Red", "Green", "Blue"}},
        Field{Field::List, "days", QRectF(300, 400, 120, 60), "Mon", {"Mon", "Tue", "Wed"}, {}, 1 << 21},
        Field{Field::Text, "notes", QRectF(100, 330, 300, 60), "", {}, {}, 1 << 12},
        Field{Field::Text, "code", QRectF(100, 290, 100, 24), "AB", {}, {}, 0, 4},
        Field{Field::Text, "pin", QRectF(250, 290, 100, 24), "", {}, {}, 1 << 13},
        Field{Field::Text, "fixed", QRectF(100, 250, 200, 24), "Locked", {}, {}, 1},
        Field{Field::Button, "go", QRectF(400, 600, 60, 24)},
        Field{Field::Signature, "sign", QRectF(300, 100, 200, 50)},
        Field{Field::Text, "bare", QRectF(100, 210, 200, 24), "No look", {}, {}, 0, 0, false},
        Field{Field::Text, "kept", QRectF(350, 210, 100, 24), "Kept", {}, {}, 0, 0, true, true},
    };
    return page;
}

// A widget's place on the page as drawn 612 pixels wide, a pixel to a point: the PDF counts up from its foot.
QRect shownAt(const QRectF &r)
{
    return QRectF(r.x(), 792 - r.y() - r.height(), r.width(), r.height()).toRect();
}

QVariantList all(int pages, int turn = 0)
{
    QVariantList layout;
    for (int i = 0; i < pages; ++i)
        layout << QVariantMap{{QStringLiteral("page"), i}, {QStringLiteral("turn"), turn}};
    return layout;
}

QVariantMap at(int index, const char *way, const QVariant &value, int page = 0)
{
    return {{QStringLiteral("page"), page}, {QStringLiteral("index"), index}, {QString::fromLatin1(way), value}};
}
QVariantMap words(int index, const QString &text) { return at(index, "text", text); }
QVariantMap tick(int index, bool on) { return at(index, "checked", on); }
QVariantMap choose(int index, const QVariantList &places) { return at(index, "chosen", places); }

// The widgets the reader reads on page 0, by their place among its annotations.
QMap<int, QVariantMap> widgetsOf(const QString &path, const QString &password = {})
{
    QMap<int, QVariantMap> widgets;
    for (const QVariant &w : PdfForms::read(path, password, 0))
        widgets.insert(w.toMap().value("index").toInt(), w.toMap());
    return widgets;
}

QVariantList places(std::initializer_list<int> list)
{
    QVariantList out;
    for (int i : list)
        out << i;
    return out;
}

int darkIn(const QImage &image, const QRect &area)
{
    int n = 0;
    for (int y = area.top(); y <= area.bottom(); ++y)
        for (int x = area.left(); x <= area.right(); ++x)
            n += QColor(image.pixel(x, y)).lightness() < 100;
    return n;
}

// Page 0 as the reader draws it, a pixel to a point, filled as `filled` says when it says anything.
QImage drawn(const QString &path, const QVariantList &filled = {}, const QString &password = {})
{
    PdfDrawing drawing;
    const QString key = drawing.open(path, password);
    drawing.fill(key, filled);
    const QImage image = PdfDrawing::draw(key, 0, 612, {}, !filled.isEmpty());
    drawing.close(key);
    return image;
}

}  // namespace

class TestPdfForms : public QObject
{
    Q_OBJECT
    QTemporaryDir dir;

    QString fresh(const QString &name, const Page &page = formPage())
    {
        const QString path = dir.filePath(name);
        save(path, writePdf({page}));
        return path;
    }

private slots:
    void a_forms_fields_are_drawn_which_qt_pdf_leaves_out()
    {
        const QString path = fresh("drawn.pdf");
        const QImage image = drawn(path);
        QCOMPARE(image.size(), QSize(612, 792));
        // Words, the group's first button, the drop-down list's option, and the words of a field with no appearance of
        // its own, which PDFium draws for it.
        QVERIFY2(darkIn(image, shownAt({100, 600, 200, 24})) > 60, "the name's words");
        QVERIFY2(darkIn(image, shownAt({100, 520, 20, 20})) > 100, "the first size, chosen");
        QCOMPARE(darkIn(image, shownAt({180, 520, 20, 20})), 0);
        QVERIFY2(darkIn(image, shownAt({100, 470, 120, 24})) > 20, "the colour");
        QVERIFY2(darkIn(image, shownAt({100, 210, 200, 24})) > 20, "a field with no appearance");
        // Not ticked, and hidden: nothing.
        QCOMPARE(darkIn(image, shownAt({100, 560, 20, 20})), 0);
        QCOMPARE(darkIn(image, shownAt({350, 210, 100, 24})), 0);

        // Qt PDF, asked for every annotation, draws not one field: why the reader's pages are drawn here.
        QBuffer bytes;
        bytes.setData(bytesOf(path));
        bytes.open(QIODevice::ReadOnly);
        QPdfDocument document;
        document.load(&bytes);
        QPdfDocumentRenderOptions options;
        options.setRenderFlags(QPdfDocumentRenderOptions::RenderFlag::Annotations);
        QImage qt = document.render(0, QSize(612, 792), options).convertToFormat(QImage::Format_ARGB32);
        QImage paper(qt.size(), QImage::Format_ARGB32);
        paper.fill(Qt::white);
        QPainter(&paper).drawImage(0, 0, qt);
        QCOMPARE(darkIn(paper, shownAt({100, 600, 200, 24})), 0);
        QCOMPARE(darkIn(paper, shownAt({100, 520, 20, 20})), 0);
    }

    void its_widgets_are_read_as_the_reader_reads_them()
    {
        const QString path = fresh("read.pdf");
        const QMap<int, QVariantMap> w = widgetsOf(path);
        // Every widget but the hidden one; the box is a mark, not a field.
        QCOMPARE(w.keys(), (QList<int>{Name, Agree, Small, Medium, Large, Colour, Days, Notes, Code, Pin, Fixed, Go, Sign, Bare}));

        const QVariantMap name = w[Name];
        QCOMPARE(name.value("page").toInt(), 0);
        QCOMPARE(name.value("name").toString(), QStringLiteral("name"));
        QCOMPARE(name.value("kind").toString(), QStringLiteral("text"));
        // In the page's points from its top left, as Qt PDF lays it out.
        QCOMPARE(name.value("rect").toRectF(), QRectF(100, 168, 200, 24));
        QCOMPARE(name.value("value").toString(), QStringLiteral("Ada Lovelace"));
        QVERIFY(name.value("fillable").toBool());
        QCOMPARE(name.value("why").toString(), QString());
        QCOMPARE(name.value("fontSize").toDouble(), 12.0);
        QCOMPARE(name.value("colour").toString(), QStringLiteral("#000000"));
        QCOMPARE(name.value("align").toInt(), 0);
        QVERIFY(!name.value("multiline").toBool());
        QCOMPARE(name.value("maxLength").toInt(), 0);

        QCOMPARE(w[Agree].value("kind").toString(), QStringLiteral("check"));
        QCOMPARE(w[Agree].value("value").toString(), QStringLiteral("Off"));
        QVERIFY(!w[Agree].value("checked").toBool());
        QCOMPARE(w[Agree].value("state").toString(), QStringLiteral("Yes"));

        // A group's buttons share its name and value; the one on is checked, each with the state it sets.
        for (const int b : {Small, Medium, Large}) {
            QCOMPARE(w[b].value("kind").toString(), QStringLiteral("radio"));
            QCOMPARE(w[b].value("name").toString(), QStringLiteral("size"));
            QCOMPARE(w[b].value("value").toString(), QStringLiteral("S"));
        }
        QVERIFY(w[Small].value("checked").toBool());
        QVERIFY(!w[Large].value("checked").toBool());
        QCOMPARE(w[Large].value("state").toString(), QStringLiteral("L"));
        QCOMPARE(w[Large].value("rect").toRectF(), QRectF(180, 252, 20, 20));

        QCOMPARE(w[Colour].value("kind").toString(), QStringLiteral("combo"));
        QCOMPARE(w[Colour].value("options").toStringList(), (QStringList{"Red", "Green", "Blue"}));
        QCOMPARE(w[Colour].value("chosen").toList(), places({0}));
        QVERIFY(!w[Colour].value("editable").toBool());
        QCOMPARE(w[Days].value("kind").toString(), QStringLiteral("list"));
        QVERIFY(w[Days].value("multiple").toBool());
        QCOMPARE(w[Days].value("chosen").toList(), places({0}));

        QVERIFY(w[Notes].value("multiline").toBool());
        QCOMPARE(w[Code].value("maxLength").toInt(), 4);
        // A password field is shown, never filled: PDFium would write the password into the file.
        QVERIFY(w[Pin].value("secret").toBool());
        QVERIFY(!w[Pin].value("fillable").toBool());
        QVERIFY(w[Pin].value("why").toString().contains(QStringLiteral("password")));

        // What is not filled here says why.
        QVERIFY(!w[Fixed].value("fillable").toBool());
        QCOMPARE(w[Fixed].value("why").toString(), QStringLiteral("This field is read only."));
        QCOMPARE(w[Go].value("kind").toString(), QStringLiteral("button"));
        QVERIFY(!w[Go].value("fillable").toBool());
        QCOMPARE(w[Sign].value("kind").toString(), QStringLiteral("signature"));
        QVERIFY(!w[Sign].value("fillable").toBool());
        QVERIFY(!w[Sign].value("why").toString().isEmpty());
        QCOMPARE(w[Bare].value("value").toString(), QStringLiteral("No look"));

        // Every page at once, a page past the end, and a file that is no PDF.
        QCOMPARE(PdfForms::read(path).size(), 14);
        QCOMPARE(PdfForms::read(path, {}, 1).size(), 0);
        save(dir.filePath("not.pdf"), "%PDF-1.7\n");
        QCOMPARE(PdfForms::read(dir.filePath("not.pdf")).size(), 0);
    }

    void a_form_is_filled_and_saved_as_a_person_fills_it()
    {
        const QString path = fresh("filled.pdf");
        const QImage before = drawn(path);
        PdfPages pages;
        const QVariantList filled{words(Name, QStringLiteral("Grace Hopper")),
                                  tick(Agree, true),
                                  tick(Large, true),
                                  choose(Colour, places({2})),
                                  choose(Days, places({0, 2})),
                                  words(Notes, QStringLiteral("one\ntwo")),
                                  words(Code, QStringLiteral("ABCD")),
                                  words(Bare, QStringLiteral("Zoë Ψ"))};
        QVERIFY2(pages.save(path, all(1), {}, {}, filled), qPrintable(pages.error()));

        const QMap<int, QVariantMap> w = widgetsOf(path);
        QCOMPARE(w[Name].value("value").toString(), QStringLiteral("Grace Hopper"));
        QVERIFY(w[Agree].value("checked").toBool());
        QCOMPARE(w[Agree].value("value").toString(), QStringLiteral("Yes"));
        QVERIFY(w[Large].value("checked").toBool());
        QVERIFY(!w[Small].value("checked").toBool());
        QCOMPARE(w[Small].value("value").toString(), QStringLiteral("L"));
        QCOMPARE(w[Colour].value("chosen").toList(), places({2}));
        QCOMPARE(w[Colour].value("value").toString(), QStringLiteral("Blue"));
        QCOMPARE(w[Days].value("chosen").toList(), places({0, 2}));
        QCOMPARE(w[Notes].value("value").toString(), QStringLiteral("one\ntwo"));
        QCOMPARE(w[Code].value("value").toString(), QStringLiteral("ABCD"));
        QCOMPARE(w[Bare].value("value").toString(), QStringLiteral("Zoë Ψ"));
        // What was not filled is as it was, the box beside them too.
        QCOMPARE(w[Fixed].value("value").toString(), QStringLiteral("Locked"));
        QCOMPARE(w[Pin].value("value").toString(), QString());
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), Hidden + 1);
        QCOMPARE(annots[Box].subtype, FPDF_ANNOT_SQUARE);
        // Each drawn anew, the field with no appearance given one, as every reader shows it.
        for (const int filledAt : {Name, Agree, Large, Colour, Days, Notes, Code, Bare})
            QVERIFY2(annots[filledAt].appearance, qPrintable(QString::number(filledAt)));

        // Drawn as filled: the tick, the third size and not the first, other words where the name was.
        const QImage after = drawn(path);
        QVERIFY(darkIn(after, shownAt({100, 560, 20, 20})) > 50);
        QVERIFY(darkIn(after, shownAt({180, 520, 20, 20})) > 50);
        QCOMPARE(darkIn(after, shownAt({100, 520, 20, 20})), 0);
        const QRect nameAt = shownAt({100, 600, 200, 24});
        QVERIFY(after.copy(nameAt) != before.copy(nameAt));
    }

    void filled_and_drawn_before_saving_writes_nothing()
    {
        const QString path = fresh("unsaved.pdf");
        const QByteArray bytes = bytesOf(path);
        PdfDrawing drawing;
        const QString key = drawing.open(path);
        const QImage before = PdfDrawing::draw(key, 0, 612);
        drawing.fill(key, {tick(Agree, true), words(Name, QStringLiteral("Grace Hopper"))});
        // Drawn filled only when asked: what the reader asks for once something is filled.
        QCOMPARE(PdfDrawing::draw(key, 0, 612), before);
        const QImage filled = PdfDrawing::draw(key, 0, 612, {}, true);
        QVERIFY(darkIn(filled, shownAt({100, 560, 20, 20})) > 50);
        const QRect nameAt = shownAt({100, 600, 200, 24});
        QVERIFY(filled.copy(nameAt) != before.copy(nameAt));
        // With a mark being taken away too.
        const QImage both = PdfDrawing::draw(key, 0, 612, {Box}, true);
        QVERIFY(darkIn(both, shownAt({100, 560, 20, 20})) > 50);
        QVERIFY(both.copy(shownAt({500, 700, 40, 40})) != before.copy(shownAt({500, 700, 40, 40})));
        // Nothing filled, nothing changed; and the file as it was throughout.
        drawing.fill(key, {});
        QCOMPARE(PdfDrawing::draw(key, 0, 612, {}, true), before);
        drawing.close(key);
        QCOMPARE(bytesOf(path), bytes);
    }

    void fields_marks_and_pages_are_one_save()
    {
        // A note before the fields: taking it away, with its pop-up, moves every field two places down the page's list,
        // so the fields are filled first, where the reader read them.
        Page page = formPage();
        page.notes = {{QRectF(400, 700, 20, 20), "a note"}};
        const QString path = fresh("together.pdf", page);
        const int moved = 2;
        PdfPages pages;
        const QVariantMap ink{{"kind", "ink"}, {"page", 0}, {"colour", "#1a1a1a"}, {"width", 2},
                              {"strokes", QVariantList{QVariant(QVariantList{QPointF(10, 10), QPointF(60, 60)})}}};
        QVERIFY2(pages.save(path, all(1, 1), {ink}, {QVariantMap{{"page", 0}, {"index", 1}}}, {words(Name + moved, QStringLiteral("Grace Hopper"))}),
                 qPrintable(pages.error()));
        QCOMPARE(readBack(path).turns, QList<int>{1});
        QList<int> subtypes;
        for (const Annot &a : annotsOf(path))
            subtypes << a.subtype;
        QVERIFY(!subtypes.contains(FPDF_ANNOT_TEXT) && !subtypes.contains(FPDF_ANNOT_POPUP));
        QVERIFY(subtypes.contains(FPDF_ANNOT_INK));
        const QMap<int, QVariantMap> w = widgetsOf(path);
        QCOMPARE(w[Name].value("value").toString(), QStringLiteral("Grace Hopper"));
        QCOMPARE(w[Agree].value("value").toString(), QStringLiteral("Off"));
    }

    void fields_on_a_turned_page_are_read_where_they_show()
    {
        const QString path = fresh("turned.pdf");
        PdfPages pages;
        QVERIFY(pages.save(path, all(1, 1)));
        // A quarter turn clockwise: the page laid out 792 points across, the name's widget standing on its side.
        const QRectF name = widgetsOf(path)[Name].value("rect").toRectF();
        QVERIFY2(qAbs(name.x() - 600) < 0.01 && qAbs(name.y() - 100) < 0.01 && qAbs(name.width() - 24) < 0.01 && qAbs(name.height() - 200) < 0.01,
                 qPrintable(QStringLiteral("%1 %2 %3 %4").arg(name.x()).arg(name.y()).arg(name.width()).arg(name.height())));
        QVERIFY2(pages.save(path, all(1), {}, {}, {words(Name, QStringLiteral("Turned"))}), qPrintable(pages.error()));
        QCOMPARE(widgetsOf(path)[Name].value("value").toString(), QStringLiteral("Turned"));
    }

    void a_locked_pdfs_fields_are_read_and_drawn_with_its_password_only()
    {
        const QString path = dir.filePath("locked form.pdf");
        save(path, writePdf({formPage()}, {}, "open sesame"));
        QCOMPARE(PdfForms::read(path).size(), 0);
        QCOMPARE(widgetsOf(path, QStringLiteral("open sesame"))[Name].value("value").toString(), QStringLiteral("Ada Lovelace"));
        QVERIFY(darkIn(drawn(path, {}, QStringLiteral("open sesame")), shownAt({100, 600, 200, 24})) > 60);
        // Read, never filled: saving it would strip its lock.
        const QByteArray bytes = bytesOf(path);
        PdfPages pages;
        QVERIFY(!pages.save(path, all(1), {}, {}, {words(Name, QStringLiteral("x"))}));
        QCOMPARE(bytesOf(path), bytes);
    }

    void an_xfa_forms_fields_are_shown_not_filled()
    {
        // The form's own default look traded for an XFA entry of the same length, so every offset holds.
        QByteArray bytes = writePdf({formPage()});
        const QByteArray look = "/DA <" + QByteArray("/Helv 0 Tf 0 g").toHex() + ">";
        QVERIFY(bytes.count(look) == 1);
        bytes.replace(look, QByteArray("/XFA []").leftJustified(look.size(), ' '));
        const QString path = dir.filePath("xfa.pdf");
        save(path, bytes);
        const QMap<int, QVariantMap> w = widgetsOf(path);
        QCOMPARE(w[Name].value("value").toString(), QStringLiteral("Ada Lovelace"));
        QVERIFY(!w[Name].value("fillable").toBool());
        QVERIFY(w[Name].value("xfa").toBool());
        QVERIFY(w[Name].value("why").toString().contains(QStringLiteral("XFA")));
        PdfPages pages;
        QVERIFY(!pages.save(path, all(1), {}, {}, {words(Name, QStringLiteral("x"))}));
        QVERIFY(pages.error().contains(QStringLiteral("XFA")));
        QCOMPARE(bytesOf(path), bytes);
    }

    void what_cannot_be_filled_leaves_the_file_as_it_was_data()
    {
        QTest::addColumn<QVariantList>("filled");
        QVariantMap noPage = words(Name, QStringLiteral("x"));
        noPage.remove(QStringLiteral("page"));
        QVariantMap noWay = words(Name, QStringLiteral("x"));
        noWay.remove(QStringLiteral("text"));
        QVariantMap twoWays = words(Name, QStringLiteral("x"));
        twoWays.insert(QStringLiteral("checked"), true);
        QTest::newRow("no page") << QVariantList{noPage};
        QTest::newRow("a page past the end") << QVariantList{at(Name, "text", QStringLiteral("x"), 1)};
        QTest::newRow("a place past the end") << QVariantList{words(99, QStringLiteral("x"))};
        QTest::newRow("a mark, not a field") << QVariantList{words(Box, QStringLiteral("x"))};
        QTest::newRow("no way of filling it") << QVariantList{noWay};
        QTest::newRow("two ways of filling it") << QVariantList{twoWays};
        QTest::newRow("words that are not words") << QVariantList{at(Name, "text", 42)};
        QTest::newRow("words for a check box") << QVariantList{words(Agree, QStringLiteral("Yes"))};
        QTest::newRow("a tick for words") << QVariantList{tick(Name, true)};
        QTest::newRow("a tick neither true nor false") << QVariantList{at(Agree, "checked", QStringLiteral("yes"))};
        QTest::newRow("a radio button unchosen") << QVariantList{tick(Small, false)};
        QTest::newRow("a read-only field") << QVariantList{words(Fixed, QStringLiteral("x"))};
        QTest::newRow("a password field") << QVariantList{words(Pin, QStringLiteral("4321"))};
        QTest::newRow("a button") << QVariantList{tick(Go, true)};
        QTest::newRow("a signature field") << QVariantList{words(Sign, QStringLiteral("x"))};
        QTest::newRow("a hidden field") << QVariantList{words(Hidden, QStringLiteral("x"))};
        QTest::newRow("more words than the field takes") << QVariantList{words(Code, QStringLiteral("ABCDE"))};
        QTest::newRow("a line break on one line") << QVariantList{words(Name, QStringLiteral("a\nb"))};
        QTest::newRow("too many words") << QVariantList{words(Notes, QString(100001, QLatin1Char('a')))};
        QTest::newRow("two options of a drop-down list") << QVariantList{choose(Colour, places({0, 1}))};
        QTest::newRow("no option of a drop-down list") << QVariantList{choose(Colour, {})};
        QTest::newRow("an option past the end") << QVariantList{choose(Colour, places({3}))};
        QTest::newRow("an option before the first") << QVariantList{choose(Colour, places({-1}))};
        QTest::newRow("an option twice") << QVariantList{choose(Days, places({0, 0}))};
        QTest::newRow("options that are not a list") << QVariantList{at(Colour, "chosen", 1)};
        QTest::newRow("words for a list that takes none") << QVariantList{words(Days, QStringLiteral("Mon"))};
        QTest::newRow("options for words") << QVariantList{choose(Name, places({0}))};
        QTest::newRow("a good entry beside a bad one") << QVariantList{words(Name, QStringLiteral("Grace Hopper")), words(Fixed, QStringLiteral("x"))};
        QTest::newRow("not an entry at all") << QVariantList{QStringLiteral("name")};
    }

    void what_cannot_be_filled_leaves_the_file_as_it_was()
    {
        QFETCH(QVariantList, filled);
        const QString path = fresh("refused.pdf");
        const QByteArray bytes = bytesOf(path);
        PdfPages pages;
        QVERIFY(!pages.save(path, all(1), {}, {}, filled));
        QVERIFY(!pages.error().isEmpty());
        QCOMPARE(bytesOf(path), bytes);
    }
};

QTEST_MAIN(TestPdfForms)
#include "tst_pdfforms.moc"
