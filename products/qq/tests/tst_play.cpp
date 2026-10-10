// Do QQ worlds play as P3.2 says? (DIRECTION P3.2)
//
// Each case builds a scene in a real window, sets it playing and lets real frames pass: Qt Quick 3D Physics steps once
// for each frame drawn, so time here is time on the screen. What is checked is where things end up (a crate at rest on
// the ground, a player who walked four metres forward), what the player's hands did (keys sent to the window, a
// gamepad's state), what was drawn (particles counted by their colour in the frame), and what a logic file does
// before and after it is saved again while the world plays.

#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QImage>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSaveFile>
#include <QSignalSpy>
#include <QTemporaryDir>
#include <QTest>
#include <QVector2D>
#include <QVector3D>
#include <QtQml/qqmlextensionplugin.h>

#include "gpu.h"
#include <QtQuick3D/qquick3dobject.h>

#include <cmath>
#include <functional>
#include <memory>
#include <numbers>

#include "input.h"
#include "sceneio.h"

// Drawn with the GPU the Studio and the Player draw with: chosen before QTEST_MAIN makes the application.
static const QString chosenGpu = qq::preferHighPerformanceGpu();

Q_IMPORT_QML_PLUGIN(QQPlugin)

namespace {

QString fixture(const QString &name)
{
    return QDir(QStringLiteral(QQ_FIXTURES)).filePath(name);
}

/// A window with a world in it, and a scene playing in the world.
struct Stage {
    QQmlEngine engine;
    std::unique_ptr<QQuickWindow> window;
    QObject *world = nullptr;
    QObject *holder = nullptr;
    QObject *scene = nullptr;
    qq::SceneIO *io = nullptr;

    bool open(const QString &sceneText, bool play = true, const QUrl &file = {})
    {
        io = engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        QQmlComponent component(&engine);
        component.setData(R"(
            import QtQuick
            import QtQuick3D
            import QQ
            Window {
                width: 960; height: 600; visible: true; color: "black"
                World {
                    objectName: "world"
                    anchors.fill: parent
                    eye: Qt.vector3d(0, 3, 9)
                    target: Qt.vector3d(0, 0.5, 0)
                    Node { objectName: "holder" }
                }
                TextInput { objectName: "typing"; width: 200; height: 24 }
            }
        )", QUrl(QStringLiteral("qrc:/tst_play/stage.qml")));
        window.reset(qobject_cast<QQuickWindow *>(component.create()));
        if (!window) {
            qWarning() << component.errorString();
            return false;
        }
        world = window->findChild<QObject *>(QStringLiteral("world"));
        holder = world->findChild<QObject *>(QStringLiteral("holder"));
        if (!QTest::qWaitForWindowExposed(window.get()))
            return false;
        // Frames flowing before anything is timed: the first ones wait on shaders being compiled.
        // frameSwapped comes from the render thread, so each count is queued. Counted through `counting`, a count still
        // queued when it goes is dropped with it; one through the window would arrive after this frame has gone and
        // write into whatever stands there by then (it overwrote a saved register and crashed a later case).
        int frames = 0;
        {
            QObject counting;
            QObject::connect(window.get(), &QQuickWindow::frameSwapped, &counting, [&frames] { ++frames; });
            for (int i = 0; i < 500 && frames < 20; ++i) {
                window->update();
                QTest::qWait(10);
            }
        }
        world->setProperty("hearing", play);
        scene = io->loadText(sceneText, file.isEmpty() ? QUrl::fromLocalFile(fixture(QStringLiteral("scenes/play.qml"))) : file,
                             holder);
        if (!scene) {
            qWarning() << io->lastError();
            return false;
        }
        if (play)
            scene->setProperty("playing", true);
        return true;
    }

    ~Stage()
    {
        // As the Studio and the Player do: the scene put away in order, before the window takes the rest.
        if (scene)
            io->discard(scene);
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    }

    QObject *entity(const QString &name) const
    {
        for (const QVariant &e : io->entities(scene)) {
            if (e.value<QObject *>()->property("name").toString() == name)
                return e.value<QObject *>();
        }
        return nullptr;
    }

    /// Let `ms` of frames pass.
    void run(int ms)
    {
        QElapsedTimer t;
        t.start();
        while (t.elapsed() < ms) {
            window->update();
            QTest::qWait(8);
        }
    }

    /// Let frames pass until `done`, for at most `ms`.
    bool runUntil(const std::function<bool()> &done, int ms)
    {
        QElapsedTimer t;
        t.start();
        while (t.elapsed() < ms) {
            if (done())
                return true;
            window->update();
            QTest::qWait(8);
        }
        return done();
    }

    /// Let `ms` of the simulation's own time pass, however long the frames take to draw; how much did pass (the last
    /// step can carry it a little over).
    double simulate(int ms)
    {
        QObject *physics = scene->property("physicsWorld").value<QObject *>();
        if (!physics)
            return 0;
        QSignalSpy steps(physics, SIGNAL(frameDone(float)));
        double done = 0;
        QElapsedTimer guard;
        guard.start();
        while (done < ms && guard.elapsed() < 20000) {
            window->update();
            QTest::qWait(4);
            for (const QList<QVariant> &step : std::as_const(steps))
                done += step.at(0).toDouble();
            steps.clear();
        }
        return done;
    }

    /// Where an entity's body is in play (its physics node), or the entity itself.
    QVector3D where(const QString &name) const
    {
        QObject *e = entity(name);
        auto *body = e->property("physics").value<QObject *>();
        return (body ? body : e)->property("scenePosition").value<QVector3D>();
    }

    qq::Input *input() { return engine.singletonInstance<qq::Input *>("QQ", "Input"); }
};

QString sceneText(const QString &entities, const QString &settings = {})
{
    return QStringLiteral("import QQ\nScene {\n    name: \"play\"\n%1    Sun {}\n    Ground { extent: 30 }\n%2}\n")
        .arg(settings, entities);
}

void writeFile(const QString &path, const QByteArray &bytes)
{
    QDir().mkpath(QFileInfo(path).absolutePath());
    QSaveFile f(path);
    QVERIFY(f.open(QIODevice::WriteOnly));
    f.write(bytes);
    QVERIFY(f.commit());
}

}  // namespace

class TstPlay : public QObject
{
    Q_OBJECT

    // Each Stage has its own engine, so its own Input: nothing held in one case reaches the next.
private slots:
    // ── physics ──

    void aDynamicShapeFallsAndComesToRestOnTheGround()
    {
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
            "    Shape { name: \"Crate\"; position: Qt.vector3d(2, 3, 0); body: \"dynamic\"; mass: 10 }\n"))));
        QObject *crate = s.entity(QStringLiteral("Crate"));
        QVERIFY(crate->property("physics").value<QObject *>());
        // Falling (the first world in a process takes a moment to start)...
        QVERIFY2(s.runUntil([&s] { return s.where(QStringLiteral("Crate")).y() < 2.6f; }, 5000), "the crate did not fall");
        // ...and at rest.
        s.run(2500);
        const QVector3D rest = s.where(QStringLiteral("Crate"));
        QVERIFY2(qAbs(rest.y() - 0.5f) < 0.05f, qPrintable(QStringLiteral("at rest at y %1").arg(rest.y())));
        QVERIFY(qAbs(rest.x() - 2) < 0.05f);
        // The entity's own place is what the file keeps, and play does not move it.
        QCOMPARE(crate->property("position").value<QVector3D>(), QVector3D(2, 3, 0));
    }

    void aStaticShapeHoldsWhatLandsOnItAndNoneLetsItThrough()
    {
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
            "    Shape { name: \"Plinth\"; position: Qt.vector3d(0, 0.25, 0); scale: Qt.vector3d(2, 0.5, 2) }\n"
            "    Shape { name: \"Ball\"; form: \"sphere\"; position: Qt.vector3d(0, 3, 0); scale: Qt.vector3d(0.5, 0.5, 0.5); "
            "body: \"dynamic\"; bounce: 0 }\n"
            "    Shape { name: \"Mist\"; position: Qt.vector3d(3, 0.5, 0); scale: Qt.vector3d(2, 0.2, 2); body: \"none\" }\n"
            "    Shape { name: \"Drop\"; form: \"sphere\"; position: Qt.vector3d(3, 3, 0); scale: Qt.vector3d(0.5, 0.5, 0.5); "
            "body: \"dynamic\"; bounce: 0 }\n"))));
        QVERIFY(!s.entity(QStringLiteral("Mist"))->property("physics").value<QObject *>());
        s.run(2500);
        // On the plinth's top (0.5), a ball of radius 0.25.
        const float ball = s.where(QStringLiteral("Ball")).y();
        QVERIFY2(qAbs(ball - 0.75f) < 0.05f, qPrintable(QStringLiteral("ball at %1").arg(ball)));
        // Through the mist to the ground.
        const float drop = s.where(QStringLiteral("Drop")).y();
        QVERIFY2(qAbs(drop - 0.25f) < 0.05f, qPrintable(QStringLiteral("drop at %1").arg(drop)));
    }

    void aCylinderAndAConeCollideAsTheirShapes()
    {
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
            "    Shape { name: \"Pillar\"; form: \"cylinder\"; position: Qt.vector3d(-2, 3, 0); scale: Qt.vector3d(1, 2, 1); "
            "body: \"dynamic\"; bounce: 0 }\n"
            "    Shape { name: \"Peak\"; form: \"cone\"; position: Qt.vector3d(2, 3, 0); body: \"dynamic\"; bounce: 0 }\n"
            "    Shape { name: \"Spire\"; form: \"cone\"; position: Qt.vector3d(0, 3, -2); scale: Qt.vector3d(1, 3, 1); "
            "body: \"dynamic\"; bounce: 0 }\n"))));
        s.run(3000);
        // Standing on their flat ends: the pillar's middle a metre up, the cone's base (where a cone is) on the ground.
        const float pillar = s.where(QStringLiteral("Pillar")).y();
        const float peak = s.where(QStringLiteral("Peak")).y();
        QVERIFY2(qAbs(pillar - 1.0f) < 0.06f, qPrintable(QStringLiteral("pillar at %1").arg(pillar)));
        QVERIFY2(qAbs(peak) < 0.06f, qPrintable(QStringLiteral("cone at %1").arg(peak)));
        const float spire = s.where(QStringLiteral("Spire")).y();  // three times as tall, still on its base
        QVERIFY2(qAbs(spire) < 0.06f, qPrintable(QStringLiteral("tall cone at %1").arg(spire)));
        // Upright: none has toppled.
        for (const char *name : {"Pillar", "Peak", "Spire"}) {
            auto *body = s.entity(QLatin1String(name))->property("physics").value<QObject *>();
            const QVector3D turn = body->property("eulerRotation").value<QVector3D>();
            QVERIFY2(qAbs(turn.x()) < 3 && qAbs(turn.z()) < 3, name);
        }
    }

    void gravityIsTheScenes()
    {
        // A rock dropped for a second of the simulation's own time falls g/2 metres: 4.9 on Earth, 0.81 on the Moon.
        for (const double g : {9.81, 1.62}) {
            Stage s;
            QVERIFY(s.open(sceneText(QStringLiteral("    Shape { name: \"Rock\"; position: Qt.vector3d(0, 20, 0); body: \"dynamic\" }\n"),
                                     QStringLiteral("    gravity: %1\n").arg(g))));
            const double t = s.simulate(1000) / 1000;
            QVERIFY(t >= 1.0 && t < 1.1);
            const double fell = 20 - s.where(QStringLiteral("Rock")).y();
            const double expected = 0.5 * g * t * t;
            QVERIFY2(qAbs(fell - expected) < expected * 0.05,
                     qPrintable(QStringLiteral("under %1: fell %2 in %3 s, not %4").arg(g).arg(fell).arg(t).arg(expected)));
        }
    }

    // ── the player, under the keyboard and a gamepad ──

    void thePlayerWalksRunsTurnsAndJumpsAtTheKeyboard()
    {
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral("    Player { name: \"Hero\"; position: Qt.vector3d(0, 0, 0) }\n"))));
        QObject *hero = s.entity(QStringLiteral("Hero"));
        QCOMPARE(s.scene->property("player").value<QObject *>(), hero);
        s.run(400);  // settle on the ground
        auto feet = [hero] { return hero->property("feet").value<QVector3D>(); };
        QVERIFY2(qAbs(feet().y()) < 0.15f, qPrintable(QStringLiteral("standing at %1").arg(feet().y())));

        // W, for a second: forward is -z, at 4.5 m a second.
        QVector3D from = feet();
        QTest::keyPress(s.window.get(), Qt::Key_W);
        s.run(1000);
        QTest::keyRelease(s.window.get(), Qt::Key_W);
        s.run(200);
        const float walked = from.z() - feet().z();
        QVERIFY2(walked > 3.0f && walked < 5.5f, qPrintable(QStringLiteral("walked %1").arg(walked)));
        QVERIFY(qAbs(feet().x() - from.x()) < 0.2f);

        // Shift and W: further in the same time.
        from = feet();
        QTest::keyPress(s.window.get(), Qt::Key_Shift);
        QTest::keyPress(s.window.get(), Qt::Key_W, Qt::ShiftModifier);
        s.run(1000);
        QTest::keyRelease(s.window.get(), Qt::Key_W, Qt::ShiftModifier);
        QTest::keyRelease(s.window.get(), Qt::Key_Shift);
        s.run(200);
        const float ran = from.z() - feet().z();
        QVERIFY2(ran > walked * 1.4f, qPrintable(QStringLiteral("ran %1, walked %2").arg(ran).arg(walked)));

        // Right arrow: turns right, which is a falling heading.
        const double heading = hero->property("heading").toDouble();
        QTest::keyPress(s.window.get(), Qt::Key_Right);
        s.run(500);
        QTest::keyRelease(s.window.get(), Qt::Key_Right);
        QVERIFY2(hero->property("heading").toDouble() < heading - 40,
                 qPrintable(QStringLiteral("heading %1").arg(hero->property("heading").toDouble())));

        // Space: up more than half a metre, and down again.
        float highest = feet().y();
        QTest::keyPress(s.window.get(), Qt::Key_Space);
        QElapsedTimer t;
        t.start();
        while (t.elapsed() < 600) {
            highest = qMax(highest, feet().y());
            s.window->update();
            QTest::qWait(8);
        }
        QTest::keyRelease(s.window.get(), Qt::Key_Space);
        QVERIFY2(highest > 0.6f, qPrintable(QStringLiteral("jumped to %1").arg(highest)));
        s.run(1200);
        QVERIFY2(qAbs(feet().y()) < 0.15f, qPrintable(QStringLiteral("landed at %1").arg(feet().y())));

        // The camera is behind, above and looking at the player.
        const QVector3D eye = hero->property("eye").value<QVector3D>();
        const double h = hero->property("heading").toDouble() * std::numbers::pi / 180;
        const QVector3D behind(float(std::sin(h)), 0, float(std::cos(h)));
        const QVector3D offset = eye - feet();
        QVERIFY(QVector3D::dotProduct(QVector3D(offset.x(), 0, offset.z()).normalized(), behind) > 0.99f);
        QVERIFY(offset.y() > 1.5f);
    }

    void aGamepadMovesAndTurnsThePlayerPastItsDeadZone()
    {
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral("    Player { name: \"Hero\" }\n"))));
        QObject *hero = s.entity(QStringLiteral("Hero"));
        qq::Input *input = s.input();
        input->setReadsHardware(false);  // this case is the pad
        s.run(600);
        auto feet = [hero] { return hero->property("feet").value<QVector3D>(); };
        auto across = [](QVector3D v) { return QVector2D(v.x(), v.z()).length(); };

        // A stick at rest drifts a little; inside the dead zone the player stands still.
        QVector3D from = feet();
        input->gamepadState(QVector2D(0.12f, 0.15f), QVector2D(0.1f, 0), false, false);
        QCOMPARE(input->move(), QVector2D(0, 0));
        QCOMPARE(input->gamepad(), true);
        s.run(500);
        QVERIFY2(across(feet() - from) < 0.02f, qPrintable(QStringLiteral("drifted %1").arg(across(feet() - from))));

        // Full forward on the left stick, half right on the right.
        const double heading = hero->property("heading").toDouble();
        input->gamepadState(QVector2D(0, 1), QVector2D(0.6f, 0), false, false);
        s.run(800);
        QVERIFY2((feet() - from).length() > 2.0f, qPrintable(QStringLiteral("moved %1").arg((feet() - from).length())));
        QVERIFY(hero->property("heading").toDouble() < heading - 20);

        // A: jump.
        input->gamepadState({}, {}, true, false);
        float highest = feet().y();
        for (int i = 0; i < 50; ++i) {
            highest = qMax(highest, feet().y());
            s.run(10);
        }
        QVERIFY2(highest > 0.5f, qPrintable(QStringLiteral("jumped to %1").arg(highest)));

        // Unplugged: nothing held.
        input->gamepadState(QVector2D(0, 1), {}, true, true, false);
        QCOMPARE(input->move(), QVector2D(0, 0));
        QCOMPARE(input->jump(), false);
        QCOMPARE(input->gamepad(), false);
    }

    void typingIntoAFieldIsNotPlaying()
    {
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral("    Player { name: \"Hero\" }\n"))));
        auto *field = s.window->findChild<QQuickItem *>(QStringLiteral("typing"));
        field->forceActiveFocus();
        QTRY_VERIFY(s.window->activeFocusItem() == field);
        QTest::keyPress(s.window.get(), Qt::Key_W);
        QCOMPARE(s.input()->move(), QVector2D(0, 0));
        QTest::keyRelease(s.window.get(), Qt::Key_W);
        field->setFocus(false);
        s.window->contentItem()->forceActiveFocus();
        QTest::keyPress(s.window.get(), Qt::Key_W);
        QCOMPARE(s.input()->move(), QVector2D(0, 1));
        QTest::keyRelease(s.window.get(), Qt::Key_W);
        QCOMPARE(s.input()->move(), QVector2D(0, 0));
    }

    // ── particles ──

    void anEmitterFillsTheAirWithItsColour()
    {
        // A vivid green no sky or ground has, counted in the frame with the emitter running and with it stopped.
        auto green = [](const QImage &frame) {
            int n = 0;
            for (int y = 0; y < frame.height(); ++y) {
                for (int x = 0; x < frame.width(); ++x) {
                    const QColor c = frame.pixelColor(x, y);
                    // Screened over the blue-grey ground the sparks turn mint, so green leads but blue follows.
                    n += c.green() > 110 && c.green() > 1.5 * c.red() && c.green() > 1.3 * c.blue() ? 1 : 0;
                }
            }
            return n;
        };
        auto frameWith = [](Stage &s, double rate, int *alive) {
            QObject *e = s.entity(QStringLiteral("Spring"));
            e->setProperty("rate", rate);
            s.run(1500);
            *alive = e->property("alive").toInt();
            return s.window->grabWindow();
        };
        const QString text = sceneText(QStringLiteral(
            "    Emitter { name: \"Spring\"; position: Qt.vector3d(0, 0.3, 0); colour: \"#20ff40\"; rate: 1; "
            "size: 0.15; speed: 1.2; spread: 0.5 }\n"));
        Stage on;
        QVERIFY(on.open(text));
        int alive = 0;
        const QImage lit = frameWith(on, 150, &alive);
        lit.save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-emitter.png")));
        Stage off;
        QVERIFY(off.open(text));
        int none = 0;
        const QImage dark = frameWith(off, 0, &none);
        QVERIFY2(alive > 100, qPrintable(QStringLiteral("%1 alive").arg(alive)));
        const int litPixels = green(lit);
        const int darkPixels = green(dark);
        QVERIFY2(litPixels > 400 && darkPixels < 40,
                 qPrintable(QStringLiteral("%1 green pixels lit, %2 without").arg(litPixels).arg(darkPixels)));
    }

    // ── sound ──

    void aSoundIsHeardWhereItIsAndOnlyInPlay()
    {
        const QString text = sceneText(QStringLiteral(
            "    Sound { name: \"Chime\"; position: Qt.vector3d(1.5, 1, 0); source: \"../chime.wav\"; reach: 12 }\n"));
        Stage editing;
        QVERIFY(editing.open(text, false));
        QVERIFY(!editing.entity(QStringLiteral("Chime"))->property("playing").value<QObject *>());
        QVERIFY(editing.world->findChildren<QObject *>().count() > 0);
        auto listeners = [](QObject *world) {
            int n = 0;
            for (QObject *o : world->findChildren<QObject *>())
                n += QString::fromLatin1(o->metaObject()->className()).startsWith(QStringLiteral("QQuick3DAudioListener"));
            return n;
        };
        QCOMPARE(listeners(editing.world), 0);  // no sound device opened while editing

        Stage playing;
        QVERIFY(playing.open(text));
        QObject *voice = playing.entity(QStringLiteral("Chime"))->property("playing").value<QObject *>();
        QVERIFY(voice);
        QCOMPARE(voice->property("source").toUrl(), QUrl::fromLocalFile(fixture(QStringLiteral("chime.wav"))));
        QCOMPARE(voice->property("scenePosition").value<QVector3D>(), QVector3D(1.5f, 1, 0));
        QCOMPARE(voice->property("distanceCutoff").toDouble(), 12.0);
        QTRY_COMPARE(listeners(playing.world), 1);
    }

    // ── logic, rebuilt while playing ──

    void logicRunsAndIsRebuiltWhenItsFileIsSavedWithoutStoppingTheWorld()
    {
        QTemporaryDir project;
        const QString logic = QDir(project.path()).filePath(QStringLiteral("logic/turn.qml"));
        writeFile(logic, "import QQ\nLogic {\n    onFrame: (dt) => target.eulerRotation.y += 90 * dt\n}\n");
        const QString text = sceneText(QStringLiteral(
            "    Shape { name: \"Box\"; position: Qt.vector3d(0, 1, 0); body: \"none\" }\n"
            "    Behaviour { name: \"Turner\"; source: \"../logic/turn.qml\"; target: \"Box\" }\n"));
        Stage s;
        QVERIFY(s.open(text, true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *box = s.entity(QStringLiteral("Box"));
        QObject *turner = s.entity(QStringLiteral("Turner"));
        auto turned = [box] { return box->property("eulerRotation").value<QVector3D>().y(); };

        // Measured by the logic's own clock, which is what it is given each frame.
        auto elapsed = [turner] { return turner->property("logic").value<QObject *>()->property("elapsed").toDouble(); };
        QVERIFY(s.runUntil([&] { return elapsed() >= 0.8; }, 5000));
        QCOMPARE(turner->property("builds").toInt(), 1);
        QCOMPARE(turner->property("error").toString(), QString());
        QVERIFY2(qAbs(turned() - 90 * elapsed()) < 2,
                 qPrintable(QStringLiteral("turned %1 in %2 s").arg(turned()).arg(elapsed())));

        // Saved again, turning the other way and glowing: picked up while playing, the box left where it was.
        writeFile(logic, "import QQ\nLogic {\n    onFrame: (dt) => {\n        target.eulerRotation.y -= 180 * dt\n"
                         "        target.emissivePower = 6\n    }\n}\n");
        QVERIFY2(s.runUntil([turner] { return turner->property("builds").toInt() == 2; }, 4000), "not rebuilt");
        const float atReload = turned();
        const double startedAt = elapsed();  // the new logic's clock starts at its build
        QVERIFY2(atReload > 60, qPrintable(QStringLiteral("reset to %1 at the rebuild").arg(atReload)));
        QVERIFY(s.runUntil([&] { return elapsed() >= startedAt + 0.4; }, 5000));
        const double ran = elapsed() - startedAt;
        QVERIFY2(qAbs(turned() - (atReload - 180 * ran)) < 3,
                 qPrintable(QStringLiteral("%1 then %2 in %3 s").arg(atReload).arg(turned()).arg(ran)));
        QCOMPARE(box->property("emissivePower").toDouble(), 6.0);

        // Saved broken: said why, and the logic that ran keeps running.
        writeFile(logic, "import QQ\nLogic {\n    onFrame: (dt) => target.eulerRotation.y -=\n");
        QVERIFY2(s.runUntil([turner] { return !turner->property("error").toString().isEmpty(); }, 4000), "no error");
        QVERIFY2(turner->property("error").toString().contains(QStringLiteral("turn.qml")),
                 qPrintable(turner->property("error").toString()));
        QCOMPARE(turner->property("builds").toInt(), 2);
        const float whileBroken = turned();
        const double brokenAt = elapsed();
        QVERIFY(s.runUntil([&] { return elapsed() >= brokenAt + 0.2; }, 5000));
        QVERIFY(turned() < whileBroken - 30);

        // Saved as something that is not logic: refused, with the reason.
        writeFile(logic, "import QtQml\nQtObject {}\n");
        QVERIFY(s.runUntil([turner] { return turner->property("error").toString().contains(QStringLiteral("not QQ logic")); },
                           4000));
        QCOMPARE(turner->property("builds").toInt(), 2);
    }

    void logicDoesNotRunWhileEditing()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/turn.qml")),
                  "import QQ\nLogic {\n    onFrame: (dt) => target.eulerRotation.y += 90 * dt\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Shape { name: \"Box\"; body: \"none\" }\n"
                           "    Behaviour { name: \"Turner\"; source: \"../logic/turn.qml\"; target: \"Box\" }\n")),
                       false, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        s.run(500);
        QCOMPARE(s.entity(QStringLiteral("Turner"))->property("builds").toInt(), 0);
        QCOMPARE(s.entity(QStringLiteral("Box"))->property("eulerRotation").value<QVector3D>().y(), 0.0f);
    }
};

QTEST_MAIN(TstPlay)
#include "tst_play.moc"
