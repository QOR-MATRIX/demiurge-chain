// QOR Media whole (ADR-094): the window loads, opens what the launcher asks in the right view, plays sound and video
// through Qt Multimedia's FFmpeg backend, shows pictures, edits text, and tells the launcher what it learned, where it
// stopped and that it closed. Every file here is made by the test or is a sample that ships with Windows; none is the
// person's.

#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonObject>
#include <QImage>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QTest>
#include <QtEndian>
#include <QtMath>
#include <QtQml/qqmlextensionplugin.h>

#include "medialink.h"

Q_IMPORT_QML_PLUGIN(QOR_MediaPlugin)

using qormedia::MediaLink;

namespace {

// One second of a 440 Hz tone: 16-bit mono PCM at 8 kHz in a WAV file.
void writeTone(const QString &path)
{
    const int rate = 8000, samples = rate;
    QByteArray pcm(samples * 2, '\0');
    for (int i = 0; i < samples; ++i)
        qToLittleEndian(qint16(8000 * qSin(2 * M_PI * 440 * i / rate)), pcm.data() + i * 2);
    QByteArray wav;
    const auto u32 = [&](quint32 v) { char b[4]; qToLittleEndian(v, b); wav.append(b, 4); };
    const auto u16 = [&](quint16 v) { char b[2]; qToLittleEndian(v, b); wav.append(b, 2); };
    wav.append("RIFF"); u32(36 + pcm.size()); wav.append("WAVE");
    wav.append("fmt "); u32(16); u16(1); u16(1); u32(rate); u32(rate * 2); u16(2); u16(16);
    wav.append("data"); u32(pcm.size()); wav.append(pcm);
    QFile f(path);
    QVERIFY(f.open(QIODevice::WriteOnly));
    f.write(wav);
}

QJsonObject find(const QJsonArray &reports, const QString &type, const QString &path)
{
    for (const QJsonValue &r : reports) {
        const QJsonObject o = r.toObject();
        if (o.value("type").toString() == type && (path.isEmpty() || o.value("path").toString() == path))
            return o;
    }
    return {};
}

}  // namespace

class TestPlayback : public QObject
{
    Q_OBJECT

    QTemporaryDir dir;
    QQmlApplicationEngine *engine = nullptr;
    MediaLink *link = nullptr;
    QQuickWindow *window = nullptr;
    QList<QQmlError> warnings;

    void open(const QString &path, const QString &kind, double position = 0)
    {
        QMetaObject::invokeMethod(link, "openRequested", Q_ARG(QString, path), Q_ARG(QString, kind), Q_ARG(double, position));
    }

    QJsonArray reportsSoFar()
    {
        // Reports are taken by the next media.next; with no launcher they wait in the queue.
        return link->pendingReports();
    }

private slots:
    void initTestCase()
    {
        QVERIFY(dir.isValid());
        writeTone(dir.filePath("tone.wav"));
        QImage picture(64, 48, QImage::Format_RGB32);
        picture.fill(QColor("#ff6a00"));
        QVERIFY(picture.save(dir.filePath("one.png")));
        QVERIFY(picture.save(dir.filePath("two.png")));
        QFile notes(dir.filePath("notes.md"));
        QVERIFY(notes.open(QIODevice::WriteOnly));
        notes.write("# Notes\n\nSome *words*.\n");
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
        QVERIFY(!window->isVisible());
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

    void sound_plays_reports_its_length_and_where_it_stopped_and_closing_is_told()
    {
        const QString tone = dir.filePath("tone.wav");
        open(tone, "audio");
        QTRY_VERIFY(window->isVisible());
        QVERIFY(QTest::qWaitForWindowActive(window));
        QCOMPARE(window->property("kind").toString(), QStringLiteral("audio"));
        QTRY_VERIFY_WITH_TIMEOUT(!find(reportsSoFar(), "learned", tone).isEmpty(), 10000);
        const double duration = find(reportsSoFar(), "learned", tone).value("duration").toDouble();
        QVERIFY2(qAbs(duration - 1.0) < 0.1, qPrintable(QString::number(duration)));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
        QVERIFY(!find(reportsSoFar(), "position", tone).isEmpty());
        QVERIFY(!find(reportsSoFar(), "closed", {}).isEmpty());
    }

    void video_decodes_through_ffmpeg()
    {
        const QString sample = QStringLiteral(
            "C:/Windows/SystemApps/Microsoft.Windows.CloudExperienceHost_cw5n1h2txyewy/media/oobe-intro.mp4");
        if (!QFile::exists(sample))
            QSKIP("Windows' sample video is not on this machine.");
        open(sample, "video");
        QTRY_VERIFY_WITH_TIMEOUT(!find(reportsSoFar(), "learned", sample).isEmpty(), 15000);
        const QJsonObject learned = find(reportsSoFar(), "learned", sample);
        QVERIFY(learned.value("duration").toDouble() > 1);
        QVERIFY(learned.value("width").toInt() > 0);
        QVERIFY(learned.value("height").toInt() > 0);
    }

    void a_picture_is_shown_with_its_size()
    {
        const QString one = dir.filePath("one.png");
        open(one, "picture");
        QTRY_VERIFY(window->isVisible());
        QVERIFY(QTest::qWaitForWindowActive(window));
        QTRY_VERIFY(!find(reportsSoFar(), "learned", one).isEmpty());
        QCOMPARE(find(reportsSoFar(), "learned", one).value("width").toInt(), 64);
        QCOMPARE(find(reportsSoFar(), "learned", one).value("height").toInt(), 48);
        // The next picture in the folder, with the arrow key.
        QTest::keyClick(window, Qt::Key_Right);
        const QString two = QDir::toNativeSeparators(dir.filePath("two.png"));
        QTRY_VERIFY(!find(reportsSoFar(), "learned", two).isEmpty());
    }

    void text_opens_in_the_editor_and_unsaved_words_hold_the_window()
    {
        const QString notes = dir.filePath("notes.md");
        open(notes, "document");
        QTRY_VERIFY(window->isVisible());
        QVERIFY(QTest::qWaitForWindowActive(window));
        QQuickItem *area = nullptr;
        QTRY_VERIFY((area = window->findChild<QQuickItem *>("editorArea")) != nullptr);
        QTRY_COMPARE(area->property("text").toString(), QStringLiteral("# Notes\n\nSome *words*.\n"));
        area->setProperty("text", QStringLiteral("# Notes\n\nChanged.\n"));
        // Something else asked for while the words are unsaved: the editor asks first and stays.
        open(dir.filePath("one.png"), "picture");
        QTest::qWait(200);
        QCOMPARE(window->property("path").toString(), notes);
        QTest::keyClick(window, Qt::Key_Escape);  // closes the question
        QTest::keyClick(window, Qt::Key_Escape);  // asks again, since the words are still unsaved
        QTest::qWait(100);
        QVERIFY(window->isVisible());
        QCOMPARE(window->property("path").toString(), notes);
        // Nothing was written while it asked.
        QFile f(notes);
        QVERIFY(f.open(QIODevice::ReadOnly));
        QCOMPARE(f.readAll(), QByteArray("# Notes\n\nSome *words*.\n"));
    }

    void a_file_qor_media_does_not_open_says_so()
    {
        const QString pdf = dir.filePath("paper.pdf");
        QFile f(pdf);
        QVERIFY(f.open(QIODevice::WriteOnly));
        f.write("%PDF-1.7\n");
        f.close();
        open(pdf, "document");
        QTRY_VERIFY(window->isVisible());
        QVERIFY(QTest::qWaitForWindowActive(window));
        QCOMPARE(window->property("path").toString(), pdf);
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!window->isVisible());
    }

    void the_launchers_colours_are_worn()
    {
        QMetaObject::invokeMethod(link, "themeRequested", Q_ARG(QVariantMap, (QVariantMap{{"accent", "#00ff88"}, {"void", "#010203"}})));
        QTRY_COMPARE(window->color(), QColor("#010203"));
    }

    void it_docks_inside_the_launchers_window_and_lifts_out_to_full_screen()
    {
        // A stand-in for the launcher's window, known to QOR Media only by its native handle, as the launcher's is.
        QWindow launcher;
        launcher.setGeometry(100, 100, 900, 600);
        launcher.show();
        QVERIFY(QTest::qWaitForWindowExposed(&launcher));
        const qreal ratio = launcher.devicePixelRatio();
        const QRect physical(qRound(40 * ratio), qRound(30 * ratio), qRound(600 * ratio), qRound(400 * ratio));
        QMetaObject::invokeMethod(link, "placeRequested", Q_ARG(QRect, physical), Q_ARG(qint64, qint64(launcher.winId())));
        open(dir.filePath("one.png"), "picture");
        QTRY_VERIFY(window->isVisible());
        QVERIFY(window->parent() != nullptr);
        QCOMPARE(window->parent()->winId(), launcher.winId());
        QCOMPARE(window->geometry(), QRect(40, 30, 600, 400));
        QVERIFY(window->flags() & Qt::FramelessWindowHint);

        // F lifts it out to full screen, and again puts it back where it sat.
        QVERIFY(QTest::qWaitForWindowActive(window));
        QTest::keyClick(window, Qt::Key_F);
        QTRY_VERIFY(window->visibility() == QWindow::FullScreen);
        QVERIFY(window->parent() == nullptr);
        QTest::keyClick(window, Qt::Key_F);
        QTRY_VERIFY(window->parent() != nullptr);
        QTRY_COMPARE(window->geometry(), QRect(40, 30, 600, 400));
    }

    void hide_pauses_and_hides()
    {
        open(dir.filePath("tone.wav"), "audio");
        QTRY_VERIFY(window->isVisible());
        QMetaObject::invokeMethod(link, "hideRequested");
        QTRY_VERIFY(!window->isVisible());
    }
};

QTEST_MAIN(TestPlayback)
#include "tst_playback.moc"
