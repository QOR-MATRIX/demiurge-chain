// The launcher link (ADR-046, ADR-102): what ARQADE says to a launcher and what it does with what it is told.

#include <QSignalSpy>
#include <QTest>

#include "arqadelink.h"
#include "fakes.h"

using arqade::ArqadeLink;
using fakes::Launcher;

class TestArqadeLink : public QObject
{
    Q_OBJECT

private:
    // A link connected, welcomed, and waiting on its first arqade.next, which is returned.
    QJsonObject welcomed(Launcher &launcher, ArqadeLink &link, const QJsonArray &capabilities = {"arqade.next", "arqade.signin", "arqade.web"})
    {
        link.connectTo(launcher.name(), "one-time-token");
        if (!launcher.accept())
            return {};
        launcher.read();  // hello
        launcher.welcome(capabilities);
        return launcher.read();
    }

private slots:
    void hello_carries_the_token_once_and_names_the_product()
    {
        Launcher launcher;
        ArqadeLink link;
        link.connectTo(launcher.name(), "one-time-token");
        QVERIFY(launcher.accept());
        const QJsonObject hello = launcher.read();
        QCOMPARE(hello.value("type").toString(), QStringLiteral("hello"));
        QCOMPARE(hello.value("product").toString(), QStringLiteral("arqade"));
        QCOMPARE(hello.value("token").toString(), QStringLiteral("one-time-token"));
        QCOMPARE(hello.value("protocol").toInt(), 1);
        QCOMPARE(hello.value("min_protocol").toInt(), 1);
    }

    void a_welcome_offering_arqade_next_starts_the_wait()
    {
        Launcher launcher;
        ArqadeLink link;
        const QJsonObject request = welcomed(launcher, link);
        QVERIFY(link.connected());
        QCOMPARE(request.value("type").toString(), QStringLiteral("request"));
        QCOMPARE(request.value("method").toString(), QStringLiteral("arqade.next"));
    }

    void a_launcher_without_arqade_next_is_not_asked()
    {
        Launcher launcher;
        ArqadeLink link;
        link.connectTo(launcher.name(), "t");
        QVERIFY(launcher.accept());
        launcher.read();
        launcher.welcome({QStringLiteral("media.next")});
        QTest::qWait(200);
        QVERIFY(!link.connected());
        QVERIFY(launcher.peer->readAll().isEmpty());
    }

    void every_command_arrives_as_its_signal_and_the_wait_goes_on()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy placed(&link, &ArqadeLink::placeRequested);
        QSignalSpy hidden(&link, &ArqadeLink::hideRequested);
        QSignalSpy themed(&link, &ArqadeLink::themeRequested);
        QSignalSpy player(&link, &ArqadeLink::playerChanged);
        QSignalSpy prefs(&link, &ArqadeLink::prefsChanged);
        QSignalSpy quit(&link, &ArqadeLink::quitRequested);

        QJsonObject request = welcomed(launcher, link);
        launcher.command(request, QJsonObject{{"type", "place"}, {"rect", QJsonObject{{"x", 10}, {"y", 20}, {"w", 800}, {"h", 600}}}, {"parent", 4242}});
        QTRY_COMPARE(placed.size(), 1);
        QCOMPARE(placed.at(0).at(0).toRect(), QRect(10, 20, 800, 600));
        QCOMPARE(placed.at(0).at(1).toLongLong(), 4242);

        request = launcher.read();
        QCOMPARE(request.value("method").toString(), QStringLiteral("arqade.next"));
        launcher.command(request, QJsonObject{{"type", "theme"}, {"colors", QJsonObject{{"accent", "#00ff88"}}}});
        QTRY_COMPARE(themed.size(), 1);
        QCOMPARE(themed.at(0).at(0).toMap().value("accent").toString(), QStringLiteral("#00ff88"));

        launcher.command(launcher.read(), QJsonObject{{"type", "player"}, {"player", QJsonObject{{"signed_in", true}, {"name", "yaldabaoth"}, {"level", 7}, {"sparks", "37,000,000 Sparks"}}}});
        QTRY_COMPARE(player.size(), 1);
        QCOMPARE(player.at(0).at(0).toMap().value("name").toString(), QStringLiteral("yaldabaoth"));
        QCOMPARE(player.at(0).at(0).toMap().value("sparks").toString(), QStringLiteral("37,000,000 Sparks"));

        launcher.command(launcher.read(), QJsonObject{{"type", "prefs"}, {"prefs", QJsonObject{{"reduced_motion", true}}}});
        QTRY_COMPARE(prefs.size(), 1);
        QVERIFY(prefs.at(0).at(0).toMap().value("reduced_motion").toBool());

        launcher.command(launcher.read(), QJsonObject{{"type", "hide"}});
        QTRY_COMPARE(hidden.size(), 1);
        QSignalSpy played(&link, &ArqadeLink::playRequested);
        launcher.command(launcher.read(), QJsonObject{{"type", "play"}, {"game", "flux"}});
        QTRY_COMPARE(played.size(), 1);
        QCOMPARE(played.at(0).at(0).toString(), QStringLiteral("flux"));
        launcher.command(launcher.read(), QJsonObject{{"type", "quit"}});
        QTRY_COMPARE(quit.size(), 1);
        QCOMPARE(launcher.read().value("method").toString(), QStringLiteral("arqade.next"));
    }

    void nothing_to_do_just_asks_again()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy placed(&link, &ArqadeLink::placeRequested);
        const QJsonObject first = welcomed(launcher, link);
        launcher.command(first, QJsonValue());
        const QJsonObject again = launcher.read();
        QCOMPARE(again.value("method").toString(), QStringLiteral("arqade.next"));
        QVERIFY(again.value("id").toInteger() != first.value("id").toInteger());
        QCOMPARE(placed.size(), 0);
    }

    void a_sign_in_waits_for_the_wait_then_goes_next()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy signedIn(&link, &ArqadeLink::signedIn);
        const QJsonObject waiting = welcomed(launcher, link);
        link.signIn(QUrl("https://id.qorsync.dev/oauth/authorize?client_id=arqade&state=s"));
        // Not sent while arqade.next is unanswered: the launcher answers one request at a time.
        QTest::qWait(100);
        QVERIFY(launcher.peer->bytesAvailable() == 0);
        QCOMPARE(link.waiting(), QStringList{"arqade.signin"});

        launcher.command(waiting, QJsonValue());
        const QJsonObject signIn = launcher.read();
        QCOMPARE(signIn.value("method").toString(), QStringLiteral("arqade.signin"));
        QCOMPARE(signIn.value("params").toObject().value("authorize_url").toString(),
                 QStringLiteral("https://id.qorsync.dev/oauth/authorize?client_id=arqade&state=s"));
        QVERIFY(link.waiting().isEmpty());

        launcher.result(signIn, {{"redirect", "https://qor-arqade-tau.vercel.app/api/auth/callback?code=c&state=s"}});
        QTRY_COMPARE(signedIn.size(), 1);
        QCOMPARE(signedIn.at(0).at(0).toUrl(), QUrl("https://qor-arqade-tau.vercel.app/api/auth/callback?code=c&state=s"));
        QCOMPARE(launcher.read().value("method").toString(), QStringLiteral("arqade.next"));
    }

    void a_newer_sign_in_replaces_one_not_yet_sent()
    {
        Launcher launcher;
        ArqadeLink link;
        const QJsonObject waiting = welcomed(launcher, link);
        link.signIn(QUrl("https://id.qorsync.dev/oauth/authorize?state=old"));
        link.signIn(QUrl("https://id.qorsync.dev/oauth/authorize?state=new"));
        QCOMPARE(link.waiting(), QStringList{"arqade.signin"});
        launcher.command(waiting, QJsonValue());
        const QJsonObject signIn = launcher.read();
        QVERIFY(signIn.value("params").toObject().value("authorize_url").toString().endsWith("state=new"));
    }

    void a_refused_sign_in_says_why_and_the_wait_goes_on()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy failed(&link, &ArqadeLink::signInFailed);
        const QJsonObject waiting = welcomed(launcher, link);
        link.signIn(QUrl("https://id.qorsync.dev/oauth/authorize"));
        launcher.command(waiting, QJsonValue());
        launcher.refuse(launcher.read(), "not_authenticated", "Sign in to the launcher first.");
        QTRY_COMPARE(failed.size(), 1);
        QCOMPARE(failed.at(0).at(0).toString(), QStringLiteral("not_authenticated"));
        QCOMPARE(failed.at(0).at(1).toString(), QStringLiteral("Sign in to the launcher first."));
        QCOMPARE(launcher.read().value("method").toString(), QStringLiteral("arqade.next"));
    }

    void an_answer_without_an_address_is_a_failure()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy signedIn(&link, &ArqadeLink::signedIn);
        QSignalSpy failed(&link, &ArqadeLink::signInFailed);
        const QJsonObject waiting = welcomed(launcher, link);
        link.signIn(QUrl("https://id.qorsync.dev/oauth/authorize"));
        launcher.command(waiting, QJsonValue());
        launcher.result(launcher.read(), {{"redirect", "/relative"}});
        QTRY_COMPARE(failed.size(), 1);
        QCOMPARE(signedIn.size(), 0);
    }

    void a_launcher_that_cannot_sign_in_is_not_asked()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy failed(&link, &ArqadeLink::signInFailed);
        QSignalSpy web(&link, &ArqadeLink::webFailed);
        welcomed(launcher, link, {"arqade.next"});
        link.signIn(QUrl("https://id.qorsync.dev/oauth/authorize"));
        link.openWeb();
        QCOMPARE(failed.size(), 1);
        QCOMPARE(failed.at(0).at(0).toString(), QStringLiteral("unsupported_method"));
        QCOMPARE(web.size(), 1);
        QVERIFY(link.waiting().isEmpty());
    }

    void the_website_is_asked_for_and_a_refusal_said()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy web(&link, &ArqadeLink::webFailed);
        const QJsonObject waiting = welcomed(launcher, link);
        link.openWeb();
        link.openWeb();
        QCOMPARE(link.waiting(), QStringList{"arqade.web"});
        launcher.command(waiting, QJsonValue());
        QJsonObject asked = launcher.read();
        QCOMPARE(asked.value("method").toString(), QStringLiteral("arqade.web"));
        launcher.result(asked, {});
        QCOMPARE(launcher.read().value("method").toString(), QStringLiteral("arqade.next"));
        QCOMPARE(web.size(), 0);
    }

    void an_answer_to_another_request_is_ignored()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy hidden(&link, &ArqadeLink::hideRequested);
        const QJsonObject first = welcomed(launcher, link);
        QJsonObject stale = first;
        stale.insert("id", first.value("id").toInteger() + 7);
        launcher.command(stale, QJsonObject{{"type", "hide"}});
        QTest::qWait(200);
        QCOMPARE(hidden.size(), 0);
    }

    void an_oversized_frame_ends_the_link()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy lost(&link, &ArqadeLink::lost);
        welcomed(launcher, link);
        launcher.sendRaw("{}", ArqadeLink::FrameLimit + 1);
        QTRY_COMPARE(lost.size(), 1);
        QVERIFY(!link.connected());
    }

    void the_launcher_going_away_after_welcome_is_lost()
    {
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy lost(&link, &ArqadeLink::lost);
        welcomed(launcher, link);
        launcher.peer->disconnectFromServer();
        QTRY_COMPARE(lost.size(), 1);
    }

    void a_refused_handshake_is_not_lost()
    {
        // Nothing was welcomed, so nothing is lost: ARQADE started by a stranger does not quit for it.
        Launcher launcher;
        ArqadeLink link;
        QSignalSpy lost(&link, &ArqadeLink::lost);
        link.connectTo(launcher.name(), "wrong");
        QVERIFY(launcher.accept());
        launcher.read();
        launcher.send({{"type", "error"}, {"id", QJsonValue()}, {"code", "not_authorised"}, {"message", "no"}});
        launcher.peer->disconnectFromServer();
        QTest::qWait(200);
        QCOMPARE(lost.size(), 0);
    }
};

QTEST_GUILESS_MAIN(TestArqadeLink)
#include "tst_arqadelink.moc"
