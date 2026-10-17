// TextFile (ADR-094 decision 3): a text file read and saved as it was written, and never a file it could not read.

#include <QFile>
#include <QTemporaryDir>
#include <QTest>

#include "textfile.h"

using qormedia::TextFile;

namespace {

QByteArray bytesOf(const QString &path)
{
    QFile f(path);
    return f.open(QIODevice::ReadOnly) ? f.readAll() : QByteArray();
}

void write(const QString &path, const QByteArray &bytes)
{
    QFile f(path);
    QVERIFY(f.open(QIODevice::WriteOnly));
    f.write(bytes);
}

}  // namespace

class TestTextFile : public QObject
{
    Q_OBJECT

private slots:
    void lf_stays_lf()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("a.txt");
        write(path, "one\ntwo\n");
        TextFile file;
        QCOMPARE(file.load(path), QStringLiteral("one\ntwo\n"));
        QVERIFY(!file.crlf());
        QVERIFY(file.save("one\ntwo\nthree\n"));
        QCOMPARE(bytesOf(path), QByteArray("one\ntwo\nthree\n"));
    }

    void crlf_is_read_as_lines_and_written_back_as_crlf()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("a.txt");
        write(path, "one\r\ntwo\r\n");
        TextFile file;
        QCOMPARE(file.load(path), QStringLiteral("one\ntwo\n"));
        QVERIFY(file.crlf());
        QVERIFY(file.save("one\ntwo\nthree"));
        QCOMPARE(bytesOf(path), QByteArray("one\r\ntwo\r\nthree"));
    }

    void a_utf8_byte_order_mark_is_kept()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("a.md");
        write(path, "\xEF\xBB\xBF# Caf\xC3\xA9\n");
        TextFile file;
        QCOMPARE(file.load(path), QStringLiteral("# Café\n"));
        QCOMPARE(file.language(), QStringLiteral("markdown"));
        QVERIFY(file.save(QStringLiteral("# Cafés\n")));
        QCOMPARE(bytesOf(path), QByteArray("\xEF\xBB\xBF# Caf\xC3\xA9s\n"));
    }

    void utf16_is_read_and_written_as_utf16()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("a.txt");
        write(path, QByteArray("\xFF\xFE" "h\0i\0\r\0\n\0", 10));
        TextFile file;
        QCOMPARE(file.load(path), QStringLiteral("hi\n"));
        QVERIFY(file.error().isEmpty());
        QVERIFY(file.save(QStringLiteral("ho\n")));
        QCOMPARE(bytesOf(path), QByteArray("\xFF\xFE" "h\0o\0\r\0\n\0", 10));
    }

    void binary_is_refused_and_never_overwritten()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("a.txt");
        const QByteArray binary("MZ\0\0\x90\0\x03", 7);
        write(path, binary);
        TextFile file;
        QVERIFY(file.load(path).isEmpty());
        QVERIFY(!file.error().isEmpty());
        QVERIFY(!file.save("typed over nothing"));
        QCOMPARE(bytesOf(path), binary);
    }

    void a_file_larger_than_the_editor_holds_is_refused_and_never_overwritten()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("big.log");
        {
            QFile f(path);
            QVERIFY(f.open(QIODevice::WriteOnly));
            QVERIFY(f.resize(TextFile::Limit + 1));
        }
        TextFile file;
        QVERIFY(file.load(path).isEmpty());
        QVERIFY(!file.error().isEmpty());
        QVERIFY(!file.save("x"));
        QCOMPARE(QFileInfo(path).size(), TextFile::Limit + 1);
    }

    void a_missing_file_is_an_error_and_not_created()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("gone.txt");
        TextFile file;
        QVERIFY(file.load(path).isEmpty());
        QVERIFY(!file.error().isEmpty());
        QVERIFY(!file.save("x"));
        QVERIFY(!QFile::exists(path));
    }

    void a_failed_load_after_a_good_one_still_refuses_to_save()
    {
        QTemporaryDir dir;
        write(dir.filePath("good.txt"), "fine");
        const QByteArray binary("\0\0\0", 3);
        write(dir.filePath("bad.txt"), binary);
        TextFile file;
        QCOMPARE(file.load(dir.filePath("good.txt")), QStringLiteral("fine"));
        QVERIFY(file.load(dir.filePath("bad.txt")).isEmpty());
        QVERIFY(!file.save("x"));
        QCOMPARE(bytesOf(dir.filePath("bad.txt")), binary);
    }
};

QTEST_GUILESS_MAIN(TestTextFile)
#include "tst_textfile.moc"
