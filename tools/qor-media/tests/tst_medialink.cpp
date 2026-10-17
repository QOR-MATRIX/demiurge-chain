// The launcher link (ADR-046, ADR-094): what QOR Media says to a launcher and what it does with what it is told. A fake
// launcher here is a QLocalServer speaking the same frames as the QOR Launcher's product host (host/session.rs).

#include <QElapsedTimer>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalServer>
#include <QLocalSocket>
#include <QSignalSpy>
#include <QTest>
#include <QUuid>
#include <QtEndian>

#include "medialink.h"

using qormedia::MediaLink;

namespace {

class Launcher
{
public:
    Launcher() { QVERIFY(server.listen(QStringLiteral("qor-media-test-") + QUuid::createUuid().toString(QUuid::Id128))); }

    QString name() const { return server.serverName(); }

    bool accept()
    {
        if (!server.waitForNewConnection(5000))
            return false;
        peer = server.nextPendingConnection();
        return peer != nullptr;
    }

    void send(const QJsonObject &frame) { sendRaw(QJsonDocument(frame).toJson(QJsonDocument::Compact)); }

    void sendRaw(const QByteArray &body, quint32 claimed = 0)
    {
        QByteArray length(4, '\0');
        qToBigEndian(claimed ? claimed : quint32(body.size()), length.data());
        peer->write(length + body);
        peer->flush();
    }

    // The next frame from QOR Media, waiting for it while the event loop runs.
    QJsonObject read()
    {
        QElapsedTimer waited;
        waited.start();
        while (waited.elapsed() < 5000) {
            buffer += peer->readAll();
            if (buffer.size() >= 4) {
                const quint32 length = qFromBigEndian<quint32>(buffer.constData());
                if (buffer.size() >= 4 + qsizetype(length)) {
                    const QByteArray body = buffer.mid(4, length);
                    buffer.remove(0, 4 + length);
                    return QJsonDocument::fromJson(body).object();
                }
            }
            QTest::qWait(10);
        }
        return {};
    }

    void welcome(const QJsonArray &capabilities = {QStringLiteral("media.next")})
    {
        send({{QStringLiteral("type"), QStringLiteral("welcome")},
              {QStringLiteral("protocol"), 1},
              {QStringLiteral("capabilities"), capabilities}});
    }

    void answer(const QJsonObject &request, const QJsonValue &command)
    {
        send({{QStringLiteral("type"), QStringLiteral("result")},
              {QStringLiteral("id"), request.value(QStringLiteral("id"))},
              {QStringLiteral("result"), QJsonObject{{QStringLiteral("command"), command}}}});
    }

    QLocalServer server;
    QLocalSocket *peer = nullptr;
    QByteArray buffer;
};

QJsonObject reportsOf(const QJsonObject &request)
{
    return request.value(QStringLiteral("params")).toObject();
}

}  // namespace

class TestMediaLink : public QObject
{
    Q_OBJECT

private:
    // A link connected, welcomed, and waiting on its first media.next, which is returned.
    QJsonObject welcomed(Launcher &launcher, MediaLink &link)
    {
        link.connectTo(launcher.name(), "one-time-token");
        if (!launcher.accept())
            return {};
        launcher.read();  // hello
        launcher.welcome();
        return launcher.read();
    }

private slots:
    void hello_carries_the_token_once_and_names_the_product()
    {
        Launcher launcher;
        MediaLink link;
        link.connectTo(launcher.name(), "one-time-token");
        QVERIFY(launcher.accept());
        const QJsonObject hello = launcher.read();
        QCOMPARE(hello.value("type").toString(), QStringLiteral("hello"));
        QCOMPARE(hello.value("product").toString(), QStringLiteral("qor-media"));
        QCOMPARE(hello.value("token").toString(), QStringLiteral("one-time-token"));
        QCOMPARE(hello.value("protocol").toInt(), 1);
    }

    void a_welcome_offering_media_next_starts_the_wait()
    {
        Launcher launcher;
        MediaLink link;
        const QJsonObject request = welcomed(launcher, link);
        QVERIFY(link.connected());
        QCOMPARE(request.value("type").toString(), QStringLiteral("request"));
        QCOMPARE(request.value("method").toString(), QStringLiteral("media.next"));
        QVERIFY(reportsOf(request).value("reports").toArray().isEmpty());
    }

    void a_launcher_without_media_next_is_not_asked()
    {
        Launcher launcher;
        MediaLink link;
        link.connectTo(launcher.name(), "t");
        QVERIFY(launcher.accept());
        launcher.read();
        launcher.welcome({QStringLiteral("something.else")});
        QTest::qWait(200);
        QVERIFY(!link.connected());
        QVERIFY(launcher.peer->readAll().isEmpty());
    }

    void open_places_then_opens_and_asks_again()
    {
        Launcher launcher;
        MediaLink link;
        QSignalSpy placed(&link, &MediaLink::placeRequested);
        QSignalSpy opened(&link, &MediaLink::openRequested);
        const QJsonObject first = welcomed(launcher, link);
        launcher.answer(first, QJsonObject{{"type", "open"},
                                           {"path", "C:\\films\\a.mkv"},
                                           {"kind", "video"},
                                           {"position", 42.5},
                                           {"rect", QJsonObject{{"x", 10}, {"y", 20}, {"w", 800}, {"h", 600}}},
                                           {"parent", 4242}});
        QTRY_COMPARE(opened.size(), 1);
        QCOMPARE(placed.size(), 1);
        QCOMPARE(placed.at(0).at(0).toRect(), QRect(10, 20, 800, 600));
        QCOMPARE(placed.at(0).at(1).toLongLong(), 4242);
        QCOMPARE(opened.at(0).at(0).toString(), QStringLiteral("C:\\films\\a.mkv"));
        QCOMPARE(opened.at(0).at(1).toString(), QStringLiteral("video"));
        QCOMPARE(opened.at(0).at(2).toDouble(), 42.5);
        const QJsonObject second = launcher.read();
        QCOMPARE(second.value("method").toString(), QStringLiteral("media.next"));
        QVERIFY(second.value("id").toInteger() != first.value("id").toInteger());
    }

    void open_without_a_rectangle_does_not_move_the_window()
    {
        Launcher launcher;
        MediaLink link;
        QSignalSpy placed(&link, &MediaLink::placeRequested);
        QSignalSpy opened(&link, &MediaLink::openRequested);
        launcher.answer(welcomed(launcher, link), QJsonObject{{"type", "open"}, {"path", "a.png"}, {"kind", "picture"}, {"position", QJsonValue()}, {"rect", QJsonValue()}});
        QTRY_COMPARE(opened.size(), 1);
        QCOMPARE(placed.size(), 0);
        QCOMPARE(opened.at(0).at(2).toDouble(), 0.0);
    }

    void place_hide_and_theme_arrive_as_signals()
    {
        Launcher launcher;
        MediaLink link;
        QSignalSpy placed(&link, &MediaLink::placeRequested);
        QSignalSpy hidden(&link, &MediaLink::hideRequested);
        QSignalSpy themed(&link, &MediaLink::themeRequested);
        launcher.answer(welcomed(launcher, link), QJsonObject{{"type", "place"}, {"rect", QJsonObject{{"x", 1}, {"y", 2}, {"w", 3}, {"h", 4}}}});
        QTRY_COMPARE(placed.size(), 1);
        QCOMPARE(placed.at(0).at(0).toRect(), QRect(1, 2, 3, 4));
        launcher.answer(launcher.read(), QJsonObject{{"type", "hide"}});
        QTRY_COMPARE(hidden.size(), 1);
        launcher.answer(launcher.read(), QJsonObject{{"type", "theme"}, {"colors", QJsonObject{{"accent", "#00ff88"}}}});
        QTRY_COMPARE(themed.size(), 1);
        QCOMPARE(themed.at(0).at(0).toMap().value("accent").toString(), QStringLiteral("#00ff88"));
    }

    void nothing_to_do_just_asks_again()
    {
        Launcher launcher;
        MediaLink link;
        QSignalSpy opened(&link, &MediaLink::openRequested);
        launcher.answer(welcomed(launcher, link), QJsonValue());
        const QJsonObject again = launcher.read();
        QCOMPARE(again.value("method").toString(), QStringLiteral("media.next"));
        QCOMPARE(opened.size(), 0);
    }

    void reports_ride_on_the_next_request_and_only_the_newest_position_of_a_file()
    {
        Launcher launcher;
        MediaLink link;
        const QJsonObject first = welcomed(launcher, link);
        link.reportPosition("a.mkv", 10);
        link.reportLearned("a.mkv", 3600, 1920, 1080);
        link.reportPosition("b.mkv", 5);
        link.reportPosition("a.mkv", 20);
        link.reportClosed();
        launcher.answer(first, QJsonValue());
        const QJsonArray reports = reportsOf(launcher.read()).value("reports").toArray();
        QCOMPARE(reports.size(), 4);
        QCOMPARE(reports.at(0).toObject().value("type").toString(), QStringLiteral("learned"));
        QCOMPARE(reports.at(0).toObject().value("duration").toDouble(), 3600.0);
        QCOMPARE(reports.at(0).toObject().value("width").toInt(), 1920);
        QCOMPARE(reports.at(1).toObject().value("path").toString(), QStringLiteral("b.mkv"));
        QCOMPARE(reports.at(2).toObject().value("path").toString(), QStringLiteral("a.mkv"));
        QCOMPARE(reports.at(2).toObject().value("position").toDouble(), 20.0);
        QCOMPARE(reports.at(3).toObject().value("type").toString(), QStringLiteral("closed"));
        QVERIFY(link.pendingReports().isEmpty());
    }

    void what_is_not_known_is_not_reported()
    {
        MediaLink link;
        link.reportLearned("a.png", 0, 640, 480);
        link.reportLearned("b.mp3", 180, 0, 0);
        link.reportPosition("c.mkv", -1);
        link.reportPosition("", 3);
        const QJsonArray held = link.pendingReports();
        QCOMPARE(held.size(), 2);
        QVERIFY(!held.at(0).toObject().contains("duration"));
        QCOMPARE(held.at(0).toObject().value("height").toInt(), 480);
        QVERIFY(!held.at(1).toObject().contains("width"));
    }

    void the_queue_holds_at_most_sixty_four()
    {
        MediaLink link;
        for (int i = 0; i < 100; ++i)
            link.reportLearned(QStringLiteral("f%1.png").arg(i), 0, 10, 10);
        QCOMPARE(link.pendingReports().size(), 64);
    }

    void an_answer_to_another_request_is_ignored()
    {
        Launcher launcher;
        MediaLink link;
        QSignalSpy opened(&link, &MediaLink::openRequested);
        const QJsonObject first = welcomed(launcher, link);
        QJsonObject stale = first;
        stale.insert("id", first.value("id").toInteger() + 7);
        launcher.answer(stale, QJsonObject{{"type", "open"}, {"path", "x.mp4"}, {"kind", "video"}});
        QTest::qWait(200);
        QCOMPARE(opened.size(), 0);
    }

    void an_oversized_frame_ends_the_link()
    {
        Launcher launcher;
        MediaLink link;
        QSignalSpy lost(&link, &MediaLink::lost);
        welcomed(launcher, link);
        launcher.sendRaw("{}", MediaLink::FrameLimit + 1);
        QTRY_COMPARE(lost.size(), 1);
        QVERIFY(!link.connected());
    }

    void the_launcher_going_away_after_welcome_is_lost()
    {
        Launcher launcher;
        MediaLink link;
        QSignalSpy lost(&link, &MediaLink::lost);
        welcomed(launcher, link);
        launcher.peer->disconnectFromServer();
        QTRY_COMPARE(lost.size(), 1);
    }

    void a_refused_handshake_is_not_lost()
    {
        // Nothing was welcomed, so nothing is lost: QOR Media started by a stranger does not quit for it.
        Launcher launcher;
        MediaLink link;
        QSignalSpy lost(&link, &MediaLink::lost);
        link.connectTo(launcher.name(), "wrong");
        QVERIFY(launcher.accept());
        launcher.read();
        launcher.send({{"type", "error"}, {"id", QJsonValue()}, {"error", QJsonObject{{"code", "refused"}}}});
        launcher.peer->disconnectFromServer();
        QTest::qWait(200);
        QCOMPARE(lost.size(), 0);
    }
};

QTEST_GUILESS_MAIN(TestMediaLink)
#include "tst_medialink.moc"
