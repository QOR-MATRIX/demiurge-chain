// Do QQ worlds play as P3.2 says? (DIRECTION P3.2)
//
// Each case builds a scene in a real window, sets it playing and lets real frames pass: Qt Quick 3D Physics steps once
// for each frame drawn, so time here is time on the screen. What is checked is where things end up (a crate at rest on
// the ground, a player who walked four metres forward), what the player's hands did (keys sent to the window, a
// gamepad's state), what was drawn (particles counted by their colour in the frame), and what a logic file does
// before and after it is saved again while the world plays. A 2D world (P3.10) is checked pixel for pixel against the
// colours tests/fixtures/make_2d.py paints.

#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QImage>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QPointer>
#include <QQmlListReference>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSaveFile>
#include <QSet>
#include <QSignalSpy>
#include <QStandardPaths>
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

#include "confinement.h"
#include "input.h"
#include "mixer.h"
#include "run.h"
#include "sceneio.h"
#include "store.h"

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

    /// Seen flat, as a Game sees a 2D scene: `ppu` pixels of art to a unit, at least `viewHeight` units tall, centred
    /// on `target`.
    void flat(double ppu, double viewHeight, const QVector3D &target)
    {
        world->setProperty("pixelsPerUnit", ppu);
        world->setProperty("viewHeight", viewHeight);
        world->setProperty("target", target);
        world->setProperty("flat", true);
    }

    /// Where a point of the world is in a grabbed frame, in its device pixels.
    QPoint devicePixel(const QVector3D &at, const QImage &frame) const
    {
        QVector3D onScreen;
        QMetaObject::invokeMethod(world, "mapFrom3DScene", Q_RETURN_ARG(QVector3D, onScreen), Q_ARG(QVector3D, at));
        return QPoint(qRound(onScreen.x() * frame.devicePixelRatio()), qRound(onScreen.y() * frame.devicePixelRatio()));
    }

    /// What a screen part drew on the World's overlay (P3.8), by the part's name, or null.
    QQuickItem *drawn(const QString &name) const
    {
        auto *overlay = world->property("overlay").value<QQuickItem *>();
        if (!overlay)
            return nullptr;
        for (QQuickItem *child : overlay->childItems()) {
            if (child->objectName() == name)
                return child;
        }
        return nullptr;
    }
};

/// A 2D scene's text: flat, 16 pixels to a unit, no ground (seen edge on, it would be a line across the view).
QString flatText(const QString &entities, const QString &settings = {})
{
    return QStringLiteral("import QQ\nScene {\n    name: \"play\"\n    view: \"2d\"\n    pixelsPerUnit: 16\n%1    Sun {}\n%2}\n")
        .arg(settings, entities);
}

/// A project folder with the 2D fixtures in its assets/, and where its scene goes.
struct FlatProject {
    QTemporaryDir dir;
    FlatProject()
    {
        QDir(dir.path()).mkpath(QStringLiteral("assets"));
        for (const char *name : {"level.tmj", "tiles.tsj", "tiles.png", "sheet.png"})
            QFile::copy(fixture(QLatin1String(name)), QDir(dir.path()).filePath(QStringLiteral("assets/") + QLatin1String(name)));
    }
    QUrl scene() const { return QUrl::fromLocalFile(QDir(dir.path()).filePath(QStringLiteral("scenes/play.qml"))); }
};

/// The colours make_2d.py paints: the sheet's eight frames, and the tileset's four tiles.
const QColor frameColours[] = {QColor(200, 40, 40), QColor(40, 200, 40), QColor(40, 40, 200), QColor(200, 200, 40),
                               QColor(200, 40, 200), QColor(40, 200, 200), QColor(240, 120, 20), QColor(120, 20, 240)};
const QColor tileColours[] = {QColor(192, 48, 40), QColor(48, 160, 80), QColor(48, 80, 192), QColor(224, 192, 64)};

/// Whether `c` is `want` as painted, give or take rounding: two levels, or six in a dark channel, which the renderer
/// holds with fewer steps.
bool painted(const QColor &c, const QColor &want)
{
    auto near = [](int got, int wanted) { return qAbs(got - wanted) <= (wanted < 48 ? 6 : 2); };
    return near(c.red(), want.red()) && near(c.green(), want.green()) && near(c.blue(), want.blue());
}

/// A scene with no window and no world, ticked by hand (P3.11): how a replay plays.
struct Headless {
    QQmlEngine engine;
    QObject *scene = nullptr;
    qq::SceneIO *io = nullptr;

    /// Play `text` as the scene in `file`, kept by `recorder` if one is given.
    bool open(const QString &text, const QUrl &file, QObject *recorder = nullptr)
    {
        io = engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        scene = io->loadText(text, file, nullptr);
        if (!scene) {
            qWarning() << io->lastError();
            return false;
        }
        scene->setProperty("autoAdvance", false);
        if (recorder)
            scene->setProperty("recorder", QVariant::fromValue(recorder));
        scene->setProperty("playing", true);
        return QTest::qWaitFor([this] {
            QVariant ready;
            QMetaObject::invokeMethod(scene, "ready", Q_RETURN_ARG(QVariant, ready));
            return ready.toBool();
        }, 10000);
    }

    ~Headless()
    {
        if (scene)
            io->discard(scene);
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    }

    int advance(double seconds)
    {
        QVariant ran;
        QMetaObject::invokeMethod(scene, "advance", Q_RETURN_ARG(QVariant, ran), Q_ARG(QVariant, seconds));
        return ran.toInt();
    }

    QObject *logicOf(const QString &behaviour) const
    {
        for (const QVariant &e : io->entities(scene)) {
            if (e.value<QObject *>()->property("name").toString() == behaviour)
                return e.value<QObject *>()->property("logic").value<QObject *>();
        }
        return nullptr;
    }
};

QVariantMap outcomeOf(QObject *scene)
{
    QVariant text;
    QMetaObject::invokeMethod(scene, "outcomeText", Q_RETURN_ARG(QVariant, text));
    return QJsonDocument::fromJson(text.toString().toUtf8()).object().toVariantMap();
}

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
    void initTestCase()
    {
        // Games' stores (P3.16) go to the test's own folders, never a person's.
        QStandardPaths::setTestModeEnabled(true);
    }

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
        writeFile(logic, "import QQ\nLogic {\n    onStep: (dt) => target.eulerRotation.y += 90 * dt\n}\n");
        const QString text = sceneText(QStringLiteral(
            "    Shape { name: \"Box\"; position: Qt.vector3d(0, 1, 0); body: \"none\" }\n"
            "    Behaviour { name: \"Turner\"; source: \"../logic/turn.qml\"; target: \"Box\" }\n"));
        Stage s;
        QVERIFY(s.open(text, true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *box = s.entity(QStringLiteral("Box"));
        QObject *turner = s.entity(QStringLiteral("Turner"));
        auto turned = [box] { return box->property("eulerRotation").value<QVector3D>().y(); };

        // Measured by the logic's own clock, which is what it is given each step.
        auto elapsed = [turner] { return turner->property("logic").value<QObject *>()->property("elapsed").toDouble(); };
        QVERIFY(s.runUntil([&] { return elapsed() >= 0.8; }, 5000));
        QCOMPARE(turner->property("builds").toInt(), 1);
        QCOMPARE(turner->property("error").toString(), QString());
        QVERIFY2(qAbs(turned() - 90 * elapsed()) < 2,
                 qPrintable(QStringLiteral("turned %1 in %2 s").arg(turned()).arg(elapsed())));

        // Saved again, turning the other way and glowing: picked up while playing, the box left where it was.
        writeFile(logic, "import QQ\nLogic {\n    onStep: (dt) => {\n        target.eulerRotation.y -= 180 * dt\n"
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
        writeFile(logic, "import QQ\nLogic {\n    onStep: (dt) => target.eulerRotation.y -=\n");
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

    // ── gameplay in logic (P3.7, ADR-105) ──

    void logicHearsItsTargetTouchAnotherEntity()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/touch.qml")),
                  "import QQ\nLogic {\n    property string met: \"\"\n"
                  "    onContact: (other, point, normal) => met += (other ? other.name : \"?\") + \";\"\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Shape { name: \"Plinth\"; position: Qt.vector3d(0, 0.25, 0); scale: Qt.vector3d(2, 0.5, 2) }\n"
                           "    Shape { name: \"Crate\"; position: Qt.vector3d(0, 3, 0); scale: Qt.vector3d(0.5, 0.5, 0.5); "
                           "body: \"dynamic\"; bounce: 0 }\n"
                           "    Behaviour { name: \"Touch\"; source: \"../logic/touch.qml\"; target: \"Crate\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *touch = s.entity(QStringLiteral("Touch"));
        auto met = [touch] {
            QObject *logic = touch->property("logic").value<QObject *>();
            return logic ? logic->property("met").toString().split(QLatin1Char(';'), Qt::SkipEmptyParts) : QStringList();
        };
        QVERIFY2(s.runUntil([&] { return met().contains(QStringLiteral("Plinth")); }, 6000), qPrintable(met().join(',')));
        // Once as they meet, not over and over while it rests there.
        s.run(1500);
        QVERIFY2(met().count(QStringLiteral("Plinth")) <= 3, qPrintable(met().join(',')));
        QVERIFY2(!met().contains(QStringLiteral("?")), qPrintable(met().join(',')));
    }

    void aTriggerHearsWhatPassesThroughIt()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/gate.qml")),
                  "import QQ\nLogic {\n    property string log: \"\"\n"
                  "    onEntered: (other) => log += \"in:\" + (other ? other.name : \"?\") + \";\"\n"
                  "    onExited: (other) => log += \"out:\" + (other ? other.name : \"?\") + \";\"\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Trigger { name: \"Gate\"; position: Qt.vector3d(0, 2.5, 0); scale: Qt.vector3d(2, 1, 2) }\n"
                           "    Shape { name: \"Ball\"; form: \"sphere\"; position: Qt.vector3d(0, 5, 0); "
                           "scale: Qt.vector3d(0.5, 0.5, 0.5); body: \"dynamic\"; bounce: 0 }\n"
                           "    Behaviour { name: \"Watch\"; source: \"../logic/gate.qml\"; target: \"Gate\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *watch = s.entity(QStringLiteral("Watch"));
        auto log = [watch] {
            QObject *logic = watch->property("logic").value<QObject *>();
            return logic ? logic->property("log").toString() : QString();
        };
        QVERIFY2(s.runUntil([&] { return log().contains(QStringLiteral("out:Ball;")); }, 6000), qPrintable(log()));
        QCOMPARE(log(), QStringLiteral("in:Ball;out:Ball;"));
        // Nothing collides with it: the ball fell through to the ground.
        s.run(1000);
        QVERIFY2(qAbs(s.where(QStringLiteral("Ball")).y() - 0.25f) < 0.05f,
                 qPrintable(QStringLiteral("ball at %1").arg(s.where(QStringLiteral("Ball")).y())));
        QCOMPARE(s.entity(QStringLiteral("Gate"))->property("inside").toInt(), 0);
    }

    void aTriggerHearsThePlayerWalkIn()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/pickup.qml")),
                  "import QQ\nLogic {\n    property string who: \"\"\n"
                  "    onEntered: (other) => { if (!who) who = other ? other.kind + \":\" + other.name : \"?\" }\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Player { name: \"Hero\" }\n"
                           "    Trigger { name: \"Orb\"; form: \"sphere\"; position: Qt.vector3d(0, 1, -2.5) }\n"
                           "    Behaviour { name: \"Pickup\"; source: \"../logic/pickup.qml\"; target: \"Orb\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        s.run(400);  // standing
        QObject *pickup = s.entity(QStringLiteral("Pickup"));
        auto who = [pickup] {
            QObject *logic = pickup->property("logic").value<QObject *>();
            return logic ? logic->property("who").toString() : QString();
        };
        QCOMPARE(who(), QString());
        QTest::keyPress(s.window.get(), Qt::Key_W);
        const bool heard = s.runUntil([&] { return !who().isEmpty(); }, 3000);
        QTest::keyRelease(s.window.get(), Qt::Key_W);
        QVERIFY2(heard, "the orb did not hear the player");
        QCOMPARE(who(), QStringLiteral("Player:Hero"));
    }

    void aRayMeetsTheFirstEntityInItsWay()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/look.qml")),
                  "import QQ\nLogic {\n"
                  "    property string down: \"\"\n    property real top: -1\n    property real facing: 0\n"
                  "    property string up: \"\"\n    property string near: \"\"\n    property string below: \"\"\n"
                  "    onFrame: (dt) => {\n"
                  "        const d = ray(Qt.vector3d(0, 5, 0), Qt.vector3d(0, -1, 0))\n"
                  "        down = d ? d.entity.name : \"nothing\"\n"
                  "        top = d ? d.point.y : -1\n"
                  "        facing = d ? d.normal.y : 0\n"
                  "        const u = ray(Qt.vector3d(0, 5, 0), Qt.vector3d(0, 1, 0))\n"
                  "        up = u ? u.entity.name : \"nothing\"\n"
                  "        const n = ray(Qt.vector3d(0, 5, 0), Qt.vector3d(0, -2, 0), 2)\n"
                  "        near = n ? n.entity.name : \"nothing\"\n"
                  "        const b = ray(target.position, Qt.vector3d(0, -1, 0))\n"
                  "        below = b ? b.entity.name : \"nothing\"\n"
                  "    }\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Shape { name: \"Block\"; position: Qt.vector3d(0, 0.5, 0) }\n"
                           "    Trigger { name: \"Mist\"; position: Qt.vector3d(0, 3, 0); scale: Qt.vector3d(2, 1, 2) }\n"
                           "    Shape { name: \"Lens\"; position: Qt.vector3d(3, 2, 0); scale: Qt.vector3d(0.5, 0.5, 0.5); "
                           "body: \"none\" }\n"
                           "    Behaviour { name: \"Look\"; source: \"../logic/look.qml\"; target: \"Lens\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *look = s.entity(QStringLiteral("Look"));
        auto value = [look](const char *name) {
            QObject *logic = look->property("logic").value<QObject *>();
            return logic ? logic->property(name) : QVariant();
        };
        QVERIFY2(s.runUntil([&] { return !value("down").toString().isEmpty(); }, 5000), "the logic did not run");
        // Down from above: through the trigger, which nothing hits in play, to the block's top face.
        QCOMPARE(value("down").toString(), QStringLiteral("Block"));
        QVERIFY2(qAbs(value("top").toDouble() - 1.0) < 0.01, qPrintable(value("top").toString()));
        QVERIFY2(value("facing").toDouble() > 0.99, qPrintable(value("facing").toString()));
        // Up into the sky: nothing. Two metres down from five: short of the block.
        QCOMPARE(value("up").toString(), QStringLiteral("nothing"));
        QCOMPARE(value("near").toString(), QStringLiteral("nothing"));
        // From inside its own target, the target is not met: the ground below is.
        QCOMPARE(value("below").toString(), QStringLiteral("Ground"));
    }

    void logicMakesAndRemovesEntitiesOnItsOwnClock()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/rain.qml")),
                  "import QQ\nLogic {\n"
                  "    property int ticks: 0\n    property int once: 0\n    property int stopped: 0\n"
                  "    property int made: 0\n    property bool removed: false\n    property bool refused: false\n"
                  "    onStarted: {\n"
                  "        every(0.1, () => ticks += 1)\n"
                  "        const stop = every(0.1, () => stopped += 1)\n"
                  "        after(0.35, () => cancel(stop))\n"
                  "        after(0.2, () => {\n"
                  "            once += 1\n"
                  "            for (let i = 0; i < 3; ++i)\n"
                  "                if (spawn(\"Shape\", { name: \"Drop\" + i, form: \"sphere\", body: \"dynamic\",\n"
                  "                                      position: Qt.vector3d(i * 2 - 2, 4, 0),\n"
                  "                                      scale: Qt.vector3d(0.5, 0.5, 0.5) }))\n"
                  "                    made += 1\n"
                  "            refused = spawn(\"Dragon\", {}) === null\n"
                  "        })\n"
                  "        after(1.5, () => removed = remove(entity(\"Drop1\")))\n"
                  "    }\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral("    Behaviour { name: \"Rain\"; source: \"../logic/rain.qml\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *rain = s.entity(QStringLiteral("Rain"));
        auto logic = [rain] { return rain->property("logic").value<QObject *>(); };
        auto elapsed = [&] { return logic() ? logic()->property("elapsed").toDouble() : 0.0; };

        QVERIFY(s.runUntil([&] { return elapsed() >= 1.2; }, 8000));
        QCOMPARE(logic()->property("made").toInt(), 3);
        QCOMPARE(logic()->property("once").toInt(), 1);
        QCOMPARE(logic()->property("refused").toBool(), true);
        QCOMPARE(logic()->property("stopped").toInt(), 3);  // at 0.1, 0.2 and 0.3, then cancelled at 0.35
        const int ticks = logic()->property("ticks").toInt();
        QVERIFY2(qAbs(ticks - int(elapsed() * 10)) <= 1,
                 qPrintable(QStringLiteral("%1 ticks in %2 s").arg(ticks).arg(elapsed())));
        // Spawned while playing, they are bodies, and they fell.
        for (const char *name : {"Drop0", "Drop1", "Drop2"}) {
            QObject *drop = s.entity(QLatin1String(name));
            QVERIFY2(drop && drop->property("physics").value<QObject *>(), name);
            QVERIFY2(s.where(QLatin1String(name)).y() < 2, name);
        }
        QPointer<QObject> body = s.entity(QStringLiteral("Drop1"))->property("physics").value<QObject *>();

        // Removed with its body.
        QVERIFY(s.runUntil([&] { return logic()->property("removed").toBool() && !s.entity(QStringLiteral("Drop1")); },
                           4000));
        QTRY_VERIFY(body.isNull());
        QVERIFY(s.entity(QStringLiteral("Drop0")) && s.entity(QStringLiteral("Drop2")));
    }

    void logicPushesDynamicBodiesAndLeadsKinematicOnes()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/kick.qml")),
                  "import QQ\nLogic {\n    property bool kicked: false\n    property bool refused: false\n"
                  "    onStarted: after(0.6, () => {\n"
                  "        kicked = target.push(Qt.vector3d(4, 0, 0))\n"
                  "        refused = !entity(\"Lift\").push(Qt.vector3d(4, 0, 0))\n"
                  "    })\n}\n");
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/lift.qml")),
                  "import QQ\nLogic {\n    onFrame: (dt) => { if (elapsed > 0.5 && target.position.y < 1.25) "
                  "target.position.y = Math.min(1.25, target.position.y + 0.8 * dt) }\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Shape { name: \"Ball\"; form: \"sphere\"; position: Qt.vector3d(-3, 0.25, 0); "
                           "scale: Qt.vector3d(0.5, 0.5, 0.5); body: \"dynamic\"; bounce: 0 }\n"
                           "    Behaviour { name: \"Kick\"; source: \"../logic/kick.qml\"; target: \"Ball\" }\n"
                           "    Shape { name: \"Lift\"; position: Qt.vector3d(0, 0.25, -4); scale: Qt.vector3d(2, 0.5, 2); "
                           "body: \"kinematic\" }\n"
                           "    Shape { name: \"Rider\"; position: Qt.vector3d(0, 0.75, -4); scale: Qt.vector3d(0.5, 0.5, 0.5); "
                           "body: \"dynamic\"; bounce: 0 }\n"
                           "    Behaviour { name: \"Raise\"; source: \"../logic/lift.qml\"; target: \"Lift\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *kick = s.entity(QStringLiteral("Kick"));
        auto logic = [kick] { return kick->property("logic").value<QObject *>(); };
        QVERIFY(s.runUntil([&] { return logic() && logic()->property("kicked").toBool(); }, 5000));
        QCOMPARE(logic()->property("refused").toBool(), true);  // a kinematic body is led, not pushed
        s.run(1500);
        QVERIFY2(s.where(QStringLiteral("Ball")).x() > -1.5f,
                 qPrintable(QStringLiteral("ball at x %1").arg(s.where(QStringLiteral("Ball")).x())));

        // The lift rose a metre, its body with it, and carried its rider up.
        QVERIFY(s.runUntil(
            [&] { return s.entity(QStringLiteral("Lift"))->property("position").value<QVector3D>().y() >= 1.249f; }, 6000));
        s.run(500);
        QVERIFY2(qAbs(s.where(QStringLiteral("Lift")).y() - 1.25f) < 0.02f,
                 qPrintable(QStringLiteral("lift's body at %1").arg(s.where(QStringLiteral("Lift")).y())));
        QVERIFY2(s.where(QStringLiteral("Rider")).y() > 1.6f,
                 qPrintable(QStringLiteral("rider at %1").arg(s.where(QStringLiteral("Rider")).y())));
    }

    // ── groups and prefabs (P3.15) ──

    void aGroupCarriesWhatIsInItAndItsBodiesFollow()
    {
        // A lift: a group holding a kinematic deck and a trigger above it. A crate lands on the deck where the group put
        // it, and the trigger hears it; then logic raises the group a little each frame, as a game moves a platform,
        // and the deck carries the crate up, the trigger going with them.
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/raise.qml")),
                  "import QQ\nLogic {\n    property bool go: false\n"
                  "    onFrame: (dt) => { if (go && target.position.y < 1.5) "
                  "target.position.y = Math.min(1.5, target.position.y + 0.8 * dt) }\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Group { name: \"Lift\"; position: Qt.vector3d(3, 0.5, 0)\n"
                           "        Shape { name: \"Deck\"; body: \"kinematic\"; scale: Qt.vector3d(2, 0.2, 2) }\n"
                           "        Trigger { name: \"Sensor\"; position: Qt.vector3d(0, 0.5, 0); scale: Qt.vector3d(1.5, 1, 1.5) }\n"
                           "    }\n"
                           "    Behaviour { name: \"Raise\"; source: \"../logic/raise.qml\"; target: \"Lift\" }\n"
                           "    Shape { name: \"Crate\"; body: \"dynamic\"; position: Qt.vector3d(3, 3, 0); "
                           "scale: Qt.vector3d(0.5, 0.5, 0.5); bounce: 0 }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        // On the deck, and staying there (not only passing on its way down): its top at 0.5 + 0.1, the crate's middle a
        // quarter above.
        QVERIFY2(s.runUntil([&] { return qAbs(s.where(QStringLiteral("Crate")).y() - 0.85f) < 0.05f; }, 5000),
                 qPrintable(QString::number(s.where(QStringLiteral("Crate")).y())));
        s.run(600);
        QVERIFY2(qAbs(s.where(QStringLiteral("Crate")).y() - 0.85f) < 0.05f,
                 qPrintable(QString::number(s.where(QStringLiteral("Crate")).y())));
        QCOMPARE(s.where(QStringLiteral("Deck")), QVector3D(3, 0.5f, 0));
        QObject *sensor = s.entity(QStringLiteral("Sensor"));
        QVERIFY(s.runUntil([&] { return sensor->property("inside").toInt() >= 1; }, 2000));
        // Raised a unit by logic.
        QObject *raise = s.entity(QStringLiteral("Raise"))->property("logic").value<QObject *>();
        QVERIFY(raise);
        raise->setProperty("go", true);
        QVERIFY(s.runUntil([&] { return s.entity(QStringLiteral("Lift"))->property("position").value<QVector3D>().y() >= 1.499f; },
                           4000));
        s.run(500);
        QVERIFY2(qAbs(s.where(QStringLiteral("Crate")).y() - 1.85f) < 0.08f,
                 qPrintable(QString::number(s.where(QStringLiteral("Crate")).y())));
        QVERIFY(qAbs(s.where(QStringLiteral("Deck")).y() - 1.5f) < 1e-4f);
        QVERIFY(qAbs(sensor->property("physics").value<QObject *>()->property("scenePosition").value<QVector3D>().y() - 2.0f) < 1e-4f);
        QVERIFY(sensor->property("inside").toInt() >= 1);
    }

    void aPrefabPlacedTwiceIsTwoAndLogicPlacesMore()
    {
        // A hoop: a trigger and the logic that counts what passes through it, in a prefab file. Two are placed in the
        // scene and logic places a third; a ball dropped through the right one is counted by that one alone.
        QTemporaryDir project;
        const QDir root(project.path());
        writeFile(root.filePath(QStringLiteral("logic/count.qml")),
                  "import QQ\nLogic {\n    property int count: 0\n    onEntered: (other) => count += 1\n}\n");
        writeFile(root.filePath(QStringLiteral("prefabs/hoop.qml")),
                  "import QQ\nGroup {\n    name: \"Hoop\"\n"
                  "    Trigger { name: \"Ring\"; position: Qt.vector3d(0, 1, 0); scale: Qt.vector3d(2, 0.5, 2) }\n"
                  "    Behaviour { name: \"Counter\"; source: \"../logic/count.qml\"; target: \"Ring\" }\n}\n");
        writeFile(root.filePath(QStringLiteral("logic/placer.qml")),
                  "import QQ\nLogic {\n    property var middle: null\n"
                  "    onStarted: middle = spawn(\"Prefab\", { name: \"Middle\", source: \"../prefabs/hoop.qml\", "
                  "position: Qt.vector3d(0, 0, -4) })\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Prefab { name: \"Left\"; source: \"../prefabs/hoop.qml\"; position: Qt.vector3d(-3, 0, 0) }\n"
                           "    Prefab { name: \"Right\"; source: \"../prefabs/hoop.qml\"; position: Qt.vector3d(3, 0, 0) }\n"
                           "    Behaviour { name: \"Placer\"; source: \"../logic/placer.qml\" }\n"
                           "    Shape { name: \"Ball\"; form: \"sphere\"; body: \"dynamic\"; position: Qt.vector3d(3, 4, 0);"
                           " scale: Qt.vector3d(0.4, 0.4, 0.4) }\n")),
                       true, QUrl::fromLocalFile(root.filePath(QStringLiteral("scenes/play.qml")))));
        auto counted = [&](const QString &prefab) {
            QObject *p = s.entity(prefab);
            if (!p)
                return -1;
            QVariant counter;
            QMetaObject::invokeMethod(p, "part", Q_RETURN_ARG(QVariant, counter), Q_ARG(QVariant, QStringLiteral("Counter")));
            QObject *logic = counter.value<QObject *>() ? counter.value<QObject *>()->property("logic").value<QObject *>() : nullptr;
            return logic ? logic->property("count").toInt() : -1;
        };
        // Each made of the file, where it stands; the third placed by logic.
        QVERIFY(s.runUntil([&] { return counted(QStringLiteral("Left")) == 0 && counted(QStringLiteral("Right")) >= 0
                                        && counted(QStringLiteral("Middle")) == 0; }, 5000));
        QVariant ring;
        QMetaObject::invokeMethod(s.entity(QStringLiteral("Right")), "part", Q_RETURN_ARG(QVariant, ring),
                                  Q_ARG(QVariant, QStringLiteral("Ring")));
        QCOMPARE(ring.value<QObject *>()->property("scenePosition").value<QVector3D>(), QVector3D(3, 1, 0));
        // The ball falls through the right one: counted there, nowhere else.
        QVERIFY2(s.runUntil([&] { return counted(QStringLiteral("Right")) == 1; }, 5000), "the right hoop never counted");
        s.run(300);
        QCOMPARE(counted(QStringLiteral("Right")), 1);
        QCOMPARE(counted(QStringLiteral("Left")), 0);
        QCOMPARE(counted(QStringLiteral("Middle")), 0);
        // The scene's own entities are the prefabs, not their parts.
        QVERIFY(!s.entity(QStringLiteral("Ring")));
        // Logic removes the third, parts and all.
        QObject *placer = s.entity(QStringLiteral("Placer"))->property("logic").value<QObject *>();
        QObject *middle = placer->property("middle").value<QObject *>();
        QVERIFY(middle);
        QVariant removed;
        QMetaObject::invokeMethod(placer, "remove", Q_RETURN_ARG(QVariant, removed), Q_ARG(QVariant, QVariant::fromValue(middle)));
        QVERIFY(removed.toBool());
        QVERIFY(s.runUntil([&] { return !s.entity(QStringLiteral("Middle")); }, 2000));
        // And one part of the left one: logic takes away what is in a prefab (or a group) too.
        QVariant leftRing;
        QMetaObject::invokeMethod(s.entity(QStringLiteral("Left")), "part", Q_RETURN_ARG(QVariant, leftRing),
                                  Q_ARG(QVariant, QStringLiteral("Ring")));
        QMetaObject::invokeMethod(placer, "remove", Q_RETURN_ARG(QVariant, removed), Q_ARG(QVariant, leftRing));
        QVERIFY(removed.toBool());
        QVERIFY(s.runUntil([&] {
            QVariant ring;
            QMetaObject::invokeMethod(s.entity(QStringLiteral("Left")), "part", Q_RETURN_ARG(QVariant, ring),
                                      Q_ARG(QVariant, QStringLiteral("Ring")));
            return !ring.value<QObject *>();
        }, 2000));
    }

    void theOrbRunSampleIsAGameMadeOfKindsAndLogic()
    {
        const QString path = QDir(QStringLiteral(QQ_SAMPLES)).filePath(QStringLiteral("orb-run/scenes/orb-run.qml"));
        QFile file(path);
        QVERIFY(file.open(QIODevice::ReadOnly));
        const QString text = QString::fromUtf8(file.readAll());
        const QUrl url = QUrl::fromLocalFile(path);

        // Kept as the Studio keeps a scene: written back, it is the same bytes.
        {
            Stage editing;
            QVERIFY(editing.open(text, false, url));
            QCOMPARE(editing.io->write(editing.scene, url), text);
        }

        // What it keeps between plays (P3.16), from a store its player has scribbled in: a best that is no number.
        qq::GameStore *kept = qq::GameStore::storeFor(qq::Confinement::packageOf(url));
        QVERIFY(kept);
        kept->clear();
        QVERIFY(kept->setValue(QStringLiteral("best"), QJSValue(QStringLiteral("lots"))));

        Stage s;
        QVERIFY(s.open(text, true, url));
        QObject *rules = s.entity(QStringLiteral("Rules"));
        auto logic = [rules] { return rules->property("logic").value<QObject *>(); };
        auto score = [&] { return logic() ? logic()->property("score").toInt() : -1; };
        QVERIFY2(s.runUntil([&] { return logic() && logic()->property("orbsOut").toInt() == 2; }, 5000),
                 qPrintable(rules->property("error").toString()));
        QCOMPARE(s.entity(QStringLiteral("Best"))->property("text").toString(), QStringLiteral("BEST 0"));
        // Each orb a prefab (P3.15): its glow and its pickup are its parts, not the scene's own entities.
        auto partOf = [&s](const QString &prefab, const QString &name) {
            QObject *p = s.entity(prefab);
            QVariant found;
            if (p)
                QMetaObject::invokeMethod(p, "part", Q_RETURN_ARG(QVariant, found), Q_ARG(QVariant, name));
            return found.value<QObject *>();
        };
        QVERIFY(s.entity(QStringLiteral("Orb 1")) && partOf(QStringLiteral("Orb 1"), QStringLiteral("Glow"))
                && partOf(QStringLiteral("Orb 1"), QStringLiteral("Pickup")));
        QCOMPARE(s.entity(QStringLiteral("Orb 1"))->property("kind").toString(), QStringLiteral("Prefab"));
        QVERIFY(!s.entity(QStringLiteral("Glow")));
        // The walls are a group (P3.15).
        QCOMPARE(s.entity(QStringLiteral("North wall"))->parent(), s.entity(QStringLiteral("Walls")));
        QCOMPARE(score(), 0);

        // Its looks (P3.14): stone walls with their relief, a slate floor, two spot lights and a dusk sky, every file
        // its own (make_looks.py writes them).
        auto picture = [](QObject *e, const char *which) {
            return e->property(which).value<QObject *>()->property("source").toUrl().toLocalFile();
        };
        for (const char *wall : {"North wall", "South wall", "East wall", "West wall"}) {
            QObject *w = s.entity(QLatin1String(wall));
            QVERIFY2(picture(w, "picture").endsWith(QStringLiteral("orb-run/assets/stone.png")), wall);
            QVERIFY2(picture(w, "relief").endsWith(QStringLiteral("orb-run/assets/stone-normal.png")), wall);
        }
        QVERIFY(picture(s.entity(QStringLiteral("Ground")), "picture").endsWith(QStringLiteral("orb-run/assets/floor.png")));
        QCOMPARE(s.entity(QStringLiteral("Arena light"))->property("kind").toString(), QStringLiteral("Spotlight"));
        QCOMPARE(s.entity(QStringLiteral("Wall wash"))->property("coneAngle").toDouble(), 40.0);
        QVERIFY(s.io->resolvedUrl(s.scene, s.scene->property("sky").toUrl()).toLocalFile()
                    .endsWith(QStringLiteral("orb-run/assets/dusk.hdr")));

        // It opens on its title, the world held still behind it (P3.8); Play starts it, and the score is on the HUD.
        auto centre = [](QQuickItem *item) {
            return item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint();
        };
        QQuickItem *title = s.drawn(QStringLiteral("Title"));
        QQuickItem *play = s.drawn(QStringLiteral("Play"));
        QQuickItem *hud = s.drawn(QStringLiteral("Score"));
        QVERIFY(title && play && hud);
        QVERIFY(s.runUntil([&] { return s.scene->property("paused").toBool() && title->opacity() > 0.99; }, 3000));
        QVERIFY(!hud->isVisible());
        s.window->grabWindow().save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-orb-run-title.png")));
        // Its music (P3.13): the title's track from the start, while the world is held.
        QObject *music = s.entity(QStringLiteral("Music"));
        auto track = [music](int index) {
            QVariant found;
            QMetaObject::invokeMethod(music, "voice", Q_RETURN_ARG(QVariant, found), Q_ARG(QVariant, index));
            QObject *voice = found.value<QObject *>();
            return voice ? voice->property("source").toUrl().fileName() : QString();  // none: let go
        };
        auto current = [music] { return music->property("current").toInt(); };
        QVERIFY(s.runUntil([&] { return track(current()) == QStringLiteral("title.wav"); }, 3000));
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(play));
        QVERIFY(s.runUntil([&] { return !s.scene->property("paused").toBool() && !title->isVisible() && hud->isVisible(); },
                           3000));
        // Play brings in the arena's track as the title's fades out, and the title's is let go.
        QCOMPARE(track(current()), QStringLiteral("arena.wav"));
        QVERIFY(s.runUntil([&] { return track(1 - current()).isEmpty(); }, 4000));

        // Walk straight ahead into the first orb: gathered, taken away with its glow, and another placed.
        s.run(400);
        QTest::keyPress(s.window.get(), Qt::Key_W);
        const bool gathered = s.runUntil([&] { return score() >= 1; }, 6000);
        QTest::keyRelease(s.window.get(), Qt::Key_W);
        QVERIFY2(gathered, "the player walked through the first orb without gathering it");
        QCOMPARE(s.entity(QStringLiteral("Score"))->property("text").toString(), QStringLiteral("ORBS 1"));
        // The best play is kept and shown; what the player scribbled counted as nothing.
        QCOMPARE(s.entity(QStringLiteral("Best"))->property("text").toString(), QStringLiteral("BEST 1"));
        QCOMPARE(kept->value(QStringLiteral("best")).toInt(), 1);
        // A chime where the orb was taken (P3.13), put away once it has played through.
        QObject *voices = s.scene->property("_voices").value<QObject *>();
        auto sounding = [voices](const QString &file) {
            for (const QVariant &v : voices->property("active").toList())
                if (QObject *voice = v.value<QObject *>(); voice && voice->property("source").toUrl().fileName() == file)
                    return voice;
            return static_cast<QObject *>(nullptr);
        };
        QPointer<QObject> chime = sounding(QStringLiteral("chime.wav"));
        QVERIFY(chime);
        QCOMPARE(chime->property("scenePosition").value<QVector3D>(), QVector3D(0, 1, -4));
        // Its lanterns, an animated glTF model, flare as the orb is taken and then sway again (P3.12).
        QObject *lantern = s.entity(QStringLiteral("Lantern west"));
        QCOMPARE(lantern->property("status").toInt(), 1);
        QCOMPARE(lantern->property("clips").toStringList(), (QStringList{QStringLiteral("idle"), QStringLiteral("flare")}));
        QCOMPARE(lantern->property("clip").toString(), QStringLiteral("flare"));
        QVERIFY(s.runUntil([&] { return lantern->property("clip").toString() == QStringLiteral("idle"); }, 3000));
        QTRY_VERIFY(!s.entity(QStringLiteral("Orb 1")));  // taken away whole, glow and pickup
        QVERIFY(s.runUntil([&] { return !chime; }, 3000));
        QVERIFY2(s.runUntil([&] { return s.entity(QStringLiteral("Orb 3")) != nullptr; }, 4000), "no new orb");
        // A new orb grows in to its full size.
        QObject *grown = partOf(QStringLiteral("Orb 3"), QStringLiteral("Glow"));
        QVERIFY(s.runUntil([&] { return qAbs(grown->property("scale").value<QVector3D>().x() - 0.6f) < 1e-4f; }, 3000));
        QCOMPARE(logic()->property("orbsOut").toInt(), 2);

        // The boulder is rolled at the player, with a rumble from where it was.
        const QVector3D boulder = s.where(QStringLiteral("Boulder"));
        bool rumbled = false;
        QVERIFY(s.runUntil([&] {
            rumbled = rumbled || sounding(QStringLiteral("rumble.wav"));
            return rumbled && (s.where(QStringLiteral("Boulder")) - boulder).length() > 1.0f;
        }, 6000));
        s.window->grabWindow().save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-orb-run.png")));

        // Pause holds it all, behind the pause screen; Resume lets it go on.
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(s.drawn(QStringLiteral("Pause"))));
        QQuickItem *paused = s.drawn(QStringLiteral("Paused"));
        QVERIFY(s.runUntil([&] { return s.scene->property("paused").toBool() && paused->opacity() > 0.99; }, 3000));
        s.window->grabWindow().save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-orb-run-paused.png")));
        // The music plays on through the pause; its switch turns the music group off, and on again.
        auto *mixer = s.engine.singletonInstance<qq::Mixer *>("QQ", "Mixer");
        auto level = [&] {
            QVariant found;
            QMetaObject::invokeMethod(music, "voice", Q_RETURN_ARG(QVariant, found), Q_ARG(QVariant, current()));
            return found.value<QObject *>() ? found.value<QObject *>()->property("volume").toDouble() : -1.0;
        };
        QVERIFY(qAbs(level() - 0.6) < 1e-6);
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(s.drawn(QStringLiteral("Music switch"))));
        QVERIFY(s.runUntil([&] { return mixer->level(QStringLiteral("music")) == 0; }, 2000));
        QCOMPARE(level(), 0.0);
        QCOMPARE(s.entity(QStringLiteral("Music switch"))->property("text").toString(), QStringLiteral("Music: off"));
        QCOMPARE(mixer->level(QStringLiteral("effects")), 1.0);
        QCOMPARE(kept->value(QStringLiteral("music")).toInt(), 0);  // kept for the next play
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(s.drawn(QStringLiteral("Music switch"))));
        QVERIFY(s.runUntil([&] { return mixer->level(QStringLiteral("music")) == 1; }, 2000));
        QVERIFY(qAbs(level() - 0.6) < 1e-6);
        QCOMPARE(kept->value(QStringLiteral("music")).toInt(), 1);
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(s.drawn(QStringLiteral("Resume"))));
        QVERIFY(s.runUntil([&] { return !s.scene->property("paused").toBool() && !paused->isVisible(); }, 3000));

        // Its pause action (P3.9): P holds the world and shows the pause screen, and P again lets it go on.
        QTest::keyClick(s.window.get(), Qt::Key_P);
        QVERIFY(s.runUntil([&] { return s.scene->property("paused").toBool() && paused->isVisible(); }, 3000));
        QTest::keyClick(s.window.get(), Qt::Key_P);
        QVERIFY(s.runUntil([&] { return !s.scene->property("paused").toBool() && !paused->isVisible(); }, 3000));
    }

    void theEmberClimbSampleIsA2DGameMadeOfKindsAndLogic()
    {
        const QString path = QDir(QStringLiteral(QQ_SAMPLES)).filePath(QStringLiteral("ember-climb/scenes/ember-climb.qml"));
        QFile file(path);
        QVERIFY(file.open(QIODevice::ReadOnly));
        const QString text = QString::fromUtf8(file.readAll());
        const QUrl url = QUrl::fromLocalFile(path);

        // Kept as the Studio keeps a scene: written back, it is the same bytes.
        {
            Stage editing;
            QVERIFY(editing.open(text, false, url));
            QCOMPARE(editing.io->write(editing.scene, url), text);
        }

        // Played and kept (P3.11): its first part is replayed at the end, with no window.
        qq::RunRecord run;
        Stage s;
        QVERIFY(s.open(text, false, url));
        s.scene->setProperty("recorder", QVariant::fromValue(static_cast<QObject *>(&run)));
        s.scene->setProperty("playing", true);
        s.flat(16, 12, QVector3D(8, 6, 0));
        QObject *rules = s.entity(QStringLiteral("Rules"));
        auto logic = [rules] { return rules->property("logic").value<QObject *>(); };
        auto gathered = [&] { return logic() ? logic()->property("gathered").toInt() : -1; };
        // Its level is drawn from Tiled, both layers, and the solid one stands.
        QCOMPARE(s.entity(QStringLiteral("Level"))->property("error").toString(), QString());
        QVERIFY(s.entity(QStringLiteral("Level"))->property("count").toInt() > 100);
        QVERIFY(s.entity(QStringLiteral("Wall"))->property("count").toInt() > 400);
        QVERIFY2(s.runUntil([&] { return s.entity(QStringLiteral("Ember 8")) != nullptr; }, 5000),
                 qPrintable(rules->property("error").toString()));
        QCOMPARE(s.entity(QStringLiteral("Embers"))->property("text").toString(), QStringLiteral("EMBERS 0 / 8"));

        // The eye stops at the level's edges: at the start, a view 20 by 12 units has its left and foot on the level's.
        auto keptInView = [&s](const QVector3D &point, double halfWidth, double halfHeight) {
            QVariant kept;
            QMetaObject::invokeMethod(s.scene, "keptInView", Q_RETURN_ARG(QVariant, kept),
                                      Q_ARG(QVariant, QVariant::fromValue(point)), Q_ARG(QVariant, halfWidth),
                                      Q_ARG(QVariant, halfHeight));
            return kept.value<QVector3D>();
        };
        QCOMPARE(keptInView(QVector3D(2.5f, 2.5f, 0), 10, 6), QVector3D(10, 6, 0));
        QCOMPARE(keptInView(QVector3D(24, 8, 0), 10, 6), QVector3D(24, 8, 0));
        // A view wider than the level is centred on it.
        QCOMPARE(keptInView(QVector3D(2.5f, 2.5f, 0), 30, 6), QVector3D(24, 6, 0));

        // It opens on its title, the world held still; Play starts the climb.
        auto centre = [](QQuickItem *item) {
            return item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint();
        };
        QQuickItem *title = s.drawn(QStringLiteral("Title"));
        QQuickItem *play = s.drawn(QStringLiteral("Play"));
        QQuickItem *hud = s.drawn(QStringLiteral("Embers"));
        QVERIFY(title && play && hud);
        QVERIFY(s.runUntil([&] { return s.scene->property("paused").toBool() && title->opacity() > 0.99; }, 3000));
        s.window->grabWindow().save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-ember-climb-title.png")));
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(play));
        QVERIFY(s.runUntil([&] { return !s.scene->property("paused").toBool() && !title->isVisible() && hud->isVisible(); },
                           3000));

        // Run right: through the first ember (gathered, and gone), the hero's look running...
        QObject *hero = s.entity(QStringLiteral("Hero"));
        QObject *look = s.entity(QStringLiteral("Hero look"));
        auto feet = [hero] { return hero->property("feet").value<QVector3D>(); };
        QVERIFY(s.runUntil([&] { return qAbs(feet().y() - 2) < 0.05f; }, 3000));
        QTest::keyPress(s.window.get(), Qt::Key_D);
        QVERIFY(s.runUntil([&] { return look->property("first").toInt() == 4; }, 2000));  // the running frames
        const bool took = s.runUntil([&] { return gathered() == 1; }, 4000);
        QVERIFY2(took, "the hero ran through the first ember without gathering it");
        QCOMPARE(s.entity(QStringLiteral("Embers"))->property("text").toString(), QStringLiteral("EMBERS 1 / 8"));
        QTRY_VERIFY(!s.entity(QStringLiteral("Ember 1")) && !s.entity(QStringLiteral("Ember 1 glow")));
        s.flat(16, 12, hero->property("lookAt").value<QVector3D>());
        s.window->grabWindow().save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-ember-climb.png")));
        // ...and on, into the first pit, and back to the start.
        bool wentFar = false;
        const bool back = s.runUntil([&] {
            wentFar = wentFar || feet().x() > 10;
            return wentFar && feet().x() < 4;
        }, 8000);
        QTest::keyRelease(s.window.get(), Qt::Key_D);
        QVERIFY2(back, qPrintable(QStringLiteral("at %1, %2").arg(feet().x()).arg(feet().y())));

        // P pauses and P goes on (its action, P3.9).
        QQuickItem *paused = s.drawn(QStringLiteral("Paused"));
        QTest::keyClick(s.window.get(), Qt::Key_P);
        QVERIFY(s.runUntil([&] { return s.scene->property("paused").toBool() && paused->isVisible(); }, 3000));
        QTest::keyClick(s.window.get(), Qt::Key_P);
        QVERIFY(s.runUntil([&] { return !s.scene->property("paused").toBool() && !paused->isVisible(); }, 3000));

        // The run so far is what the keyboard and the mouse did, so it is kept here: what follows places the hero by hand.
        QMetaObject::invokeMethod(s.scene, "finishRecord");
        s.scene->setProperty("recorder", QVariant());
        const QVariantMap kept = run.outcome().value(QStringLiteral("Rules")).toMap();
        QCOMPARE(kept.value(QStringLiteral("gathered")).toInt(), 1);

        // Every other ember, the hero put on each in turn: the forge is lit, the time shown, the world held.
        QObject *body = hero->property("physics").value<QObject *>();
        QVERIFY(body);
        for (int i = 2; i <= 8 && gathered() < 8; ++i) {
            QObject *ember = s.entity(QStringLiteral("Ember %1").arg(i));
            QVERIFY(ember);
            const QVector3D at = ember->property("position").value<QVector3D>();
            QMetaObject::invokeMethod(hero, "place", Q_ARG(QVariant, QVariant::fromValue(QVector3D(at.x(), at.y(), 0))));
            QVERIFY2(s.runUntil([&] { return gathered() >= i; }, 3000), qPrintable(QStringLiteral("ember %1").arg(i)));
        }
        QQuickItem *lit = s.drawn(QStringLiteral("Lit"));
        QVERIFY(s.runUntil([&] { return s.scene->property("paused").toBool() && lit->opacity() > 0.99; }, 3000));
        QVERIFY(s.entity(QStringLiteral("Took"))->property("text").toString().startsWith(QStringLiteral("Climbed in 0:")));
        s.window->grabWindow().save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-ember-climb-won.png")));

        // Again: the embers back, none gathered, the hero at the start, climbing.
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(s.drawn(QStringLiteral("Again"))));
        QVERIFY(s.runUntil([&] { return !s.scene->property("paused").toBool() && hud->isVisible() && !lit->isVisible(); }, 3000));
        QCOMPARE(gathered(), 0);
        QVERIFY(s.runUntil([&] { return s.entity(QStringLiteral("Ember 8")) != nullptr; }, 2000));
        QVERIFY(s.runUntil([&] { return feet().x() < 4; }, 2000));

        // The kept run, played again with no window and no physics world: the title's Play, the first ember, the pit and
        // the pause, to the same outcome.
        QQmlEngine engine;
        const QVariantMap again = qq::replayRun(&engine, url, &run);
        QCOMPARE(again.value(QStringLiteral("problem")).toString(), QString());
        QVERIFY2(again.value(QStringLiteral("matches")).toBool(),
                 qPrintable(QJsonDocument(QJsonObject::fromVariantMap(again)).toJson(QJsonDocument::Compact) + " vs "
                            + QJsonDocument(QJsonObject::fromVariantMap(run.outcome())).toJson(QJsonDocument::Compact)));
    }

    // ── steps, seeds and replay (P3.11, ADR-105) ──

    void logicStepsSixtyTimesASecondWhateverTheFrameRate()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/clock.qml")),
                  "import QQ\nLogic {\n    property int stepped: 0\n    property int fired: 0\n    property int frames: 0\n"
                  "    onStarted: every(0.25, () => fired += 1)\n"
                  "    onStep: (dt) => stepped += 1\n"
                  "    onFrame: (dt) => frames += 1\n}\n");
        const QString text = sceneText(QStringLiteral("    Behaviour { name: \"Clock\"; source: \"../logic/clock.qml\" }\n"));
        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")));

        // Two seconds of play, drawn at 30, 60 and 144 frames a second and at uneven ones: always 120 steps, and the
        // timer every quarter second fires eight times.
        const QList<QList<double>> rates{QList<double>(60, 1.0 / 30), QList<double>(120, 1.0 / 60),
                                         QList<double>(288, 1.0 / 144)};
        QList<double> uneven;
        for (int i = 0; i < 100; ++i)
            uneven << (i % 2 ? 0.031 : 0.009);
        for (const QList<double> &frames : rates + QList<QList<double>>{uneven}) {
            Headless h;
            QVERIFY(h.open(text, file));
            int ran = 0;
            for (double frame : frames)
                ran += h.advance(frame);
            QObject *clock = h.logicOf(QStringLiteral("Clock"));
            QVERIFY(clock);
            QCOMPARE(ran, 120);
            QCOMPARE(h.scene->property("steps").toInt(), 120);
            QCOMPARE(clock->property("stepped").toInt(), 120);
            QCOMPARE(clock->property("fired").toInt(), 8);
            QCOMPARE(clock->property("elapsed").toDouble(), 2.0);
            QCOMPARE(clock->property("frames").toInt(), 0);  // no window, no frames drawn
        }

        // A frame half a second long is at most eight steps: the game slows rather than leaps.
        Headless h;
        QVERIFY(h.open(text, file));
        QCOMPARE(h.advance(0.5), 8);
        // Paused, the clock still ticks (screens answer) but takes no steps.
        h.scene->setProperty("paused", true);
        QCOMPARE(h.advance(0.1), 6);
        QCOMPARE(h.scene->property("steps").toInt(), 8);
        QCOMPARE(h.scene->property("ticks").toInt(), 14);
    }

    void randomNumbersFollowTheSeed()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/dice.qml")),
                  "import QQ\nLogic {\n    property var rolls: []\n"
                  "    onStarted: { const r = []; for (let i = 0; i < 10; ++i) r.push(randomInt(1, 6)); rolls = r }\n"
                  "    outcome: rolls\n}\n");
        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")));
        auto rollsWith = [&](int seed, int *played = nullptr) {
            Headless h;
            const bool opened = h.open(sceneText(QStringLiteral("    Behaviour { name: \"Dice\"; source: \"../logic/dice.qml\" }\n"),
                                                 QStringLiteral("    seed: %1\n").arg(seed)),
                                       file);
            if (!opened)
                return QVariantList();
            h.advance(1.0 / 60);
            if (played)
                *played = h.scene->property("playSeed").toInt();
            return h.logicOf(QStringLiteral("Dice"))->property("rolls").toList();
        };
        // The same rolls on every machine for seed 42 (tst_scene pins the generator itself).
        const QVariantList fortyTwo = rollsWith(42);
        QCOMPARE(fortyTwo, (QVariantList{4, 1, 1, 1, 6, 2, 6, 5, 3, 4}));
        QCOMPARE(rollsWith(42), fortyTwo);
        QVERIFY(rollsWith(43) != fortyTwo);
        // Seed 0 draws a new seed for each play, and says which.
        int first = 0;
        int second = 0;
        rollsWith(0, &first);
        rollsWith(0, &second);
        QVERIFY(first > 0 && second > 0 && first != second);
    }

    void aRunIsRecordedAndReplayedWithNoWindowToTheSameOutcome()
    {
        // A small game of chance and skill: three coins at spots chosen by the seed, a hero who runs and jumps for
        // them on a Tiled level, and a timer. Its outcome is everything a run decides, the hero's exact place among it.
        FlatProject project;
        writeFile(QDir(project.dir.path()).filePath(QStringLiteral("logic/coins.qml")),
                  "import QQ\nLogic {\n    property int coins: 0\n    property int ticksAt: 0\n    property var spots: []\n"
                  "    onStarted: {\n"
                  "        const s = []\n"
                  "        for (let i = 0; i < 3; ++i) {\n"
                  "            const at = Qt.vector3d(-4 + randomInt(0, 7), 1.5, 0)\n"
                  "            s.push(at.x)\n"
                  "            const coin = spawn(\"Trigger\", { name: \"Coin \" + i, position: at, scale: Qt.vector3d(0.5, 0.5, 1) })\n"
                  "            coin.entered.connect((other) => { if (other && other.kind === \"Player2D\" && !removing(coin)) {\n"
                  "                coins += 1; remove(coin) } })\n"
                  "        }\n"
                  "        spots = s\n"
                  "        every(0.5, () => ticksAt += 1)\n"
                  "    }\n"
                  "    outcome: ({ coins: coins, timer: ticksAt, spots: spots, x: scene.player ? scene.player.atX : 0,\n"
                  "               y: scene.player ? scene.player.atY : 0, doing: scene.player ? scene.player.activity : \"\" })\n"
                  "}\n");
        const QString text = flatText(QStringLiteral(
            "    TileMap { name: \"Level\"; position: Qt.vector3d(-5, 5, 0); source: \"../assets/level.tmj\" }\n"
            "    Player2D { name: \"Hero\"; position: Qt.vector3d(-4, 1, 0); jumpHeight: 1.5 }\n"
            "    Behaviour { name: \"Coins\"; source: \"../logic/coins.qml\" }\n"), QStringLiteral("    seed: 1234\n"));
        // Kept where a replay reads it, as a game's scene is.
        writeFile(project.scene().toLocalFile(), text.toUtf8());

        // Played in a window, at whatever rate it draws, with the keyboard, and kept.
        qq::RunRecord run;
        {
            Stage s;
            QVERIFY(s.open(text, false, project.scene()));
            s.flat(16, 7, QVector3D(0, 2.5f, 0));
            s.scene->setProperty("recorder", QVariant::fromValue(static_cast<QObject *>(&run)));
            s.scene->setProperty("playing", true);
            QObject *hero = s.entity(QStringLiteral("Hero"));
            auto feet = [hero] { return hero->property("feet").value<QVector3D>(); };
            QVERIFY(s.runUntil([&] { return hero->property("grounded").toBool(); }, 4000));
            QTest::keyPress(s.window.get(), Qt::Key_D);
            s.run(700);
            QTest::keyPress(s.window.get(), Qt::Key_Space);
            s.run(250);
            QTest::keyRelease(s.window.get(), Qt::Key_Space);
            s.run(600);
            QTest::keyRelease(s.window.get(), Qt::Key_D);
            QTest::keyPress(s.window.get(), Qt::Key_A);
            s.run(500);
            QTest::keyRelease(s.window.get(), Qt::Key_A);
            s.run(300);
            QVERIFY(feet().x() > -4.5f);
            QMetaObject::invokeMethod(s.scene, "finishRecord");
            QCOMPARE(run.seed(), 1234);
            QVERIFY(run.ticks() > 100);
            const QVariantMap kept = run.outcome().value(QStringLiteral("Coins")).toMap();
            QCOMPARE(kept.value(QStringLiteral("spots")).toList().size(), 3);
            QCOMPARE(kept.value(QStringLiteral("timer")).toInt(), run.ticks() / 30);  // every half second of steps
            QCOMPARE(kept.value(QStringLiteral("x")).toDouble(), double(hero->property("atX").toDouble()));
        }

        // Played again with no window and no physics world: the same outcome, to the last bit of the hero's place.
        QQmlEngine engine;
        QVariantMap again = qq::replayRun(&engine, project.scene(), &run);
        QCOMPARE(again.value(QStringLiteral("problem")).toString(), QString());
        QCOMPARE(again.value(QStringLiteral("ticks")).toInt(), run.ticks());
        QVERIFY2(again.value(QStringLiteral("matches")).toBool(),
                 qPrintable(QJsonDocument(QJsonObject::fromVariantMap(again)).toJson(QJsonDocument::Compact) + " vs "
                            + QJsonDocument(QJsonObject::fromVariantMap(run.outcome())).toJson(QJsonDocument::Compact)));
        // And once more, read back from the file a run is kept in.
        qq::RunRecord reread;
        QVERIFY(reread.fromJson(run.toJson()));
        QVERIFY(qq::replayRun(&engine, project.scene(), &reread).value(QStringLiteral("matches")).toBool());

        // A run told differently does not reach it: another seed, or hands that never went right.
        qq::RunRecord otherSeed;
        QVERIFY(otherSeed.fromJson(run.toJson()));
        otherSeed.setSeed(4321);
        QVERIFY(!qq::replayRun(&engine, project.scene(), &otherSeed).value(QStringLiteral("matches")).toBool());
        QJsonObject edited = QJsonDocument::fromJson(run.toJson()).object();
        edited.insert(QStringLiteral("hands"), QJsonArray());
        qq::RunRecord idle;
        QVERIFY(idle.fromJson(QJsonDocument(edited).toJson()));
        QVERIFY(!qq::replayRun(&engine, project.scene(), &idle).value(QStringLiteral("matches")).toBool());
    }

    void aTriggerHearsA2DPlayerByQQsOwnMeasure()
    {
        // Replayed, so with no window and no physics world: what reaches the trigger is QQ's kinematics and its own
        // measure, and nothing of PhysX's. The run: the hero holds right for a second.
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/gate.qml")),
                  "import QQ\nLogic {\n    property var log: []\n"
                  "    onEntered: (other) => log = log.concat([\"in \" + other.name])\n"
                  "    onExited: (other) => log = log.concat([\"out \" + other.name])\n"
                  "    outcome: ({ log: log, x: entity(\"Hero\").atX, y: entity(\"Hero\").atY,\n"
                  "               physics: scene.physicsWorld !== null })\n}\n");
        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")));
        writeFile(file.toLocalFile(), flatText(QStringLiteral(
                      "    Shape { name: \"Floor\"; position: Qt.vector3d(0, -0.5, 0); scale: Qt.vector3d(40, 1, 2) }\n"
                      "    Trigger { name: \"Gate\"; position: Qt.vector3d(3, 0.5, 0) }\n"
                      "    Player2D { name: \"Hero\"; position: Qt.vector3d(0, 0, 0); speed: 5 }\n"
                      "    Behaviour { name: \"Gate log\"; source: \"../logic/gate.qml\"; target: \"Gate\" }\n")).toUtf8());
        qq::RunRecord run;
        run.setSeed(1);
        run.capture(0, {{QStringLiteral("move"), QVariantList{1.0, 0.0}}, {QStringLiteral("look"), QVariantList{0.0, 0.0}},
                        {QStringLiteral("held"), QStringList()}, {QStringLiteral("edges"), QVariantList()}}, {});
        run.setTicks(60);
        QQmlEngine engine;
        const QVariantMap result = qq::replayRun(&engine, file, &run);
        QCOMPARE(result.value(QStringLiteral("problem")).toString(), QString());
        const QVariantMap gate = result.value(QStringLiteral("outcome")).toMap().value(QStringLiteral("Gate log")).toMap();
        // At 5 units a second for 60 steps: in through the gate (2.5 to 3.5) and out the far side, standing on the floor.
        QCOMPARE(gate.value(QStringLiteral("log")).toStringList(), (QStringList{QStringLiteral("in Hero"), QStringLiteral("out Hero")}));
        QCOMPARE(gate.value(QStringLiteral("x")).toDouble(), 5.0);
        QCOMPARE(gate.value(QStringLiteral("y")).toDouble(), 0.5);
        QCOMPARE(gate.value(QStringLiteral("physics")).toBool(), false);  // no physics world was made
    }

    // ── saving (P3.16, ADR-105) ──

    void aGameKeepsWhatItSavesFromOnePlayToTheNext()
    {
        // A game that counts its plays and keeps a profile the first time; what it cannot keep, it is told.
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/keeper.qml")),
                  "import QQ\nLogic {\n    property int plays: 0\n    property string name: \"\"\n"
                  "    property bool refused: false\n"
                  "    onStarted: {\n"
                  "        plays = saved(\"plays\", 0) + 1\n"
                  "        save(\"plays\", plays)\n"
                  "        const profile = saved(\"profile\", null)\n"
                  "        name = profile ? profile.name : \"\"\n"
                  "        if (!profile)\n"
                  "            save(\"profile\", { name: \"Ada\", levels: [1, 2, 3] })\n"
                  "        refused = !save(\"where\", Qt.vector3d(1, 2, 3))\n"
                  "    }\n}\n");
        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")));
        const QString text = sceneText(QStringLiteral("    Behaviour { name: \"Keeper\"; source: \"../logic/keeper.qml\"; target: \"\" }\n"));
        writeFile(file.toLocalFile(), text.toUtf8());
        qq::GameStore *store = qq::GameStore::storeFor(QUrl::fromLocalFile(project.path()));
        QVERIFY(store && store->keys().isEmpty());

        for (int play = 1; play <= 2; ++play) {
            Headless h;
            QVERIFY(h.open(text, file));
            h.advance(0.1);
            QObject *logic = h.logicOf(QStringLiteral("Keeper"));
            QVERIFY(logic);
            QCOMPARE(logic->property("plays").toInt(), play);
            QCOMPARE(logic->property("name").toString(), play == 1 ? QString() : QStringLiteral("Ada"));
            QVERIFY(logic->property("refused").toBool());
        }
        // On disk, as the game's next start reads it; what was refused is not there.
        store->flush();
        qq::GameStore fromFile(store->game());
        QCOMPARE(fromFile.value(QStringLiteral("plays")).toInt(), 2);
        QCOMPARE(fromFile.keys(), (QStringList{QStringLiteral("plays"), QStringLiteral("profile")}));
        QCOMPARE(fromFile.value(QStringLiteral("profile")).toMap().value(QStringLiteral("levels")).toList().size(), 3);
        store->clear();
        store->flush();
    }

    void aReplayPlaysFromTheStoreAsItsRunFoundItAndKeepsNothing()
    {
        // A game that reads what it kept and keeps one more: its outcome rests on the store.
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/count.qml")),
                  "import QQ\nLogic {\n    property int found: -1\n"
                  "    onStarted: { found = saved(\"count\", 0); save(\"count\", found + 1) }\n"
                  "    outcome: ({ found: found, now: saved(\"count\", -1), steps: steps })\n}\n");
        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")));
        const QString text = sceneText(QStringLiteral("    Behaviour { name: \"Counter\"; source: \"../logic/count.qml\"; target: \"\" }\n"));
        writeFile(file.toLocalFile(), text.toUtf8());
        qq::GameStore *store = qq::GameStore::storeFor(QUrl::fromLocalFile(project.path()));
        QVERIFY(store->setValue(QStringLiteral("count"), QJSValue(5)));

        // Played and kept: the run found 5 and left 6, and its record has the store as it found it.
        qq::RunRecord run;
        {
            Headless h;
            QVERIFY(h.open(text, file, &run));
            h.advance(0.5);
            QMetaObject::invokeMethod(h.scene, "finishRecord");
        }
        QCOMPARE(run.store().value(QStringLiteral("count")).toInt(), 5);
        const QVariantMap kept = run.outcome().value(QStringLiteral("Counter")).toMap();
        QCOMPARE(kept.value(QStringLiteral("found")).toInt(), 5);
        QCOMPARE(kept.value(QStringLiteral("now")).toInt(), 6);
        QCOMPARE(store->value(QStringLiteral("count")).toInt(), 6);

        // The store moves on (another play, or the player's own hand); the replay still plays from what the run found,
        // and reaches its outcome.
        QVERIFY(store->setValue(QStringLiteral("count"), QJSValue(100)));
        store->flush();
        QQmlEngine engine;
        const QVariantMap again = qq::replayRun(&engine, file, &run);
        QCOMPARE(again.value(QStringLiteral("problem")).toString(), QString());
        QVERIFY2(again.value(QStringLiteral("matches")).toBool(),
                 qPrintable(QJsonDocument(QJsonObject::fromVariantMap(again)).toJson(QJsonDocument::Compact)));
        // And keeps nothing: the store is as it was, in memory and on disk.
        QCOMPARE(store->value(QStringLiteral("count")).toInt(), 100);
        store->flush();
        QCOMPARE(qq::GameStore(store->game()).value(QStringLiteral("count")).toInt(), 100);

        // Read back from its file, the run carries its store; told it found another, it does not reach its outcome.
        qq::RunRecord reread;
        QVERIFY(reread.fromJson(run.toJson()));
        QVERIFY(qq::replayRun(&engine, file, &reread).value(QStringLiteral("matches")).toBool());
        reread.setStore({{QStringLiteral("count"), 7}});
        QVERIFY(!qq::replayRun(&engine, file, &reread).value(QStringLiteral("matches")).toBool());
        store->clear();
        store->flush();
    }

    // ── the game's host (ADR-117) ──

    void aGameAsksItsHostAndHearsWhatItIsToldAtTheNextTickAndAReplayHearsItToo()
    {
        // A game that asks as it starts, and keeps what it hears and the step it heard it at: its outcome.
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/ask.qml")),
                  "import QQ\nLogic {\n    property var heard: []\n"
                  "    onStarted: askHost({ ask: \"hello\", n: 1 })\n"
                  "    onTold: (message) => heard = heard.concat([{ at: steps, message: message,\n"
                  "        lists: Array.isArray(message.names) && Array.isArray(message.deep.more) }])\n"
                  "    outcome: ({ heard: heard })\n}\n");
        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")));
        const QString text = sceneText(QStringLiteral("    Behaviour { name: \"Asker\"; source: \"../logic/ask.qml\"; target: \"\" }\n"));
        writeFile(file.toLocalFile(), text.toUtf8());

        qq::RunRecord run;
        {
            Headless h;
            QVERIFY(h.open(text, file, &run));
            QSignalSpy asked(h.scene, SIGNAL(asked(QVariant)));
            h.advance(1.0 / 60);
            // Asked at once, a JSON value as the game wrote it.
            QCOMPARE(asked.size(), 1);
            QVariant question = asked.first().first();
            if (question.canConvert<QJSValue>())
                question = question.value<QJSValue>().toVariant();
            QCOMPARE(question.toMap().value(QStringLiteral("ask")).toString(), QStringLiteral("hello"));
            QCOMPARE(question.toMap().value(QStringLiteral("n")).toInt(), 1);

            // Told, it hears nothing until the next tick, which delivers it.
            auto tell = [&h](const QVariant &message) {
                QVariant ok;
                QMetaObject::invokeMethod(h.scene, "tell", Q_RETURN_ARG(QVariant, ok), Q_ARG(QVariant, message));
                return ok.toBool();
            };
            QVERIFY(tell(QVariantMap{{QStringLiteral("answer"), 42}, {QStringLiteral("names"), QStringList{"a", "b"}},
                                     {QStringLiteral("deep"), QVariantMap{{QStringLiteral("more"), QVariantList{1, 2}}}}}));
            QObject *logic = h.logicOf(QStringLiteral("Asker"));
            QVERIFY(logic);
            auto heard = [logic] {
                const QVariant v = logic->property("heard");
                return v.canConvert<QJSValue>() ? v.value<QJSValue>().toVariant().toList() : v.toList();
            };
            QCOMPARE(heard().size(), 0);
            h.advance(1.0 / 60);
            QCOMPARE(heard().size(), 1);
            // Heard in the second tick, before its step: the logic has taken one step.
            const QVariantMap first = heard().first().toMap();
            QCOMPARE(first.value(QStringLiteral("at")).toInt(), 1);
            QCOMPARE(first.value(QStringLiteral("message")).toMap().value(QStringLiteral("answer")).toInt(), 42);
            // A JSON value as JavaScript has one: its lists, however deep, are arrays.
            QVERIFY(first.value(QStringLiteral("lists")).toBool());

            // More than 16 KiB is refused, and nothing is told.
            QVERIFY(!tell(QString(20000, QLatin1Char('x'))));
            QVERIFY(tell(QStringLiteral("again")));
            h.advance(5.0 / 60);
            QCOMPARE(heard().size(), 2);
            QCOMPARE(heard().last().toMap().value(QStringLiteral("message")).toString(), QStringLiteral("again"));
            QMetaObject::invokeMethod(h.scene, "finishRecord");
        }

        // The run keeps what was told, at the ticks it was heard, so its replay reaches the same outcome, from its file too.
        QQmlEngine engine;
        const QVariantMap again = qq::replayRun(&engine, file, &run);
        QCOMPARE(again.value(QStringLiteral("problem")).toString(), QString());
        QVERIFY2(again.value(QStringLiteral("matches")).toBool(),
                 qPrintable(QJsonDocument(QJsonObject::fromVariantMap(again)).toJson(QJsonDocument::Compact)));
        qq::RunRecord reread;
        QVERIFY(reread.fromJson(run.toJson()));
        QCOMPARE(reread.toldAt(1).size(), 1);
        QVERIFY(qq::replayRun(&engine, file, &reread).value(QStringLiteral("matches")).toBool());

        // Without what it was told, the run reaches another outcome.
        QJsonObject withoutTold = QJsonDocument::fromJson(run.toJson()).object();
        withoutTold.remove(QStringLiteral("told"));
        qq::RunRecord deaf;
        QVERIFY(deaf.fromJson(QJsonDocument(withoutTold).toJson()));
        QVERIFY(!qq::replayRun(&engine, file, &deaf).value(QStringLiteral("matches")).toBool());
    }

    // ── animation (P3.12, ADR-105) ──

    void aModelPlaysItsGltfClipsOnTheGamesClock()
    {
        // tests/fixtures/rig.gltf (make_rig.py): a bar skinned to Root and Tip, which "bend" turns 90 degrees at 1 s and
        // back at 2 s, and a box whose morph target "puff" doubles it over 1 s.
        QTemporaryDir project;
        QDir(project.path()).mkpath(QStringLiteral("assets"));
        QVERIFY(QFile::copy(fixture(QStringLiteral("rig.gltf")), QDir(project.path()).filePath(QStringLiteral("assets/rig.gltf"))));
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Shape { name: \"Wall\"; position: Qt.vector3d(0, 1, -2); scale: Qt.vector3d(10, 6, 0.2); "
                           "colour: \"#101418\"; body: \"none\" }\n"
                           "    Prop { name: \"Rig\"; source: \"../assets/rig.gltf\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        s.world->setProperty("eye", QVector3D(0.5f, 1, 5));
        s.world->setProperty("target", QVector3D(0.5f, 1, 0));
        s.scene->setProperty("autoAdvance", false);  // the clock is ticked by hand here, step by step
        QObject *rig = s.entity(QStringLiteral("Rig"));
        QVERIFY(s.runUntil([rig] { return rig->property("status").toInt() == 1; }, 5000));
        QCOMPARE(rig->property("clips").toStringList(), (QStringList{QStringLiteral("bend"), QStringLiteral("puff")}));
        auto part = [rig](const QString &name) {
            QVariant found;
            QMetaObject::invokeMethod(rig, "part", Q_RETURN_ARG(QVariant, found), Q_ARG(QVariant, name));
            return found.value<QObject *>();
        };
        auto ticks = [&s](int n) {
            for (int i = 0; i < n; ++i)
                QMetaObject::invokeMethod(s.scene, "tick");
            s.run(80);  // a few frames, so the pose is drawn
        };
        QObject *tip = part(QStringLiteral("Tip"));
        QVERIFY(tip);
        auto turned = [tip] { return tip->property("eulerRotation").value<QVector3D>().z(); };
        // The model is drawn bright in front of a dark wall: a point is the model's when it is bright.
        auto ember = [&s](const QImage &frame, const QVector3D &at) {
            const QColor c = frame.pixelColor(s.devicePixel(at, frame));
            return c.red() + c.green() + c.blue() > 3 * 120;
        };

        // At rest the bar stands upright: its upper half is drawn above the joint, nothing to its left.
        QImage frame = s.window->grabWindow();
        frame.save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-rig-rest.png")));
        QVERIFY(ember(frame, QVector3D(0, 1.7f, 0)));
        QVERIFY(!ember(frame, QVector3D(-0.7f, 1, 0)));

        // "bend", a step at a time: 45 degrees at half a second, 90 at one, and the upper half drawn to the left.
        rig->setProperty("clip", QStringLiteral("bend"));
        QCOMPARE(rig->property("clipLength").toDouble(), 2.0);
        ticks(30);
        QVERIFY2(qAbs(turned() - 45) < 1, qPrintable(QString::number(turned())));
        ticks(30);
        QVERIFY2(qAbs(turned() - 90) < 1, qPrintable(QString::number(turned())));
        frame = s.window->grabWindow();
        QVERIFY(ember(frame, QVector3D(-0.7f, 1, 0)));
        QVERIFY(!ember(frame, QVector3D(0, 1.7f, 0)));
        // Looping: two and a half seconds in is half a second in.
        ticks(90);
        QVERIFY2(qAbs(rig->property("clipTime").toDouble() - 0.5) < 1e-6, qPrintable(rig->property("clipTime").toString()));
        QVERIFY(qAbs(turned() - 45) < 1);
        // Paused, it holds; twice as fast, twice as far.
        s.scene->setProperty("paused", true);
        ticks(30);
        QVERIFY(qAbs(rig->property("clipTime").toDouble() - 0.5) < 1e-6);
        s.scene->setProperty("paused", false);
        rig->setProperty("clipSpeed", 2.0);
        ticks(15);
        QVERIFY(qAbs(rig->property("clipTime").toDouble() - 1.0) < 1e-6);
        QVERIFY(qAbs(turned() - 90) < 1);

        // "puff", once: its morph weight rises to 1 and holds, and the box is drawn twice as big.
        QObject *puff = part(QStringLiteral("Puff"));
        QVERIFY(puff);
        auto weight = [puff] {
            QQmlListReference targets(puff, "morphTargets");
            return targets.count() > 0 ? targets.at(0)->property("weight").toDouble() : -1.0;
        };
        rig->setProperty("clipSpeed", 1.0);
        rig->setProperty("clipLoops", false);
        rig->setProperty("clip", QStringLiteral("puff"));
        QVERIFY(qAbs(turned()) < 1);  // "bend" let go: the bar upright again
        auto gilt = [&s](const QImage &f, const QVector3D &at) {
            const QColor c = f.pixelColor(s.devicePixel(at, f));
            return c.red() + c.green() + c.blue() > 3 * 120;
        };
        frame = s.window->grabWindow();
        QVERIFY(!gilt(frame, QVector3D(1.95f, 1, 0.5f)));
        ticks(30);
        QVERIFY2(qAbs(weight() - 0.5) < 0.02, qPrintable(QString::number(weight())));
        QCOMPARE(rig->property("clipDone").toBool(), false);
        ticks(60);
        QVERIFY2(qAbs(weight() - 1) < 1e-3, qPrintable(QString::number(weight())));
        QCOMPARE(rig->property("clipDone").toBool(), true);
        frame = s.window->grabWindow();
        QVERIFY(gilt(frame, QVector3D(1.95f, 1, 0.5f)));
        s.window->grabWindow().save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-rig.png")));
    }

    void aClipPosesItsJointsOnEachStepInAReplay()
    {
        // Replayed, so with no window and no frames drawn: logic reading a joint in its step reads where the clip has
        // put it on that step, as it would in play.
        QTemporaryDir project;
        const QDir root(project.path());
        root.mkpath(QStringLiteral("assets"));
        QVERIFY(QFile::copy(fixture(QStringLiteral("rig.gltf")), root.filePath(QStringLiteral("assets/rig.gltf"))));
        writeFile(root.filePath(QStringLiteral("logic/watch.qml")),
                  "import QQ\nLogic {\n    property real at30: -1\n    property real at60: -1\n"
                  "    onStep: (dt) => {\n"
                  "        const tip = entity(\"Rig\").part(\"Tip\")\n"
                  "        if (steps === 30) at30 = tip.eulerRotation.z\n"
                  "        if (steps === 60) at60 = tip.eulerRotation.z\n"
                  "    }\n"
                  "    outcome: ({ at30: at30, at60: at60 })\n}\n");
        const QUrl file = QUrl::fromLocalFile(root.filePath(QStringLiteral("scenes/play.qml")));
        writeFile(file.toLocalFile(), sceneText(QStringLiteral(
                      "    Prop { name: \"Rig\"; source: \"../assets/rig.gltf\"; clip: \"bend\" }\n"
                      "    Behaviour { name: \"Watch\"; source: \"../logic/watch.qml\" }\n")).toUtf8());
        qq::RunRecord run;
        run.setSeed(1);
        run.setTicks(60);
        QQmlEngine engine;
        const QVariantMap result = qq::replayRun(&engine, file, &run);
        QCOMPARE(result.value(QStringLiteral("problem")).toString(), QString());
        const QVariantMap watch = result.value(QStringLiteral("outcome")).toMap().value(QStringLiteral("Watch")).toMap();
        QVERIFY2(qAbs(watch.value(QStringLiteral("at30")).toDouble() - 45) < 1, qPrintable(watch.value(QStringLiteral("at30")).toString()));
        QVERIFY2(qAbs(watch.value(QStringLiteral("at60")).toDouble() - 90) < 1, qPrintable(watch.value(QStringLiteral("at60")).toString()));
    }

    void aTweenEasesAnyFieldOnTheGamesClock()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/ease.qml")),
                  "import QQ\nLogic {\n    property var log: []\n    property int stopped: 0\n"
                  "    onStarted: {\n"
                  "        const box = entity(\"Box\")\n"
                  "        tween(box, \"position\", Qt.vector3d(6, 1, 0), 1, \"linear\", () => log = log.concat([\"arrived \" + steps]))\n"
                  "        tween(box, \"colour\", \"#000000\", 0.5, \"outCubic\")\n"
                  "        tween(box, \"metalness\", 1, 0.5, \"inQuad\")\n"
                  "        log = log.concat([\"nothing \" + tween(box, \"nothing\", 1, 1)])\n"
                  "        const crate = entity(\"Crate\")\n"
                  "        tween(crate, \"position\", Qt.vector3d(10, 1, 0), 1, \"linear\", () => log = log.concat([\"crate first\"]))\n"
                  "        after(0.5, () => tween(crate, \"position\", Qt.vector3d(-2, 1, 0), 0.5, \"outBack\"))\n"
                  "        stopped = tween(entity(\"Ball\"), \"position\", Qt.vector3d(0, 5, 0), 1)\n"
                  "        after(0.25, () => cancel(stopped))\n"
                  "    }\n"
                  "    outcome: log\n}\n");
        Headless h;
        QVERIFY(h.open(sceneText(QStringLiteral(
                           "    Shape { name: \"Box\"; position: Qt.vector3d(0, 1, 0); colour: \"#ffffff\"; body: \"none\" }\n"
                           "    Shape { name: \"Crate\"; position: Qt.vector3d(0, 1, 0); body: \"none\" }\n"
                           "    Shape { name: \"Ball\"; position: Qt.vector3d(0, 1, 0); body: \"none\" }\n"
                           "    Behaviour { name: \"Ease\"; source: \"../logic/ease.qml\" }\n")),
                       QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *box = h.io->entities(h.scene).value(2).value<QObject *>();
        QObject *crate = h.io->entities(h.scene).value(3).value<QObject *>();
        QObject *ball = h.io->entities(h.scene).value(4).value<QObject *>();
        QCOMPARE(box->property("name").toString(), QStringLiteral("Box"));
        auto ticks = [&h](int n) {
            for (int i = 0; i < n; ++i)
                QMetaObject::invokeMethod(h.scene, "tick");
        };
        // Half way through each, on the step it should be: linear half way, outCubic seven eighths, inQuad a quarter.
        ticks(15);
        QCOMPARE(box->property("position").value<QVector3D>().x(), 1.5f);
        QVERIFY(qAbs(box->property("colour").value<QColor>().redF() - 0.125) < 0.01);
        QVERIFY(qAbs(box->property("metalness").toDouble() - 0.25) < 1e-6);
        // The ball's tween is cancelled here, where it is.
        const float ballAt = ball->property("position").value<QVector3D>().y();
        QVERIFY(ballAt > 1 && ballAt < 5);
        ticks(15);
        QCOMPARE(box->property("position").value<QVector3D>().x(), 3.0f);
        QCOMPARE(box->property("colour").value<QColor>(), QColor(Qt::black));
        QCOMPARE(box->property("metalness").toDouble(), 1.0);
        QCOMPARE(ball->property("position").value<QVector3D>().y(), ballAt);
        ticks(30);
        // Arrived on its sixtieth step; the crate's second tween took over from its first, which never arrived.
        QCOMPARE(box->property("position").value<QVector3D>(), QVector3D(6, 1, 0));
        QVERIFY(qAbs(crate->property("position").value<QVector3D>().x() + 2) < 1e-5);
        QObject *ease = h.logicOf(QStringLiteral("Ease"));
        QCOMPARE(ease->property("log").toStringList(), (QStringList{QStringLiteral("nothing 0"), QStringLiteral("arrived 60")}));
    }

    void aStateMachineEntersStepsAndLeavesItsStates()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/door.qml")),
                  "import QQ\nLogic {\n    property var log: []\n"
                  "    readonly property var door: machine({ start: \"shut\",\n"
                  "        shut: { enter: () => log = log.concat([\"enter shut\"]), exit: () => log = log.concat([\"exit shut\"]),\n"
                  "                on: { knock: \"opening\" } },\n"
                  "        opening: { enter: () => log = log.concat([\"enter opening\"]),\n"
                  "                   step: (dt) => { if (door.steps >= 30) door.go(\"open\") } },\n"
                  "        open: { enter: () => log = log.concat([\"enter open after \" + door.previous]),\n"
                  "                on: { slam: () => \"shut\" } } })\n"
                  "    onStarted: after(0.25, () => { const knocked = door.send(\"knock\"); log = log.concat([\"knock \" + knocked]) })\n"
                  "    function slam() { return door.send(\"slam\") }\n"
                  "    function nowhere() { return door.go(\"nowhere\") }\n"
                  "    outcome: ({ state: door.state, log: log })\n}\n");
        Headless h;
        QVERIFY(h.open(sceneText(QStringLiteral("    Behaviour { name: \"Door\"; source: \"../logic/door.qml\" }\n")),
                       QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *door = h.logicOf(QStringLiteral("Door"));
        auto ticks = [&h](int n) {
            for (int i = 0; i < n; ++i)
                QMetaObject::invokeMethod(h.scene, "tick");
        };
        auto state = [door] { return door->property("door").toMap().value(QStringLiteral("state")).toString(); };
        ticks(14);
        QCOMPARE(state(), QStringLiteral("shut"));
        ticks(1);  // the knock, a quarter of a second in
        QCOMPARE(state(), QStringLiteral("opening"));
        ticks(28);
        QCOMPARE(state(), QStringLiteral("opening"));
        ticks(1);  // thirty steps in the state, the knock's own among them
        QCOMPARE(state(), QStringLiteral("open"));
        QVariant answer;
        QMetaObject::invokeMethod(door, "slam", Q_RETURN_ARG(QVariant, answer));
        QCOMPARE(answer.toBool(), true);
        QCOMPARE(state(), QStringLiteral("shut"));
        QMetaObject::invokeMethod(door, "nowhere", Q_RETURN_ARG(QVariant, answer));
        QCOMPARE(answer.toBool(), false);
        QCOMPARE(state(), QStringLiteral("shut"));
        QCOMPARE(door->property("log").toStringList(),
                 (QStringList{QStringLiteral("enter shut"), QStringLiteral("exit shut"), QStringLiteral("enter opening"),
                              QStringLiteral("knock true"), QStringLiteral("enter open after opening"),
                              QStringLiteral("enter shut")}));
    }

    // ── sound (P3.13, ADR-105) ──

    void logicPlaysASoundOnceWhereItAsks()
    {
        QTemporaryDir project;
        const QDir root(project.path());
        root.mkpath(QStringLiteral("assets"));
        QVERIFY(QFile::copy(fixture(QStringLiteral("chime.wav")), root.filePath(QStringLiteral("assets/chime.wav"))));
        writeFile(root.filePath(QStringLiteral("logic/shots.qml")),
                  "import QQ\nLogic {\n    property var placed: null\n    property var everywhere: null\n"
                  "    onStarted: {\n"
                  "        placed = play(\"../assets/chime.wav\", { at: Qt.vector3d(2, 1, 0), volume: 0.5, reach: 8 })\n"
                  "        everywhere = play(\"../assets/chime.wav\", { group: \"ui\" })\n"
                  "    }\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral("    Behaviour { name: \"Shots\"; source: \"../logic/shots.qml\" }\n")),
                       true, QUrl::fromLocalFile(root.filePath(QStringLiteral("scenes/play.qml")))));
        QObject *shots = s.entity(QStringLiteral("Shots"));
        auto logic = [shots] { return shots->property("logic").value<QObject *>(); };
        QVERIFY(s.runUntil([&] { return logic() && logic()->property("placed").value<QObject *>(); }, 5000));
        QPointer<QObject> placed = logic()->property("placed").value<QObject *>();
        QPointer<QObject> everywhere = logic()->property("everywhere").value<QObject *>();
        QVERIFY(everywhere);
        // Where logic said, as loud and as far as it said; and one heard everywhere alike, in the ui group.
        QVERIFY(QString::fromLatin1(placed->metaObject()->className()).contains(QStringLiteral("SpatialSound")));
        QCOMPARE(placed->property("source").toUrl(), QUrl::fromLocalFile(root.filePath(QStringLiteral("assets/chime.wav"))));
        QCOMPARE(placed->property("scenePosition").value<QVector3D>(), QVector3D(2, 1, 0));
        QCOMPARE(placed->property("volume").toDouble(), 0.5);
        QCOMPARE(placed->property("distanceCutoff").toDouble(), 8.0);
        QVERIFY(QString::fromLatin1(everywhere->metaObject()->className()).contains(QStringLiteral("AmbientSound")));
        QCOMPARE(everywhere->property("volume").toDouble(), 1.0);
        // The groups' levels are heard at once: ui at a half, then everything at a half.
        auto *mixer = s.engine.singletonInstance<qq::Mixer *>("QQ", "Mixer");
        mixer->setLevel(QStringLiteral("ui"), 0.5);
        QCOMPARE(everywhere->property("volume").toDouble(), 0.5);
        QCOMPARE(placed->property("volume").toDouble(), 0.5);
        mixer->setLevel(QStringLiteral("master"), 0.5);
        QCOMPARE(everywhere->property("volume").toDouble(), 0.25);
        QCOMPARE(placed->property("volume").toDouble(), 0.25);
        // Put away once the second-long file has played through.
        QObject *voices = s.scene->property("_voices").value<QObject *>();
        QCOMPARE(voices->property("active").toList().size(), 2);
        QVERIFY(s.runUntil([&] { return !placed && !everywhere; }, 3000));
        QCOMPARE(voices->property("active").toList().size(), 0);
    }

    void aScenesMusicFadesFromOneTrackToTheNext()
    {
        QTemporaryDir project;
        const QDir root(project.path());
        root.mkpath(QStringLiteral("assets"));
        for (const char *name : {"assets/first.wav", "assets/second.wav"})
            QVERIFY(QFile::copy(fixture(QStringLiteral("chime.wav")), root.filePath(QLatin1String(name))));
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Music { name: \"Music\"; source: \"../assets/first.wav\"; volume: 0.8; fade: 0.6 }\n")),
                       true, QUrl::fromLocalFile(root.filePath(QStringLiteral("scenes/play.qml")))));
        QObject *music = s.entity(QStringLiteral("Music"));
        auto voice = [music](int index) {
            QVariant found;
            QMetaObject::invokeMethod(music, "voice", Q_RETURN_ARG(QVariant, found), Q_ARG(QVariant, index));
            return found.value<QObject *>();
        };
        auto track = [&](int index) { return voice(index) ? voice(index)->property("source").toUrl().fileName() : QStringLiteral("?"); };
        auto level = [&](int index) { return voice(index) ? voice(index)->property("volume").toDouble() : -1.0; };
        // One voice, with the track; the other is made only when there is a next one.
        QVERIFY(s.runUntil([&] { return voice(0) != nullptr; }, 3000));
        QVERIFY(!voice(1));
        QCOMPARE(track(0), QStringLiteral("first.wav"));
        QVERIFY(qAbs(level(0) - 0.8) < 1e-6);  // a voice's volume is a float
        QCOMPARE(voice(0)->property("loops").toInt(), -1);  // for ever
        // In the music group: its level is heard.
        auto *mixer = s.engine.singletonInstance<qq::Mixer *>("QQ", "Mixer");
        mixer->setLevel(QStringLiteral("music"), 0.5);
        QVERIFY(qAbs(level(0) - 0.4) < 1e-6);
        mixer->setLevel(QStringLiteral("music"), 1);

        // The next track: the first fades out as the second fades in, and is let go when it has.
        music->setProperty("source", QStringLiteral("../assets/second.wav"));
        QCOMPARE(track(1), QStringLiteral("second.wav"));
        s.run(250);
        QVERIFY2(level(0) > 0.05 && level(0) < 0.75, qPrintable(QString::number(level(0))));
        QVERIFY2(level(1) > 0.05 && level(1) < 0.75, qPrintable(QString::number(level(1))));
        QVERIFY(s.runUntil([&] { return qAbs(level(1) - 0.8) < 1e-6 && !voice(0); }, 2000));
        // Stopped, it lets its voices go.
        s.scene->setProperty("playing", false);
        QVERIFY(!voice(1));
    }

    void aSoundIsPannedToTheEyesSideAsABrowserHearsIt()
    {
        // As a browser hears them (WebVoice; on this computer only made and measured): a sound on the eye's right is
        // panned right, one on its left, left; quieter with distance; and a one-shot the same.
        QTemporaryDir project;
        const QDir root(project.path());
        root.mkpath(QStringLiteral("assets"));
        QVERIFY(QFile::copy(fixture(QStringLiteral("chime.wav")), root.filePath(QStringLiteral("assets/chime.wav"))));
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Sound { name: \"Right\"; position: Qt.vector3d(4, 1, 0); source: \"../assets/chime.wav\" }\n"
                           "    Sound { name: \"Left\"; position: Qt.vector3d(-4, 1, 0); source: \"../assets/chime.wav\"; volume: 0.5 }\n")),
                       false, QUrl::fromLocalFile(root.filePath(QStringLiteral("scenes/play.qml")))));
        s.world->setProperty("eye", QVector3D(0, 1, 6));
        s.world->setProperty("target", QVector3D(0, 1, 0));
        QObject *right = s.entity(QStringLiteral("Right"));
        QObject *left = s.entity(QStringLiteral("Left"));
        right->setProperty("web", true);
        left->setProperty("web", true);
        s.scene->property("_voices").value<QObject *>()->setProperty("web", true);
        s.scene->setProperty("playing", true);
        QVERIFY(s.runUntil([&] { return right->property("playing").value<QObject *>() && left->property("playing").value<QObject *>(); }, 3000));
        QObject *heardRight = right->property("playing").value<QObject *>();
        QObject *heardLeft = left->property("playing").value<QObject *>();
        QVERIFY(QString::fromLatin1(heardRight->metaObject()->className()).contains(QStringLiteral("WebVoice")));
        // The eye at (0, 1, 6) looking down -z: (4, 1, 0) is to its right, 4/sqrt(52) of the way round.
        const double side = 4 / std::sqrt(52.0);
        QVERIFY2(qAbs(heardRight->property("pan").toDouble() - side) < 1e-3, qPrintable(heardRight->property("pan").toString()));
        QVERIFY2(qAbs(heardLeft->property("pan").toDouble() + side) < 1e-3, qPrintable(heardLeft->property("pan").toString()));
        QVERIFY(qAbs(heardRight->property("volume").toDouble() - 1 / std::sqrt(52.0)) < 1e-3);
        QVERIFY(qAbs(heardLeft->property("volume").toDouble() - 0.5 / std::sqrt(52.0)) < 1e-3);
        // The eye turned to look at the right one: now ahead, so in the middle.
        s.world->setProperty("target", QVector3D(4, 1, 0));
        s.run(100);
        QVERIFY2(qAbs(heardRight->property("pan").toDouble()) < 1e-3, qPrintable(heardRight->property("pan").toString()));
        // A one-shot on the left, heard as a browser would.
        s.world->setProperty("target", QVector3D(0, 1, 0));
        QObject *voices = s.scene->property("_voices").value<QObject *>();
        QVariant shot;
        const QVariant chime = QUrl::fromLocalFile(root.filePath(QStringLiteral("assets/chime.wav")));
        QVariantMap options;
        options.insert(QStringLiteral("at"), QVariant::fromValue(QVector3D(-3, 1, 6)));
        QMetaObject::invokeMethod(voices, "oneShot", Q_RETURN_ARG(QVariant, shot), Q_ARG(QVariant, chime),
                                  Q_ARG(QVariant, QVariant(options)));
        QObject *oneShot = shot.value<QObject *>();
        QVERIFY(oneShot);
        s.run(50);
        QVERIFY2(oneShot->property("pan").toDouble() < -0.99, qPrintable(oneShot->property("pan").toString()));
    }

    void noSoundIsMadeInAReplay()
    {
        // A replay has no ears: a placed Sound, the Music and a sound logic plays are all silent, and it says so.
        QTemporaryDir project;
        const QDir root(project.path());
        root.mkpath(QStringLiteral("assets"));
        QVERIFY(QFile::copy(fixture(QStringLiteral("chime.wav")), root.filePath(QStringLiteral("assets/chime.wav"))));
        writeFile(root.filePath(QStringLiteral("logic/ears.qml")),
                  "import QQ\nLogic {\n    property bool tried: false\n    property bool silent: false\n"
                  "    onStarted: { silent = play(\"../assets/chime.wav\") === null; tried = true }\n"
                  "    outcome: ({ tried: tried, silent: silent, sound: entity(\"Chime\").playing === null,\n"
                  "               music: entity(\"Music\").voice(0) === null })\n}\n");
        const QUrl file = QUrl::fromLocalFile(root.filePath(QStringLiteral("scenes/play.qml")));
        writeFile(file.toLocalFile(), sceneText(QStringLiteral(
                      "    Sound { name: \"Chime\"; position: Qt.vector3d(1, 1, 0); source: \"../assets/chime.wav\" }\n"
                      "    Music { name: \"Music\"; source: \"../assets/chime.wav\" }\n"
                      "    Behaviour { name: \"Ears\"; source: \"../logic/ears.qml\" }\n")).toUtf8());
        qq::RunRecord run;
        run.setTicks(10);
        QQmlEngine engine;
        const QVariantMap result = qq::replayRun(&engine, file, &run);
        QCOMPARE(result.value(QStringLiteral("problem")).toString(), QString());
        const QVariantMap ears = result.value(QStringLiteral("outcome")).toMap().value(QStringLiteral("Ears")).toMap();
        QCOMPARE(ears.value(QStringLiteral("tried")).toBool(), true);
        QCOMPARE(ears.value(QStringLiteral("silent")).toBool(), true);
        QCOMPARE(ears.value(QStringLiteral("sound")).toBool(), true);
        QCOMPARE(ears.value(QStringLiteral("music")).toBool(), true);
    }

    // ── the game's interface (P3.8, ADR-105) ──

    void aScreenDrawsWhatLogicSetsOverTheWorld()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/hud.qml")),
                  "import QQ\nLogic {\n    onStarted: {\n        entity(\"Score\").text = \"Orbs 3\"\n"
                  "        entity(\"Health\").value = 2\n    }\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    ScreenText { name: \"Score\"; anchor: \"top-left\"; insetX: 40; insetY: 40; text: \"0\"; "
                           "size: 48; colour: \"#20ff40\" }\n"
                           "    ScreenBar { name: \"Health\"; anchor: \"top-right\"; insetX: 40; insetY: 48; width: 400; "
                           "height: 24; value: 4; maximum: 4; colour: \"#ff2040\" }\n"
                           "    Behaviour { name: \"Hud\"; source: \"../logic/hud.qml\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QQuickItem *score = s.drawn(QStringLiteral("Score"));
        QQuickItem *health = s.drawn(QStringLiteral("Health"));
        QVERIFY(score && health);
        QObject *scoreText = s.entity(QStringLiteral("Score"))->property("content").value<QObject *>();
        QVERIFY(s.runUntil([&] { return scoreText->property("text").toString() == QStringLiteral("Orbs 3"); }, 5000));

        // Placed by anchor and inset, in points of a screen 720 tall: the window is 600, so a point is 600/720 pixels.
        const double unit = s.window->height() / 720.0;
        QVERIFY2(qAbs(score->x() - 40 * unit) < 1 && qAbs(score->y() - 40 * unit) < 1,
                 qPrintable(QStringLiteral("score at %1, %2").arg(score->x()).arg(score->y())));
        QVERIFY2(qAbs(health->x() + health->width() + 40 * unit - s.window->width()) < 1,
                 qPrintable(QStringLiteral("bar ends at %1").arg(health->x() + health->width())));
        // Half full: 2 of 4.
        auto *fill = health->findChild<QQuickItem *>(QStringLiteral("fill"));
        QVERIFY(fill);
        QVERIFY(s.runUntil([&] { return qAbs(fill->width() - (health->width() - 2) / 2) < 2; }, 3000));

        // Drawn over the world: the text's green and the bar's red are in the frame where they should be.
        s.run(300);
        const QImage frame = s.window->grabWindow();
        frame.save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("play-screen.png")));
        auto count = [&frame](QRect area, auto match) {
            int n = 0;
            for (int y = area.top(); y <= area.bottom(); ++y)
                for (int x = area.left(); x <= area.right(); ++x)
                    n += match(frame.pixelColor(x, y)) ? 1 : 0;
            return n;
        };
        // The frame is in device pixels; the items are placed in the window's own (a display may be scaled).
        const qreal dpr = frame.devicePixelRatio();
        auto device = [dpr](QRectF r) { return QRectF(r.topLeft() * dpr, r.size() * dpr).toRect(); };
        const QRect textArea = device(score->mapRectToScene(QRectF(0, 0, score->width(), score->height())));
        const QRect barArea = device(health->mapRectToScene(QRectF(0, 0, health->width(), health->height())));
        const int green = count(textArea, [](QColor c) { return c.green() > 150 && c.green() > 2 * c.red(); });
        const int red = count(barArea, [](QColor c) { return c.red() > 150 && c.red() > 2 * c.green(); });
        QVERIFY2(green > 60, qPrintable(QStringLiteral("%1 green pixels in the text").arg(green)));
        QVERIFY2(red > 150, qPrintable(QStringLiteral("%1 red pixels in the bar").arg(red)));
    }

    void screensShowAndHideAndPauseHoldsTheWorld()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/hold.qml")),
                  "import QQ\nLogic {\n    onStarted: after(0.3, () => pause())\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    ScreenText { name: \"Hud\"; screen: \"hud\"; text: \"HUD\" }\n"
                           "    ScreenText { name: \"Menu\"; screen: \"pause\"; anchor: \"centre\"; text: \"PAUSED\" }\n"
                           "    Shape { name: \"Rock\"; position: Qt.vector3d(0, 30, 0); body: \"dynamic\" }\n"
                           "    Behaviour { name: \"Hold\"; source: \"../logic/hold.qml\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QQuickItem *hud = s.drawn(QStringLiteral("Hud"));
        QQuickItem *menu = s.drawn(QStringLiteral("Menu"));
        QVERIFY(hud && menu);
        QVERIFY(hud->isVisible());
        QVERIFY(!menu->isVisible());  // the scene starts on "hud" alone

        // Logic pauses the world: the rock and the logic's own clock hold still.
        QObject *hold = s.entity(QStringLiteral("Hold"));
        auto elapsed = [hold] {
            QObject *logic = hold->property("logic").value<QObject *>();
            return logic ? logic->property("elapsed").toDouble() : 0.0;
        };
        QVERIFY(s.runUntil([&] { return s.scene->property("paused").toBool(); }, 5000));
        s.run(100);
        const float heldAt = s.where(QStringLiteral("Rock")).y();
        const double clock = elapsed();
        s.run(600);
        QVERIFY2(qAbs(s.where(QStringLiteral("Rock")).y() - heldAt) < 1e-3,
                 qPrintable(QStringLiteral("rock moved from %1 to %2").arg(heldAt).arg(s.where(QStringLiteral("Rock")).y())));
        QCOMPARE(elapsed(), clock);
        QVERIFY2(clock < 0.4, qPrintable(QStringLiteral("paused at %1 s").arg(clock)));

        // The pause screen shown over it; then hidden, and the world goes on.
        QMetaObject::invokeMethod(s.scene, "showScreen", Q_ARG(QVariant, QStringLiteral("pause")));
        QVERIFY(s.runUntil([&] { return menu->isVisible() && menu->opacity() > 0.99; }, 2000));
        QMetaObject::invokeMethod(s.scene, "hideScreen", Q_ARG(QVariant, QStringLiteral("pause")));
        s.scene->setProperty("paused", false);
        QVERIFY(s.runUntil([&] { return !menu->isVisible(); }, 2000));
        QVERIFY(s.runUntil([&] { return s.where(QStringLiteral("Rock")).y() < heldAt - 1; }, 3000));
        QVERIFY(elapsed() > clock);

        // Only one screen: the title to the game, the game to its end.
        QMetaObject::invokeMethod(s.scene, "onlyScreen", Q_ARG(QVariant, QStringLiteral("pause")));
        QVERIFY(s.runUntil([&] { return menu->isVisible() && !hud->isVisible(); }, 2000));
    }

    void aButtonAnswersOnlyWhileItsScreenIsShownAndTheWorldPlays()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/press.qml")),
                  "import QQ\nLogic {\n    property int presses: 0\n    onClicked: presses += 1\n}\n");
        const QString text = sceneText(QStringLiteral(
            "    ScreenButton { name: \"Go\"; anchor: \"centre\"; insetX: 0; insetY: 0; text: \"Go\" }\n"
            "    ScreenButton { name: \"Later\"; screen: \"pause\"; anchor: \"centre\"; insetX: 0; insetY: 120; text: \"Later\" }\n"
            "    Behaviour { name: \"Press\"; source: \"../logic/press.qml\"; target: \"Go\" }\n"));
        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")));
        auto centre = [](QQuickItem *item) {
            return item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint();
        };

        Stage editing;
        QVERIFY(editing.open(text, false, file));
        QQuickItem *still = editing.drawn(QStringLiteral("Go"));
        QVERIFY(still && still->isVisible());
        QSignalSpy unpressed(editing.entity(QStringLiteral("Go")), SIGNAL(clicked()));
        QTest::mouseClick(editing.window.get(), Qt::LeftButton, {}, centre(still));
        QCOMPARE(unpressed.count(), 0);  // while editing, a button is drawn and does nothing

        Stage s;
        QVERIFY(s.open(text, true, file));
        QQuickItem *go = s.drawn(QStringLiteral("Go"));
        QQuickItem *later = s.drawn(QStringLiteral("Later"));
        QVERIFY(go && later);
        QVERIFY2(go->height() >= 48 * s.window->height() / 720.0 - 0.5, "a target a finger finds");
        QObject *press = s.entity(QStringLiteral("Press"));
        QVERIFY(s.runUntil([press] { return press->property("logic").value<QObject *>() != nullptr; }, 5000));
        s.run(300);
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(go));
        QVERIFY(s.runUntil([press] { return press->property("logic").value<QObject *>()->property("presses").toInt() == 1; },
                           2000));

        // Its screen hidden, a button is not pressed where it would be.
        QSignalSpy hidden(s.entity(QStringLiteral("Later")), SIGNAL(clicked()));
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(later));
        s.run(100);
        QCOMPARE(hidden.count(), 0);
        QMetaObject::invokeMethod(s.scene, "showScreen", Q_ARG(QVariant, QStringLiteral("pause")));
        QVERIFY(s.runUntil([later] { return later->opacity() > 0.99; }, 2000));
        QTest::mouseClick(s.window.get(), Qt::LeftButton, {}, centre(later));
        QTRY_COMPARE(hidden.count(), 1);
    }

    void withReducedMotionAScreenAppearsAtOnce()
    {
        const QString text = sceneText(QStringLiteral(
            "    ScreenText { name: \"Note\"; screen: \"note\"; anchor: \"centre\"; text: \"Note\" }\n"));
        for (const bool reduced : {false, true}) {
            Stage s;
            QVERIFY(s.open(text));
            auto *look = s.engine.singletonInstance<QObject *>("QQ", "Look");
            QVERIFY(look);
            look->setProperty("reducedMotion", reduced);
            QCOMPARE(look->property("fade").toInt(), reduced ? 0 : 260);  // the standard duration (ADR-108)
            QQuickItem *note = s.drawn(QStringLiteral("Note"));
            QVERIFY(note && !note->isVisible());
            QMetaObject::invokeMethod(s.scene, "showScreen", Q_ARG(QVariant, QStringLiteral("note")));
            s.run(30);
            if (reduced)
                QCOMPARE(note->opacity(), 1.0);
            else
                QVERIFY2(note->opacity() < 1.0, qPrintable(QStringLiteral("opacity %1 at once").arg(note->opacity())));
            QVERIFY(s.runUntil([note] { return note->opacity() > 0.999; }, 1000));
        }
    }

    void aScreenButtonAnswersTheKeyboardAndAReplayPressesItToo()
    {
        // ADR-108: a game's menu is reached by keyboard as well as by pointer. Tab gives the shown button the focus and
        // its ring; Enter presses it, delivered at the scene's next tick as a click is, so a run keeps it and a replay
        // presses it at the same moment.
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/go.qml")),
                  "import QQ\nLogic {\n    property int presses: 0\n"
                  "    onStarted: showScreen(\"title\")\n"
                  "    onClicked: presses += 1\n"
                  "    outcome: ({ presses: presses, steps: steps })\n}\n");
        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")));
        const QString text = sceneText(QStringLiteral(
            "    ScreenButton { name: \"Go\"; screen: \"title\"; anchor: \"centre\"; text: \"Go\" }\n"
            "    Behaviour { name: \"Counter\"; source: \"../logic/go.qml\"; target: \"Go\" }\n"));
        writeFile(file.toLocalFile(), text.toUtf8());

        qq::RunRecord run;
        {
            Stage s;
            QVERIFY(s.open(text, false, file));
            s.scene->setProperty("recorder", QVariant::fromValue(static_cast<QObject *>(&run)));
            s.scene->setProperty("playing", true);
            QObject *counter = s.entity(QStringLiteral("Counter"));
            auto logic = [counter] { return counter->property("logic").value<QObject *>(); };
            QQuickItem *go = nullptr;
            QVERIFY(s.runUntil([&] {
                go = s.drawn(QStringLiteral("Go"));
                return logic() && go && go->isVisible() && go->opacity() > 0.99;
            }, 4000));
            QTest::keyClick(s.window.get(), Qt::Key_Tab);
            s.run(50);
            QQuickItem *focused = s.window->activeFocusItem();
            QVERIFY2(focused && (focused == go || go->isAncestorOf(focused)), "Tab did not reach the shown button");
            QTest::keyClick(s.window.get(), Qt::Key_Return);
            QVERIFY(s.runUntil([&] { return logic()->property("presses").toInt() == 1; }, 2000));
            s.run(200);
            QMetaObject::invokeMethod(s.scene, "finishRecord");
        }
        QCOMPARE(run.outcome().value(QStringLiteral("Counter")).toMap().value(QStringLiteral("presses")).toInt(), 1);
        // Played again with no window and no keyboard: the press is in the record, at the same tick.
        QQmlEngine engine;
        const QVariantMap again = qq::replayRun(&engine, file, &run);
        QCOMPARE(again.value(QStringLiteral("problem")).toString(), QString());
        QVERIFY2(again.value(QStringLiteral("matches")).toBool(),
                 qPrintable(QJsonDocument(QJsonObject::fromVariantMap(again)).toJson(QJsonDocument::Compact)));
    }

    void aScreenImageDrawsAPictureFromTheProjectCrisply()
    {
        QTemporaryDir project;
        QDir(project.path()).mkpath(QStringLiteral("assets"));
        QVERIFY(QFile::copy(fixture(QStringLiteral("mark.png")), QDir(project.path()).filePath(QStringLiteral("assets/mark.png"))));
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    ScreenImage { name: \"Mark\"; anchor: \"centre\"; insetX: 0; insetY: 0; width: 160; "
                           "height: 160; source: \"../assets/mark.png\"; crisp: true }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *mark = s.entity(QStringLiteral("Mark"));
        QVERIFY(s.runUntil([mark] { return mark->property("ready").toBool(); }, 3000));
        QObject *image = mark->property("content").value<QObject *>();
        QCOMPARE(image->property("smooth").toBool(), false);
        s.run(200);
        // Its middle is the ember of the mark; a pixel art edge stays hard (gilt, then clear, no blend between).
        QQuickItem *drawn = s.drawn(QStringLiteral("Mark"));
        const QImage frame = s.window->grabWindow();
        const QPoint middle = (drawn->mapToScene(QPointF(drawn->width() / 2, drawn->height() / 2)) * frame.devicePixelRatio())
                                  .toPoint();
        const QColor ember = frame.pixelColor(middle);
        QVERIFY2(ember.red() > 230 && ember.green() > 80 && ember.green() < 130 && ember.blue() < 30,
                 qPrintable(ember.name()));
    }

    // ── input actions (P3.9, ADR-105) ──

    void anActionIsHeldFromTheKeyboardTheMouseThePadAndAFinger()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/log.qml")),
                  "import QQ\nLogic {\n    property string log: \"\"\n"
                  "    onPressed: (action) => log += \"+\" + action + \";\"\n"
                  "    onReleased: (action) => log += \"-\" + action + \";\"\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    InputAction { name: \"fire\"; keys: \"F, Mouse Right\"; pad: \"X\" }\n"
                           "    InputAction { name: \"boost\" }\n"
                           "    ScreenButton { name: \"Boost\"; anchor: \"bottom-right\"; text: \"Boost\"; action: \"boost\" }\n"
                           "    Behaviour { name: \"Log\"; source: \"../logic/log.qml\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        qq::Input *input = s.input();
        input->setReadsHardware(false);
        QObject *logged = s.entity(QStringLiteral("Log"));
        QVERIFY(s.runUntil([logged] { return logged->property("logic").value<QObject *>() != nullptr; }, 5000));
        auto log = [logged] { return logged->property("logic").value<QObject *>()->property("log").toString(); };

        // Presses reach logic at the next tick of the game's clock (P3.11): wait for them, then let a few more ticks
        // pass, so what should not have arrived has had its chance.
        auto heard = [&](const QString &want) {
            s.runUntil([&] { return log() == want; }, 1000);
            s.run(60);
            return log() == want;
        };        auto clear = [logged] { logged->property("logic").value<QObject *>()->setProperty("log", QString()); };
        s.run(200);
        clear();

        // The keyboard: held while the key is.
        QTest::keyPress(s.window.get(), Qt::Key_F);
        QVERIFY2(heard(QStringLiteral("+fire;")), qPrintable(log()));
        QVERIFY(input->held(QStringLiteral("fire")));
        QTest::keyRelease(s.window.get(), Qt::Key_F);
        QVERIFY2(heard(QStringLiteral("+fire;-fire;")), qPrintable(log()));
        QVERIFY(!input->held(QStringLiteral("fire")));

        // The mouse, away from any button.
        clear();
        QTest::mouseClick(s.window.get(), Qt::RightButton, {}, QPoint(200, 120));
        QVERIFY2(heard(QStringLiteral("+fire;-fire;")), qPrintable(log()));

        // The pad: X.
        clear();
        input->gamepadButtons({QStringLiteral("X")});
        input->gamepadButtons({});
        QVERIFY2(heard(QStringLiteral("+fire;-fire;")), qPrintable(log()));

        // A finger on a button that holds an action: the touch control.
        clear();
        QQuickItem *boost = s.drawn(QStringLiteral("Boost"));
        QVERIFY(boost);
        const QPoint centre = boost->mapToScene(QPointF(boost->width() / 2, boost->height() / 2)).toPoint();
        QTest::mousePress(s.window.get(), Qt::LeftButton, {}, centre);
        QVERIFY(input->held(QStringLiteral("boost")));
        QTest::mouseRelease(s.window.get(), Qt::LeftButton, {}, centre);
        QVERIFY2(heard(QStringLiteral("+boost;-boost;")), qPrintable(log()));

        // Jump and run are actions too, built in with their old bindings.
        clear();
        QTest::keyPress(s.window.get(), Qt::Key_Space);
        QTest::keyRelease(s.window.get(), Qt::Key_Space);
        QVERIFY2(heard(QStringLiteral("+jump;-jump;")), qPrintable(log()));
        QCOMPARE(input->describe(QStringLiteral("run")), QStringLiteral("Shift / Pad LS / Pad LB / Pad RB"));
        QCOMPARE(input->describe(QStringLiteral("fire")), QStringLiteral("F / Mouse Right / Pad X"));
    }

    void aPlayerRebindsAnActionByPressingWhatTheyWant()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/log.qml")),
                  "import QQ\nLogic {\n    property string log: \"\"\n"
                  "    onPressed: (action) => log += \"+\" + action + \";\"\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    InputAction { name: \"fire\"; keys: \"F\"; pad: \"X\" }\n"
                           "    Behaviour { name: \"Log\"; source: \"../logic/log.qml\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        qq::Input *input = s.input();
        input->setReadsHardware(false);
        QObject *logged = s.entity(QStringLiteral("Log"));
        QVERIFY(s.runUntil([logged] { return logged->property("logic").value<QObject *>() != nullptr; }, 5000));
        QObject *logic = logged->property("logic").value<QObject *>();
        auto log = [logic] { return logic->property("log").toString(); };

        // Presses reach logic at the next tick of the game's clock (P3.11): wait for them, then let a few more ticks
        // pass, so what should not have arrived has had its chance.
        auto heard = [&](const QString &want) {
            s.runUntil([&] { return log() == want; }, 1000);
            s.run(60);
            return log() == want;
        };
        // Asked from logic, as a controls screen would: the next key the player presses is the binding, and that press
        // does nothing else.
        QMetaObject::invokeMethod(logic, "rebind", Q_ARG(QVariant, QStringLiteral("fire")));
        QCOMPARE(input->listening(), QStringLiteral("fire"));
        QTest::keyPress(s.window.get(), Qt::Key_G);
        QTest::keyRelease(s.window.get(), Qt::Key_G);
        QCOMPARE(input->listening(), QString());
        QVERIFY2(heard(QString()), qPrintable(log()));
        QCOMPARE(input->describe(QStringLiteral("fire")), QStringLiteral("G / Pad X"));
        QTest::keyPress(s.window.get(), Qt::Key_F);
        QTest::keyRelease(s.window.get(), Qt::Key_F);
        QVERIFY2(heard(QString()), qPrintable(log()));  // the old key no longer fires
        QTest::keyPress(s.window.get(), Qt::Key_G);
        QTest::keyRelease(s.window.get(), Qt::Key_G);
        QVERIFY2(heard(QStringLiteral("+fire;")), qPrintable(log()));

        // The pad the same way: Y, held while it is named, counts for nothing until it is let go.
        input->listen(QStringLiteral("fire"));
        input->gamepadButtons({QStringLiteral("Y")});
        QCOMPARE(input->describe(QStringLiteral("fire")), QStringLiteral("G / Pad Y"));
        QVERIFY2(heard(QStringLiteral("+fire;")), qPrintable(log()));
        input->gamepadButtons({});
        input->gamepadButtons({QStringLiteral("Y")});
        QVERIFY2(heard(QStringLiteral("+fire;+fire;")), qPrintable(log()));
        input->gamepadButtons({});

        // A built-in one too: jump on J, and Space no longer.
        input->listen(QStringLiteral("jump"));
        QTest::keyPress(s.window.get(), Qt::Key_J);
        QTest::keyRelease(s.window.get(), Qt::Key_J);
        QTest::keyPress(s.window.get(), Qt::Key_Space);
        QCOMPARE(input->jump(), false);
        QTest::keyRelease(s.window.get(), Qt::Key_Space);
        QTest::keyPress(s.window.get(), Qt::Key_J);
        QCOMPARE(input->jump(), true);
        QTest::keyRelease(s.window.get(), Qt::Key_J);
    }

    void thePointerPicksWhatItIsOver()
    {
        QTemporaryDir project;
        writeFile(QDir(project.path()).filePath(QStringLiteral("logic/aim.qml")),
                  "import QQ\nLogic {\n    property string over: \"\"\n"
                  "    onFrame: (dt) => { const p = pointing(); over = p ? p.entity.name : \"nothing\" }\n}\n");
        Stage s;
        QVERIFY(s.open(sceneText(QStringLiteral(
                           "    Shape { name: \"Target\"; position: Qt.vector3d(0, 1, 0); scale: Qt.vector3d(1.5, 1.5, 1.5) }\n"
                           "    Behaviour { name: \"Aim\"; source: \"../logic/aim.qml\" }\n")),
                       true, QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/play.qml")))));
        QObject *aim = s.entity(QStringLiteral("Aim"));
        auto over = [aim] {
            QObject *logic = aim->property("logic").value<QObject *>();
            return logic ? logic->property("over").toString() : QString();
        };
        QVERIFY(s.runUntil([&] { return !over().isEmpty(); }, 5000));
        // Where the target is on the screen, as the world projects it.
        QVector3D onScreen;
        QVERIFY(QMetaObject::invokeMethod(s.world, "mapFrom3DScene", Q_RETURN_ARG(QVector3D, onScreen),
                                          Q_ARG(QVector3D, QVector3D(0, 1, 0))));
        QTest::mouseMove(s.window.get(), QPoint(int(onScreen.x()), int(onScreen.y())));
        QVERIFY2(s.runUntil([&] { return over() == QStringLiteral("Target"); }, 2000), qPrintable(over()));
        QTest::mouseMove(s.window.get(), QPoint(s.window->width() / 2, 8));  // the sky
        QVERIFY2(s.runUntil([&] { return over() == QStringLiteral("nothing"); }, 2000), qPrintable(over()));
        QTest::mouseMove(s.window.get(), QPoint(s.window->width() / 2, s.window->height() - 8));  // the ground
        QVERIFY2(s.runUntil([&] { return over() == QStringLiteral("Ground"); }, 2000), qPrintable(over()));
    }

    // ── 2D (P3.10, ADR-105) ──

    void aCrispSpriteIsDrawnPixelForPixelAtAWholeZoom()
    {
        FlatProject project;
        Stage s;
        QVERIFY(s.open(flatText(QStringLiteral(
                           "    Sprite { name: \"Frame\"; source: \"../assets/sheet.png\"; columns: 4; rows: 2; frame: 5 }\n")),
                       true, project.scene()));
        s.flat(16, 12, QVector3D(0, 0, 0));
        QObject *sprite = s.entity(QStringLiteral("Frame"));
        // An 8-pixel frame, at 16 pixels to a unit: half a unit across.
        QCOMPARE(sprite->property("size").value<QVector3D>(), QVector3D(0.5f, 0.5f, 1));
        s.run(300);
        const int zoom = s.world->property("zoom").toInt();
        QVERIFY(zoom >= 1);
        const QImage frame = s.window->grabWindow();
        QCOMPARE(qRound(s.world->property("magnification").toDouble() * frame.devicePixelRatio()), 16 * zoom);

        // Every art pixel is a square of `zoom` device pixels, its edges sharp: the white block at the frame's top-left
        // is two art pixels each way, then the frame's colour. Frame 5 is on the sheet's second row: a sheet read
        // upside down would show frame 1.
        const QPoint corner = s.devicePixel(QVector3D(-0.25f, 0.25f, 0), frame);
        auto at = [&](int x, int y) { return frame.pixelColor(corner + QPoint(x, y)); };
        const QColor colour = frameColours[5];
        QVERIFY2(painted(at(0, 0), Qt::white), qPrintable(at(0, 0).name()));
        QVERIFY2(painted(at(2 * zoom - 1, 2 * zoom - 1), Qt::white), qPrintable(at(2 * zoom - 1, 2 * zoom - 1).name()));
        QVERIFY2(painted(at(2 * zoom, 0), colour), qPrintable(at(2 * zoom, 0).name()));
        QVERIFY2(painted(at(0, 2 * zoom), colour), qPrintable(at(0, 2 * zoom).name()));
        QVERIFY2(painted(at(8 * zoom - 1, 8 * zoom - 1), colour), qPrintable(at(8 * zoom - 1, 8 * zoom - 1).name()));
        // And nothing of it past its edges.
        for (const QPoint &outside : {QPoint(-1, 4 * zoom), QPoint(8 * zoom, 4 * zoom), QPoint(4 * zoom, -1), QPoint(4 * zoom, 8 * zoom)})
            QVERIFY2(!painted(at(outside.x(), outside.y()), colour), qPrintable(at(outside.x(), outside.y()).name()));

        // Flipped, it faces the other way: the white block is at the top-right.
        sprite->setProperty("flip", true);
        s.run(150);
        const QImage flipped = s.window->grabWindow();
        QVERIFY(painted(flipped.pixelColor(corner + QPoint(8 * zoom - 1, 0)), Qt::white));
        QVERIFY(painted(flipped.pixelColor(corner), colour));
    }

    void aSheetPlaysItsFramesAndHoldsWhenTheWorldPauses()
    {
        FlatProject project;
        Stage s;
        QVERIFY(s.open(flatText(QStringLiteral(
                           "    Sprite { name: \"Runner\"; source: \"../assets/sheet.png\"; columns: 4; rows: 2; "
                           "first: 4; count: 4; fps: 6 }\n")),
                       true, project.scene()));
        s.flat(16, 12, QVector3D(0, 0, 0));
        QObject *runner = s.entity(QStringLiteral("Runner"));
        // It plays the four frames of the second row, from the first of them, and no others.
        QSet<int> seen;
        QElapsedTimer t;
        t.start();
        while (t.elapsed() < 1200) {
            seen.insert(runner->property("frame").toInt());
            s.window->update();
            QTest::qWait(8);
        }
        QCOMPARE(seen, (QSet<int>{4, 5, 6, 7}));
        // What is drawn is the frame it is on (read either side of the grab, so a frame turning over is not checked).
        bool checked = false;
        for (int attempt = 0; attempt < 10 && !checked; ++attempt) {
            const int before = runner->property("frame").toInt();
            const QImage frame = s.window->grabWindow();
            if (runner->property("frame").toInt() != before)
                continue;
            const QColor middle = frame.pixelColor(s.devicePixel(QVector3D(0.1f, -0.1f, 0), frame));
            QVERIFY2(painted(middle, frameColours[before]), qPrintable(QStringLiteral("frame %1 drawn as %2").arg(before).arg(middle.name())));
            checked = true;
        }
        QVERIFY(checked);
        // Paused, the frames hold.
        s.scene->setProperty("paused", true);
        const int held = runner->property("frame").toInt();
        s.run(600);
        QCOMPARE(runner->property("frame").toInt(), held);
    }

    void aTiledLevelIsDrawnTileForTileAndStoodOn()
    {
        FlatProject project;
        Stage s;
        // The map's top-left at (-5, 5): ten units across and five down, its floor's top at y 1.
        QVERIFY(s.open(flatText(QStringLiteral(
                           "    TileMap { name: \"Level\"; position: Qt.vector3d(-5, 5, 0); source: \"../assets/level.tmj\" }\n"
                           "    Player2D { name: \"Hero\"; position: Qt.vector3d(-3, 2, 0); jumpHeight: 1.5 }\n")),
                       true, project.scene()));
        s.flat(16, 7, QVector3D(0, 2.5f, 0));
        QObject *level = s.entity(QStringLiteral("Level"));
        QCOMPARE(level->property("error").toString(), QString());
        QCOMPARE(level->property("count").toInt(), 16);
        QVERIFY(level->property("physics").value<QObject *>());
        QCOMPARE(level->property("extent").toSizeF(), QSizeF(10, 5));
        s.run(300);

        // Tile for tile, as painted, each with its white corner where Tiled put it.
        const int zoom = s.world->property("zoom").toInt();
        const QImage frame = s.window->grabWindow();
        auto tile = [&](int column, int row) { return s.devicePixel(QVector3D(-5.0f + column, 5.0f - row, 0), frame); };
        auto at = [&](const QPoint &p) { return frame.pixelColor(p); };
        const QPoint floor = tile(0, 4);
        QVERIFY2(painted(at(floor), Qt::white), qPrintable(at(floor).name()));
        QVERIFY2(painted(at(floor + QPoint(zoom, zoom)), tileColours[0]), qPrintable(at(floor + QPoint(zoom, zoom)).name()));
        QVERIFY(painted(at(tile(3, 4) + QPoint(8 * zoom, 8 * zoom)), tileColours[1]));
        // The mirrored green tile has its corner at the top-right.
        const QPoint mirrored = tile(4, 4);
        QVERIFY2(painted(at(mirrored + QPoint(16 * zoom - 1, 0)), Qt::white), qPrintable(at(mirrored + QPoint(16 * zoom - 1, 0)).name()));
        QVERIFY2(painted(at(mirrored), tileColours[1]), qPrintable(at(mirrored).name()));
        QVERIFY(painted(at(tile(9, 0) + QPoint(8 * zoom, 8 * zoom)), tileColours[2]));  // the wall
        QVERIFY(painted(at(tile(5, 1) + QPoint(8 * zoom, 8 * zoom)), tileColours[3]));  // the ledge
        // Only the first layer: the decor layer's blue tile is not drawn.
        QVERIFY(!painted(at(tile(2, 2) + QPoint(8 * zoom, 8 * zoom)), tileColours[2]));

        // The hero falls to the floor...
        QObject *hero = s.entity(QStringLiteral("Hero"));
        QCOMPARE(s.scene->property("player").value<QObject *>(), hero);
        auto feet = [hero] { return hero->property("feet").value<QVector3D>(); };
        // ...and stands on it to within half an art pixel (a thirty-second of a unit), not floating above it.
        QVERIFY2(s.runUntil([&] { return qAbs(feet().y() - 1) < 0.03f && hero->property("activity").toString() == QStringLiteral("idle"); }, 4000),
                 qPrintable(QStringLiteral("feet at %1, %2").arg(feet().y()).arg(hero->property("activity").toString())));
        // ...runs right under the ledge to the wall, and stops against it.
        QTest::keyPress(s.window.get(), Qt::Key_D);
        QVERIFY(s.runUntil([&] { return hero->property("activity").toString() == QStringLiteral("run"); }, 1000));
        s.run(2200);
        QTest::keyRelease(s.window.get(), Qt::Key_D);
        s.run(200);
        QVERIFY2(qAbs(feet().x() - 3.625f) < 0.15f, qPrintable(QStringLiteral("stopped at %1").arg(feet().x())));
        QVERIFY2(qAbs(feet().y() - 1) < 0.03f, qPrintable(QStringLiteral("feet at %1").arg(feet().y())));
        QCOMPARE(hero->property("facingLeft").toBool(), false);
        QCOMPARE(feet().z(), 0.0f);

        // A jump of a unit and a half, and down again.
        float highest = feet().y();
        bool rose = false;
        QTest::keyPress(s.window.get(), Qt::Key_Space);
        QElapsedTimer t;
        t.start();
        while (t.elapsed() < 900) {
            highest = qMax(highest, feet().y());
            rose = rose || hero->property("activity").toString() == QStringLiteral("jump");
            s.window->update();
            QTest::qWait(8);
        }
        QTest::keyRelease(s.window.get(), Qt::Key_Space);
        QVERIFY(rose);
        QVERIFY2(highest > 2.2f && highest < 2.8f, qPrintable(QStringLiteral("jumped to %1").arg(highest)));
        QVERIFY(s.runUntil([&] { return qAbs(feet().y() - 1) < 0.1f; }, 2000));

        // Left: it turns to face that way.
        QTest::keyPress(s.window.get(), Qt::Key_A);
        s.run(300);
        QTest::keyRelease(s.window.get(), Qt::Key_A);
        QCOMPARE(hero->property("facingLeft").toBool(), true);
    }

    void aTopDownPlayerMovesAcrossThePlaneAndCarriesItsLook()
    {
        FlatProject project;
        Stage s;
        QVERIFY(s.open(flatText(QStringLiteral(
                           "    Sprite { name: \"Look\"; source: \"../assets/sheet.png\"; columns: 4; rows: 2 }\n"
                           "    Player2D { name: \"Hero\"; style: \"top-down\"; position: Qt.vector3d(0, 0, 0); look: \"Look\" }\n")),
                       true, project.scene()));
        s.flat(16, 12, QVector3D(0, 0, 0));
        QObject *hero = s.entity(QStringLiteral("Hero"));
        QObject *look = s.entity(QStringLiteral("Look"));
        auto feet = [hero] { return hero->property("feet").value<QVector3D>(); };
        s.run(500);
        // No gravity: it stays where it was put.
        QVERIFY2(qAbs(feet().y()) < 0.05f, qPrintable(QStringLiteral("drifted to %1").arg(feet().y())));

        // W moves it up the screen, D to the right, each at its speed.
        QVector3D from = feet();
        QTest::keyPress(s.window.get(), Qt::Key_W);
        s.run(1000);
        QTest::keyRelease(s.window.get(), Qt::Key_W);
        s.run(150);
        const float up = feet().y() - from.y();
        QVERIFY2(up > 3.5f && up < 6.0f, qPrintable(QStringLiteral("went up %1").arg(up)));
        QVERIFY(qAbs(feet().x() - from.x()) < 0.1f);
        from = feet();
        QTest::keyPress(s.window.get(), Qt::Key_D);
        s.run(600);
        QTest::keyRelease(s.window.get(), Qt::Key_D);
        s.run(150);
        QVERIFY(feet().x() - from.x() > 2.0f);
        QVERIFY(qAbs(feet().y() - from.y()) < 0.1f);

        // The look goes where it goes, its foot at its feet.
        const QVector3D carried = look->property("position").value<QVector3D>();
        QVERIFY2((carried - (feet() + QVector3D(0, 0.25f, 0))).length() < 0.05f,
                 qPrintable(QStringLiteral("look at %1, %2").arg(carried.x()).arg(carried.y())));
    }

    void aDynamicBodyInA2DSceneStaysOnThePlane()
    {
        Stage s;
        QVERIFY(s.open(flatText(QStringLiteral(
            "    Shape { name: \"Floor\"; position: Qt.vector3d(0, -0.5, 0); scale: Qt.vector3d(20, 1, 4) }\n"
            "    Shape { name: \"Crate\"; position: Qt.vector3d(0, 3, 0); eulerRotation: Qt.vector3d(0, 0, 30); "
            "body: \"dynamic\" }\n"))));
        s.flat(16, 12, QVector3D(0, 1, 0));
        QObject *crate = s.entity(QStringLiteral("Crate"));
        QVERIFY(s.runUntil([&s] { return s.where(QStringLiteral("Crate")).y() < 2.5f; }, 5000));
        // Pushed into the depth and spun about every axis: it moves only on the plane and turns only about the depth axis.
        QVERIFY(QMetaObject::invokeMethod(crate, "push", Q_ARG(QVariant, QVariant::fromValue(QVector3D(2, 0, 8)))));
        s.run(1500);
        const QVector3D at = s.where(QStringLiteral("Crate"));
        QVERIFY2(qAbs(at.z()) < 0.01f, qPrintable(QStringLiteral("moved to z %1").arg(at.z())));
        QVERIFY(at.x() > 0.2f);
        const QVector3D turned = crate->property("physics").value<QObject *>()->property("eulerRotation").value<QVector3D>();
        QVERIFY2(qAbs(turned.x()) < 0.5f && qAbs(turned.y()) < 0.5f,
                 qPrintable(QStringLiteral("turned %1, %2, %3").arg(turned.x()).arg(turned.y()).arg(turned.z())));
    }
};

QTEST_MAIN(TstPlay)
#include "tst_play.moc"
