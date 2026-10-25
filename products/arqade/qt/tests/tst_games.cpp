// ARQADE's games rebuilt in QQ (P7.20, ADR-102 decision 2), played in ARQADE's window by QQ's runtime. Each game's card
// opens it in place of the home, and it is played as a person plays it, by the mouse on its screen buttons, the
// keyboard and the gamepad:
//
//   Orbital: a miss, a pause, and ten locks to complete the orbit.
//   Void Runner: lanes changed by each of them, hazards dodged and gems caught, then three hazards met to end the run.
//   Synapse: a press while the signal shows is no answer; three signals repeated, then a node out of turn.
//   Rift Survivor: moved each way there is; steered clear until the signal evolves and a perk is chosen; then left
//   standing until its hull breaks.
//
// And the two arenas (P7.24), against another person on ARQADE's server, here a stand-in that signs in as the website
// does, keeps matches as arena-store.ts keeps them, and plays the opponent when the test says:
//
//   Signed out, an arena says so and offers nothing to do.
//   Flux Four: a match opened, a challenger joining, four pulses dropped by Space, a number key, the pad and the
//   pointer, the match won; then a rematch's lobby closed before anyone joins.
//   Rift Reversi: someone's open lobby joined, discs placed by the keys and the pointer and turned over, a move the
//   server refuses said so, and the match conceded once the player is sure.
//
// The run ARQADE kept of each replays in QQ to the same outcome, and so does each run kept in
// products/arqade/tests/fixtures/, which the game's rules replay in Node (tests/<game>.test.mjs). The launcher's reduced
// motion reaches the game, and Esc and Back return to the home.
//
// It opens a real window, so it needs a desktop session. With ARQADE_SHOT set to a folder, it saves what it drew there;
// with ARQADE_RUN set to a folder, it writes each run it played there, as <game>-run.json (how the fixtures were made).

#include <QDateTime>
#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QGuiApplication>
#include <QJSValue>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalSpy>
#include <QTest>
#include <QtQml/qqmlextensionplugin.h>

#include <cmath>
#include <functional>

#include "arqadelink.h"
#include "arqadeserver.h"
#include "fakes.h"
#include "fonts.h"
#include "input.h"
#include "run.h"

Q_IMPORT_QML_PLUGIN(ARQADEPlugin)
Q_IMPORT_QML_PLUGIN(QQPlugin)

using arqade::ArqadeLink;
using arqade::ArqadeServer;

namespace {

const QUrl kOrbital(QStringLiteral("qrc:/games/orbital/scenes/orbital.qml"));
const QUrl kVoidRunner(QStringLiteral("qrc:/games/void-runner/scenes/void-runner.qml"));
const QUrl kSynapse(QStringLiteral("qrc:/games/synapse/scenes/synapse.qml"));
const QUrl kRiftSurvivor(QStringLiteral("qrc:/games/rift-survivor/scenes/rift-survivor.qml"));
const QUrl kFluxFour(QStringLiteral("qrc:/games/arenas/scenes/flux-four.qml"));
const QUrl kRiftReversi(QStringLiteral("qrc:/games/arenas/scenes/rift-reversi.qml"));
// Rift Survivor's rules (logic/rules.mjs): a run's length in steps.
constexpr int Rules_RUN_STEPS = 5400;
// Orbital's rules (logic/rules.mjs): a whole turn, and half the capture arc.
constexpr double kTurn = 6.283185307179586;
constexpr double kArc = 0.29;

void save(QQuickWindow *window, const QString &name)
{
    const QByteArray folder = qgetenv("ARQADE_SHOT");
    if (folder.isEmpty())
        return;
    QDir().mkpath(QString::fromLocal8Bit(folder));
    window->grabWindow().save(QDir(QString::fromLocal8Bit(folder)).filePath(name));
}

QQuickItem *find(QQuickItem *item, const std::function<bool(QQuickItem *)> &wanted)
{
    if (!item)
        return nullptr;
    if (wanted(item))
        return item;
    for (QQuickItem *child : item->childItems())
        if (QQuickItem *found = find(child, wanted))
            return found;
    return nullptr;
}

// How far the satellite is from the arc's middle, round the orbit, as the rules measure it.
double apart(double angle, double target)
{
    double d = angle - target;
    d -= std::floor(d / kTurn) * kTurn;
    return std::min(d, kTurn - d);
}

QVariantMap asMap(const QVariant &v)
{
    return v.canConvert<QJSValue>() ? v.value<QJSValue>().toVariant().toMap() : v.toMap();
}

QVariantList asList(const QVariant &v)
{
    return v.canConvert<QJSValue>() ? v.value<QJSValue>().toVariant().toList() : v.toList();
}


// ARQADE as it signs a browser in (tst_arqadeserver's stand-in does the same), with an arena as arena-store.ts keeps
// one: matches created, joined, moved in, resigned and cancelled by the player ("yaldabaoth"), and by an opponent
// ("sophia") whom the test moves for. The moves are lib/arena-engine.ts's rules, in brief. Only its own origin may post.
class Arcade : public fakes::Http
{
public:
    static constexpr const char *Me = "p-me";
    static constexpr const char *Them = "p-1";

    struct Match
    {
        QString id, game, status = QStringLiteral("waiting"), host, guest;
        QList<int> cells;
        int turn = 1, winner = -1, placed = -1, revision = 0, moves = 0;
        QList<int> flipped;
    };

    QList<Match> matches;
    /// A post refused once, in these words.
    QString refuseNext;
    int created = 0;

    Arcade()
    {
        route = [this](const fakes::Request &r) { return answer(r); };
    }

    QUrl authorize() const
    {
        return QUrl(QStringLiteral("http://localhost:%1/oauth/authorize?client_id=arqade&state=s").arg(server.serverPort()));
    }
    QUrl callback() const { return base().resolved(QUrl("api/auth/callback?code=c&state=s")); }

    Match *find(const QString &id)
    {
        for (Match &m : matches)
            if (m.id == id)
                return &m;
        return nullptr;
    }
    /// The player's match of `game` most recently made or joined.
    Match *latest(const QString &game)
    {
        for (auto it = matches.rbegin(); it != matches.rend(); ++it)
            if (it->game == game && (it->host == QLatin1String(Me) || it->guest == QLatin1String(Me)))
                return &*it;
        return nullptr;
    }

    static QList<int> flips(const QList<int> &cells, int pos, int turn)
    {
        QList<int> out;
        if (pos < 0 || pos >= 36 || cells[pos])
            return out;
        for (int dx = -1; dx <= 1; ++dx) {
            for (int dy = -1; dy <= 1; ++dy) {
                if (!dx && !dy)
                    continue;
                int x = pos % 6 + dx, y = pos / 6 + dy;
                QList<int> line;
                while (x >= 0 && x < 6 && y >= 0 && y < 6 && cells[y * 6 + x] == 3 - turn) {
                    line << y * 6 + x;
                    x += dx;
                    y += dy;
                }
                if (!line.isEmpty() && x >= 0 && x < 6 && y >= 0 && y < 6 && cells[y * 6 + x] == turn)
                    out << line;
            }
        }
        return out;
    }

    static QList<int> legal(const Match &m)
    {
        QList<int> out;
        if (m.winner >= 0)
            return out;
        if (m.game == QLatin1String("flux")) {
            for (int c = 0; c < 7; ++c)
                if (!m.cells[c])
                    out << c;
            return out;
        }
        for (int i = 0; i < 36; ++i)
            if (!flips(m.cells, i, m.turn).isEmpty())
                out << i;
        return out;
    }

    /// A move by whoever's turn it is: false if it is not a legal one.
    static bool play(Match &m, int choice)
    {
        if (!legal(m).contains(choice))
            return false;
        const int turn = m.turn;
        m.flipped.clear();
        m.moves += 1;
        m.revision += 1;
        if (m.game == QLatin1String("flux")) {
            for (int row = 5; row >= 0; --row) {
                if (!m.cells[row * 7 + choice]) {
                    m.placed = row * 7 + choice;
                    break;
                }
            }
            m.cells[m.placed] = turn;
            static const int ways[4][2] = {{1, 0}, {0, 1}, {1, 1}, {1, -1}};
            for (const auto &way : ways) {
                int count = 1;
                for (int sign : {-1, 1}) {
                    int x = m.placed % 7 + way[0] * sign, y = m.placed / 7 + way[1] * sign;
                    while (x >= 0 && x < 7 && y >= 0 && y < 6 && m.cells[y * 7 + x] == turn) {
                        count += 1;
                        x += way[0] * sign;
                        y += way[1] * sign;
                    }
                }
                if (count >= 4)
                    m.winner = turn;
            }
            if (m.winner < 0) {
                m.turn = 3 - turn;
                if (!m.cells.contains(0))
                    m.winner = 0;
            }
        } else {
            m.flipped = flips(m.cells, choice, turn);
            m.cells[choice] = turn;
            m.placed = choice;
            for (int i : m.flipped)
                m.cells[i] = turn;
            m.turn = 3 - turn;
            if (legal(m).isEmpty()) {
                m.turn = turn;
                if (legal(m).isEmpty()) {
                    const qsizetype a = m.cells.count(1), b = m.cells.count(2);
                    m.winner = a == b ? 0 : a > b ? 1 : 2;
                }
            }
        }
        if (m.winner >= 0)
            m.status = QStringLiteral("done");
        return true;
    }

    Match &make(const QString &game, const QString &host)
    {
        Match m;
        m.id = QUuid::createUuid().toString(QUuid::WithoutBraces);
        m.game = game;
        m.host = host;
        m.cells = QList<int>(game == QLatin1String("flux") ? 42 : 36, 0);
        if (game == QLatin1String("reversi")) {
            m.cells[14] = 2;
            m.cells[15] = 1;
            m.cells[20] = 1;
            m.cells[21] = 2;
        }
        matches << m;
        return matches.last();
    }

    // The opponent, as the test moves for them.
    QString opponentOpens(const QString &game) { return make(game, QLatin1String(Them)).id; }
    void opponentJoins(const QString &id)
    {
        Match *m = find(id);
        m->guest = QLatin1String(Them);
        m->status = QStringLiteral("active");
        m->revision += 1;
    }
    bool opponentMoves(const QString &id, int choice)
    {
        Match *m = find(id);
        const int seat = m->host == QLatin1String(Them) ? 1 : 2;
        return m->status == QLatin1String("active") && m->turn == seat && play(*m, choice);
    }

    QJsonObject snapshot(const Match &m) const
    {
        const double now = double(QDateTime::currentMSecsSinceEpoch());
        auto who = [](const QString &id) -> QJsonValue {
            if (id.isEmpty())
                return QJsonValue::Null;
            return QJsonObject{{"id", id}, {"name", id == QLatin1String(Me) ? "yaldabaoth" : "sophia"}, {"qorVerified", true}};
        };
        QJsonArray cells, flipped, moves;
        for (int v : m.cells)
            cells << v;
        for (int v : m.flipped)
            flipped << v;
        if (m.status == QLatin1String("active"))
            for (int v : legal(m))
                moves << v;
        return {{"id", m.id}, {"game", m.game}, {"status", m.status},
                {"state", QJsonObject{{"cells", cells}, {"turn", m.turn},
                                      {"winner", m.winner < 0 ? QJsonValue(QJsonValue::Null) : QJsonValue(m.winner)},
                                      {"placed", m.placed}, {"flipped", flipped}}},
                {"revision", m.revision}, {"moves", m.moves}, {"deadline", now + 90000}, {"serverNow", now},
                {"host", who(m.host)}, {"guest", who(m.guest)},
                {"seat", m.host == QLatin1String(Me) ? 1 : m.guest == QLatin1String(Me) ? 2 : 0}, {"legal", moves}};
    }

    QByteArray live() const
    {
        QJsonArray rows;
        // The player's own first, as the live arcade lists them.
        for (int pass = 0; pass < 2; ++pass) {
            for (const Match &m : matches) {
                const int seat = m.host == QLatin1String(Me) ? 1 : m.guest == QLatin1String(Me) ? 2 : 0;
                if ((pass == 0) != (seat > 0) || (m.status != QLatin1String("waiting") && m.status != QLatin1String("active")))
                    continue;
                rows << QJsonObject{{"id", m.id}, {"game", m.game}, {"status", m.status}, {"moves", m.moves}, {"updated", 0},
                                    {"alias", m.host == QLatin1String(Me) ? "yaldabaoth" : "sophia"}, {"seat", seat}};
            }
        }
        return QJsonDocument(QJsonObject{{"me", QJsonObject{{"id", Me}, {"name", "yaldabaoth"}, {"qorVerified", true}}},
                                         {"online", 2}, {"matches", rows}, {"standings", QJsonArray()},
                                         {"messages", QJsonArray()}, {"serverNow", 0}, {"refreshMs", 2000}})
            .toJson(QJsonDocument::Compact);
    }

    fakes::Response answer(const fakes::Request &r)
    {
        const QString path = r.url.path();
        const QList<QPair<QByteArray, QByteArray>> json = {{"Content-Type", "application/json"}};
        const auto said = [&](int status, const QJsonObject &body) {
            return fakes::Response{status, json, QJsonDocument(body).toJson(QJsonDocument::Compact)};
        };
        const auto refused = [&](int status, const QString &why) { return said(status, {{"error", why}}); };
        if (path == QLatin1String("/api/auth/login"))
            return {302,
                    {{"Location", authorize().toEncoded()},
                     {"Set-Cookie", "arq_login=L1; Path=/api/auth; HttpOnly; SameSite=Lax; Max-Age=600"}},
                    {}};
        if (path == QLatin1String("/api/auth/callback")) {
            if (r.cookie("arq_login") != "L1")
                return {302, {{"Location", "/?qor_error=refused#qor-identity"}}, {}};
            return {302,
                    {{"Location", "/?qor=signed_in#qor-identity"},
                     {"Set-Cookie", "arq_session=S1; Path=/; HttpOnly; SameSite=Lax; Max-Age=28800"},
                     {"Set-Cookie", "arq_login=; Path=/api/auth; HttpOnly; SameSite=Lax; Max-Age=0"}},
                    {}};
        }
        if (r.cookie("arq_session") != "S1")
            return refused(401, QStringLiteral("Sign in with QOR ID to join the live arcade."));
        if (path == QLatin1String("/api/live"))
            return {200, json, live()};
        if (!path.startsWith(QLatin1String("/api/matches")))
            return {404, {}, {}};
        const QUrl origin(QString::fromUtf8(r.headers.value("origin")));
        if (r.method == "POST" && (!origin.isValid() || origin.authority().toUtf8() != r.headers.value("host")))
            return refused(403, QStringLiteral("This request must come from the arcade."));
        if (r.method == "POST" && !refuseNext.isEmpty()) {
            const QString why = refuseNext;
            refuseNext.clear();
            return refused(400, why);
        }
        if (path == QLatin1String("/api/matches") && r.method == "POST") {
            created += 1;
            return said(201, snapshot(make(r.json().value("game").toString(), QLatin1String(Me))));
        }
        Match *m = find(path.mid(13));
        if (!m)
            return refused(404, QStringLiteral("This match could not be found."));
        if (r.method == "GET")
            return said(200, snapshot(*m));
        const QJsonObject body = r.json();
        const QString action = body.value("action").toString();
        if (action == QLatin1String("join")) {
            if (m->host == QLatin1String(Me))
                return refused(400, QStringLiteral("A second player must join this match."));
            if (m->status != QLatin1String("waiting"))
                return refused(409, QStringLiteral("This lobby is no longer available."));
            m->guest = QLatin1String(Me);
            m->status = QStringLiteral("active");
            m->revision += 1;
            return said(200, snapshot(*m));
        }
        const int seat = m->host == QLatin1String(Me) ? 1 : m->guest == QLatin1String(Me) ? 2 : 0;
        if (!seat)
            return refused(403, QStringLiteral("Only the match players can make moves."));
        if (action == QLatin1String("cancel") && m->status == QLatin1String("waiting")) {
            m->status = QStringLiteral("cancelled");
            m->revision += 1;
            return said(200, snapshot(*m));
        }
        if (m->status != QLatin1String("active"))
            return refused(409, QStringLiteral("This match is not active."));
        if (body.value("revision").toInt() != m->revision)
            return refused(409, QStringLiteral("The board has changed. Refreshing the match."));
        if (action == QLatin1String("resign")) {
            m->winner = 3 - seat;
            m->status = QStringLiteral("done");
            m->revision += 1;
            return said(200, snapshot(*m));
        }
        if (action != QLatin1String("move") || m->turn != seat)
            return refused(400, QStringLiteral("Wait for your turn."));
        if (!play(*m, body.value("cell").toInt(-1)))
            return refused(400, QStringLiteral("That move is not available."));
        return said(200, snapshot(*m));
    }
};

}  // namespace

class TestGames : public QObject
{
    Q_OBJECT

    QQmlApplicationEngine *engine = nullptr;
    ArqadeLink *link = nullptr;
    ArqadeServer *server = nullptr;
    QQuickWindow *window = nullptr;
    qq::Input *input = nullptr;
    Arcade *arcade = nullptr;
    fakes::Launcher *launcher = nullptr;
    QStringList warnings;
    /// The Behaviour whose logic is the game being played.
    QString behaviour = QStringLiteral("Orbital");

    QQuickItem *home() const { return window->findChild<QQuickItem *>(QStringLiteral("home")); }
    QQuickItem *game() const { return window->findChild<QQuickItem *>(QStringLiteral("game")); }
    QObject *scene() const { return game() ? game()->property("scene").value<QObject *>() : nullptr; }

    // The game's logic: the Behaviour of its name, and what it holds.
    QObject *logic() const
    {
        if (!game())
            return nullptr;
        for (const QVariant &e : asList(game()->property("entityList"))) {
            QObject *o = e.value<QObject *>();
            if (o && o->property("kind").toString() == QLatin1String("Behaviour")
                && o->property("name").toString() == behaviour)
                return o->property("logic").value<QObject *>();
        }
        return nullptr;
    }
    // The rules' game as it stands: Orbital's { phase, angle, target, locks, misses, ... }, Void Runner's { phase, lane,
    // coming, hull, gems, steps, ... }.
    QVariantMap state() const { return logic() ? asMap(logic()->property("game")) : QVariantMap(); }

    // The text a screen part shows, by its name.
    QString screenText(const QString &name) const
    {
        QQuickItem *drawn = find(game(), [&](QQuickItem *i) { return i->objectName() == name; });
        QQuickItem *text = find(drawn, [](QQuickItem *i) { return i->inherits("QQuickText"); });
        return text ? text->property("text").toString() : QString();
    }

    // An entity of the scene being played, by its name and kind (a screen button may share a shape's name).
    QObject *entityNamed(const QString &name, const QString &kind = QStringLiteral("Shape")) const
    {
        if (!game())
            return nullptr;
        for (const QVariant &e : asList(game()->property("entityList"))) {
            QObject *o = e.value<QObject *>();
            if (o && o->property("name").toString() == name && o->property("kind").toString() == kind)
                return o;
        }
        return nullptr;
    }

    // The sounds playing now from a place in the world, whose file ends `file`: QQ's placed voices, Qt's SpatialSound,
    // which alone have a reach (`distanceCutoff`). A sound heard everywhere has none.
    QList<QObject *> placedVoices(const QString &file) const
    {
        QList<QObject *> found;
        QObject *voices = scene() ? scene()->property("_voices").value<QObject *>() : nullptr;
        if (!voices)
            return found;
        for (const QVariant &v : asList(voices->property("active"))) {
            QObject *voice = v.value<QObject *>();
            if (voice && voice->property("distanceCutoff").isValid()
                && voice->property("source").toUrl().toString().endsWith(file))
                found << voice;
        }
        return found;
    }

    // A sound of `file` is playing from within `within` of `where`, in the scene.
    bool heardNear(const QString &file, const QVector3D &where, double within) const
    {
        for (QObject *voice : placedVoices(file))
            if ((voice->property("position").value<QVector3D>() - where).length() <= within)
                return true;
        return false;
    }

    // A screen button, clicked where it is drawn, as a person clicks it.
    bool click(const QString &name)
    {
        QQuickItem *drawn = find(game(), [&](QQuickItem *i) { return i->objectName() == name && i->isVisible(); });
        if (!drawn)
            return false;
        const QPointF at = drawn->mapToScene(QPointF(drawn->width() / 2, drawn->height() / 2));
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, at.toPoint());
        return true;
    }

    // A screen button clicked once its screen has faded in: waited for, then clicked once. (QTRY_VERIFY would click
    // again as it checks what it waited for.)
    bool clickShown(const QString &name)
    {
        QElapsedTimer waited;
        waited.start();
        while (!find(game(), [&](QQuickItem *i) { return i->objectName() == name && i->isVisible(); })
               && waited.elapsed() < 5000)
            QTest::qWait(20);
        return click(name);
    }

    // A game's card pressed from the keyboard, as a person tabbing to it presses it: Space on the card with focus. (Not
    // by the mouse: on Windows the real pointer, wherever it is, can take a synthetic one's hover and press away.) Then
    // its title, and the phase its rules start in (none for an arena, whose server keeps its rules).
    void open(const QString &name, const QString &kind, const QUrl &file, const QString &title,
              const QString &phase = QStringLiteral("title"))
    {
        QQuickItem *card = find(window->contentItem(), [&](QQuickItem *i) {
            return i->property("here").toBool() && i->property("name").toString() == name;
        });
        QVERIFY2(card, qPrintable(name + QStringLiteral("'s card is not one that plays here")));
        card->forceActiveFocus(Qt::TabFocusReason);
        QTRY_VERIFY(card->hasActiveFocus());
        QTest::keyClick(window, Qt::Key_Space);

        QTRY_COMPARE(window->property("playing").toString(), kind);
        QVERIFY(!home()->isVisible());
        QVERIFY(game());
        // A game that does not start says why: its problem, and what QML warned of.
        QTRY_VERIFY2_WITH_TIMEOUT(game()->property("settled").toBool(),
                                  qPrintable(game()->property("problem").toString() + '\n' + warnings.join('\n')), 15000);
        QCOMPARE(game()->property("problem").toString(), QString());
        QCOMPARE(QUrl(game()->property("file").toUrl()), file);
        QTRY_VERIFY(logic());
        if (!phase.isEmpty())
            QTRY_COMPARE(state().value("phase").toString(), phase);
        QTRY_COMPARE(screenText(QStringLiteral("Title")), title);
    }

    // The screens the scene shows now.
    QStringList screens() const
    {
        QStringList out;
        if (scene())
            for (const QVariant &v : asList(scene()->property("shown")))
                out << v.toString();
        return out;
    }

    // Signed in to the stand-in ARQADE through the stand-in launcher, as the app signs in with the launcher's QOR ID.
    void signIn()
    {
        if (server->state() == ArqadeServer::State::SignedIn)
            return;
        link->connectTo(launcher->name(), "t");
        QVERIFY(launcher->accept());
        launcher->read();
        launcher->welcome();
        const QJsonObject waiting = launcher->read();
        server->setLauncherSignedIn(true);
        QTRY_COMPARE(link->waiting(), QStringList{QStringLiteral("arqade.signin")});
        launcher->command(waiting, QJsonValue());
        const QJsonObject asked = launcher->read();
        launcher->result(asked, {{"redirect", arcade->callback().toString()}});
        launcher->read();  // the next arqade.next, left waiting
        QTRY_COMPARE(server->state(), ArqadeServer::State::SignedIn);
    }

    // The arena's match as it stands in the game: arena.mjs's reading of the server's snapshot.
    QVariantMap match() const { return asMap(state().value("match")); }
    int cursor() const { return state().value("cursor").toInt(); }

    // Where the board's cell `i` is drawn, in the window: what the pointer aims at.
    QPoint cellAt(int i) const
    {
        QObject *cell = asList(logic()->property("cells")).value(i).value<QObject *>();
        if (!cell)
            return {};
        QVector3D at;
        QMetaObject::invokeMethod(game(), "mapFrom3DScene", Q_RETURN_ARG(QVector3D, at),
                                  Q_ARG(QVector3D, cell->property("scenePosition").value<QVector3D>()));
        return game()->mapToScene(QPointF(at.x(), at.y())).toPoint();
    }

    // The disc or pulse on cell `i`, or null.
    QObject *discOn(int i) const { return asList(logic()->property("discs")).value(i).value<QObject *>(); }

    // The pad's button held for a moment.
    void pad(const QString &button)
    {
        input->gamepadButtons({button});
        QTest::qWait(40);
        input->gamepadButtons({});
    }

    // The run ARQADE kept of the game being played, closed on what has been played: QQ plays it again, with no window
    // and no physics world, to the outcome it reached. With ARQADE_RUN set, it is written there as <as>-run.json.
    void keptRunReplays(const QUrl &file, const QString &as, QVariantMap *outcome)
    {
        QVERIFY2(game(), "no game is being played");
        QObject *view = game()->parent();
        while (view && !view->property("record").isValid())
            view = view->parent();
        QVERIFY(view);
        auto *record = qobject_cast<qq::RunRecord *>(view->property("record").value<QObject *>());
        QVERIFY(record);
        QVERIFY(QMetaObject::invokeMethod(scene(), "finishRecord"));
        QVERIFY(record->seed() != 0);
        *outcome = asMap(record->outcome().value(behaviour));

        const QByteArray json = record->toJson();
        if (const QByteArray to = qgetenv("ARQADE_RUN"); !to.isEmpty()) {
            QDir().mkpath(QString::fromLocal8Bit(to));
            QFile out(QDir(QString::fromLocal8Bit(to)).filePath(as + QStringLiteral("-run.json")));
            QVERIFY(out.open(QIODevice::WriteOnly));
            out.write(json);
        }

        qq::RunRecord again;
        QVERIFY(again.fromJson(json));
        QQmlEngine headless;
        const QVariantMap result = qq::replayRun(&headless, file, &again);
        QVERIFY2(result.value("problem").toString().isEmpty(), qPrintable(result.value("problem").toString()));
        QVERIFY2(result.value("matches").toBool(),
                 QJsonDocument(QJsonObject::fromVariantMap(result)).toJson(QJsonDocument::Compact).constData());
    }

    // A run kept beside the game's rules, which Node replays with logic/rules.mjs: QQ's own replay reaches what it says.
    void fixtureReplays(const QString &as, const QUrl &file)
    {
        QFile kept(QStringLiteral(ARQADE_RUNS "/") + as + QStringLiteral("-run.json"));
        QVERIFY2(kept.open(QIODevice::ReadOnly), qPrintable(kept.fileName()));
        qq::RunRecord run;
        QString why;
        QVERIFY2(run.fromJson(kept.readAll(), &why), qPrintable(why));
        QQmlEngine headless;
        const QVariantMap result = qq::replayRun(&headless, file, &run);
        QVERIFY2(result.value("problem").toString().isEmpty(), qPrintable(result.value("problem").toString()));
        QVERIFY2(result.value("matches").toBool(),
                 QJsonDocument(QJsonObject::fromVariantMap(result)).toJson(QJsonDocument::Compact).constData());
    }

    // Space, or the pad's A, held for a moment: the scene takes it at its next tick.
    void lock(bool pad = false)
    {
        if (pad)
            input->gamepadButtons({QStringLiteral("A")});
        else
            QTest::keyPress(window, Qt::Key_Space);
        QTest::qWait(40);
        if (pad)
            input->gamepadButtons({});
        else
            QTest::keyRelease(window, Qt::Key_Space);
    }

    // Void Runner: one lane over, by the keyboard (A or D, or every other time the arrows), the pad's d-pad, or the Left
    // and Right buttons on the screen.
    // Rift Survivor: which of the eight ways, or none, keeps the ship furthest from what threatens it (enemies, rifts not
    // yet spent, the arena's edges) and nearest the closest fragment, as tests/rift-survivor.test.mjs's pilot chooses.
    static std::pair<int, int> riftWay(const QVariantMap &now)
    {
        static const int dirs[9][2] = {{0, 0}, {1, 0}, {-1, 0}, {0, 1}, {0, -1}, {1, 1}, {1, -1}, {-1, 1}, {-1, -1}};
        const double gx = now.value("x").toDouble(), gy = now.value("y").toDouble();
        const QVariantList enemies = asList(now.value("enemies")), pulses = asList(now.value("pulses")),
                           fragments = asList(now.value("fragments"));
        int best = 0;
        double bestScore = -1e300;
        for (int i = 0; i < 9; ++i) {
            const double k = dirs[i][0] && dirs[i][1] ? 0.7071067811865476 : 1;
            const double x = gx + dirs[i][0] * k * 30, y = gy + dirs[i][1] * k * 30;
            double s = 0;
            for (const QVariant &v : enemies) {
                const QVariantMap e = asMap(v);
                const double d = std::pow(e.value("x").toDouble() - x, 2) + std::pow(e.value("y").toDouble() - y, 2);
                s -= 40000 / (d + 1);
            }
            for (const QVariant &v : pulses) {
                const QVariantMap p = asMap(v);
                if (p.value("age").toInt() < 102
                    && std::pow(p.value("x").toDouble() - x, 2) + std::pow(p.value("y").toDouble() - y, 2) < 120 * 120)
                    s -= 50;
            }
            double nearest = -1;
            for (const QVariant &v : fragments) {
                const QVariantMap f = asMap(v);
                const double d = std::pow(f.value("x").toDouble() - x, 2) + std::pow(f.value("y").toDouble() - y, 2);
                if (nearest < 0 || d < nearest)
                    nearest = d;
            }
            if (nearest >= 0)
                s += 8000 / (nearest + 400);
            s -= std::pow((x - 500) / 500, 4) + std::pow((y - 310) / 310, 4);
            if (s > bestScore) {
                bestScore = s;
                best = i;
            }
        }
        return {dirs[best][0], dirs[best][1]};
    }

    // Rift Survivor: W, A, S and D held for a way, and let go for any other.
    QSet<int> steering;
    void steer(std::pair<int, int> way)
    {
        QSet<int> want;
        if (way.first > 0)
            want << Qt::Key_D;
        if (way.first < 0)
            want << Qt::Key_A;
        if (way.second > 0)
            want << Qt::Key_S;
        if (way.second < 0)
            want << Qt::Key_W;
        for (int key : steering)
            if (!want.contains(key))
                QTest::keyRelease(window, Qt::Key(key));
        for (int key : want)
            if (!steering.contains(key))
                QTest::keyPress(window, Qt::Key(key));
        steering = want;
    }

    // Synapse: node `n` (0 to 3) pressed by the keys 1 to 4 (way 0), the pad's X, Y, B and A (1), or its button (2).
    void node(int n, int way)
    {
        static const QStringList pad = {QStringLiteral("X"), QStringLiteral("Y"), QStringLiteral("B"), QStringLiteral("A")};
        static const QStringList buttons = {QStringLiteral("Origin"), QStringLiteral("Echo"), QStringLiteral("Pulse"),
                                            QStringLiteral("Aether")};
        if (way == 0) {
            QTest::keyClick(window, Qt::Key(Qt::Key_1 + n));
        } else if (way == 1) {
            input->gamepadButtons({pad[n]});
            QTest::qWait(20);
            input->gamepadButtons({});
        } else {
            click(buttons[n]);
        }
    }

    enum class By { Keys, Arrows, Pad, Touch };
    void shift(int by, By how)
    {
        const bool right = by > 0;
        switch (how) {
        case By::Keys:
            QTest::keyClick(window, right ? Qt::Key_D : Qt::Key_A);
            break;
        case By::Arrows:
            QTest::keyClick(window, right ? Qt::Key_Right : Qt::Key_Left);
            break;
        case By::Pad:
            input->gamepadButtons({right ? QStringLiteral("Right") : QStringLiteral("Left")});
            QTest::qWait(20);
            input->gamepadButtons({});
            break;
        case By::Touch:
            click(right ? QStringLiteral("Right") : QStringLiteral("Left"));
            break;
        }
    }

    // Void Runner: the lane a pilot wants now. Toward the next gem to arrive, away from a hazard about to; or, steering
    // into trouble, toward the next hazard.
    int wanted(const QVariantMap &now, bool intoHazards) const
    {
        const int lane = now.value("lane").toInt();
        auto soonest = [&](int l) {
            QVariantMap next;
            for (const QVariant &v : asList(now.value("coming"))) {
                const QVariantMap thing = asMap(v);
                if (thing.value("lane").toInt() == l && thing.value("z").toDouble() > 0.6
                    && (next.isEmpty() || thing.value("z").toDouble() > next.value("z").toDouble()))
                    next = thing;
            }
            return next;
        };
        auto worth = [&](int l) {
            const QVariantMap next = soonest(l);
            if (next.isEmpty())
                return 0;
            const bool gem = next.value("gem").toBool();
            return intoHazards ? (gem ? -1 : 1) : (gem ? 1 : -10);
        };
        int best = lane;
        for (int l = 0; l < 3; ++l)
            if (worth(l) > worth(best) || (worth(l) == worth(best) && std::abs(l - lane) < std::abs(best - lane)))
                best = l;
        // Two lanes over crosses the middle one, and the two presses can reach consecutive ticks: as a person would,
        // wait for a hazard about to reach the middle lane to pass rather than cross it as it arrives.
        if (!intoHazards && std::abs(best - lane) == 2) {
            const QVariantMap middle = soonest(1);
            if (!middle.isEmpty() && !middle.value("gem").toBool() && middle.value("z").toDouble() > 0.85)
                return lane;
        }
        return best;
    }

private slots:
    void initTestCase()
    {
        arqade::loadFonts();
        link = new ArqadeLink(this);
        // ARQADE's server and the launcher stood in for here: nothing here reaches the network. Until the arenas' tests
        // sign in, nobody is signed in.
        arcade = new Arcade;
        launcher = new fakes::Launcher;
        server = new ArqadeServer(link, arcade->base(), this);
        engine = new QQmlApplicationEngine(this);
        connect(engine, &QQmlEngine::warnings, this, [this](const QList<QQmlError> &list) {
            for (const QQmlError &e : list)
                warnings << e.toString();
        });
        engine->setInitialProperties({{"link", QVariant::fromValue(link)},
                                      {"server", QVariant::fromValue(server)},
                                      {"launched", false}});
        engine->loadFromModule("ARQADE", "Main");
        QCOMPARE(engine->rootObjects().size(), 1);
        window = qobject_cast<QQuickWindow *>(engine->rootObjects().first());
        QVERIFY(window);
        window->resize(1440, 900);
        QVERIFY(QTest::qWaitForWindowExposed(window));
        // The pad is this test's alone: a real one on this PC moves nothing.
        input = engine->singletonInstance<qq::Input *>("QQ", "Input");
        QVERIFY(input);
        input->setReadsHardware(false);
    }

    // The four games and the two arenas all play here.
    void every_card_plays_its_game_here()
    {
        for (const char *name : {"Orbital", "Void Runner", "Synapse", "Rift Survivor", "Flux Four", "Rift Reversi"})
            QVERIFY2(find(window->contentItem(), [&](QQuickItem *i) {
                         return i->property("name").toString() == QLatin1String(name) && i->property("here").toBool();
                     }),
                     name);
    }

    // A test that failed says what QML warned of on the way, and where the game stood: its screens and its state.
    void cleanup()
    {
        if (!QTest::currentTestFailed())
            return;
        if (!warnings.isEmpty())
            qWarning().noquote() << "QML warned:" << warnings.join('\n');
        if (logic())
            qWarning().noquote() << "Screens:" << screens().join(", ") << "\nState:"
                                 << QJsonDocument(QJsonObject::fromVariantMap(state())).toJson(QJsonDocument::Compact);
    }

    void cleanupTestCase()
    {
        delete launcher;
        delete arcade;
    }

    void orbitals_card_opens_the_game_in_arqades_window()
    {
        behaviour = QStringLiteral("Orbital");
        open(QStringLiteral("Orbital"), QStringLiteral("orbital"), kOrbital, QStringLiteral("ORBITAL"));
        if (QTest::currentTestFailed())
            return;
        QTest::qWait(500);
        save(window, "orbital-title.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void the_launchers_reduced_motion_reaches_the_game()
    {
        QObject *look = engine->singletonInstance<QObject *>("QQ", "Look");
        QVERIFY(look);
        emit link->prefsChanged({{"reduced_motion", true}});
        QVERIFY(look->property("reducedMotion").toBool());
        emit link->prefsChanged({{"reduced_motion", false}});
        QVERIFY(!look->property("reducedMotion").toBool());
    }

    void play_starts_the_orbit_and_a_lock_outside_the_arc_misses()
    {
        QVERIFY(click(QStringLiteral("Play")));
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("playing"));
        QTRY_VERIFY(state().value("angle").toDouble() > 0);
        QTRY_COMPARE(screenText(QStringLiteral("Locks")), QStringLiteral("LOCKS 0 / 10"));

        // Well clear of the arc, a lock misses.
        QTRY_VERIFY_WITH_TIMEOUT(apart(state().value("angle").toDouble(), state().value("target").toDouble()) > kArc + 0.6,
                                 10000);
        lock();
        QTRY_COMPARE(state().value("misses").toInt(), 1);
        QCOMPARE(state().value("locks").toInt(), 0);
        QTRY_COMPARE(screenText(QStringLiteral("Misses")), QStringLiteral("MISSES 1 / 3"));
        QTRY_COMPARE(screenText(QStringLiteral("Message")), QStringLiteral("SIGNAL MISSED"));
        save(window, "orbital-missed.png");
    }

    void p_or_the_pads_start_pauses_the_world_and_resume_carries_on()
    {
        QTest::keyClick(window, Qt::Key_P);
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("paused"));
        QVERIFY(scene()->property("paused").toBool());
        QTRY_COMPARE(screenText(QStringLiteral("Paused")), QStringLiteral("PAUSED"));
        const double held = state().value("angle").toDouble();
        QTest::qWait(400);
        QCOMPARE(state().value("angle").toDouble(), held);
        save(window, "orbital-paused.png");

        // Space while paused is no lock.
        lock();
        QTest::qWait(100);
        QCOMPARE(state().value("misses").toInt() + state().value("locks").toInt(), 1);

        QVERIFY(click(QStringLiteral("Resume")));
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("playing"));
        QVERIFY(!scene()->property("paused").toBool());
        QTRY_VERIFY(state().value("angle").toDouble() != held);

        // The pad's Start pauses, and again carries on.
        for (const char *phase : {"paused", "playing"}) {
            input->gamepadButtons({QStringLiteral("Start")});
            QTest::qWait(40);
            input->gamepadButtons({});
            QTRY_COMPARE(state().value("phase").toString(), QString::fromLatin1(phase));
        }
    }

    void ten_locks_in_the_arc_by_space_and_the_pad_complete_the_orbit()
    {
        for (int locks = 1; locks <= 10; ++locks) {
            // Into the arc, by a margin a few ticks deep, and locked: by Space, and every other time by the pad's A.
            QTRY_VERIFY_WITH_TIMEOUT(apart(state().value("angle").toDouble(), state().value("target").toDouble())
                                         < kArc * 0.55,
                                     10000);
            lock(locks % 2 == 0);
            QTRY_COMPARE(state().value("locks").toInt(), locks);
            if (locks == 1) {
                QTRY_COMPARE(screenText(QStringLiteral("Message")), QStringLiteral("ORBIT LOCKED"));
                save(window, "orbital-locked.png");
                // The lock is heard from the satellite.
                QObject *satellite = entityNamed(QStringLiteral("Satellite"));
                QVERIFY(satellite);
                QTRY_VERIFY(heardNear(QStringLiteral("lock.wav"), satellite->property("scenePosition").value<QVector3D>(), 1.0));
            }
        }
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("over"));
        QCOMPARE(state().value("misses").toInt(), 1);
        QTRY_COMPARE(screenText(QStringLiteral("Result")), QStringLiteral("ORBIT COMPLETE"));
        QCOMPARE(screenText(QStringLiteral("Score")), QStringLiteral("SCORE 2,000"));
        QTest::qWait(400);
        save(window, "orbital-complete.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // What ARQADE kept is the run: QQ plays it again, with no window and no physics world, to the outcome it reached.
    void the_run_arqade_kept_replays_to_its_outcome()
    {
        QVariantMap outcome;
        keptRunReplays(kOrbital, QStringLiteral("orbital"), &outcome);
        if (QTest::currentTestFailed())
            return;
        QCOMPARE(outcome.value("locks").toInt(), 10);
        QCOMPARE(outcome.value("misses").toInt(), 1);
        QCOMPARE(outcome.value("score").toInt(), 2000);
        QCOMPARE(outcome.value("over").toBool(), true);
    }

    void the_kept_run_replays_in_qq_as_the_rules_replay_it()
    {
        fixtureReplays(QStringLiteral("orbital"), kOrbital);
    }

    void esc_returns_to_the_home()
    {
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_COMPARE(window->property("playing").toString(), QString());
        QTRY_VERIFY(!game());
        QVERIFY(home()->isVisible());
        QTest::qWait(300);
        save(window, "home-again.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // Back, on the bar, as well.
    void back_on_the_bar_returns_too()
    {
        window->setProperty("playing", QStringLiteral("orbital"));
        QTRY_VERIFY(game());
        QTRY_VERIFY_WITH_TIMEOUT(game()->property("settled").toBool(), 15000);
        QTRY_VERIFY(logic());
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("title"));
        QQuickItem *back = window->findChild<QQuickItem *>(QStringLiteral("back"));
        QVERIFY(back);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier,
                          back->mapToScene(QPointF(back->width() / 2, back->height() / 2)).toPoint());
        QTRY_COMPARE(window->property("playing").toString(), QString());
        QTRY_VERIFY(!game());
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void void_runners_card_opens_it_and_play_starts_the_run()
    {
        behaviour = QStringLiteral("Void Runner");
        open(QStringLiteral("Void Runner"), QStringLiteral("void"), kVoidRunner, QStringLiteral("VOID RUNNER"));
        if (QTest::currentTestFailed())
            return;
        QTest::qWait(400);
        save(window, "void-runner-title.png");
        QVERIFY(click(QStringLiteral("Play")));
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("playing"));
        QTRY_VERIFY(state().value("steps").toInt() > 0);
        QTRY_COMPARE(screenText(QStringLiteral("Hull")), QStringLiteral("HULL 3 / 3"));
        QCOMPARE(state().value("lane").toInt(), 1);
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // Steered for ten seconds or more as a person would: away from each hazard about to arrive, toward each gem, until it
    // has caught one (a seed may throw nothing but hazards for a while). Each change of lane is made a different way from
    // the one before, by the keys, the arrows, the pad and the screen's buttons in turn, until each has been used.
    void a_pilot_dodges_hazards_and_catches_gems_by_keys_arrows_pad_and_touch()
    {
        const int hull = state().value("hull").toInt();
        const QList<By> ways = {By::Keys, By::Arrows, By::Pad, By::Touch};
        QList<int> used(ways.size(), 0);
        int moves = 0;
        bool seenThrown = false;
        int frame = 0;
        while ((state().value("steps").toInt() < 600 || used.contains(0) || state().value("gems").toInt() == 0)
               && state().value("phase").toString() == QLatin1String("playing")) {
            const QVariantMap now = state();
            seenThrown = seenThrown || !asList(now.value("coming")).isEmpty();
            const int to = wanted(now, false);
            const int lane = now.value("lane").toInt();
            if (to != lane) {
                // Two lanes over at once if need be.
                const int way = moves++ % ways.size();
                for (int i = 0; i < std::abs(to - lane); ++i)
                    shift(to > lane ? +1 : -1, ways[way]);
                QTRY_COMPARE_WITH_TIMEOUT(state().value("lane").toInt(), to, 500);
                used[way] += 1;
                // A change of lane is heard from the ship, not everywhere.
                if (moves == 1)
                    QTRY_VERIFY(!placedVoices(QStringLiteral("shift.wav")).isEmpty());
            }
            if (++frame == 300)
                save(window, "void-runner-running.png");
            QTest::qWait(4);
        }
        QVERIFY(seenThrown);
        QVERIFY2(!used.contains(0), "a way of changing lane was never needed before the run ended");
        const QVariantMap now = state();
        QCOMPARE(now.value("phase").toString(), QStringLiteral("playing"));
        QCOMPARE(now.value("hull").toInt(), hull);
        QVERIFY2(now.value("gems").toInt() > 0, "the run ended before a gem was caught");
        QTRY_VERIFY(screenText(QStringLiteral("Score")).startsWith(QStringLiteral("SCORE ")));
        save(window, "void-runner-dodging.png");
    }

    // Then steered into what is thrown: three hazards breach the hull, and the run ends.
    void three_hazards_met_end_the_run()
    {
        while (state().value("phase").toString() == QLatin1String("playing")) {
            const QVariantMap now = state();
            const int to = wanted(now, true);
            const int lane = now.value("lane").toInt();
            for (int i = 0; i < std::abs(to - lane); ++i)
                shift(to > lane ? +1 : -1, By::Keys);
            if (now.value("hull").toInt() == 2 && screenText(QStringLiteral("Message")) == QLatin1String("HULL BREACHED"))
                save(window, "void-runner-breached.png");
            QTest::qWait(4);
            QVERIFY2(state().value("steps").toInt() < 2700, "the run lasted to the end steering into hazards");
        }
        const QVariantMap end = state();
        QCOMPARE(end.value("phase").toString(), QStringLiteral("over"));
        QCOMPARE(end.value("hull").toInt(), 0);
        QTRY_COMPARE(screenText(QStringLiteral("Result")), QStringLiteral("HULL BREACHED"));
        QVERIFY(screenText(QStringLiteral("Final")).startsWith(QStringLiteral("SCORE ")));
        QTest::qWait(400);
        save(window, "void-runner-over.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void void_runners_kept_run_replays_to_its_outcome()
    {
        QVariantMap outcome;
        keptRunReplays(kVoidRunner, QStringLiteral("void-runner"), &outcome);
        if (QTest::currentTestFailed())
            return;
        QCOMPARE(outcome.value("hull").toInt(), 0);
        QCOMPARE(outcome.value("survived").toBool(), false);
        QCOMPARE(outcome.value("over").toBool(), true);
        QVERIFY(outcome.value("gems").toInt() > 0);
        QCOMPARE(QStringLiteral("SCORE ") + QLocale(QLocale::English, QLocale::UnitedKingdom).toString(outcome.value("score").toInt()),
                 screenText(QStringLiteral("Final")));
    }

    void void_runners_kept_run_replays_in_qq_as_its_rules_replay_it()
    {
        fixtureReplays(QStringLiteral("void-runner"), kVoidRunner);
    }

    void esc_leaves_void_runner_too()
    {
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_COMPARE(window->property("playing").toString(), QString());
        QTRY_VERIFY(!game());
        QVERIFY(home()->isVisible());
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void synapses_card_opens_it_and_the_signal_shows_itself()
    {
        behaviour = QStringLiteral("Synapse");
        open(QStringLiteral("Synapse"), QStringLiteral("synapse"), kSynapse, QStringLiteral("SYNAPSE"));
        if (QTest::currentTestFailed())
            return;
        QTest::qWait(400);
        save(window, "synapse-title.png");
        QVERIFY(click(QStringLiteral("Play")));
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("playing"));
        QCOMPARE(state().value("stage").toString(), QStringLiteral("watch"));
        QTRY_COMPARE(screenText(QStringLiteral("Stage")), QStringLiteral("OBSERVE THE SIGNAL"));
        // A node pressed while the signal shows is no answer.
        node(0, 0);
        QTest::qWait(60);
        QCOMPARE(state().value("cursor").toInt(), 0);
        QCOMPARE(state().value("phase").toString(), QStringLiteral("playing"));
        // Its one node lit, part way through its lighting.
        QTRY_VERIFY(state().value("shown").toInt() >= 40);
        save(window, "synapse-signal.png");
        QTRY_COMPARE_WITH_TIMEOUT(state().value("stage").toString(), QStringLiteral("repeat"), 5000);
        QTRY_COMPARE(screenText(QStringLiteral("Stage")), QStringLiteral("REPEAT 0 / 1"));
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // Three signals repeated, node by node, each node pressed a different way from the one before: the keys 1 to 4, the
    // pad's X, Y, B and A, and the buttons along the foot of the screen.
    void three_signals_repeated_by_keys_pad_and_the_buttons()
    {
        int way = 0;
        for (int round = 1; round <= 3; ++round) {
            QTRY_COMPARE_WITH_TIMEOUT(state().value("stage").toString(), QStringLiteral("repeat"), 10000);
            const QVariantList signal = asList(state().value("signal"));
            QCOMPARE(signal.size(), round);
            for (int i = 0; i < signal.size(); ++i) {
                node(signal[i].toInt(), way++ % 3);
                // Each node's note is heard from the node.
                const int n = signal[i].toInt();
                // The nodes are made by the logic as play starts: its own list has them.
                const QVariantList nodes = asList(logic()->property("nodes"));
                QCOMPARE(nodes.size(), 4);
                QObject *pressed = nodes[n].value<QObject *>();
                QVERIFY(pressed);
                QTRY_VERIFY(heardNear(QStringLiteral("node%1.wav").arg(n + 1),
                                      pressed->property("scenePosition").value<QVector3D>(), 0.01));
                if (i + 1 < signal.size())
                    QTRY_COMPARE(state().value("cursor").toInt(), i + 1);
                else
                    QTRY_COMPARE(asList(state().value("signal")).size(), round + 1);
            }
            if (round == 3)
                save(window, "synapse-repeated.png");
        }
        QCOMPARE(state().value("phase").toString(), QStringLiteral("playing"));
        QTRY_COMPARE(screenText(QStringLiteral("Length")), QStringLiteral("SIGNAL 4 / 8"));
    }

    void a_node_out_of_turn_loses_the_signal()
    {
        QTRY_COMPARE_WITH_TIMEOUT(state().value("stage").toString(), QStringLiteral("repeat"), 10000);
        const int right = asList(state().value("signal")).first().toInt();
        node((right + 1) % 4, 0);
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("over"));
        QTRY_COMPARE(screenText(QStringLiteral("Result")), QStringLiteral("SIGNAL LOST"));
        QCOMPARE(screenText(QStringLiteral("Final")), QStringLiteral("SCORE 450"));
        QTest::qWait(400);
        save(window, "synapse-lost.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void synapses_kept_run_replays_to_its_outcome()
    {
        QVariantMap outcome;
        keptRunReplays(kSynapse, QStringLiteral("synapse"), &outcome);
        if (QTest::currentTestFailed())
            return;
        QCOMPARE(outcome.value("score").toInt(), 450);
        QCOMPARE(outcome.value("length").toInt(), 4);
        QCOMPARE(outcome.value("mastered").toBool(), false);
        QCOMPARE(outcome.value("over").toBool(), true);
    }

    void synapses_kept_run_replays_in_qq_as_its_rules_replay_it()
    {
        fixtureReplays(QStringLiteral("synapse"), kSynapse);
    }

    void esc_leaves_synapse_too()
    {
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_COMPARE(window->property("playing").toString(), QString());
        QTRY_VERIFY(!game());
        QVERIFY(home()->isVisible());
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void rift_survivors_card_opens_it_and_play_starts_the_run()
    {
        behaviour = QStringLiteral("Rift Survivor");
        open(QStringLiteral("Rift Survivor"), QStringLiteral("rift"), kRiftSurvivor, QStringLiteral("RIFT SURVIVOR"));
        if (QTest::currentTestFailed())
            return;
        QTest::qWait(400);
        save(window, "rift-title.png");
        QVERIFY(click(QStringLiteral("Play")));
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("playing"));
        QTRY_COMPARE(screenText(QStringLiteral("Hull")), QStringLiteral("HULL 4 / 4"));
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // Held for a moment each way there is, the ship moves that way: D on the keys, the left arrow, the pad's d-pad down,
    // and the screen's Up button under the pointer.
    void the_ship_moves_by_keys_arrows_pad_and_the_screens_buttons()
    {
        struct Way { const char *name; double dx, dy; std::function<void(bool)> hold; };
        const QList<Way> ways = {
            {"D", 1, 0, [this](bool down) { down ? QTest::keyPress(window, Qt::Key_D) : QTest::keyRelease(window, Qt::Key_D); }},
            {"the left arrow", -1, 0,
             [this](bool down) { down ? QTest::keyPress(window, Qt::Key_Left) : QTest::keyRelease(window, Qt::Key_Left); }},
            {"the d-pad's down", 0, 1,
             [this](bool down) { input->gamepadButtons(down ? QStringList{QStringLiteral("Down")} : QStringList{}); }},
            {"the Up button", 0, -1, [this](bool down) {
                 QQuickItem *drawn = find(game(), [](QQuickItem *i) { return i->objectName() == QLatin1String("Up"); });
                 const QPoint at = drawn->mapToScene(QPointF(drawn->width() / 2, drawn->height() / 2)).toPoint();
                 down ? QTest::mousePress(window, Qt::LeftButton, Qt::NoModifier, at)
                      : QTest::mouseRelease(window, Qt::LeftButton, Qt::NoModifier, at);
             }},
        };
        for (const Way &way : ways) {
            const double x = state().value("x").toDouble(), y = state().value("y").toDouble();
            way.hold(true);
            QTest::qWait(150);
            way.hold(false);
            QTest::qWait(30);
            const double mx = state().value("x").toDouble() - x, my = state().value("y").toDouble() - y;
            QVERIFY2(mx * way.dx > 20 || my * way.dy > 20,
                     qPrintable(QStringLiteral("%1 moved the ship %2, %3").arg(QLatin1String(way.name)).arg(mx).arg(my)));
            QVERIFY2(std::abs(mx * way.dy) + std::abs(my * way.dx) < 1, way.name);
        }
        QVERIFY(!asMap(state().value("held")).value("up").toBool());
    }

    // Steered as a person would, by the keys, away from enemies and armed rifts and toward fragments, until ten kills
    // evolve the signal; the world holds while a perk is chosen, here Fracture by its button.
    void a_pilot_keeps_away_until_the_signal_evolves_and_a_perk_is_chosen()
    {
        QVERIFY(logic());
        int frame = 0;
        bool killHeard = false;
        while (state().value("phase").toString() == QLatin1String("playing") && state().value("steps").toInt() < 3600) {
            steer(riftWay(state()));
            if (++frame == 400)
                save(window, "rift-running.png");
            // A kill is heard from where the enemy was.
            killHeard = killHeard || !placedVoices(QStringLiteral("kill.wav")).isEmpty();
            QTest::qWait(4);
        }
        QVERIFY2(killHeard, "no kill was heard from a place in the arena");
        steer({0, 0});
        QCOMPARE(state().value("phase").toString(), QStringLiteral("choosing"));
        QVERIFY(state().value("hull").toInt() > 0);
        QVERIFY(state().value("kills").toInt() >= 10);
        QTRY_COMPARE(screenText(QStringLiteral("Evolved")), QStringLiteral("SIGNAL EVOLVED  ·  LEVEL 2"));
        const int steps = state().value("steps").toInt();
        QTest::qWait(300);
        QCOMPARE(state().value("steps").toInt(), steps);
        QVERIFY(scene()->property("paused").toBool());
        save(window, "rift-evolved.png");
        QVERIFY(click(QStringLiteral("Fracture")));
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("playing"));
        QCOMPARE(state().value("spread").toInt(), 1);
        QTRY_VERIFY(state().value("steps").toInt() > steps);
    }

    // Then left standing still: enemies and the rifts that open under it breach the hull four times, a perk chosen by its
    // key each time the signal evolves meanwhile.
    void standing_still_the_hull_breaks()
    {
        QVERIFY(logic());
        QElapsedTimer waited;
        waited.start();
        while (state().value("phase").toString() != QLatin1String("over")) {
            QVERIFY2(waited.elapsed() < 120000, "two minutes passed and the hull held");
            if (state().value("phase").toString() == QLatin1String("choosing"))
                QTest::keyClick(window, Qt::Key_1);
            if (state().value("hull").toInt() == 3 && state().value("hold").toInt() > 80)
                save(window, "rift-breached.png");
            QTest::qWait(10);
            QVERIFY2(state().value("steps").toInt() < Rules_RUN_STEPS, "the run lasted to the end standing still");
        }
        QCOMPARE(state().value("hull").toInt(), 0);
        QTRY_COMPARE(screenText(QStringLiteral("Result")), QStringLiteral("HULL BROKEN"));
        QVERIFY(screenText(QStringLiteral("Final")).startsWith(QStringLiteral("SCORE ")));
        QTest::qWait(400);
        save(window, "rift-over.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void rift_survivors_kept_run_replays_to_its_outcome()
    {
        QVariantMap outcome;
        keptRunReplays(kRiftSurvivor, QStringLiteral("rift-survivor"), &outcome);
        if (QTest::currentTestFailed())
            return;
        QCOMPARE(outcome.value("hull").toInt(), 0);
        QCOMPARE(outcome.value("survived").toBool(), false);
        QCOMPARE(outcome.value("over").toBool(), true);
        QVERIFY(outcome.value("level").toInt() >= 2);
        QCOMPARE(QStringLiteral("SCORE ") + QLocale(QLocale::English, QLocale::UnitedKingdom).toString(outcome.value("score").toInt()),
                 screenText(QStringLiteral("Final")));
    }

    void rift_survivors_kept_run_replays_in_qq_as_its_rules_replay_it()
    {
        fixtureReplays(QStringLiteral("rift-survivor"), kRiftSurvivor);
    }

    void esc_leaves_rift_survivor_too()
    {
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_COMPARE(window->property("playing").toString(), QString());
        QTRY_VERIFY(!game());
        QVERIFY(home()->isVisible());
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // Signed out, an arena says so, and offers nothing to do: no match to open or join.
    void an_arena_signed_out_says_so_and_offers_nothing()
    {
        behaviour = QStringLiteral("Arena");
        QCOMPARE_NE(server->state(), ArqadeServer::State::SignedIn);
        open(QStringLiteral("Flux Four"), QStringLiteral("flux"), kFluxFour, QStringLiteral("FLUX FOUR"), QString());
        if (QTest::currentTestFailed())
            return;
        // In the server's words, as the rankings say it: here, started by hand, there is no launcher to sign in with.
        QVERIFY(!server->message().isEmpty());
        QTRY_COMPARE(screenText(QStringLiteral("Status")), server->message());
        QVERIFY(screens().contains(QStringLiteral("lobby")));
        QVERIFY(!screens().contains(QStringLiteral("lobby-create")));
        QVERIFY(!screens().contains(QStringLiteral("lobby-join")));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!game());
        QCOMPARE(arcade->created, 0);
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // Signed in, a match opened from Flux Four's lobby waits for a challenger, and plays once one joins.
    void flux_four_opens_a_match_and_a_challenger_joins()
    {
        behaviour = QStringLiteral("Arena");
        signIn();
        if (QTest::currentTestFailed())
            return;
        open(QStringLiteral("Flux Four"), QStringLiteral("flux"), kFluxFour, QStringLiteral("FLUX FOUR"), QString());
        if (QTest::currentTestFailed())
            return;
        QTRY_COMPARE(screenText(QStringLiteral("Status")), QStringLiteral("0 open lobbies"));
        QTRY_VERIFY(screens().contains(QStringLiteral("lobby-create")));
        QTest::qWait(400);
        save(window, "flux-lobby.png");
        QVERIFY(clickShown(QStringLiteral("Create")));
        QTRY_VERIFY(screens().contains(QStringLiteral("waiting")));
        QCOMPARE(arcade->created, 1);
        QVERIFY(screenText(QStringLiteral("Waiting line")).startsWith(QStringLiteral("Your lobby is open.")));
        const QString id = match().value("id").toString();
        QCOMPARE(id, arcade->latest("flux")->id);
        QTest::qWait(400);
        save(window, "flux-waiting.png");

        arcade->opponentJoins(id);
        QTRY_COMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("YOUR MOVE"));
        QVERIFY(screens().contains(QStringLiteral("match")));
        QCOMPARE(screenText(QStringLiteral("Host")), QStringLiteral("yaldabaoth  (you)"));
        QCOMPARE(screenText(QStringLiteral("Guest")), QStringLiteral("sophia"));
        QVERIFY(screenText(QStringLiteral("Clock")).endsWith(QStringLiteral(" s")));
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // Four pulses dropped into one column, each a different way: Space after the right arrow, the number key 5, the
    // pad's A, and the pointer, which aims as it moves over the board and drops as it presses. Sophia drops hers in
    // the first column meanwhile, and the fourth wins the match.
    void four_pulses_dropped_by_space_a_number_the_pad_and_the_pointer_win_flux_four()
    {
        QVERIFY(logic());
        const QString id = match().value("id").toString();
        const std::function<void()> ways[] = {
            [this] { QTest::keyClick(window, Qt::Key_Right); QTRY_COMPARE(cursor(), 4); QTest::keyClick(window, Qt::Key_Space); },
            [this] { QTest::keyClick(window, Qt::Key_5); },
            [this] { pad(QStringLiteral("A")); },
            [this] {
                // Aimed away first, so the pointer is what brings it back.
                QTest::keyClick(window, Qt::Key_Left);
                QTRY_COMPARE(cursor(), 3);
                const QPoint at = cellAt(18);
                QTest::mouseMove(window, at);
                QTRY_COMPARE(cursor(), 4);
                save(window, "flux-aimed.png");
                QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, at);
            },
        };
        const int landed[] = {39, 32, 25, 18};
        for (int k = 0; k < 4; ++k) {
            QTRY_COMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("YOUR MOVE"));
            ways[k]();
            QTRY_COMPARE(match().value("moves").toInt(), 2 * k + 1);
            QCOMPARE(match().value("placed").toInt(), landed[k]);
            // It comes to rest with a sound from where it lands.
            QTRY_VERIFY(discOn(landed[k]));
            QTRY_VERIFY(heardNear(QStringLiteral("drop.wav"),
                                  discOn(landed[k])->property("scenePosition").value<QVector3D>(), 0.01));
            if (k == 3)
                break;
            QTRY_COMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("THEIR MOVE"));
            if (k == 1)
                save(window, "flux-playing.png");
            QVERIFY(arcade->opponentMoves(id, 0));
        }
        QTRY_COMPARE(screenText(QStringLiteral("Result")), QStringLiteral("VICTORY"));
        QCOMPARE(screenText(QStringLiteral("Result line")), QStringLiteral("Beautifully played."));
        QCOMPARE(asList(logic()->property("line")).size(), 4);
        QCOMPARE(arcade->find(id)->winner, 1);
        QTest::qWait(500);
        save(window, "flux-won.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void flux_fours_kept_run_replays_to_its_outcome()
    {
        QVariantMap outcome;
        keptRunReplays(kFluxFour, QStringLiteral("flux-four"), &outcome);
        if (QTest::currentTestFailed())
            return;
        QCOMPARE(outcome.value("result").toString(), QStringLiteral("won"));
        QCOMPARE(outcome.value("seat").toInt(), 1);
        QCOMPARE(outcome.value("moves").toInt(), 7);
        QCOMPARE(outcome.value("match").toString(), arcade->latest("flux")->id);
    }

    void flux_fours_kept_run_replays_in_qq_as_its_rules_replay_it()
    {
        fixtureReplays(QStringLiteral("flux-four"), kFluxFour);
    }

    // Play again opens a new lobby; closed before anyone joins, it says so, and the lobby is a button away.
    void a_rematchs_lobby_closed_before_anyone_joins()
    {
        QVERIFY(clickShown(QStringLiteral("Rematch")));
        QTRY_COMPARE(arcade->created, 2);
        QTRY_VERIFY(screens().contains(QStringLiteral("waiting")));
        QVERIFY(clickShown(QStringLiteral("Close lobby")));
        QTRY_VERIFY(screens().contains(QStringLiteral("closed")));
        QCOMPARE(arcade->latest("flux")->status, QStringLiteral("cancelled"));
        QVERIFY(clickShown(QStringLiteral("Find another")));
        QTRY_VERIFY(screens().contains(QStringLiteral("lobby")));
        QTRY_COMPARE(screenText(QStringLiteral("Status")), QStringLiteral("0 open lobbies"));
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_VERIFY(!game());
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // An open lobby taken by someone else just before the player joins it: the server says so, and the lobby shows
    // what is open now.
    void rift_reversi_says_when_an_open_lobby_was_taken_first()
    {
        behaviour = QStringLiteral("Arena");
        signIn();
        if (QTest::currentTestFailed())
            return;
        const QString taken = arcade->opponentOpens(QStringLiteral("reversi"));
        open(QStringLiteral("Rift Reversi"), QStringLiteral("reversi"), kRiftReversi, QStringLiteral("RIFT REVERSI"),
             QString());
        if (QTest::currentTestFailed())
            return;
        QTRY_COMPARE(screenText(QStringLiteral("Status")), QStringLiteral("1 open lobby"));
        Arcade::Match *m = arcade->find(taken);
        m->guest = QStringLiteral("p-2");
        m->status = QStringLiteral("active");
        m->revision += 1;
        QVERIFY(clickShown(QStringLiteral("Join")));
        QTRY_VERIFY(screens().contains(QStringLiteral("notice")));
        QCOMPARE(screenText(QStringLiteral("Notice")), QStringLiteral("This lobby is no longer available."));
        // The live arcade is read every 3 seconds while the lobby shows.
        QTRY_COMPARE_WITH_TIMEOUT(screenText(QStringLiteral("Status")), QStringLiteral("0 open lobbies"), 10000);
        QVERIFY(screens().contains(QStringLiteral("lobby")));
        QVERIFY(!screens().contains(QStringLiteral("lobby-join")));
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // Sophia's open lobby joined from Rift Reversi's: she moves first, and the player answers by the keys.
    void rift_reversi_joins_an_open_lobby_and_plays_by_the_keys()
    {
        QVERIFY(logic());
        const QString id = arcade->opponentOpens(QStringLiteral("reversi"));
        QTRY_COMPARE_WITH_TIMEOUT(screenText(QStringLiteral("Status")), QStringLiteral("1 open lobby"), 10000);
        QTRY_VERIFY(screens().contains(QStringLiteral("lobby-join")));
        // A button draws its words in capitals.
        QTRY_COMPARE(screenText(QStringLiteral("Join")).toLower(), QStringLiteral("join sophia"));
        QVERIFY(clickShown(QStringLiteral("Join")));
        QTRY_COMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("THEIR MOVE"));
        QCOMPARE(match().value("seat").toInt(), 2);
        QCOMPARE(screenText(QStringLiteral("Guest")), QStringLiteral("yaldabaoth  (you)   2"));

        QVERIFY(arcade->opponentMoves(id, 22));
        QTRY_COMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("YOUR MOVE"));
        // Each legal move marked, and the cursor on one.
        const QList<int> legal = Arcade::legal(*arcade->find(id));
        QTRY_VERIFY(legal.contains(cursor()));
        int marked = 0;
        for (const QVariant &v : asList(logic()->property("marks")))
            marked += v.value<QObject *>()->property("visible").toBool() ? 1 : 0;
        QCOMPARE(marked, int(legal.size()) - 1);
        // Down a row and back by the arrows; E jumps to the next legal move, and Space places a disc there.
        const int was = cursor();
        QTest::keyClick(window, Qt::Key_Down);
        QTRY_COMPARE(cursor(), std::min(was + 6, 30 + was % 6));
        QTest::keyClick(window, Qt::Key_Up);
        QTRY_COMPARE(cursor(), was);
        QTest::keyClick(window, Qt::Key_E);
        QList<int> sorted = legal;
        std::sort(sorted.begin(), sorted.end());
        const int next = sorted[(sorted.indexOf(was) + 1) % sorted.size()];
        QTRY_COMPARE(cursor(), next);
        save(window, "reversi-aimed.png");
        QTest::keyClick(window, Qt::Key_Space);
        QTRY_COMPARE(match().value("placed").toInt(), next);
        const QVariantList flipped = asList(match().value("flipped"));
        QVERIFY(!flipped.isEmpty());
        QTRY_VERIFY(discOn(next));
        QTRY_VERIFY(heardNear(QStringLiteral("place.wav"), discOn(next)->property("scenePosition").value<QVector3D>(), 0.01));
        QTRY_VERIFY(!placedVoices(QStringLiteral("flip.wav")).isEmpty());
        // Each disc it bracketed turns to the guest's colour.
        for (const QVariant &v : flipped)
            QTRY_COMPARE(discOn(v.toInt())->property("colour").value<QColor>(), QColor(QStringLiteral("#85c3cf")));
        QTRY_COMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("THEIR MOVE"));
        save(window, "reversi-turned.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    // A move the server refuses says why, and the player can move again; then by the pointer, and a concession, once
    // the player is sure.
    void a_refused_move_says_why_and_a_concession_waits_until_the_player_is_sure()
    {
        QVERIFY(logic());
        const QString id = match().value("id").toString();
        QVERIFY(arcade->find(id));
        QVERIFY(arcade->opponentMoves(id, Arcade::legal(*arcade->find(id)).first()));
        QTRY_COMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("YOUR MOVE"));
        arcade->refuseNext = QStringLiteral("The board has changed. Refreshing the match.");
        pad(QStringLiteral("A"));
        QTRY_VERIFY(screens().contains(QStringLiteral("notice")));
        QCOMPARE(screenText(QStringLiteral("Notice")), QStringLiteral("The board has changed. Refreshing the match."));
        QCOMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("YOUR MOVE"));
        save(window, "reversi-refused.png");

        const QList<int> legal = Arcade::legal(*arcade->find(id));
        const int target = legal.last();
        const QPoint at = cellAt(target);
        QTest::mouseMove(window, at);
        QTRY_COMPARE(cursor(), target);
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, at);
        QTRY_COMPARE(match().value("placed").toInt(), target);
        QTRY_COMPARE(screenText(QStringLiteral("Turn")), QStringLiteral("THEIR MOVE"));

        QVERIFY(clickShown(QStringLiteral("Concede")));
        QTRY_VERIFY(screens().contains(QStringLiteral("concede")));
        QVERIFY(clickShown(QStringLiteral("Keep playing")));
        QTRY_VERIFY(!screens().contains(QStringLiteral("concede")));
        QCOMPARE(arcade->find(id)->status, QStringLiteral("active"));
        QVERIFY(clickShown(QStringLiteral("Concede")));
        QTRY_VERIFY(screens().contains(QStringLiteral("concede")));
        QTest::qWait(300);
        save(window, "reversi-conceding.png");
        QVERIFY(clickShown(QStringLiteral("Concede match")));
        QTRY_COMPARE(screenText(QStringLiteral("Result")), QStringLiteral("MATCH COMPLETE"));
        QCOMPARE(arcade->find(id)->winner, 1);
        QVERIFY(screenText(QStringLiteral("Result line")).startsWith(QStringLiteral("Another world awaits.")));
        QTest::qWait(400);
        save(window, "reversi-over.png");
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }

    void rift_reversis_kept_run_replays_to_its_outcome()
    {
        QVariantMap outcome;
        keptRunReplays(kRiftReversi, QStringLiteral("rift-reversi"), &outcome);
        if (QTest::currentTestFailed())
            return;
        QCOMPARE(outcome.value("result").toString(), QStringLiteral("lost"));
        QCOMPARE(outcome.value("seat").toInt(), 2);
        QCOMPARE(outcome.value("moves").toInt(), 4);
    }

    void rift_reversis_kept_run_replays_in_qq_as_its_rules_replay_it()
    {
        fixtureReplays(QStringLiteral("rift-reversi"), kRiftReversi);
    }

    void esc_leaves_the_arena_too()
    {
        QTest::keyClick(window, Qt::Key_Escape);
        QTRY_COMPARE(window->property("playing").toString(), QString());
        QTRY_VERIFY(!game());
        QVERIFY(home()->isVisible());
        QVERIFY2(warnings.isEmpty(), qPrintable(warnings.join('\n')));
    }
};

QTEST_MAIN(TestGames)
#include "tst_games.moc"
