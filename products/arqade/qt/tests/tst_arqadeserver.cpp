// ARQADE's server, as the app signs in to it with the launcher's QOR ID, reads its rankings (ADR-102, P7.19) and plays
// its arenas (P7.24). A fake ARQADE answers HTTP; a fake launcher answers the link; QOR ID's sign-in page lives at
// "localhost", another host, as it would in life.

#include <QNetworkCookieJar>
#include <QSignalSpy>
#include <QTest>

#include "arqadelink.h"
#include "arqadeserver.h"
#include "fakes.h"

using arqade::ArqadeLink;
using arqade::ArqadeServer;
using fakes::Http;
using fakes::Launcher;
using fakes::Request;
using fakes::Response;

namespace {

const QByteArray Live = R"({
  "me": {"id": "p-me", "name": "yaldabaoth", "qorVerified": true},
  "online": 3,
  "matches": [{"id": "00000000-0000-4000-8000-0000000000aa", "game": "reversi", "status": "waiting", "moves": 0,
               "updated": 0, "alias": "sophia", "seat": 0}],
  "messages": [], "serverNow": 0, "refreshMs": 2000,
  "standings": [
    {"player": "p-1", "game": "all", "wins": "9", "losses": "1", "draws": "0", "points": "27", "alias": "sophia"},
    {"player": "p-me", "game": "all", "wins": 4, "losses": 2, "draws": 1, "points": 13, "alias": "yaldabaoth"},
    {"player": "p-1", "game": "flux", "wins": "5", "losses": "0", "draws": "0", "points": "15", "alias": "sophia"},
    {"player": "p-me", "game": "reversi", "wins": "4", "losses": "2", "draws": "1", "points": "13", "alias": "yaldabaoth"}
  ]
})";

// ARQADE as it signs a browser in: login sends to QOR ID and sets arq_login; the callback, given it back, sets
// arq_session; the live arcade answers whoever holds a session.
class Arqade : public Http
{
public:
    Arqade()
    {
        route = [this](const Request &r) -> Response {
            const QString path = r.url.path();
            if (path == "/api/auth/login") {
                logins += 1;
                if (unconfigured)
                    return {302, {{"Location", "/?qor_error=unconfigured#qor-identity"}}, {}};
                return {302,
                        {{"Location", authorize().toEncoded()},
                         {"Set-Cookie", "arq_login=L1; Path=/api/auth; HttpOnly; SameSite=Lax; Max-Age=600"}},
                        {}};
            }
            if (path == "/api/auth/callback") {
                if (r.cookie("arq_login") != "L1" || r.url.query() != "code=c&state=s")
                    return {302, {{"Location", "/?qor_error=refused#qor-identity"}}, {}};
                return {302,
                        {{"Location", "/?qor=signed_in#qor-identity"},
                         {"Set-Cookie", "arq_session=S1; Path=/; HttpOnly; SameSite=Lax; Max-Age=28800"},
                         {"Set-Cookie", "arq_login=; Path=/api/auth; HttpOnly; SameSite=Lax; Max-Age=0"}},
                        {}};
            }
            // The arenas, as arena-store.ts keeps them: signed in, and a post only as JSON from ARQADE's own pages,
            // whose origin names the host it was sent to (origin.ts).
            if (path.startsWith("/api/matches")) {
                if (r.cookie("arq_session") != "S1")
                    return {401, {{"Content-Type", "application/json"}}, R"({"error":"Sign in with QOR ID to join the live arcade."})"};
                const QUrl origin(QString::fromUtf8(r.headers.value("origin")));
                if (r.method == "POST" && (!origin.isValid() || origin.authority().toUtf8() != r.headers.value("host")
                                           || !r.headers.value("content-type").startsWith("application/json")))
                    return {403, {{"Content-Type", "application/json"}}, R"({"error":"This request must come from the arcade."})"};
                if (refuseNext) {
                    refuseNext = false;
                    return {409, {{"Content-Type", "application/json"}}, R"({"error":"Wait for your turn."})"};
                }
                QJsonObject match{{"id", MatchId}, {"game", r.method == "POST" && path == "/api/matches"
                                                               ? r.json().value("game").toString() : QStringLiteral("flux")},
                                  {"status", "waiting"}, {"revision", ++revision}, {"seat", 1},
                                  {"state", QJsonObject{{"cells", QJsonArray()}, {"turn", 1}}}};
                return {path == "/api/matches" ? 201 : 200, {{"Content-Type", "application/json"}},
                        QJsonDocument(match).toJson(QJsonDocument::Compact)};
            }
            if (path == "/api/live") {
                lives += 1;
                if (r.cookie("arq_session") != "S1" || expired)
                    return {401, {{"Content-Type", "application/json"}}, R"({"error":"Sign in with QOR ID to join the live arcade."})"};
                return {200, {{"Content-Type", "application/json"}}, Live};
            }
            return {404, {}, {}};
        };
    }

    QUrl authorize() const
    {
        return QUrl(QStringLiteral("http://localhost:%1/oauth/authorize?client_id=arqade&state=s").arg(server.serverPort()));
    }

    QUrl callback() const { return base().resolved(QUrl("api/auth/callback?code=c&state=s")); }

    static constexpr const char *MatchId = "00000000-0000-4000-8000-000000000001";

    int logins = 0;
    int lives = 0;
    int revision = 0;
    bool unconfigured = false;
    bool expired = false;
    bool refuseNext = false;
};

}  // namespace

class TestArqadeServer : public QObject
{
    Q_OBJECT

private:
    // A link welcomed by the launcher, its first arqade.next returned.
    static QJsonObject welcomed(Launcher &launcher, ArqadeLink &link)
    {
        link.connectTo(launcher.name(), "t");
        if (!launcher.accept())
            return {};
        launcher.read();
        launcher.welcome();
        return launcher.read();
    }

    // Answer the waiting arqade.next with nothing; the request after it, which is returned, is the sign-in if one waits.
    static QJsonObject signInRequest(Launcher &launcher, const QJsonObject &waiting)
    {
        launcher.command(waiting, QJsonValue());
        return launcher.read();
    }

    // The requests the stand-in ARQADE has had to its arena routes, in order.
    static QList<Request> matchCalls(const Arqade &arqade)
    {
        QList<Request> calls;
        for (const Request &r : arqade.seen)
            if (r.url.path().startsWith(QLatin1String("/api/matches")))
                calls.append(r);
        return calls;
    }

    // Signed in all the way through, the launcher's next arqade.next put in `next`.
    static void signIn(Arqade &arqade, Launcher &launcher, ArqadeLink &link, ArqadeServer &server, QJsonObject &next)
    {
        const QJsonObject waiting = welcomed(launcher, link);
        server.setLauncherSignedIn(true);
        QTRY_COMPARE(link.waiting(), QStringList{"arqade.signin"});
        const QJsonObject asked = signInRequest(launcher, waiting);
        launcher.result(asked, {{"redirect", arqade.callback().toString()}});
        next = launcher.read();
        QTRY_COMPARE(server.state(), ArqadeServer::State::SignedIn);
    }

private slots:
    void it_signs_in_through_the_launcher_and_reads_the_rankings()
    {
        Arqade arqade;
        Launcher launcher;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        server.setActive(true);

        const QJsonObject waiting = welcomed(launcher, link);
        server.setLauncherSignedIn(true);
        QTRY_COMPARE(server.state(), ArqadeServer::State::SigningIn);
        QTRY_COMPARE(link.waiting(), QStringList{"arqade.signin"});

        // The launcher is handed QOR ID's page exactly as ARQADE sent it, and nothing else.
        const QJsonObject asked = signInRequest(launcher, waiting);
        QCOMPARE(asked.value("method").toString(), QStringLiteral("arqade.signin"));
        QCOMPARE(QUrl(asked.value("params").toObject().value("authorize_url").toString()), arqade.authorize());

        launcher.result(asked, {{"redirect", arqade.callback().toString()}});
        QTRY_COMPARE(server.state(), ArqadeServer::State::SignedIn);
        QTRY_VERIFY(!server.standings().isEmpty());

        QCOMPARE(server.online(), 3);
        QCOMPARE(server.me().value("name").toString(), QStringLiteral("yaldabaoth"));
        const QVariantList all = server.standings().value("all").toList();
        QCOMPARE(all.size(), 2);
        // Sums sent as strings, as Postgres sends them, are read as whole numbers.
        QCOMPARE(all.at(0).toMap().value("points").toLongLong(), 27);
        QCOMPARE(all.at(0).toMap().value("rank").toInt(), 1);
        QCOMPARE(all.at(0).toMap().value("me").toBool(), false);
        QCOMPARE(all.at(1).toMap().value("rank").toInt(), 2);
        QCOMPARE(all.at(1).toMap().value("me").toBool(), true);
        QCOMPARE(server.standings().value("flux").toList().size(), 1);
        QCOMPARE(server.standings().value("reversi").toList().at(0).toMap().value("draws").toLongLong(), 1);
        QVERIFY(server.message().isEmpty());
    }

    void a_sign_in_that_leads_elsewhere_is_not_followed()
    {
        Arqade arqade;
        Launcher launcher;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        const QJsonObject waiting = welcomed(launcher, link);
        server.setLauncherSignedIn(true);
        QTRY_COMPARE(link.waiting(), QStringList{"arqade.signin"});
        const QJsonObject asked = signInRequest(launcher, waiting);
        // Another host: not followed, so its code is never sent anywhere.
        launcher.result(asked, {{"redirect", arqade.authorize().resolved(QUrl("/api/auth/callback?code=c&state=s")).toString()}});
        QTRY_COMPARE(server.state(), ArqadeServer::State::SignedOut);
        QVERIFY(!server.message().isEmpty());
        for (const Request &r : arqade.seen)
            QVERIFY(r.url.path() != "/api/auth/callback");
    }

    void a_refused_sign_in_says_why()
    {
        Arqade arqade;
        Launcher launcher;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        const QJsonObject waiting = welcomed(launcher, link);
        server.setLauncherSignedIn(true);
        QTRY_COMPARE(link.waiting(), QStringList{"arqade.signin"});
        launcher.refuse(signInRequest(launcher, waiting), "not_authenticated", "Sign in to QOR ID first.");
        QTRY_COMPARE(server.state(), ArqadeServer::State::SignedOut);
        QVERIFY(server.message().contains("QOR ID"));
    }

    void an_unconfigured_server_is_said_so()
    {
        Arqade arqade;
        arqade.unconfigured = true;
        Launcher launcher;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        welcomed(launcher, link);
        server.setLauncherSignedIn(true);
        QTRY_COMPARE(server.state(), ArqadeServer::State::SignedOut);
        QVERIFY(server.message().contains("not set up"));
        QVERIFY(link.waiting().isEmpty());
    }

    void a_session_that_ends_is_renewed_once_and_not_in_a_loop()
    {
        Arqade arqade;
        Launcher launcher;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        server.setActive(true);
        QJsonObject waiting;
        signIn(arqade, launcher, link, server, waiting);
        if (QTest::currentTestFailed())
            return;
        QTRY_COMPARE(arqade.lives, 1);
        QCOMPARE(arqade.logins, 1);

        // The session ends at ARQADE: the app signs in again, once.
        arqade.expired = true;
        server.refresh();
        QTRY_COMPARE(arqade.logins, 2);
        QTRY_COMPARE(link.waiting(), QStringList{"arqade.signin"});
        const QJsonObject asked = signInRequest(launcher, waiting);
        launcher.result(asked, {{"redirect", arqade.callback().toString()}});
        // Signed in again, but still refused: it stops there.
        QTRY_COMPARE(server.state(), ArqadeServer::State::SignedOut);
        QTest::qWait(300);
        QCOMPARE(arqade.logins, 2);
    }

    void signing_out_of_the_launcher_forgets_the_session()
    {
        Arqade arqade;
        Launcher launcher;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        server.setActive(true);
        QJsonObject waiting;
        signIn(arqade, launcher, link, server, waiting);
        if (QTest::currentTestFailed())
            return;
        QTRY_VERIFY(!server.standings().isEmpty());

        server.setLauncherSignedIn(false);
        QCOMPARE(server.state(), ArqadeServer::State::SignedOut);
        QVERIFY(server.standings().isEmpty());
        QVERIFY(server.me().isEmpty());
        const int asked = arqade.lives;
        server.refresh();
        QTest::qWait(200);
        QCOMPARE(arqade.lives, asked);
    }

    void without_a_launcher_it_does_not_try()
    {
        Arqade arqade;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        server.signIn();
        QCOMPARE(server.state(), ArqadeServer::State::SignedOut);
        QVERIFY(server.message().contains("QOR Launcher"));
        QTest::qWait(100);
        QVERIFY(arqade.seen.isEmpty());
    }

    void a_server_that_is_not_there_is_unreachable()
    {
        Launcher launcher;
        ArqadeLink link;
        QUrl nowhere;
        {
            QTcpServer closed;
            closed.listen(QHostAddress::LocalHost);
            nowhere = QUrl(QStringLiteral("http://127.0.0.1:%1/").arg(closed.serverPort()));
        }
        ArqadeServer server(&link, nowhere);
        welcomed(launcher, link);
        server.setLauncherSignedIn(true);
        QTRY_COMPARE(server.state(), ArqadeServer::State::Unreachable);
        QVERIFY(server.message().contains("could not be reached"));
    }

    void only_this_server_is_ours()
    {
        ArqadeServer server(nullptr, QUrl("https://qor-arqade-tau.vercel.app/"));
        QVERIFY(server.isOurs(QUrl("https://qor-arqade-tau.vercel.app/api/auth/callback?code=c")));
        QVERIFY(server.isOurs(QUrl("https://QOR-ARQADE-TAU.vercel.app:443/x")));
        QVERIFY(!server.isOurs(QUrl("http://qor-arqade-tau.vercel.app/api/auth/callback")));
        QVERIFY(!server.isOurs(QUrl("https://qor-arqade-tau.vercel.app.example.com/")));
        QVERIFY(!server.isOurs(QUrl("https://qor-arqade-tau.vercel.app:8443/")));
        QVERIFY(!server.isOurs(QUrl("https://id.qorsync.dev/oauth/authorize")));
    }

    // The arenas (P7.24), played as the website plays them: the open matches read from the live arcade, a match created,
    // asked after and moved in, each post from ARQADE's own origin, as JSON, with the session; a refusal in the
    // server's own words.
    void the_arenas_are_played_through_it_as_the_website_plays_them()
    {
        Arqade arqade;
        Launcher launcher;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        server.setActive(true);
        QJsonObject next;
        signIn(arqade, launcher, link, server, next);
        QTRY_COMPARE(server.matches().size(), 1);
        const QVariantMap open = server.matches().first().toMap();
        QCOMPARE(open.value("game").toString(), QStringLiteral("reversi"));
        QCOMPARE(open.value("status").toString(), QStringLiteral("waiting"));
        QCOMPARE(open.value("alias").toString(), QStringLiteral("sophia"));
        QCOMPARE(open.value("seat").toInt(), 0);

        QSignalSpy answered(&server, &ArqadeServer::matchAnswered);
        QSignalSpy refused(&server, &ArqadeServer::matchRefused);
        // Each call is answered once; a refusal it should not have had fails with the server's words.
        const auto heard = [&](qsizetype calls) {
            QTRY_COMPARE(answered.size() + refused.size(), calls);
            QVERIFY2(refused.isEmpty(), qPrintable(refused.value(0).value(0).toString()));
        };

        server.createMatch("flux");
        heard(1);
        QCOMPARE(matchCalls(arqade).size(), 1);
        const Request created = matchCalls(arqade).last();
        QCOMPARE(created.method, QByteArray("POST"));
        QCOMPARE(created.url.path(), QStringLiteral("/api/matches"));
        QCOMPARE(created.headers.value("origin"), QByteArray("http://127.0.0.1:") + QByteArray::number(arqade.server.serverPort()));
        QCOMPARE(created.json().value("game").toString(), QStringLiteral("flux"));
        QCOMPARE(answered.last().first().toMap().value("id").toString(), QString::fromLatin1(Arqade::MatchId));
        QCOMPARE(answered.last().first().toMap().value("game").toString(), QStringLiteral("flux"));
        // Each answer names the match its request asked about: for a new one, none.
        QCOMPARE(answered.last().at(1).toString(), QString());

        server.fetchMatch(Arqade::MatchId);
        heard(2);
        QCOMPARE(matchCalls(arqade).last().method, QByteArray("GET"));
        QCOMPARE(matchCalls(arqade).last().url.path(), QStringLiteral("/api/matches/") + Arqade::MatchId);
        QCOMPARE(answered.last().at(1).toString(), QString::fromLatin1(Arqade::MatchId));

        server.actOnMatch(Arqade::MatchId, "move", 3, 2);
        heard(3);
        const QJsonObject move = matchCalls(arqade).last().json();
        QCOMPARE(move.value("action").toString(), QStringLiteral("move"));
        QCOMPARE(move.value("cell").toInt(), 3);
        QCOMPARE(move.value("revision").toInt(), 2);

        // A join carries no cell.
        server.actOnMatch(Arqade::MatchId, "join", 0, 3);
        heard(4);
        QVERIFY(!matchCalls(arqade).last().json().contains("cell"));

        arqade.refuseNext = true;
        server.actOnMatch(Arqade::MatchId, "move", 4, 4);
        QTRY_COMPARE(refused.size(), 1);
        QCOMPARE(refused.last().first().toString(), QStringLiteral("Wait for your turn."));
        QCOMPARE(refused.last().at(1).toString(), QString::fromLatin1(Arqade::MatchId));
        QCOMPARE(answered.size(), 4);
    }

    // A match's id goes into an address, so only a match's id is sent; an action or a game the arenas do not have is
    // refused before anything is sent.
    void a_match_call_with_a_bad_id_action_or_game_sends_nothing()
    {
        Arqade arqade;
        Launcher launcher;
        ArqadeLink link;
        ArqadeServer server(&link, arqade.base());
        QJsonObject next;
        signIn(arqade, launcher, link, server, next);
        QSignalSpy refused(&server, &ArqadeServer::matchRefused);
        server.fetchMatch("../live");
        server.actOnMatch(Arqade::MatchId, "delete", 0, 0);
        server.actOnMatch("00000000-0000-4000-8000-00000000000Z", "move", 0, 0);
        server.createMatch("chess");
        QCOMPARE(refused.size(), 4);
        QCOMPARE(refused.at(1).at(1).toString(), QString::fromLatin1(Arqade::MatchId));
        QCOMPARE(refused.at(3).at(1).toString(), QString());
        QTest::qWait(200);
        QVERIFY(matchCalls(arqade).isEmpty());
    }
};

QTEST_GUILESS_MAIN(TestArqadeServer)
#include "tst_arqadeserver.moc"
