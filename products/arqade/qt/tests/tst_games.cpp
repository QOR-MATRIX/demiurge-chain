// ARQADE's games rebuilt in QQ (P7.20, ADR-102 decision 2), played in ARQADE's window by QQ's runtime. Each game's card
// opens it in place of the home, and it is played as a person plays it, by the mouse on its screen buttons, the
// keyboard and the gamepad:
//
//   Orbital: a miss, a pause, and ten locks to complete the orbit.
//   Void Runner: lanes changed by each of them, hazards dodged and gems caught, then three hazards met to end the run.
//
// The run ARQADE kept of each replays in QQ to the same outcome, and so does each run kept in
// products/arqade/tests/fixtures/, which the game's rules replay in Node (tests/<game>.test.mjs). The launcher's reduced
// motion reaches the game, and Esc and Back return to the home.
//
// It opens a real window, so it needs a desktop session. With ARQADE_SHOT set to a folder, it saves what it drew there;
// with ARQADE_RUN set to a folder, it writes each run it played there, as <game>-run.json (how the fixtures were made).

#include <QDir>
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

}  // namespace

class TestGames : public QObject
{
    Q_OBJECT

    QQmlApplicationEngine *engine = nullptr;
    ArqadeLink *link = nullptr;
    ArqadeServer *server = nullptr;
    QQuickWindow *window = nullptr;
    qq::Input *input = nullptr;
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

    // A game's card pressed from the keyboard, as a person tabbing to it presses it: Space on the card with focus. (Not
    // by the mouse: on Windows the real pointer, wherever it is, can take a synthetic one's hover and press away.) Then
    // its title.
    void open(const QString &name, const QString &kind, const QUrl &file, const QString &title)
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
        QTRY_VERIFY_WITH_TIMEOUT(game()->property("settled").toBool(), 15000);
        QCOMPARE(game()->property("problem").toString(), QString());
        QCOMPARE(QUrl(game()->property("file").toUrl()), file);
        QTRY_VERIFY(logic());
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("title"));
        QTRY_COMPARE(screenText(QStringLiteral("Title")), title);
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
        return best;
    }

private slots:
    void initTestCase()
    {
        arqade::loadFonts();
        link = new ArqadeLink(this);
        // A server nobody answers at: nothing here reaches the network.
        server = new ArqadeServer(link, QUrl("http://127.0.0.1:9/"), this);
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

    void the_rebuilt_games_cards_play_here_and_the_others_open_the_website()
    {
        for (const char *name : {"Orbital", "Void Runner"})
            QVERIFY2(find(window->contentItem(), [&](QQuickItem *i) {
                         return i->property("name").toString() == QLatin1String(name) && i->property("here").toBool();
                     }),
                     name);
        for (const char *name : {"Synapse", "Rift Survivor"})
            QVERIFY2(find(window->contentItem(), [&](QQuickItem *i) {
                         return i->property("name").toString() == QLatin1String(name) && i->property("here").isValid()
                             && !i->property("here").toBool();
                     }),
                     name);
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
};

QTEST_MAIN(TestGames)
#include "tst_games.moc"
