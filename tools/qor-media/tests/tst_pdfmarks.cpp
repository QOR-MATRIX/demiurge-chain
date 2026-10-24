// A PDF's marks (ADR-094 decision 3, ADR-114 decision 1, L9.5). PdfMarks writes words highlighted, underlined and
// struck through, notes, ink and a picture of a signature, drawn or typed, through PDFium, each the annotation it is
// with an appearance of its own; reads a PDF's own marks, and takes them away with the pop-ups that show their words.
// Every PDF is written by the test; what is saved is read back through PDFium and drawn as the reader draws it. Words
// are marked where Qt PDF's selection puts them, which is what the reader hands over, on a page as it is and turned.

#include <QBuffer>
#include <QFontDatabase>
#include <QPainter>
#include <QPdfDocument>
#include <QPdfSelection>
#include <QTemporaryDir>
#include <QTest>

#include <cmath>

#include "pdfdrawing.h"
#include "pdfmarks.h"
#include "pdfpages.h"
#include "testpdf.h"

using qormedia::PdfDrawing;
using qormedia::PdfMarks;
using qormedia::PdfPages;
using namespace testpdf;

namespace {

const QString kYellow = QStringLiteral("#ffd60a");
const QString kRed = QStringLiteral("#e5383b");
const QString kBlue = QStringLiteral("#3a86ff");
const QString kBlack = QStringLiteral("#1a1a1a");

QVariantMap keep(int page, int turn = 0)
{
    return {{QStringLiteral("page"), page}, {QStringLiteral("turn"), turn}};
}

QVariantList all(int pages)
{
    QVariantList layout;
    for (int i = 0; i < pages; ++i)
        layout << keep(i);
    return layout;
}

QVariantMap mark(const char *kind, int page, const QString &colour, QVariantMap rest = {})
{
    rest.insert(QStringLiteral("kind"), QString::fromLatin1(kind));
    rest.insert(QStringLiteral("page"), page);
    rest.insert(QStringLiteral("colour"), colour);
    return rest;
}

QVariantList points(std::initializer_list<QPointF> list)
{
    QVariantList out;
    for (const QPointF &p : list)
        out << p;
    return out;
}

// Lists of points, each kept a list: a list braced around a single list would copy it instead.
QVariantList lists(std::initializer_list<QVariantList> each)
{
    QVariantList out;
    for (const QVariantList &l : each)
        out << QVariant(l);
    return out;
}

// A page's words as the reader selects them: Qt PDF's selection of all of a page's text, its runs as rectangles.
QVariantList wordsOn(const QString &path, int page)
{
    QBuffer bytes;
    bytes.setData(bytesOf(path));
    bytes.open(QIODevice::ReadOnly);
    QPdfDocument document;
    document.load(&bytes);
    return PdfMarks::rectsOf(QVariant::fromValue(document.getAllText(page).bounds()));
}

// A page as the reader shows it, on its white paper, a pixel for each of its points across.
QImage shown(const QString &path, int page, int width, const QList<int> &hidden = {})
{
    PdfDrawing drawing;
    const QString key = drawing.open(path);
    const QImage image = PdfDrawing::draw(key, page, width, hidden);
    drawing.close(key);
    QImage paper(image.size(), QImage::Format_RGB32);
    paper.fill(Qt::white);
    QPainter(&paper).drawImage(0, 0, image);
    return paper;
}

template<typename Test>
int count(const QImage &image, const QRect &area, Test test)
{
    int n = 0;
    const QRect inside = area.intersected(image.rect());
    for (int y = inside.top(); y <= inside.bottom(); ++y)
        for (int x = inside.left(); x <= inside.right(); ++x)
            n += test(QColor(image.pixel(x, y)));
    return n;
}

bool closeTo(const QColor &p, const QColor &c, int by = 48)
{
    return std::abs(p.red() - c.red()) < by && std::abs(p.green() - c.green()) < by && std::abs(p.blue() - c.blue()) < by;
}
bool dark(const QColor &p)
{
    return p.red() < 90 && p.green() < 90 && p.blue() < 90;
}
// A word's ink, under a highlight's colour or not.
bool inked(const QColor &p)
{
    return p.lightness() < 100;
}
// A highlight's yellow, a part of it over white.
bool highlit(const QColor &p)
{
    return p.red() > 230 && p.green() > 205 && p.blue() > 110 && p.blue() < 200;
}

// Whether every character of a page lies in one of `quads`, all in the PDF's own space.
bool covers(const QList<QRectF> &quads, const QList<QRectF> &chars)
{
    for (const QRectF &c : chars)
        if (!std::any_of(quads.cbegin(), quads.cend(), [&](const QRectF &q) { return q.adjusted(-1, -1, 1, 1).contains(c.center()); }))
            return false;
    return !chars.isEmpty();
}

QRect pixels(const QVariant &r)
{
    return r.toRectF().toAlignedRect();
}

}  // namespace

class TestPdfMarks : public QObject
{
    Q_OBJECT

    QTemporaryDir dir;

    QString fresh(const QString &name, const QList<Page> &pages)
    {
        const QString path = dir.filePath(name);
        save(path, writePdf(pages));
        return path;
    }

private slots:
    void initTestCase()
    {
        QVERIFY(dir.isValid());
    }

    void words_are_highlighted_through_so_they_still_read()
    {
        const QString path = fresh("highlight.pdf", {{{"alpha one", "beta two"}, {}}});
        const QVariantList words = wordsOn(path, 0);
        QCOMPARE(words.size(), 2);
        PdfPages pages;
        QVERIFY2(pages.save(path, all(1), {mark("highlight", 0, kYellow, {{"rects", words}})}), qPrintable(pages.error()));
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 1);
        const Annot &a = annots.first();
        QCOMPARE(a.subtype, FPDF_ANNOT_HIGHLIGHT);
        QVERIFY(a.appearance);
        QVERIFY(a.printed);
        QCOMPARE(a.quads.size(), 2);
        QVERIFY(std::abs(a.opacity - PdfMarks::HighlightOpacity) < 0.01);
        // Over the words, where the PDF has them, and all within the annotation's own rectangle.
        QVERIFY(covers(a.quads, charBoxes(path, 0)));
        for (const QRectF &q : a.quads)
            QVERIFY(a.rect.adjusted(-0.01, -0.01, 0.01, 0.01).contains(q));
        // Drawn as the reader draws it: yellow over the words, and the words still dark through it.
        const QImage page = shown(path, 0, 612);
        for (const QVariant &r : words) {
            const QRect at = pixels(r);
            QVERIFY2(count(page, at, highlit) > at.width() * at.height() / 3, qPrintable(QString::number(count(page, at, highlit))));
            QVERIFY(count(page, at, inked) > 40);
        }
        QVERIFY(bytesOf(path).contains("/ExtGState"));
    }

    void words_are_underlined_and_struck_through()
    {
        const QString path = fresh("lines.pdf", {{{"alpha one", "beta two"}, {}}});
        const QVariantList words = wordsOn(path, 0);
        QCOMPARE(words.size(), 2);
        PdfPages pages;
        QVERIFY2(pages.save(path, all(1),
                            {mark("underline", 0, kRed, {{"rects", QVariantList{words[0]}}}),
                             mark("strikeout", 0, kBlue, {{"rects", QVariantList{words[1]}}})}),
                 qPrintable(pages.error()));
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 2);
        QCOMPARE(annots[0].subtype, FPDF_ANNOT_UNDERLINE);
        QCOMPARE(annots[1].subtype, FPDF_ANNOT_STRIKEOUT);
        const QImage page = shown(path, 0, 612);
        // Red along the first line's foot and nowhere above its middle; blue across the second's middle, not its foot.
        const QRect first = pixels(words[0]), second = pixels(words[1]);
        // Each by its hue, since a line a point and a half thick is shared, softened, between rows of pixels.
        const auto red = [](const QColor &p) { return p.red() - qMax(p.green(), p.blue()) > 60; };
        const auto blue = [](const QColor &p) { return p.blue() - qMax(p.red(), p.green()) > 40; };
        QVERIFY(count(page, QRect(first.left(), first.bottom() - 3, first.width(), 4), red) > first.width());
        QCOMPARE(count(page, QRect(first.left(), first.top(), first.width(), first.height() / 2), red), 0);
        const int middle = second.center().y();
        QVERIFY(count(page, QRect(second.left(), middle - 3, second.width(), 7), blue) > second.width() / 2);
        QCOMPARE(count(page, QRect(second.left(), second.bottom() - 2, second.width(), 3), blue), 0);
    }

    void a_note_keeps_its_words_where_it_was_put()
    {
        const QString path = fresh("note.pdf", {{{"words"}, {}}});
        const QString words = QStringLiteral("Call back über ✓\nSecond line 中文");
        PdfPages pages;
        QVERIFY2(pages.save(path, all(1), {mark("note", 0, kYellow, {{"x", 300}, {"y", 200}, {"text", words}})}), qPrintable(pages.error()));
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 1);
        QCOMPARE(annots[0].subtype, FPDF_ANNOT_TEXT);
        QCOMPARE(annots[0].contents, words);
        QVERIFY(annots[0].appearance);
        // 300 across and 200 down is 592 up from the foot of a 792-point page; the note is 20 points square.
        QVERIFY(annots[0].rect.contains(QRectF(300, 572, 20, 20)));
        QVERIFY(annots[0].rect.width() < 24);
        const QImage page = shown(path, 0, 612);
        QVERIFY(count(page, QRect(302, 202, 10, 6), [](const QColor &p) { return closeTo(p, QColor(kYellow)); }) > 40);
        // And read back as the reader reads a PDF's own marks.
        const QVariantList marks = PdfMarks::read(path);
        QCOMPARE(marks.size(), 1);
        const QVariantMap note = marks.first().toMap();
        QCOMPARE(note.value("kind").toString(), QStringLiteral("note"));
        QCOMPARE(note.value("text").toString(), words);
        const QRectF at = note.value("rect").toRectF();
        QVERIFY2(std::abs(at.x() - 300) < 1.5 && std::abs(at.y() - 200) < 1.5, qPrintable(QString("%1 %2").arg(at.x()).arg(at.y())));
    }

    void ink_keeps_its_strokes()
    {
        const QString path = fresh("ink.pdf", {{{"words"}, {}}});
        const QVariantList strokes{points({{100, 300}, {200, 320}, {300, 300}}), points({{100, 400}})};
        PdfPages pages;
        QVERIFY2(pages.save(path, all(1), {mark("ink", 0, kRed, {{"strokes", strokes}, {"width", 4}})}), qPrintable(pages.error()));
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 1);
        QCOMPARE(annots[0].subtype, FPDF_ANNOT_INK);
        QCOMPARE(annots[0].strokes, 2);
        QCOMPARE(annots[0].border, 4.f);
        QVERIFY(annots[0].appearance);
        const QImage page = shown(path, 0, 612);
        const auto red = [](const QColor &p) { return closeTo(p, QColor(kRed)); };
        // Along the line, at its bend, and the dot a single point makes; nothing far from either.
        QVERIFY(count(page, QRect(196, 316, 9, 9), red) > 10);
        QVERIFY(count(page, QRect(97, 397, 7, 7), red) > 3);
        QCOMPARE(count(page, QRect(150, 340, 100, 40), red), 0);
    }

    void a_signature_is_a_picture_drawn_or_typed()
    {
        QCOMPARE(PdfMarks::signatureFace(), QStringLiteral("Great Vibes"));
        QVERIFY(PdfMarks::typed(QString()).isEmpty());
        QVERIFY(PdfMarks::typed(QStringLiteral("   ")).isEmpty());
        const QVariantMap typed = PdfMarks::typed(QStringLiteral("Ada Lovelace"));
        const double aspect = typed.value("aspect").toDouble();
        QVERIFY2(aspect > 2, qPrintable(QString::number(aspect)));
        const QVariantList shapes = typed.value("shapes").toList();
        QVERIFY(shapes.size() > 10);
        for (const QVariant &shape : shapes)
            for (const QVariant &p : shape.toList()) {
                const QPointF at = p.toPointF();
                QVERIFY(at.x() >= -0.001 && at.x() <= 1.001 && at.y() >= -0.001 && at.y() <= 1.001);
            }

        const QString path = fresh("signed.pdf", {{{"words"}, {}}});
        const QRectF drawn(100, 500, 200, 80), placed(100, 620, 200, 200 / aspect);
        PdfPages pages;
        QVERIFY2(pages.save(path, all(1),
                            {mark("signature", 0, kBlack, {{"x", drawn.x()}, {"y", drawn.y()}, {"width", drawn.width()}, {"height", drawn.height()},
                                                          {"strokes", lists({points({{0, 0.5}, {0.5, 0}, {1, 1}})})}, {"line", 0.05}}),
                             mark("signature", 0, kBlue, {{"x", placed.x()}, {"y", placed.y()}, {"width", placed.width()}, {"height", placed.height()},
                                                         {"shapes", shapes}})}),
                 qPrintable(pages.error()));
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 2);
        for (const Annot &a : annots) {
            QCOMPARE(a.subtype, FPDF_ANNOT_STAMP);
            QCOMPARE(a.contents, QStringLiteral("Signature"));
            QVERIFY(a.appearance);
        }
        // Each in the box it was placed in: 500 to 580 down is 212 to 292 up.
        QVERIFY(annots[0].rect.adjusted(-0.01, -0.01, 0.01, 0.01).contains(QRectF(100, 212, 200, 80)));
        QVERIFY(QRectF(100, 212, 200, 80).adjusted(-3, -3, 3, 3).contains(annots[0].rect));
        const QImage page = shown(path, 0, 612);
        QVERIFY(count(page, drawn.toAlignedRect(), dark) > 200);
        QVERIFY(count(page, placed.toAlignedRect(), [](const QColor &p) { return closeTo(p, QColor(kBlue)); }) > 300);
        QCOMPARE(count(page, QRect(330, 480, 200, 200), [](const QColor &p) { return p.saturation() > 60 || dark(p); }), 0);
    }

    void a_mark_on_a_turned_page_lands_on_its_words()
    {
        const QString path = fresh("turned.pdf", {{{"alpha one", "beta two"}, {}}});
        PdfPages pages;
        QVERIFY(pages.save(path, {keep(0, 1)}));
        QCOMPARE(readBack(path).turns, QList<int>{1});
        // Laid out turned, the page is 792 across; the words run down it.
        const QVariantList words = wordsOn(path, 0);
        QCOMPARE(words.size(), 2);
        QVERIFY(words[0].toRectF().height() > words[0].toRectF().width());
        QVERIFY(pages.save(path, all(1), {mark("highlight", 0, kYellow, {{"rects", words}})}));
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 1);
        QVERIFY(covers(annots[0].quads, charBoxes(path, 0)));
        const QImage page = shown(path, 0, 792);
        QCOMPARE(page.size(), QSize(792, 612));
        for (const QVariant &r : words) {
            const QRect at = pixels(r);
            QVERIFY(count(page, at, highlit) > at.width() * at.height() / 3);
        }
    }

    void marks_and_page_changes_are_one_save()
    {
        const QString path = fresh("together.pdf", {{{"alpha one"}, {}}, {{"beta two"}, {}}, {{"gamma"}, {}}});
        const QVariantList gamma = wordsOn(path, 2);
        PdfPages pages;
        // The last page highlighted and turned to the front, the second deleted, a note on the first.
        QVERIFY2(pages.save(path, {keep(2, 1), keep(0)},
                            {mark("highlight", 2, kYellow, {{"rects", gamma}}), mark("note", 0, kYellow, {{"x", 40}, {"y", 40}, {"text", "first"}}),
                             mark("note", 1, kYellow, {{"x", 40}, {"y", 40}, {"text", "gone with its page"}})}),
                 qPrintable(pages.error()));
        const Held held = readBack(path);
        QCOMPARE(held.text, (QStringList{"gamma", "alpha one"}));
        QCOMPARE(held.turns, (QList<int>{1, 0}));
        const QList<Annot> annots = annotsOf(path);
        QCOMPARE(annots.size(), 2);
        QCOMPARE(annots[0].page, 0);
        QCOMPARE(annots[0].subtype, FPDF_ANNOT_HIGHLIGHT);
        QVERIFY(covers(annots[0].quads, charBoxes(path, 0)));
        QCOMPARE(annots[1].page, 1);
        QCOMPARE(annots[1].contents, QStringLiteral("first"));
    }

    void the_pdfs_own_marks_are_read_and_taken_away()
    {
        // A link, a red box and a note with its pop-up, in that order among the page's annotations.
        const QString path = dir.filePath("own.pdf");
        Page page{{"words"}, {{QRectF(72, 650, 100, 30), 0, {}}}, {QRectF(100, 300, 200, 100)}, {{QRectF(400, 600, 20, 20), "Hello there"}}};
        save(path, writePdf({page, {{"two"}, {}}}));
        QCOMPARE(annotsOf(path).size(), 4);
        const QVariantList marks = PdfMarks::read(path);
        // The link and the pop-up are not marks.
        QCOMPARE(marks.size(), 2);
        const QVariantMap box = marks[0].toMap(), note = marks[1].toMap();
        QCOMPARE(box.value("kind").toString(), QStringLiteral("shape"));
        QCOMPARE(box.value("page").toInt(), 0);
        QCOMPARE(box.value("index").toInt(), 1);
        // 300 to 400 up is 392 to 492 down the page as the reader lays it out.
        QCOMPARE(box.value("rect").toRectF(), QRectF(100, 392, 200, 100));
        QCOMPARE(note.value("kind").toString(), QStringLiteral("note"));
        QCOMPARE(note.value("index").toInt(), 2);
        QCOMPARE(note.value("text").toString(), QStringLiteral("Hello there"));
        // A page at a time, as the reader asks: the second has none, and a page it lacks has none either.
        QCOMPARE(PdfMarks::read(path, QString(), 0), marks);
        QVERIFY(PdfMarks::read(path, QString(), 1).isEmpty());
        QVERIFY(PdfMarks::read(path, QString(), 5).isEmpty());
        QVERIFY(count(shown(path, 0, 612), QRect(100, 392, 200, 100), [](const QColor &p) { return closeTo(p, Qt::red); }) > 200);

        PdfPages pages;
        QVERIFY2(pages.save(path, all(2), {}, {QVariantMap{{"page", 0}, {"index", 1}}, QVariantMap{{"page", 0}, {"index", 2}}}),
                 qPrintable(pages.error()));
        // Only the link is left: the note took its pop-up with it.
        const QList<Annot> left = annotsOf(path);
        QCOMPARE(left.size(), 1);
        QCOMPARE(left.first().subtype, FPDF_ANNOT_LINK);
        QVERIFY(PdfMarks::read(path).isEmpty());
        QCOMPARE(count(shown(path, 0, 612), QRect(100, 392, 200, 100), [](const QColor &p) { return closeTo(p, Qt::red); }), 0);
    }

    void a_page_is_drawn_without_the_marks_being_taken_away()
    {
        const QString path = dir.filePath("two-boxes.pdf");
        save(path, writePdf({{{"words"}, {}, {QRectF(100, 300, 200, 100), QRectF(100, 100, 200, 100)}}}));
        const auto red = [](const QColor &p) { return closeTo(p, Qt::red); };
        // 300 to 400 up is 392 to 492 down; 100 to 200 up is 592 to 692.
        const QRect upper(98, 390, 10, 104), lower(98, 590, 10, 104);
        const QImage both = shown(path, 0, 612);
        QVERIFY(count(both, upper, red) > 200 && count(both, lower, red) > 200);
        const QImage without = shown(path, 0, 612, {0});
        QCOMPARE(without.size(), QSize(612, 792));
        QCOMPARE(count(without, upper, red), 0);
        QVERIFY(count(without, lower, red) > 200);
        QVERIFY(count(without, QRect(72, 70, 120, 30), dark) > 40);
        // A place the page does not have hides nothing.
        const QImage still = shown(path, 0, 612, {7});
        QVERIFY(count(still, upper, red) > 200 && count(still, lower, red) > 200);
        // And nothing is written.
        QCOMPARE(annotsOf(path).size(), 2);
        QCOMPARE(PdfDrawing::openCount(), 0);
    }

    void a_locked_pdfs_marks_are_read_with_its_password_only()
    {
        const QString path = dir.filePath("locked-note.pdf");
        save(path, writePdf({{{"secret"}, {}, {}, {{QRectF(400, 600, 20, 20), "sealed words"}}}}, {}, "open sesame"));
        QVERIFY(PdfMarks::read(path).isEmpty());
        QVERIFY(PdfMarks::read(path, QStringLiteral("nope")).isEmpty());
        const QVariantList marks = PdfMarks::read(path, QStringLiteral("open sesame"));
        QCOMPARE(marks.size(), 1);
        QCOMPARE(marks.first().toMap().value("text").toString(), QStringLiteral("sealed words"));
    }

    void what_cannot_be_followed_leaves_the_file_as_it_was_data()
    {
        QTest::addColumn<QVariantList>("added");
        QTest::addColumn<QVariantList>("removed");
        const QVariantList words{QRectF(72, 80, 100, 20)};
        const QVariantMap box{{"x", 100}, {"y", 100}, {"width", 100}, {"height", 40}};
        const auto sign = [&](QVariantMap rest) {
            rest.insert(box);
            return mark("signature", 0, kBlack, rest);
        };
        QVariantList many;
        for (int i = 0; i < 200001; ++i)
            many << QPointF(i % 500, 100);
        QTest::newRow("a kind it does not know") << QVariantList{mark("scribble", 0, kRed, {{"rects", words}})} << QVariantList{};
        QTest::newRow("a page it does not have") << QVariantList{mark("highlight", 2, kYellow, {{"rects", words}})} << QVariantList{};
        QTest::newRow("a page before the first") << QVariantList{mark("highlight", -1, kYellow, {{"rects", words}})} << QVariantList{};
        QVariantMap half = mark("highlight", 0, kYellow, {{"rects", words}});
        half.insert("page", 0.5);
        QTest::newRow("half a page") << QVariantList{half} << QVariantList{};
        QTest::newRow("no colour") << QVariantList{mark("highlight", 0, QStringLiteral("nope"), {{"rects", words}})} << QVariantList{};
        QTest::newRow("no words") << QVariantList{mark("highlight", 0, kYellow, {{"rects", QVariantList{}}})} << QVariantList{};
        QTest::newRow("words with no width") << QVariantList{mark("underline", 0, kRed, {{"rects", QVariantList{QRectF(72, 80, 0, 20)}}})} << QVariantList{};
        QTest::newRow("a place that is not a number") << QVariantList{mark("note", 0, kYellow, {{"x", std::nan("")}, {"y", 10}, {"text", "x"}})} << QVariantList{};
        QTest::newRow("a place without end") << QVariantList{mark("note", 0, kYellow, {{"x", INFINITY}, {"y", 10}, {"text", "x"}})} << QVariantList{};
        QTest::newRow("a note too long") << QVariantList{mark("note", 0, kYellow, {{"x", 1}, {"y", 1}, {"text", QString(100001, 'a')}})} << QVariantList{};
        QTest::newRow("ink with no width") << QVariantList{mark("ink", 0, kRed, {{"width", 0}, {"strokes", lists({points({{1, 1}})})}})} << QVariantList{};
        QTest::newRow("ink too wide") << QVariantList{mark("ink", 0, kRed, {{"width", 101}, {"strokes", lists({points({{1, 1}})})}})} << QVariantList{};
        QTest::newRow("an empty stroke") << QVariantList{mark("ink", 0, kRed, {{"width", 2}, {"strokes", lists({QVariantList{}})}})} << QVariantList{};
        QTest::newRow("too many points") << QVariantList{mark("ink", 0, kRed, {{"width", 2}, {"strokes", lists({many})}})} << QVariantList{};
        QTest::newRow("a signature both drawn and typed")
            << QVariantList{sign({{"strokes", lists({points({{0, 0}, {1, 1}})})}, {"line", 0.05}, {"shapes", lists({points({{0, 0}, {1, 0}, {1, 1}})})}})}
            << QVariantList{};
        QTest::newRow("a signature's line past its box") << QVariantList{sign({{"strokes", lists({points({{0, 0}, {1, 1}})})}, {"line", 2}})} << QVariantList{};
        QTest::newRow("a signature with no box")
            << QVariantList{mark("signature", 0, kBlack, {{"x", 1}, {"y", 1}, {"width", 100}, {"height", 0}, {"shapes", lists({points({{0, 0}, {1, 0}, {1, 1}})})}})}
            << QVariantList{};
        QTest::newRow("a mark it does not have") << QVariantList{} << QVariantList{QVariantMap{{"page", 0}, {"index", 9}}};
        QTest::newRow("a link is not a mark") << QVariantList{} << QVariantList{QVariantMap{{"page", 0}, {"index", 0}}};
        QTest::newRow("a mark on a page it does not have") << QVariantList{} << QVariantList{QVariantMap{{"page", 3}, {"index", 1}}};
        QTest::newRow("a place before the first") << QVariantList{} << QVariantList{QVariantMap{{"page", 0}, {"index", -1}}};
        // One good mark beside a bad one: neither is written.
        QTest::newRow("a good mark and a bad one")
            << QVariantList{mark("highlight", 0, kYellow, {{"rects", words}}), mark("highlight", 0, kYellow, {{"rects", QVariantList{}}})} << QVariantList{};
    }
    void what_cannot_be_followed_leaves_the_file_as_it_was()
    {
        QFETCH(QVariantList, added);
        QFETCH(QVariantList, removed);
        const QString path = dir.filePath("refused.pdf");
        save(path, writePdf({{{"words"}, {{QRectF(72, 650, 100, 30), 0, {}}}, {QRectF(100, 300, 200, 100)}}}));
        const QByteArray before = bytesOf(path);
        PdfPages pages;
        QVERIFY(!pages.save(path, all(1), added, removed));
        QVERIFY(!pages.error().isEmpty());
        QCOMPARE(bytesOf(path), before);
    }

    void rects_of_reads_a_selections_geometry()
    {
        const QList<QPolygonF> polygons{QPolygonF(QRectF(10, 20, 30, 40)), QPolygonF({{0, 0}, {5, 0}}), QPolygonF(QRectF(1, 2, 3, 4))};
        const QVariantList rects = PdfMarks::rectsOf(QVariant::fromValue(polygons));
        // The line with no height is no run of words.
        QCOMPARE(rects, (QVariantList{QRectF(10, 20, 30, 40), QRectF(1, 2, 3, 4)}));
        const QVariantList asLists = lists({points({{10, 20}, {40, 20}, {40, 60}, {10, 60}})});
        QCOMPARE(PdfMarks::rectsOf(asLists), (QVariantList{QRectF(10, 20, 30, 40)}));
        QVERIFY(PdfMarks::rectsOf(QStringLiteral("not a geometry")).isEmpty());
        QVERIFY(PdfMarks::rectsOf(QVariant()).isEmpty());
    }

    void the_colours_are_what_a_pdf_holds()
    {
        const QVariantList colours = PdfMarks::colours();
        QCOMPARE(colours.size(), 6);
        for (const QVariant &c : colours) {
            QVERIFY(!c.toMap().value("name").toString().isEmpty());
            QVERIFY(QColor::fromString(c.toMap().value("colour").toString()).isValid());
        }
        QCOMPARE(colours.first().toMap().value("colour").toString(), kYellow);
    }
};

QTEST_MAIN(TestPdfMarks)
#include "tst_pdfmarks.moc"
