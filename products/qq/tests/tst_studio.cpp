// Does QQ Studio do what P3.1 says, in its real window? (DIRECTION P3.1)
//
// Each case opens the Studio's own Main.qml, with a temporary project, and works it the way a person does: a click in
// the viewport, a drag on a handle, Save. What is checked is what changed in the scene and what reached the disk.

#include <QDir>
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

#include "sceneio.h"

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
        QCOMPARE(s.entities().size(), 5);  // a sun, the ground, and three shapes
        QVERIFY(s.entity(QStringLiteral("Core")));
        const QString notice = s.window->property("notice").toString();
        QVERIFY2(notice.contains(QStringLiteral("Imported 3 shapes")) && notice.contains(QStringLiteral("emitter")),
                 qPrintable(notice));
        QCOMPARE(s.window->property("sceneName").toString(), QStringLiteral("first-light"));
    }
};

QTEST_MAIN(TstStudio)
#include "tst_studio.moc"
