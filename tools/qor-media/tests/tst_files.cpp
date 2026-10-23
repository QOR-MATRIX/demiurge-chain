// Files (ADR-094): a file's kind by its extension, as the launcher's index shelves it, and the files beside it.

#include <QDir>
#include <QFile>
#include <QTemporaryDir>
#include <QTest>

#include "files.h"

using qormedia::Files;

class TestFiles : public QObject
{
    Q_OBJECT

private slots:
    void kinds_by_extension_whatever_the_case()
    {
        QCOMPARE(Files::kindOf("C:/a/film.MKV"), QStringLiteral("video"));
        QCOMPARE(Files::kindOf("song.flac"), QStringLiteral("audio"));
        QCOMPARE(Files::kindOf("photo.Jpeg"), QStringLiteral("picture"));
        QCOMPARE(Files::kindOf("notes.md"), QStringLiteral("document"));
        QCOMPARE(Files::kindOf("main.rs"), QStringLiteral("document"));
        QCOMPARE(Files::kindOf("paper.pdf"), QStringLiteral("document"));
        QCOMPARE(Files::kindOf("setup.exe"), QStringLiteral("other"));
        QCOMPARE(Files::kindOf("no-extension"), QStringLiteral("other"));
    }

    void text_is_what_the_editor_opens()
    {
        QVERIFY(Files::isText("a.txt"));
        QVERIFY(Files::isText("a.md"));
        QVERIFY(Files::isText("a.json"));
        QVERIFY(Files::isText("a.csv"));
        QVERIFY(!Files::isText("a.pdf"));
        QVERIFY(!Files::isText("a.docx"));
        QVERIFY(!Files::isText("a.png"));
        QCOMPARE(Files::textLanguage("a.markdown"), QStringLiteral("markdown"));
        QCOMPARE(Files::textLanguage("a.qml"), QStringLiteral("code"));
        QCOMPARE(Files::textLanguage("a.log"), QStringLiteral("plain"));
    }

    void a_pdf_is_what_the_reader_opens()
    {
        QVERIFY(Files::isPdf("C:/a/Paper.PDF"));
        QVERIFY(!Files::isPdf("a.pdf.txt"));
        QVERIFY(!Files::isPdf("a.docx"));
        QVERIFY(!Files::isPdf("pdf"));
    }

    void siblings_are_the_same_kind_in_the_order_people_read_numbers()
    {
        QTemporaryDir dir;
        QVERIFY(dir.isValid());
        for (const char *name : {"shot 10.jpg", "shot 2.png", "Shot 1.JPG", "clip.mp4", "notes.txt", ".hidden.jpg"}) {
            QFile f(dir.filePath(QString::fromUtf8(name)));
            QVERIFY(f.open(QIODevice::WriteOnly));
            f.write("x");
        }
        QDir().mkdir(dir.filePath("folder.jpg"));
        const QStringList found = Files::siblings(dir.filePath("shot 2.png"));
        QStringList names;
        for (const QString &p : found)
            names << Files::nameOf(p);
        QCOMPARE(names, (QStringList{"Shot 1.JPG", "shot 2.png", "shot 10.jpg"}));
        QCOMPARE(found.at(1), dir.filePath("shot 2.png"));
        QCOMPARE(found.at(2), QDir::toNativeSeparators(dir.filePath("shot 10.jpg")));
    }

    void urls_are_local_files()
    {
        const QUrl url = Files::urlOf("C:/My Films/a #1.mkv");
        QVERIFY(url.isLocalFile());
        QCOMPARE(url.toLocalFile(), QStringLiteral("C:/My Films/a #1.mkv"));
    }
};

QTEST_GUILESS_MAIN(TestFiles)
#include "tst_files.moc"
