// Does QQ Studio do what P3.1 and P3.2 say, in its real window? (DIRECTION P3.1, P3.2)
//
// Each case opens the Studio's own Main.qml, with a temporary project, and works it the way a person does: a click in
// the viewport, a drag on a handle, Save, Play and Stop. What is checked is what changed in the scene and what reached
// the disk.

#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QFileInfo>
#include <QMouseEvent>
#include <QPointingDevice>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QStandardPaths>
#include <QTemporaryDir>
#include <QTest>
#include <QVector3D>
#include <QWheelEvent>
#include <cmath>
#include <QtQml/qqmlextensionplugin.h>

#include "gpu.h"
#include "mixer.h"

#include "sceneio.h"
#include "store.h"

// Drawn with the GPU the Studio and the Player draw with: chosen before QTEST_MAIN makes the application.
static const QString chosenGpu = qq::preferHighPerformanceGpu();

Q_IMPORT_QML_PLUGIN(QQPlugin)
Q_IMPORT_QML_PLUGIN(QQ_StudioPlugin)

namespace {

QString fixture(const QString &name)
{
    return QDir(QStringLiteral(QQ_FIXTURES)).filePath(name);
}

QString readAll(const QString &path)
{
    QFile f(path);
    return f.open(QIODevice::ReadOnly) ? QString::fromUtf8(f.readAll()) : QString();
}

/// One open Studio over a temporary project.
struct Studio {
    QTemporaryDir project;
    QQmlApplicationEngine engine;
    QQuickWindow *window = nullptr;

    bool open(bool versioned = true)
    {
        if (versioned)
            QDir(project.path()).mkdir(QStringLiteral(".git"));
        engine.setInitialProperties({{QStringLiteral("model"), QUrl::fromLocalFile(fixture(QStringLiteral("orb.gltf")))},
                                     {QStringLiteral("project"), QUrl::fromLocalFile(project.path())}});
        engine.loadFromModule("QQ.Studio", "Main");
        if (engine.rootObjects().isEmpty())
            return false;
        window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        if (!window || !QTest::qWaitForWindowExposed(window))
            return false;
        return QTest::qWaitFor([this] { return window->property("settled").toBool(); }, 15000);
    }

    ~Studio()
    {
        // As the Studio does when it quits.
        if (window)
            QMetaObject::invokeMethod(window, "release");
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    }

    QObject *scene() const { return window->property("scene").value<QObject *>(); }
    QObject *selection() const { return window->property("selection").value<QObject *>(); }
    QVariantList entities() const { return window->property("entityList").toList(); }

    QObject *entity(const QString &name) const
    {
        for (const QVariant &e : entities()) {
            if (e.value<QObject *>()->property("name").toString() == name)
                return e.value<QObject *>();
        }
        return nullptr;
    }

    QVariant call(const char *function, const QVariant &a = {}, const QVariant &b = {})
    {
        QVariant result;
        if (!a.isValid())
            QMetaObject::invokeMethod(window, function, Q_RETURN_ARG(QVariant, result));
        else if (!b.isValid())
            QMetaObject::invokeMethod(window, function, Q_RETURN_ARG(QVariant, result), Q_ARG(QVariant, a));
        else
            QMetaObject::invokeMethod(window, function, Q_RETURN_ARG(QVariant, result), Q_ARG(QVariant, a), Q_ARG(QVariant, b));
        return result;
    }

    /// Where a point in the scene appears in the window, through the given view ("world" or "overlay").
    QPointF onScreen(const char *view, const QVector3D &point) const
    {
        auto *item = window->findChild<QQuickItem *>(QLatin1String(view));
        QVector3D mapped;
        QMetaObject::invokeMethod(item, "mapFrom3DScene", Q_RETURN_ARG(QVector3D, mapped), Q_ARG(QVector3D, point));
        return item->mapToScene(QPointF(mapped.x(), mapped.y()));
    }

    /// Let `ms` of frames pass: play moves only as frames are drawn.
    void run(int ms)
    {
        QElapsedTimer t;
        t.start();
        while (t.elapsed() < ms) {
            window->update();
            QTest::qWait(8);
        }
    }

    void click(const char *objectName)
    {
        auto *item = window->findChild<QQuickItem *>(QLatin1String(objectName));
        QVERIFY2(item, objectName);
        QTest::mouseClick(window, Qt::LeftButton, {},
                          item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint());
    }

    QObject *playScene() const { return window->property("playScene").value<QObject *>(); }

    QObject *played(const QString &name) const
    {
        auto *io = const_cast<QQmlApplicationEngine &>(engine).singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        for (const QVariant &e : io->entities(playScene())) {
            if (e.value<QObject *>()->property("name").toString() == name)
                return e.value<QObject *>();
        }
        return nullptr;
    }

    void settle(int frames = 6)
    {
        for (int i = 0; i < frames; ++i) {
            window->update();
            QTest::qWait(30);
        }
    }

    /// A left-button drag from `from` to `to`, in small steps, as a hand makes it.
    void drag(QPointF from, QPointF to)
    {
        QTest::mousePress(window, Qt::LeftButton, {}, from.toPoint());
        for (int i = 1; i <= 12; ++i) {
            const QPointF at = from + (to - from) * (i / 12.0);
            QMouseEvent move(QEvent::MouseMove, at, window->mapToGlobal(at), Qt::NoButton, Qt::LeftButton, {},
                             QPointingDevice::primaryPointingDevice());
            QGuiApplication::sendEvent(window, &move);
            QTest::qWait(16);
        }
        QTest::mouseRelease(window, Qt::LeftButton, {}, to.toPoint());
        settle(2);
    }
};

}  // namespace

class TstStudio : public QObject
{
    Q_OBJECT

private slots:
    void initTestCase()
    {
        QQuickStyle::setStyle(QStringLiteral("Material"));
        // Games' stores (P3.16) go to the test's own folders, never a person's.
        QStandardPaths::setTestModeEnabled(true);
    }

    void aNewStudioOpensALitSceneInItsProject()
    {
        Studio s;
        QVERIFY(s.open());
        QCOMPARE(s.entities().size(), 4);
        QVERIFY(s.entity(QStringLiteral("Orb")));
        QCOMPARE(s.entity(QStringLiteral("Orb"))->property("status").toInt(), 1);  // Ready
        QCOMPARE(s.window->property("dirty").toBool(), true);  // never saved
        QCOMPARE(s.window->property("projectInfo").toMap().value(QStringLiteral("repository")).toBool(), true);
    }

    void clickingTheModelInTheViewportSelectsIt()
    {
        Studio s;
        QVERIFY(s.open());
        s.settle();
        const QPointF orb = s.onScreen("world", s.entity(QStringLiteral("Orb"))->property("scenePosition").value<QVector3D>());
        QTest::mouseClick(s.window, Qt::LeftButton, {}, orb.toPoint());
        QTRY_VERIFY(s.selection());
        QCOMPARE(s.selection()->property("name").toString(), QStringLiteral("Orb"));
        // A click on the empty sky selects nothing.
        QTest::mouseClick(s.window, Qt::LeftButton, {}, s.onScreen("world", QVector3D(0, 40, -200)).toPoint());
        QTRY_VERIFY(!s.selection());
    }

    void draggingTheXHandleMovesAlongXOnly()
    {
        Studio s;
        QVERIFY(s.open());
        QObject *plinth = s.entity(QStringLiteral("Plinth"));
        s.window->setProperty("selection", QVariant::fromValue(plinth));
        s.settle();
        const QVector3D before = plinth->property("position").value<QVector3D>();
        // The X arrow's shaft, half way along: the gizmo is one unit long, scaled to its on-screen size.
        QObject *gizmo = s.window->findChild<QObject *>(QStringLiteral("gizmo"));
        QVERIFY(gizmo);
        const double handle = gizmo->property("size").toDouble();
        const QPointF from = s.onScreen("overlay", before + QVector3D(float(handle * 0.6), 0, 0));
        const QPointF to = from + (s.onScreen("overlay", before + QVector3D(1, 0, 0)) - s.onScreen("overlay", before));
        s.drag(from, to);

        const QVector3D after = plinth->property("position").value<QVector3D>();
        QVERIFY2(qAbs(after.x() - before.x() - 1.0f) < 0.15f,
                 qPrintable(QStringLiteral("x moved %1").arg(after.x() - before.x())));
        QCOMPARE(after.y(), before.y());
        QCOMPARE(after.z(), before.z());
        QCOMPARE(s.window->property("dirty").toBool(), true);
    }

    void draggingTheCentreHandleScalesEvenly()
    {
        Studio s;
        QVERIFY(s.open());
        QObject *plinth = s.entity(QStringLiteral("Plinth"));
        s.window->setProperty("selection", QVariant::fromValue(plinth));
        s.settle();
        const QVector3D before = plinth->property("scale").value<QVector3D>();
        const QPointF centre = s.onScreen("overlay", plinth->property("scenePosition").value<QVector3D>());
        s.drag(centre, centre + QPointF(60, 0));
        const QVector3D after = plinth->property("scale").value<QVector3D>();
        const float factor = after.x() / before.x();
        QVERIFY2(factor > 1.3f, qPrintable(QStringLiteral("grew by %1").arg(factor)));
        QVERIFY(qAbs(after.y() / before.y() - factor) < 0.001f);
        QVERIFY(qAbs(after.z() / before.z() - factor) < 0.001f);
        // The position is not touched by scaling.
        QCOMPARE(plinth->property("position").value<QVector3D>(), QVector3D(0, 0.25f, 0));
    }

    void savingWritesACanonicalSceneAndCarriesTheModelIntoTheProject()
    {
        Studio s;
        QVERIFY(s.open());
        s.window->setProperty("sceneName", QStringLiteral("First Light"));
        QVERIFY(s.call("saveScene").toBool());
        QCOMPARE(s.window->property("dirty").toBool(), false);
        QVERIFY(s.window->property("notice").toString().contains(QStringLiteral("Commit it in Projects")));

        const QDir root(s.project.path());
        const QString file = root.filePath(QStringLiteral("scenes/first-light.qml"));
        QVERIFY(QFileInfo::exists(file));
        QVERIFY(QFileInfo::exists(root.filePath(QStringLiteral("assets/orb.gltf"))));
        const QString saved = readAll(file);
        QVERIFY(saved.contains(QStringLiteral("        source: \"../assets/orb.gltf\"\n")));
        QVERIFY(!saved.contains(QDir::fromNativeSeparators(s.project.path())));
        // Canonical: read back and written again, the same bytes.
        auto *io = s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        QObject *again = io->load(QUrl::fromLocalFile(file), nullptr);
        QVERIFY2(again, qPrintable(io->lastError()));
        QCOMPARE(io->write(again, QUrl::fromLocalFile(file)), saved);
        delete again;

        // One edit: unsaved, and saving again changes exactly one line.
        s.entity(QStringLiteral("Plinth"))->setProperty("roughness", 0.9);
        s.call("refresh");
        QCOMPARE(s.window->property("dirty").toBool(), true);
        QVERIFY(s.call("saveScene").toBool());
        const QStringList a = saved.split(QLatin1Char('\n'));
        const QStringList b = readAll(file).split(QLatin1Char('\n'));
        QCOMPARE(a.size(), b.size());
        QStringList changed;
        for (qsizetype i = 0; i < a.size(); ++i) {
            if (a[i] != b[i])
                changed << b[i];
        }
        QCOMPARE(changed, QStringList{QStringLiteral("        roughness: 0.9")});
        QCOMPARE(s.window->property("projectInfo").toMap().value(QStringLiteral("scenes")).toStringList(),
                 QStringList{QStringLiteral("first-light")});

        // A picture on a shape and the scene's own sky, chosen from outside the project (P3.14): saving brings both in,
        // and the file names them relative to itself.
        s.entity(QStringLiteral("Plinth"))->setProperty("image", QUrl::fromLocalFile(fixture(QStringLiteral("checker.png"))));
        s.scene()->setProperty("sky", QUrl::fromLocalFile(fixture(QStringLiteral("sky.hdr"))));
        s.call("refresh");
        QVERIFY(s.call("saveScene").toBool());
        QVERIFY(QFileInfo::exists(root.filePath(QStringLiteral("assets/checker.png"))));
        QVERIFY(QFileInfo::exists(root.filePath(QStringLiteral("assets/sky.hdr"))));
        const QString pictured = readAll(file);
        QVERIFY(pictured.contains(QStringLiteral("        image: \"../assets/checker.png\"\n")));
        QVERIFY(pictured.contains(QStringLiteral("    sky: \"../assets/sky.hdr\"\n")));
        // And the world shows the sky from the project's copy.
        QObject *world = s.window->findChild<QObject *>(QStringLiteral("world"));
        QCOMPARE(world->property("sky").toUrl(), QUrl::fromLocalFile(root.filePath(QStringLiteral("assets/sky.hdr"))));
    }

    void aSavedSceneOpensAgainAsItWasSaved()
    {
        Studio s;
        QVERIFY(s.open());
        s.entity(QStringLiteral("Plinth"))->setProperty("colour", QColor(QStringLiteral("#ff6a00")));
        QVERIFY(s.call("saveScene").toBool());
        s.call("newScene");
        QCOMPARE(s.entity(QStringLiteral("Plinth"))->property("colour").value<QColor>(), QColor(QStringLiteral("#2b3140")));
        s.call("openScene", QStringLiteral("untitled"));
        QCOMPARE(s.window->property("dirty").toBool(), false);
        QCOMPARE(s.entity(QStringLiteral("Plinth"))->property("colour").value<QColor>(), QColor(QStringLiteral("#ff6a00")));
    }

    void anEntityIsAddedAndDeleted()
    {
        Studio s;
        QVERIFY(s.open());
        s.call("add", QStringLiteral("Shape"), QVariantMap{{QStringLiteral("name"), QStringLiteral("Cube")},
                                                           {QStringLiteral("form"), QStringLiteral("cube")}});
        QCOMPARE(s.entities().size(), 5);
        QCOMPARE(s.selection()->property("name").toString(), QStringLiteral("Cube"));
        s.call("add", QStringLiteral("Shape"), QVariantMap{{QStringLiteral("name"), QStringLiteral("Cube")}});
        QVERIFY(s.entity(QStringLiteral("Cube 2")));  // names stay unique
        s.call("removeSelection");
        QTRY_COMPARE(s.entities().size(), 5);
        QVERIFY(!s.entity(QStringLiteral("Cube 2")));
    }

    void aTwoDimensionalSceneIsImportedWithWhatCannotComeYetSaid()
    {
        Studio s;
        QVERIFY(s.open());
        s.call("importScene", QUrl::fromLocalFile(fixture(QStringLiteral("first-light.qq.json"))));
        QCOMPARE(s.entities().size(), 6);  // a sun, the ground, three shapes and the core's sparks
        QVERIFY(s.entity(QStringLiteral("Core")));
        QVERIFY(s.entity(QStringLiteral("Core sparks")));
        const QString notice = s.window->property("notice").toString();
        QVERIFY2(notice.contains(QStringLiteral("Imported 4 entities")) && notice.contains(QStringLiteral("follow"))
                     && notice.contains(QStringLiteral("motion")),
                 qPrintable(notice));
        QCOMPARE(s.window->property("sceneName").toString(), QStringLiteral("first-light"));
    }

    void aTwoDimensionalSceneIsEditedFlat()
    {
        // P3.10: a 2D scene opens framed as its game starts, is panned by a drag and zoomed by the wheel, and its handles
        // move things on the plane.
        Studio s;
        QVERIFY(s.open());
        s.call("importScene", QUrl::fromLocalFile(fixture(QStringLiteral("first-light.qq.json"))));
        auto *world = s.window->findChild<QQuickItem *>(QStringLiteral("world"));
        QTRY_VERIFY(world->property("flat").toBool());
        QCOMPARE(world->property("viewHeight").toDouble(), 5.4);
        s.settle();
        QObject *core = s.entity(QStringLiteral("Core"));
        const QVector3D corePlace = core->property("scenePosition").value<QVector3D>();
        // The core is the middle of the preview, so it is the middle of the view.
        const QPointF middle = world->mapToScene(QPointF(world->width() / 2, world->height() / 2));
        QPointF seen = s.onScreen("world", corePlace);
        QVERIFY2((seen - middle).manhattanLength() < 2, qPrintable(QStringLiteral("at %1, %2").arg(seen.x()).arg(seen.y())));

        // The handles see it where the world does, and there is no depth arrow.
        s.window->setProperty("selection", QVariant::fromValue(core));
        s.settle();
        QVERIFY((s.onScreen("overlay", corePlace) - seen).manhattanLength() < 1.5);
        QObject *gizmo = s.window->findChild<QObject *>(QStringLiteral("gizmo"));
        QCOMPARE(gizmo->property("flat").toBool(), true);
        const double handle = gizmo->property("size").toDouble();
        const QPointF from = s.onScreen("overlay", corePlace + QVector3D(float(handle * 0.6), 0, 0));
        const QPointF to = from + (s.onScreen("overlay", corePlace + QVector3D(1, 0, 0)) - s.onScreen("overlay", corePlace));
        s.drag(from, to);
        const QVector3D moved = core->property("position").value<QVector3D>();
        QVERIFY2(qAbs(moved.x() - corePlace.x() - 1.0f) < 0.1f, qPrintable(QStringLiteral("x moved %1").arg(moved.x() - corePlace.x())));
        QCOMPARE(moved.y(), corePlace.y());
        QCOMPARE(moved.z(), 0.0f);

        // A drag on nothing pans: what was under the pointer stays under it.
        s.window->setProperty("selection", QVariant());
        s.settle();
        const QVector3D place = core->property("scenePosition").value<QVector3D>();
        seen = s.onScreen("world", place);
        s.drag(middle + QPointF(-200, -120), middle + QPointF(-80, -60));
        const QPointF panned = s.onScreen("world", place) - seen;
        QVERIFY2(qAbs(panned.x() - 120) < 2 && qAbs(panned.y() - 60) < 2,
                 qPrintable(QStringLiteral("moved %1, %2").arg(panned.x()).arg(panned.y())));

        // Scrolling forward shows fewer units, larger.
        const double tall = world->property("viewHeight").toDouble();
        QWheelEvent wheel(middle, s.window->mapToGlobal(middle), QPoint(), QPoint(0, 240), Qt::NoButton, Qt::NoModifier,
                          Qt::NoScrollPhase, false);
        QGuiApplication::sendEvent(s.window, &wheel);
        s.settle();
        QVERIFY2(world->property("viewHeight").toDouble() < tall * 0.85,
                 qPrintable(QStringLiteral("%1 to %2").arg(tall).arg(world->property("viewHeight").toDouble())));
        QCOMPARE(world->property("magnification").toDouble(), world->height() / world->property("viewHeight").toDouble());
    }

    void playThenStopLeavesTheSceneExactlyAsItWas()
    {
        Studio s;
        QVERIFY(s.open());
        s.call("add", QStringLiteral("Shape"), QVariantMap{{QStringLiteral("name"), QStringLiteral("Crate")},
                                                           {QStringLiteral("body"), QStringLiteral("dynamic")},
                                                           {QStringLiteral("position"), QVector3D(3, 4, 0)}});
        s.call("add", QStringLiteral("Player"), QVariantMap{{QStringLiteral("position"), QVector3D(0, 0, 3)}});
        QObject *crate = s.entity(QStringLiteral("Crate"));
        s.window->setProperty("selection", QVariant::fromValue(crate));
        auto *io = s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        const QUrl file = s.window->property("sceneFile").toUrl();
        const QString before = io->write(s.scene(), file);
        const bool dirtyBefore = s.window->property("dirty").toBool();
        const int countBefore = int(s.entities().size());

        // Play, from its button.
        s.click("playButton");
        QTRY_VERIFY(s.window->property("playing").toBool());
        QVERIFY(s.playScene() && s.playScene() != s.scene());
        QVERIFY(!s.call("saveScene").toBool());  // nothing is saved from play

        // The crate falls in play; the scene's crate stays where it was put.
        s.run(2500);
        QObject *falling = s.played(QStringLiteral("Crate"));
        QVERIFY(falling);
        const float y = falling->property("physics").value<QObject *>()->property("scenePosition").value<QVector3D>().y();
        QVERIFY2(qAbs(y - 0.5f) < 0.06f, qPrintable(QStringLiteral("crate at %1").arg(y)));
        QCOMPARE(crate->property("position").value<QVector3D>(), QVector3D(3, 4, 0));

        // The keys walk the player, and the eye follows it.
        QObject *player = s.played(QStringLiteral("Player"));
        QCOMPARE(s.window->property("follow").value<QObject *>(), player);
        const float startZ = player->property("feet").value<QVector3D>().z();
        QTest::keyPress(s.window, Qt::Key_W);
        s.run(700);
        QTest::keyRelease(s.window, Qt::Key_W);
        s.run(100);
        const float walked = startZ - player->property("feet").value<QVector3D>().z();
        QVERIFY2(walked > 1.5f, qPrintable(QStringLiteral("walked %1").arg(walked)));
        QObject *world = s.window->findChild<QObject *>(QStringLiteral("world"));
        QCOMPARE(world->property("eye").value<QVector3D>(), player->property("eye").value<QVector3D>());

        // The mouse looks around: a drag to the right turns the player right (a falling heading), at a quarter of a
        // degree a pixel, and a drag down tilts the eye to look further down, at a fifth.
        const double heading = player->property("heading").toDouble();
        const double pitch = player->property("pitch").toDouble();
        const QPointF from = s.onScreen("world", player->property("lookAt").value<QVector3D>());
        s.drag(from, from + QPointF(160, 40));
        s.run(200);
        const double turned = heading - player->property("heading").toDouble();
        QVERIFY2(turned > 32 && turned < 48, qPrintable(QStringLiteral("turned %1").arg(turned)));
        const double tilted = player->property("pitch").toDouble() - pitch;
        QVERIFY2(tilted > 5 && tilted < 11, qPrintable(QStringLiteral("tilted %1").arg(tilted)));

        // The game turns its music down (as its options screen would, P3.13).
        auto *mixer = s.engine.singletonInstance<qq::Mixer *>("QQ", "Mixer");
        mixer->setLevel(QStringLiteral("music"), 0.25);

        // Esc: the copy is gone, and the scene, its unsaved state, the selection and the levels are as they were.
        QTest::keyClick(s.window, Qt::Key_Escape);
        QTRY_VERIFY(!s.window->property("playing").toBool());
        QVERIFY(!s.playScene());
        QCOMPARE(io->write(s.scene(), file), before);
        QCOMPARE(s.window->property("dirty").toBool(), dirtyBefore);
        QCOMPARE(int(s.entities().size()), countBefore);
        QCOMPARE(s.selection(), crate);
        QCOMPARE(mixer->level(QStringLiteral("music")), 1.0);
        QVERIFY(s.window->property("notice").toString().contains(QStringLiteral("as it was")));

        // And plays again from the same start.
        QVERIFY(s.call("play").toBool());
        s.run(200);
        const QObject *again = s.played(QStringLiteral("Crate"))->property("physics").value<QObject *>();
        QVERIFY(again->property("scenePosition").value<QVector3D>().y() > 3.0f);
        QVERIFY(s.call("stop").toBool());
    }

    // ── editing (P3.15) ──

    void everyEditIsUndoneAndRedoneWithTheKeys()
    {
        Studio s;
        QVERIFY(s.open());
        s.window->setProperty("sceneName", QStringLiteral("edits"));
        QVERIFY(s.call("saveScene").toBool());
        auto *io = s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        const QUrl file = s.window->property("sceneFile").toUrl();
        auto text = [&] { return io->write(s.scene(), file); };
        auto key = [&](Qt::Key k, Qt::KeyboardModifiers m) {
            QTest::keyClick(s.window, k, m);
            s.settle(2);
        };
        auto history = [&](const char *which) {
            return s.window->property("history").value<QObject *>()->property(which).toBool();
        };
        const QString saved = text();
        QVERIFY(!history("canUndo"));

        // A handle dragged: one step.
        QObject *plinth = s.entity(QStringLiteral("Plinth"));
        s.window->setProperty("selection", QVariant::fromValue(plinth));
        s.settle();
        const QVector3D at = plinth->property("position").value<QVector3D>();
        const double size = s.window->findChild<QObject *>(QStringLiteral("gizmo"))->property("size").toDouble();
        const QPointF from = s.onScreen("overlay", at + QVector3D(float(size * 0.6), 0, 0));
        s.drag(from, from + (s.onScreen("overlay", at + QVector3D(1, 0, 0)) - s.onScreen("overlay", at)));
        const QString moved = text();
        QVERIFY(moved != saved);
        QVERIFY(history("canUndo"));

        // An entity added: another.
        QVERIFY(s.call("add", QStringLiteral("Shape"), QVariantMap{{QStringLiteral("name"), QStringLiteral("Crate")}})
                    .value<QObject *>());
        const QString added = text();

        // Three quick changes of one field in the inspector, as a slider drag makes them: one step.
        QObject *inspector = s.window->findChild<QObject *>(QStringLiteral("inspector"));
        for (const char *colour : {"#ff0000", "#00ff00", "#0000ff"})
            QMetaObject::invokeMethod(inspector, "set", Q_ARG(QVariant, QStringLiteral("colour")),
                                      Q_ARG(QVariant, QString::fromLatin1(colour)));
        const QString coloured = text();
        QVERIFY(coloured.contains(QStringLiteral("colour: \"#0000ff\"")));

        // Back, step by step, with Ctrl+Z: the colour, the crate, the move; and unsaved no more.
        key(Qt::Key_Z, Qt::ControlModifier);
        QCOMPARE(text(), added);
        QVERIFY(s.selection());  // still selected, by name
        QCOMPARE(s.selection()->property("name").toString(), QStringLiteral("Crate"));
        key(Qt::Key_Z, Qt::ControlModifier);
        QCOMPARE(text(), moved);
        QVERIFY(!s.entity(QStringLiteral("Crate")));
        QVERIFY(s.window->property("dirty").toBool());
        key(Qt::Key_Z, Qt::ControlModifier);
        QCOMPARE(text(), saved);
        QVERIFY(!s.window->property("dirty").toBool());
        QVERIFY(!history("canUndo"));
        key(Qt::Key_Z, Qt::ControlModifier);  // nothing further back
        QCOMPARE(text(), saved);

        // And forward again with Ctrl+Y and Ctrl+Shift+Z.
        key(Qt::Key_Y, Qt::ControlModifier);
        QCOMPARE(text(), moved);
        key(Qt::Key_Z, Qt::ControlModifier | Qt::ShiftModifier);
        QCOMPARE(text(), added);
        s.click("redoButton");
        s.settle(2);
        QCOMPARE(text(), coloured);
        QVERIFY(!history("canRedo"));

        // An edit after an undo leaves nothing to redo.
        key(Qt::Key_Z, Qt::ControlModifier);
        QVERIFY(history("canRedo"));
        s.entity(QStringLiteral("Crate"))->setProperty("roughness", 0.9);
        s.call("refresh");
        QVERIFY(!history("canRedo"));
        QVERIFY(text().contains(QStringLiteral("roughness: 0.9")));
        // and undoing it goes back to before it, not to what was undone.
        key(Qt::Key_Z, Qt::ControlModifier);
        QCOMPARE(text(), added);
    }

    void aSelectionIsDuplicatedBesideItself()
    {
        Studio s;
        QVERIFY(s.open());
        auto *io = s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        QObject *plinth = s.entity(QStringLiteral("Plinth"));
        s.window->setProperty("selection", QVariant::fromValue(plinth));
        QTest::keyClick(s.window, Qt::Key_D, Qt::ControlModifier);
        s.settle(2);
        QObject *copy = s.entity(QStringLiteral("Plinth 2"));
        QVERIFY(copy);
        QCOMPARE(s.selection(), copy);
        // Every field the same but its name, and a unit along X.
        for (const char *field : {"scale", "eulerRotation", "form", "colour", "roughness", "metalness", "body"})
            QCOMPARE(copy->property(field), plinth->property(field));
        QCOMPARE(copy->property("position").value<QVector3D>(),
                 plinth->property("position").value<QVector3D>() + QVector3D(1, 0, 0));
        // The two stay apart: the copy moved, the original stays.
        copy->setProperty("position", QVector3D(5, 0, 0));
        QCOMPARE(plinth->property("position").value<QVector3D>(), QVector3D(0, 0.25f, 0));
        // A copy of the copy is the third, and a model's file comes along.
        s.click("duplicateButton");
        s.settle(2);
        QVERIFY(s.entity(QStringLiteral("Plinth 3")));
        QObject *orb = s.entity(QStringLiteral("Orb"));
        s.window->setProperty("selection", QVariant::fromValue(orb));
        QVERIFY(s.call("duplicateSelection").value<QObject *>());
        QObject *orb2 = s.entity(QStringLiteral("Orb 2"));
        QCOMPARE(io->resolvedUrl(orb2, orb2->property("source").toUrl()),
                 io->resolvedUrl(orb, orb->property("source").toUrl()));
        // Undone, it is gone.
        QTest::keyClick(s.window, Qt::Key_Z, Qt::ControlModifier);
        s.settle(2);
        QVERIFY(!s.entity(QStringLiteral("Orb 2")));
        QVERIFY(s.entity(QStringLiteral("Plinth 3")));
    }

    void withSnappingHandlesMoveAlongTheGridTurnAndScaleByWholeSteps()
    {
        Studio s;
        QVERIFY(s.open());
        s.window->setProperty("snapStep", 0.25);
        QObject *plinth = s.entity(QStringLiteral("Plinth"));
        s.window->setProperty("selection", QVariant::fromValue(plinth));
        s.settle();
        const QVector3D at = plinth->property("position").value<QVector3D>();
        const double size = s.window->findChild<QObject *>(QStringLiteral("gizmo"))->property("size").toDouble();
        // Along X by about 0.9 of a unit (a hand's drag is never exact): it lands on the grid; Y and Z as they were.
        QPointF from = s.onScreen("overlay", at + QVector3D(float(size * 0.6), 0, 0));
        s.drag(from, from + (s.onScreen("overlay", at + QVector3D(0.9f, 0, 0)) - s.onScreen("overlay", at)));
        QVector3D now = plinth->property("position").value<QVector3D>();
        QVERIFY2(now.x() >= 0.5f && now.x() <= 1.25f && std::fmod(double(now.x()), 0.25) == 0.0,
                 qPrintable(QString::number(now.x())));
        QCOMPARE(now.y(), at.y());
        QCOMPARE(now.z(), at.z());
        // Turned by the knob: a whole number of 15-degree steps.
        s.settle();
        now = plinth->property("position").value<QVector3D>();
        from = s.onScreen("overlay", now + QVector3D(float(size * 0.55), float(size * 0.55), 0));
        s.drag(from, from + QPointF(53, 0));
        const double turned = plinth->property("eulerRotation").value<QVector3D>().y();
        QVERIFY2(turned != 0 && std::fmod(std::abs(turned), 15.0) < 1e-6, qPrintable(QString::number(turned)));
        // Scaled by the centre: by tenths.
        const QVector3D scale = plinth->property("scale").value<QVector3D>();
        from = s.onScreen("overlay", now);
        s.drag(from, from + QPointF(47, 0));
        const double factor = plinth->property("scale").value<QVector3D>().x() / scale.x();
        QVERIFY2(factor > 1.05 && std::abs(factor * 10 - std::round(factor * 10)) < 1e-4, qPrintable(QString::number(factor)));
    }

    void theSceneIsATreeOfGroupsAndAGroupIsKeptAsAPrefab()
    {
        Studio s;
        QVERIFY(s.open());
        auto *io = s.engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        auto key = [&](Qt::Key k, Qt::KeyboardModifiers m) {
            QTest::keyClick(s.window, k, m);
            s.settle(2);
        };
        QObject *plinth = s.entity(QStringLiteral("Plinth"));
        const QVector3D at = plinth->property("scenePosition").value<QVector3D>();

        // Ctrl+G: the plinth in a new group, where it stood, and the list a tree.
        s.window->setProperty("selection", QVariant::fromValue(plinth));
        key(Qt::Key_G, Qt::ControlModifier);
        QObject *group = s.entity(QStringLiteral("Group"));
        QVERIFY(group);
        QCOMPARE(s.selection(), group);
        QCOMPARE(plinth->parent(), group);
        QCOMPARE(plinth->property("scenePosition").value<QVector3D>(), at);
        QVariant depth;
        QMetaObject::invokeMethod(s.window, "depthOf", Q_RETURN_ARG(QVariant, depth), Q_ARG(QVariant, QVariant::fromValue(plinth)));
        QCOMPARE(depth.toInt(), 1);
        // The group moved, the plinth goes with it.
        group->setProperty("position", group->property("position").value<QVector3D>() + QVector3D(2, 0, 0));
        QCOMPARE(plinth->property("scenePosition").value<QVector3D>(), at + QVector3D(2, 0, 0));
        s.call("refresh");

        // Out to the scene's top by the inspector's choice, where it is; and back in.
        s.window->setProperty("selection", QVariant::fromValue(plinth));
        QVERIFY(s.call("moveSelectionInto", QVariant::fromValue<QObject *>(nullptr)).toBool());
        QCOMPARE(plinth->parent(), s.scene());
        QCOMPARE(plinth->property("position").value<QVector3D>(), at + QVector3D(2, 0, 0));
        QVERIFY(s.call("moveSelectionInto", QVariant::fromValue(group)).toBool());
        QCOMPARE(plinth->parent(), group);

        // Ctrl+Shift+G: the group's contents out of it, where they stand, and the group gone; undone, it is back.
        s.window->setProperty("selection", QVariant::fromValue(group));
        key(Qt::Key_G, Qt::ControlModifier | Qt::ShiftModifier);
        QVERIFY(!s.entity(QStringLiteral("Group")));
        QCOMPARE(s.entity(QStringLiteral("Plinth"))->parent(), s.scene());
        key(Qt::Key_Z, Qt::ControlModifier);
        QVERIFY(s.entity(QStringLiteral("Group")));
        QCOMPARE(s.entity(QStringLiteral("Plinth"))->parent(), s.entity(QStringLiteral("Group")));

        // Kept as a prefab: written to prefabs/, and a prefab of it placed where the group stood.
        group = s.entity(QStringLiteral("Group"));
        group->setProperty("name", QStringLiteral("Stand"));
        s.window->setProperty("selection", QVariant::fromValue(group));
        s.click("prefabButton");
        s.settle(2);
        const QString file = QDir(s.project.path()).filePath(QStringLiteral("prefabs/stand.qml"));
        QVERIFY(QFileInfo::exists(file));
        QVERIFY(readAll(file).startsWith(QStringLiteral("import QQ\n\nGroup {\n    name: \"Stand\"\n")));
        QObject *stand = s.entity(QStringLiteral("Stand"));
        QVERIFY(stand);
        QCOMPARE(stand->property("kind").toString(), QStringLiteral("Prefab"));
        QCOMPARE(stand->property("position").value<QVector3D>(), at + QVector3D(2, 0, 0));
        // Its parts are its file's: not the scene's own entities, so not in the list, and a click on one picks it.
        QVERIFY(!s.entity(QStringLiteral("Plinth")));
        QVariant part;
        QMetaObject::invokeMethod(stand, "part", Q_RETURN_ARG(QVariant, part), Q_ARG(QVariant, QStringLiteral("Plinth")));
        QObject *made = part.value<QObject *>();
        QVERIFY(made);
        QCOMPARE(made->property("scenePosition").value<QVector3D>(), at + QVector3D(2, 0, 0));
        s.window->setProperty("selection", QVariant());
        s.settle(4);
        QTest::mouseClick(s.window, Qt::LeftButton, {}, s.onScreen("world", made->property("scenePosition").value<QVector3D>()).toPoint());
        s.settle(2);
        QCOMPARE(s.selection(), stand);

        // Saved, the scene keeps the prefab, not its parts; placed again, it is a second.
        QVERIFY(s.call("saveScene").toBool());
        const QString saved = readAll(s.window->property("sceneFile").toUrl().toLocalFile());
        QVERIFY(saved.contains(QStringLiteral("        source: \"../prefabs/stand.qml\"\n")));
        QVERIFY(!saved.contains(QStringLiteral("\"Plinth\"")));
        QVERIFY(s.call("add", QStringLiteral("Prefab"),
                       QVariantMap{{QStringLiteral("name"), QStringLiteral("Stand")},
                                   {QStringLiteral("source"), QUrl::fromLocalFile(file)}}).value<QObject *>());
        QVERIFY(s.entity(QStringLiteral("Stand 2")));
        QCOMPARE(io->everyEntity(s.scene()).size() - io->entities(s.scene()).size(), 4);  // two parts each: a root and a plinth

        // Unpacked, the first is a group of its parts where it stood, to change; kept back, it is written over its file
        // and the second, made again from it, has the change too.
        s.window->setProperty("selection", QVariant::fromValue(s.entity(QStringLiteral("Stand"))));
        s.click("unpackButton");
        s.settle(2);
        QObject *unpacked = s.entity(QStringLiteral("Stand"));
        QCOMPARE(unpacked->property("kind").toString(), QStringLiteral("Group"));
        QObject *plinthAgain = s.entity(QStringLiteral("Plinth"));
        QVERIFY(plinthAgain);
        QCOMPARE(plinthAgain->property("scenePosition").value<QVector3D>(), at + QVector3D(2, 0, 0));
        plinthAgain->setProperty("colour", QColor(QStringLiteral("#ff6a00")));
        s.window->setProperty("selection", QVariant::fromValue(unpacked));
        s.click("prefabButton");
        s.settle(2);
        QVERIFY(readAll(file).contains(QStringLiteral("colour: \"#ff6a00\"")));
        QVERIFY(!QFileInfo::exists(QDir(s.project.path()).filePath(QStringLiteral("prefabs/stand-2.qml"))));
        QVariant second;
        QMetaObject::invokeMethod(s.entity(QStringLiteral("Stand 2")), "part", Q_RETURN_ARG(QVariant, second),
                                  Q_ARG(QVariant, QStringLiteral("Plinth")));
        QCOMPARE(second.value<QObject *>()->property("colour").value<QColor>(), QColor(QStringLiteral("#ff6a00")));
    }

    void theProjectsFilesAreListedAndUsedByAClick()
    {
        Studio s;
        QVERIFY(s.open());
        const QDir root(s.project.path());
        root.mkpath(QStringLiteral("assets"));
        for (const char *name : {"orb.gltf", "checker.png", "chime.wav", "sky.hdr"})
            QVERIFY(QFile::copy(fixture(QLatin1String(name)), root.filePath(QStringLiteral("assets/") + QLatin1String(name))));
        s.call("chooseProject", QUrl::fromLocalFile(s.project.path()));
        s.settle(2);
        QObject *panel = s.window->findChild<QObject *>(QStringLiteral("projectPanel"));
        QVERIFY(panel);
        auto file = [&](const QString &kind) {
            for (const QVariant &f : panel->property("files").toList())
                if (f.toMap().value(QStringLiteral("kind")).toString() == kind)
                    return f;
            return QVariant();
        };
        QVERIFY(file(QStringLiteral("model")).isValid() && file(QStringLiteral("picture")).isValid()
                && file(QStringLiteral("sound")).isValid() && file(QStringLiteral("sky")).isValid());

        // A click on the model in the list adds it as a Prop.
        auto *list = s.window->findChild<QQuickItem *>(QStringLiteral("assetList"));
        QVERIFY(list);
        const QVariantList shown = panel->property("shown").toList();
        int row = -1;
        for (int i = 0; i < shown.size(); ++i)
            if (shown.at(i).toMap().value(QStringLiteral("kind")).toString() == QStringLiteral("model"))
                row = i;
        QVERIFY(row >= 0);
        QQuickItem *item = nullptr;
        QMetaObject::invokeMethod(list, "itemAtIndex", Q_RETURN_ARG(QQuickItem *, item), Q_ARG(int, row));
        QVERIFY(item);
        QTest::mouseClick(s.window, Qt::LeftButton, {}, item->mapToScene(QPointF(item->width() / 2, item->height() / 2)).toPoint());
        s.settle(2);
        QObject *orb = s.entity(QStringLiteral("orb"));
        QVERIFY(orb);
        QCOMPARE(orb->property("kind").toString(), QStringLiteral("Prop"));
        QCOMPARE(orb->property("source").toUrl(), QUrl::fromLocalFile(root.filePath(QStringLiteral("assets/orb.gltf"))));

        // The picture on the selected plinth; the sky the scene's; the sound a Sound.
        s.window->setProperty("selection", QVariant::fromValue(s.entity(QStringLiteral("Plinth"))));
        QVERIFY(s.call("useFile", file(QStringLiteral("picture"))).value<QObject *>());
        QCOMPARE(s.entity(QStringLiteral("Plinth"))->property("image").toUrl(),
                 QUrl::fromLocalFile(root.filePath(QStringLiteral("assets/checker.png"))));
        s.call("useFile", file(QStringLiteral("sky")));
        QCOMPARE(s.scene()->property("sky").toUrl(), QUrl::fromLocalFile(root.filePath(QStringLiteral("assets/sky.hdr"))));
        s.call("useFile", file(QStringLiteral("sound")));
        QCOMPARE(s.entity(QStringLiteral("chime"))->property("kind").toString(), QStringLiteral("Sound"));
        // The filter narrows the list by name.
        s.window->findChild<QObject *>(QStringLiteral("assetFilter"))->setProperty("text", QStringLiteral("chec"));
        QCOMPARE(panel->property("shown").toList().size(), 1);
    }

    void logicIsWrittenInTheStudioAndCheckedByLine()
    {
        Studio s;
        QVERIFY(s.open());
        QObject *behaviour = s.call("addBehaviour").value<QObject *>();
        QVERIFY(behaviour);
        s.window->setProperty("selection", QVariant::fromValue(behaviour));
        s.settle(2);
        s.click("editLogicButton");
        s.settle(2);
        QObject *editor = s.window->findChild<QObject *>(QStringLiteral("logicEditor"));
        QCOMPARE(editor->property("name").toString(), QStringLiteral("behaviour"));
        const QString path = QDir(s.project.path()).filePath(QStringLiteral("logic/behaviour.qml"));
        QCOMPARE(editor->property("text").toString(), readAll(path));
        QObject *code = s.window->findChild<QObject *>(QStringLiteral("logicText"));
        QObject *status = s.window->findChild<QObject *>(QStringLiteral("logicStatus"));
        QCOMPARE(status->property("text").toString(), QStringLiteral("Builds"));

        // A closing brace taken away: after a moment's quiet, said to be wrong, by its line.
        const QString good = editor->property("text").toString();
        QString broken = good;
        broken.chop(2);  // the last "}\n"
        code->setProperty("text", broken);
        QTRY_VERIFY(editor->property("problems").toList().size() > 0);
        QVERIFY(status->property("text").toString().contains(QStringLiteral("problem")));
        QVERIFY(editor->property("dirty").toBool());

        // Mended, and saved with Ctrl+S while it has the keys: the file is written, the scene is not.
        const QString mended = good + QStringLiteral("// changed in the Studio\n");
        code->setProperty("text", mended);
        QMetaObject::invokeMethod(code, "forceActiveFocus");
        s.settle(1);
        const QUrl sceneFile = s.window->property("sceneFile").toUrl();
        QTest::keyClick(s.window, Qt::Key_S, Qt::ControlModifier);
        s.settle(2);
        QCOMPARE(readAll(path), mended);
        QVERIFY(!editor->property("dirty").toBool());
        QTRY_COMPARE(status->property("text").toString(), QStringLiteral("Builds"));
        QVERIFY(!QFileInfo::exists(sceneFile.toLocalFile()));
    }

    void f3ShowsWhatTheWorldCostsAndCollidersInPlay()
    {
        Studio s;
        QVERIFY(s.open());
        s.call("add", QStringLiteral("Shape"), QVariantMap{{QStringLiteral("name"), QStringLiteral("Crate")},
                                                           {QStringLiteral("body"), QStringLiteral("dynamic")},
                                                           {QStringLiteral("position"), QVector3D(1, 3, 0)}});
        QTest::keyClick(s.window, Qt::Key_F3);
        s.settle(4);
        auto *overlay = s.window->findChild<QQuickItem *>(QStringLiteral("statsOverlay"));
        QVERIFY(overlay && overlay->isVisible());
        QObject *frames = s.window->findChild<QObject *>(QStringLiteral("statsFrames"));
        QTRY_VERIFY(frames->property("text").toString().contains(QStringLiteral("fps")));
        QObject *draws = s.window->findChild<QObject *>(QStringLiteral("statsDraws"));
        QTRY_VERIFY(!draws->property("text").toString().startsWith(QStringLiteral("0 draw")));
        // In play, every body drawn as its collider, and counted.
        QVERIFY(s.call("play").toBool());
        QTRY_VERIFY(s.playScene() && s.playScene()->property("physicsWorld").value<QObject *>());
        QObject *physics = s.playScene()->property("physicsWorld").value<QObject *>();
        QTRY_VERIFY(physics->property("forceDebugDraw").toBool());
        QObject *counted = s.window->findChild<QObject *>(QStringLiteral("statsScene"));
        QTRY_VERIFY(!counted->property("text").toString().contains(QStringLiteral(" 0 bodies")));
        // F3 again: gone, and the colliders with it.
        QTest::keyClick(s.window, Qt::Key_F3);
        s.settle(2);
        QVERIFY(!overlay->isVisible());
        QVERIFY(!physics->property("forceDebugDraw").toBool());
        QVERIFY(s.call("stop").toBool());
    }

    void theProjectPanelSaysWhatTheGameKeepsAndForgetsIt()
    {
        Studio s;
        QVERIFY(s.open());
        qq::GameStore *store = qq::GameStore::storeFor(QUrl::fromLocalFile(s.project.path()));
        QVERIFY(store);
        QObject *panel = s.window->findChild<QObject *>(QStringLiteral("projectPanel"));
        QCOMPARE(panel->property("store").value<QObject *>(), store);
        QObject *summary = s.window->findChild<QObject *>(QStringLiteral("savedSummary"));
        QCOMPARE(summary->property("text").toString(), QStringLiteral("Nothing kept between plays"));

        // A play whose logic keeps something: the panel says what, once play stops.
        const QString logic = QDir(s.project.path()).filePath(QStringLiteral("logic/keeper.qml"));
        QDir().mkpath(QFileInfo(logic).absolutePath());
        {
            QFile f(logic);
            QVERIFY(f.open(QIODevice::WriteOnly));
            f.write("import QQ\nLogic {\n    onStarted: save(\"best\", 3)\n}\n");
        }
        QVERIFY(s.call("add", QStringLiteral("Behaviour"),
                       QVariantMap{{QStringLiteral("name"), QStringLiteral("Keeper")},
                                   {QStringLiteral("source"), QUrl::fromLocalFile(logic)}})
                    .value<QObject *>());
        QVERIFY(s.call("play").toBool());
        QTRY_COMPARE(store->value(QStringLiteral("best")).toInt(), 3);
        QVERIFY(s.call("stop").toBool());
        QTRY_VERIFY(summary->property("text").toString().startsWith(QStringLiteral("Kept between plays: best")));

        // Forget: nothing kept, as for someone who has never played it.
        auto *forget = s.window->findChild<QQuickItem *>(QStringLiteral("forgetSaved"));
        QVERIFY(forget && forget->isVisible() && forget->isEnabled());
        s.click("forgetSaved");
        s.settle(2);
        QVERIFY(store->keys().isEmpty());
        QCOMPARE(summary->property("text").toString(), QStringLiteral("Nothing kept between plays"));
        QVERIFY(!forget->isEnabled());
        store->flush();
        QVERIFY(!QFileInfo::exists(qq::GameStore::placeOf(store->game())));
    }

    void aNewBehaviourIsWrittenIntoTheProjectAndNeverOverAFile()
    {
        Studio s;
        QVERIFY(s.open());
        const QDir root(s.project.path());
        root.mkpath(QStringLiteral("logic"));
        QFile mine(root.filePath(QStringLiteral("logic/behaviour.qml")));
        QVERIFY(mine.open(QIODevice::WriteOnly));
        mine.write("// mine\n");
        mine.close();

        s.window->setProperty("selection", QVariant::fromValue(s.entity(QStringLiteral("Plinth"))));
        QVariant made = s.call("addBehaviour");
        QObject *behaviour = made.value<QObject *>();
        QVERIFY(behaviour);
        QCOMPARE(behaviour->property("target").toString(), QStringLiteral("Plinth"));
        QCOMPARE(readAll(root.filePath(QStringLiteral("logic/behaviour.qml"))), QStringLiteral("// mine\n"));
        const QString written = readAll(root.filePath(QStringLiteral("logic/behaviour-2.qml")));
        QVERIFY2(written.contains(QStringLiteral("Logic {")), qPrintable(written));

        // In play the template turns what it drives, 90 degrees for each second of play its logic has seen. (Measured
        // by the logic's own clock: the first play in a process waits a moment for physics to start.)
        QVERIFY(s.call("play").toBool());
        QObject *logic = s.played(behaviour->property("name").toString())->property("logic").value<QObject *>();
        QVERIFY(logic);
        QElapsedTimer guard;
        guard.start();
        while (logic->property("elapsed").toDouble() < 0.6 && guard.elapsed() < 5000)
            s.run(50);
        const double elapsed = logic->property("elapsed").toDouble();
        const float turned = s.played(QStringLiteral("Plinth"))->property("eulerRotation").value<QVector3D>().y();
        QVERIFY2(elapsed >= 0.6 && qAbs(turned - 90 * elapsed) < 2,
                 qPrintable(QStringLiteral("turned %1 in %2 s").arg(turned).arg(elapsed)));
        QCOMPARE(s.window->property("playProblem").toString(), QString());
        QVERIFY(s.call("stop").toBool());

        // Saved, the scene keeps the logic by a path inside the project.
        QVERIFY(s.call("saveScene").toBool());
        QVERIFY(readAll(root.filePath(QStringLiteral("scenes/untitled.qml")))
                    .contains(QStringLiteral("        source: \"../logic/behaviour-2.qml\"\n")));
    }
};

QTEST_MAIN(TstStudio)
#include "tst_studio.moc"
