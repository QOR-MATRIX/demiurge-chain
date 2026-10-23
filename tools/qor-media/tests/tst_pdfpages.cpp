// A PDF's pages changed (ADR-094 decision 3, L9.5): PdfPages turns, deletes and moves them through PDFium and writes the
// file back whole or not at all. A list it cannot follow, a locked or protected PDF, and a file that cannot be replaced
// each leave the file exactly as it was. Every PDF is written by the test; what is saved is read back through PDFium.

#include <QDir>
#include <QFile>
#include <QTemporaryDir>
#include <QTest>
#include <QUrl>

#include "pdfpages.h"
#include "testpdf.h"

using qormedia::PdfPages;
using namespace testpdf;

namespace {

QVariantMap keep(int page, int turn = 0)
{
    return {{QStringLiteral("page"), page}, {QStringLiteral("turn"), turn}};
}

}  // namespace

class TestPdfPages : public QObject
{
    Q_OBJECT

    QTemporaryDir dir;

    // A fresh three-page PDF: "alpha one", "beta two", "gamma".
    QString three(const QString &name)
    {
        const QString path = dir.filePath(name);
        save(path, writePdf({{{"alpha one"}, {}}, {{"beta two"}, {}}, {{"gamma"}, {}}}));
        return path;
    }

private slots:
    void initTestCase()
    {
        QVERIFY(dir.isValid());
        // PdfPages sets PDFium up on first use; the reading back here needs it too.
        QVERIFY(!PdfPages::changeable(dir.filePath("nothing.pdf")));
    }

    void turns_deletions_and_moves_are_written()
    {
        const QString path = three("a.pdf");
        PdfPages pages;
        QVERIFY2(pages.save(path, {keep(2, 1), keep(0)}), qPrintable(pages.error()));
        const Held held = readBack(path);
        QVERIFY(held.read);
        QCOMPARE(held.text, (QStringList{"gamma", "alpha one"}));
        QCOMPARE(held.turns, (QList<int>{1, 0}));
        QVERIFY(pages.error().isEmpty());
    }

    void every_page_in_its_place_is_the_same_pages()
    {
        const QString path = three("b.pdf");
        PdfPages pages;
        QVERIFY(pages.save(path, {keep(0), keep(1), keep(2)}));
        const Held held = readBack(path);
        QCOMPARE(held.text, (QStringList{"alpha one", "beta two", "gamma"}));
        QCOMPARE(held.turns, (QList<int>{0, 0, 0}));
    }

    void the_whole_order_reversed()
    {
        const QString path = three("c.pdf");
        PdfPages pages;
        QVERIFY(pages.save(path, {keep(2), keep(1), keep(0)}));
        QCOMPARE(readBack(path).text, (QStringList{"gamma", "beta two", "alpha one"}));
    }

    void a_turn_adds_to_what_the_page_already_has()
    {
        const QString path = three("d.pdf");
        PdfPages pages;
        QVERIFY(pages.save(path, {keep(0, 3), keep(1), keep(2)}));
        QVERIFY(pages.save(path, {keep(0, 2), keep(1), keep(2)}));
        QCOMPARE(readBack(path).turns, (QList<int>{1, 0, 0}));
    }

    void a_list_it_cannot_follow_leaves_the_file_as_it_was_data()
    {
        QTest::addColumn<QVariantList>("layout");
        QTest::newRow("nothing kept") << QVariantList{};
        QTest::newRow("a page twice") << QVariantList{keep(0), keep(0)};
        QTest::newRow("a page it does not have") << QVariantList{keep(0), keep(3)};
        QTest::newRow("a page before the first") << QVariantList{keep(-1)};
        QTest::newRow("a turn too far") << QVariantList{keep(0, 4)};
        QTest::newRow("a turn backwards") << QVariantList{keep(0, -1)};
        QTest::newRow("not an entry") << QVariantList{QVariant(1)};
    }
    void a_list_it_cannot_follow_leaves_the_file_as_it_was()
    {
        QFETCH(QVariantList, layout);
        const QString path = three("e.pdf");
        const QByteArray before = bytesOf(path);
        PdfPages pages;
        QVERIFY(!pages.save(path, layout));
        QVERIFY(!pages.error().isEmpty());
        QCOMPARE(bytesOf(path), before);
    }

    void a_locked_or_protected_pdf_is_left_as_it_was()
    {
        const QString locked = dir.filePath("locked.pdf");
        save(locked, writePdf({{{"secret"}, {}}, {{"more"}, {}}}, {}, "open sesame"));
        const QString guarded = dir.filePath("guarded.pdf");
        save(guarded, writePdf({{{"guarded"}, {}}, {{"more"}, {}}}, {}, {}, true));
        // The protected one opens without a password, so the reader shows it; its pages are still not changed.
        QVERIFY(readBack(guarded).read);
        for (const QString &path : {locked, guarded}) {
            QVERIFY(!PdfPages::changeable(path));
            const QByteArray before = bytesOf(path);
            PdfPages pages;
            QVERIFY(!pages.save(path, {keep(1)}));
            QVERIFY(!pages.error().isEmpty());
            QCOMPARE(bytesOf(path), before);
        }
    }

    void a_file_that_cannot_be_replaced_is_left_as_it_was_and_nothing_else_written()
    {
        QTemporaryDir own;
        const QString path = own.filePath("read-only.pdf");
        save(path, writePdf({{{"alpha one"}, {}}, {{"beta two"}, {}}}));
        const QByteArray before = bytesOf(path);
        QVERIFY(QFile::setPermissions(path, QFile::ReadOwner | QFile::ReadUser | QFile::ReadGroup | QFile::ReadOther));
        PdfPages pages;
        const bool saved = pages.save(path, {keep(1)});
        QFile::setPermissions(path, QFile::ReadOwner | QFile::WriteOwner | QFile::ReadUser | QFile::WriteUser);
        QVERIFY(!saved);
        QVERIFY(!pages.error().isEmpty());
        QCOMPARE(bytesOf(path), before);
        QCOMPARE(QDir(own.path()).entryList(QDir::Files), QStringList{"read-only.pdf"});
    }

    void changeable_says_which_pdfs_can_be_changed()
    {
        QVERIFY(PdfPages::changeable(three("f.pdf")));
        const QString broken = dir.filePath("broken.pdf");
        save(broken, "%PDF-1.7\n");
        QVERIFY(!PdfPages::changeable(broken));
        QVERIFY(!PdfPages::changeable(dir.filePath("missing.pdf")));
    }

    void a_copy_for_saving_is_made_and_only_such_a_copy_discarded()
    {
        const QString path = three("g.pdf");
        PdfPages pages;
        const QUrl copy = pages.copyForSaving(path);
        QVERIFY(copy.isLocalFile());
        QVERIFY(copy.toLocalFile() != path);
        QCOMPARE(bytesOf(copy.toLocalFile()), bytesOf(path));
        // Anything but such a copy is left alone.
        QVERIFY(!pages.discard(QUrl::fromLocalFile(path)));
        QVERIFY(QFile::exists(path));
        QVERIFY(pages.discard(copy));
        QVERIFY(!QFile::exists(copy.toLocalFile()));
    }

    void a_copy_of_a_read_only_pdf_is_still_discarded()
    {
        // Windows copies the read-only mark with the file; the copy must still go.
        const QString path = three("h.pdf");
        QVERIFY(QFile::setPermissions(path, QFile::ReadOwner | QFile::ReadUser | QFile::ReadGroup | QFile::ReadOther));
        PdfPages pages;
        const QUrl copy = pages.copyForSaving(path);
        QFile::setPermissions(path, QFile::ReadOwner | QFile::WriteOwner | QFile::ReadUser | QFile::WriteUser);
        QVERIFY(copy.isLocalFile());
        QVERIFY(pages.discard(copy));
        QVERIFY(!QFile::exists(copy.toLocalFile()));
    }
};

QTEST_MAIN(TestPdfPages)
#include "tst_pdfpages.moc"
