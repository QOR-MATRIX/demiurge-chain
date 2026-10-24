// ARQADE's games rebuilt in QQ (P7.20, ADR-102 decision 2), played in ARQADE's window by QQ's runtime. Orbital's card
// opens the game in place of the home; it is played as a person plays it, by the mouse on its screen buttons, the
// keyboard and the gamepad: a miss, a pause, and ten locks to complete the orbit. The run ARQADE kept replays in QQ to the same outcome,
// and so does the run kept in products/arqade/tests/fixtures/, which ARQADE's rules replay in Node (orbital.test.mjs).
// The launcher's reduced motion reaches the game, and Esc returns to the home.
//
// It opens a real window, so it needs a desktop session. With ARQADE_SHOT set to a folder, it saves what it drew there;
// with ARQADE_RUN set to a file, it writes the run it played there (how the fixture was made).

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

    QQuickItem *home() const { return window->findChild<QQuickItem *>(QStringLiteral("home")); }
    QQuickItem *game() const { return window->findChild<QQuickItem *>(QStringLiteral("game")); }
    QObject *scene() const { return game() ? game()->property("scene").value<QObject *>() : nullptr; }

    // Orbital's logic: the Behaviour named "Orbital", and what it holds.
    QObject *logic() const
    {
        if (!game())
            return nullptr;
        const QVariant list = game()->property("entityList");
        const QVariantList entities = list.canConvert<QJSValue>() ? list.value<QJSValue>().toVariant().toList()
                                                                  : list.toList();
        for (const QVariant &e : entities) {
            QObject *o = e.value<QObject *>();
            if (o && o->property("kind").toString() == QLatin1String("Behaviour")
                && o->property("name").toString() == QLatin1String("Orbital"))
                return o->property("logic").value<QObject *>();
        }
        return nullptr;
    }
    // The rules' game as it stands: { phase, angle, target, locks, misses, ... }.
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

    void orbitals_card_opens_the_game_in_arqades_window()
    {
        QQuickItem *card = find(window->contentItem(), [](QQuickItem *i) {
            return i->property("here").toBool() && i->property("name").toString() == QLatin1String("Orbital");
        });
        QVERIFY2(card, "Orbital's card is not one that plays here");
        // The other three still open today's version.
        for (const char *name : {"Void Runner", "Synapse", "Rift Survivor"})
            QVERIFY2(find(window->contentItem(), [&](QQuickItem *i) {
                         return i->property("name").toString() == QLatin1String(name) && i->property("here").isValid()
                             && !i->property("here").toBool();
                     }),
                     name);

        // Scrolled into view, and pressed.
        auto *page = window->findChild<QQuickItem *>("page");
        QVERIFY(page);
        const qreal below = card->mapToScene(QPointF(0, card->height())).y() - window->height() + 40;
        if (below > 0)
            page->setProperty("contentY", page->property("contentY").toReal() + below);
        QTest::qWait(300);
        // The pointer on the card first, and the card saying so, so the press lands where the card is now.
        const QPoint middle = card->mapToScene(QPointF(card->width() / 2, card->height() / 2)).toPoint();
        QVERIFY2(window->contentItem()->contains(middle),
                 qPrintable(QStringLiteral("the card's middle, %1, %2, is off the window").arg(middle.x()).arg(middle.y())));
        QTest::mouseMove(window, middle);
        QTRY_VERIFY2(card->property("hovered").toBool(), "the pointer is not on Orbital's card");
        QTest::mouseClick(window, Qt::LeftButton, Qt::NoModifier, middle);

        QTRY_COMPARE(window->property("playing").toString(), QStringLiteral("orbital"));
        QVERIFY(!home()->isVisible());
        QVERIFY(game());
        QTRY_VERIFY_WITH_TIMEOUT(game()->property("settled").toBool(), 15000);
        QCOMPARE(game()->property("problem").toString(), QString());
        QCOMPARE(QUrl(game()->property("file").toUrl()), kOrbital);
        QTRY_VERIFY(logic());
        QTRY_COMPARE(state().value("phase").toString(), QStringLiteral("title"));
        QTRY_COMPARE(screenText(QStringLiteral("Title")), QStringLiteral("ORBITAL"));
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
        QObject *view = game()->parent();
        while (view && !view->property("record").isValid())
            view = view->parent();
        QVERIFY(view);
        auto *record = qobject_cast<qq::RunRecord *>(view->property("record").value<QObject *>());
        QVERIFY(record);
        QVERIFY(QMetaObject::invokeMethod(scene(), "finishRecord"));
        const QVariantMap outcome = asMap(record->outcome().value(QStringLiteral("Orbital")));
        QCOMPARE(outcome.value("locks").toInt(), 10);
        QCOMPARE(outcome.value("misses").toInt(), 1);
        QCOMPARE(outcome.value("score").toInt(), 2000);
        QCOMPARE(outcome.value("over").toBool(), true);
        QVERIFY(record->seed() != 0);

        const QByteArray json = record->toJson();
        if (const QByteArray to = qgetenv("ARQADE_RUN"); !to.isEmpty()) {
            QFile out(QString::fromLocal8Bit(to));
            QVERIFY(out.open(QIODevice::WriteOnly));
            out.write(json);
        }

        qq::RunRecord again;
        QVERIFY(again.fromJson(json));
        QQmlEngine headless;
        const QVariantMap result = qq::replayRun(&headless, kOrbital, &again);
        QVERIFY2(result.value("problem").toString().isEmpty(), qPrintable(result.value("problem").toString()));
        QVERIFY2(result.value("matches").toBool(),
                 QJsonDocument(QJsonObject::fromVariantMap(result)).toJson(QJsonDocument::Compact).constData());
    }

    // The run kept beside ARQADE's rules, which Node replays with logic/rules.mjs: QQ's own replay reaches what it says.
    void the_kept_run_replays_in_qq_as_the_rules_replay_it()
    {
        QFile file(QStringLiteral(ARQADE_RUNS "/orbital-run.json"));
        QVERIFY2(file.open(QIODevice::ReadOnly), qPrintable(file.fileName()));
        qq::RunRecord kept;
        QString why;
        QVERIFY2(kept.fromJson(file.readAll(), &why), qPrintable(why));
        QQmlEngine headless;
        const QVariantMap result = qq::replayRun(&headless, kOrbital, &kept);
        QVERIFY2(result.value("problem").toString().isEmpty(), qPrintable(result.value("problem").toString()));
        QVERIFY2(result.value("matches").toBool(),
                 QJsonDocument(QJsonObject::fromVariantMap(result)).toJson(QJsonDocument::Compact).constData());
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

    // Back, on the bar, as well; and a game played again starts afresh, from a new seed.
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
};

QTEST_MAIN(TestGames)
#include "tst_games.moc"
