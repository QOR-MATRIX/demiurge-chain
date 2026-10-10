// Does QQ Studio do what P3.1 and P3.2 say, in its real window? (DIRECTION P3.1, P3.2)
//
// Each case opens the Studio's own Main.qml, with a temporary project, and works it the way a person does: a click in
// the viewport, a drag on a handle, Save, Play and Stop. What is checked is what changed in the scene and what reached
// the disk.

#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QMouseEvent>
#include <QPointingDevice>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QTest>
#include <QVector3D>
#include <QtQml/qqmlextensionplugin.h>

#include "gpu.h"

#include "sceneio.h"

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
    void initTestCase() { QQuickStyle::setStyle(QStringLiteral("Material")); }

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

        // Esc: the copy is gone, and the scene, its unsaved state and the selection are as they were.
        QTest::keyClick(s.window, Qt::Key_Escape);
        QTRY_VERIFY(!s.window->property("playing").toBool());
        QVERIFY(!s.playScene());
        QCOMPARE(io->write(s.scene(), file), before);
        QCOMPARE(s.window->property("dirty").toBool(), dirtyBefore);
        QCOMPARE(int(s.entities().size()), countBefore);
        QCOMPARE(s.selection(), crate);
        QVERIFY(s.window->property("notice").toString().contains(QStringLiteral("as it was")));

        // And plays again from the same start.
        QVERIFY(s.call("play").toBool());
        s.run(200);
        const QObject *again = s.played(QStringLiteral("Crate"))->property("physics").value<QObject *>();
        QVERIFY(again->property("scenePosition").value<QVector3D>().y() > 3.0f);
        QVERIFY(s.call("stop").toBool());
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
