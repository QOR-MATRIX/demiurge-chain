// Does the QQ Player keep a game to its own package? (ADR-087 decision 2)
//
// Each case builds a game in a temporary package (a project folder with scenes/, logic/ and assets/), confines a real
// engine to it as the Player does (Confinement::apply), plays the scene, and lets its logic try one kind of reach
// beyond the package. What the logic got back is read from its own properties, what reached the disk or the network
// is checked from outside, and every refusal must be on Confinement's record. The first case is the other side: a game
// that keeps to its own files plays, and nothing of it is refused.

#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QImage>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickWindow>
#include <QTcpServer>
#include <QTemporaryDir>
#include <QTest>
#include <QtQml/qqmlextensionplugin.h>

#include <memory>

#include "confinement.h"
#include "gpu.h"
#include "sceneio.h"

static const QString chosenGpu = qq::preferHighPerformanceGpu();

Q_IMPORT_QML_PLUGIN(QQPlugin)

namespace {

QString fixture(const QString &name)
{
    return QDir(QStringLiteral(QQ_FIXTURES)).filePath(name);
}

void write(const QString &path, const QByteArray &bytes)
{
    QDir().mkpath(QFileInfo(path).absolutePath());
    QFile f(path);
    QVERIFY(f.open(QIODevice::WriteOnly));
    f.write(bytes);
}

/// A game in a package of its own, played in a confined engine; and a folder outside the package to aim at.
struct Game {
    QTemporaryDir package;
    QTemporaryDir outside;
    QQmlEngine engine;
    std::unique_ptr<QQuickWindow> window;
    QObject *scene = nullptr;
    qq::SceneIO *io = nullptr;
    int refusalsBefore = 0;

    QString in(const QString &path) const { return QDir(package.path()).filePath(path); }
    QString out(const QString &path) const { return QDir(outside.path()).filePath(path); }
    QString outUrl(const QString &path) const { return QUrl::fromLocalFile(out(path)).toString(); }

    /// Play a scene whose one Behaviour runs `logic` (a whole logic file), with `extra` entities beside it.
    bool play(const QByteArray &logic, const QString &extra = {})
    {
        write(in(QStringLiteral("logic/probe.qml")), logic);
        const QString text = QStringLiteral(
            "import QQ\nScene {\n    name: \"game\"\n    Sun {}\n    Ground {}\n%1"
            "    Behaviour { name: \"Probe\"; source: \"../logic/probe.qml\"; target: \"\" }\n}\n").arg(extra);
        write(in(QStringLiteral("scenes/game.qml")), text.toUtf8());

        refusalsBefore = int(qq::Confinement::refusals().size());
        qq::Confinement::apply(&engine, QUrl::fromLocalFile(package.path()));
        io = engine.singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        QQmlComponent component(&engine);
        component.setData(R"(
            import QtQuick
            import QtQuick3D
            import QQ
            Window {
                width: 640; height: 400; visible: true
                World { objectName: "world"; anchors.fill: parent; Node { objectName: "holder" } }
            }
        )", QUrl(QStringLiteral("qrc:/tst_lockdown/stage.qml")));
        window.reset(qobject_cast<QQuickWindow *>(component.create()));
        if (!window || !QTest::qWaitForWindowExposed(window.get()))
            return false;
        QObject *holder = window->findChild<QObject *>(QStringLiteral("holder"));
        scene = io->load(QUrl::fromLocalFile(in(QStringLiteral("scenes/game.qml"))), holder);
        if (!scene) {
            qWarning() << io->lastError();
            return false;
        }
        scene->setProperty("playing", true);
        return true;
    }

    ~Game()
    {
        if (scene)
            io->discard(scene);
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    }

    QObject *behaviour() const
    {
        for (const QVariant &e : io->entities(scene)) {
            if (e.value<QObject *>()->property("name").toString() == QStringLiteral("Probe"))
                return e.value<QObject *>();
        }
        return nullptr;
    }

    /// The logic, once built (or null, with the Behaviour's error saying why).
    QObject *logic()
    {
        QObject *b = behaviour();
        (void)QTest::qWaitFor([b] { return b->property("logic").value<QObject *>() || !b->property("error").toString().isEmpty(); },
                              10000);
        return b->property("logic").value<QObject *>();
    }

    QStringList newRefusals() const { return qq::Confinement::refusals().mid(refusalsBefore); }
};

}  // namespace

class TstLockdown : public QObject
{
    Q_OBJECT

private slots:
    void aGameThatKeepsToItsOwnFilesPlaysAndNothingIsRefused()
    {
        Game g;
        QVERIFY(QDir().mkpath(g.in(QStringLiteral("assets"))));
        QVERIFY(QFile::copy(fixture(QStringLiteral("orb.gltf")), g.in(QStringLiteral("assets/orb.gltf"))));
        QVERIFY(g.play(QByteArrayLiteral(
                           "import QtQuick\nimport QtQuick3D\nimport QtQuick3D.Physics\nimport QQ\n"
                           "Logic {\n    property bool ran: false\n    onFrame: (dt) => ran = true\n}\n"),
                       QStringLiteral("    Prop { name: \"Orb\"; source: \"../assets/orb.gltf\" }\n")));
        QObject *logic = g.logic();
        QVERIFY2(logic, qPrintable(g.behaviour()->property("error").toString()));
        QTRY_VERIFY(logic->property("ran").toBool());
        QObject *orb = nullptr;
        for (const QVariant &e : g.io->entities(g.scene))
            if (e.value<QObject *>()->property("name").toString() == QStringLiteral("Orb"))
                orb = e.value<QObject *>();
        QTRY_VERIFY2(orb->property("status").toInt() == 1,  // Prop.Ready
                     qPrintable(QStringLiteral("status %1: %2; refused: %3").arg(orb->property("status").toInt())
                                    .arg(orb->property("errorString").toString(), g.newRefusals().join(QStringLiteral(" | ")))));
        QVERIFY2(g.newRefusals().isEmpty(), qPrintable(g.newRefusals().join(QLatin1Char('\n'))));
    }

    void importsOutsideTheListAreRefused_data()
    {
        QTest::addColumn<QString>("module");
        QTest::newRow("storage") << QStringLiteral("QtQuick.LocalStorage");
        QTest::newRow("camera and microphone") << QStringLiteral("QtMultimedia");
        QTest::newRow("settings and paths") << QStringLiteral("QtCore");
        QTest::newRow("dialogs") << QStringLiteral("QtQuick.Dialogs");
        QTest::newRow("labs settings") << QStringLiteral("Qt.labs.settings");
    }

    void importsOutsideTheListAreRefused()
    {
        QFETCH(QString, module);
        Game g;
        QVERIFY(g.play(QStringLiteral("import %1\nimport QQ\nLogic {}\n").arg(module).toUtf8()));
        // The known limit (confinement.h): Qt's spatial audio library links Qt Multimedia's QML library, which
        // registers its types when it loads, and a registered module is imported without its qmldir. Natively only a
        // person's own games play (ADR-087 decision 3); in a browser, the browser asks before a camera or microphone.
        QEXPECT_FAIL("camera and microphone", "Qt Multimedia's types are registered by Qt's spatial audio library", Abort);
        QVERIFY(!g.logic());
        const QString error = g.behaviour()->property("error").toString();
        QVERIFY2(error.contains(module), qPrintable(error));
        QVERIFY2(g.newRefusals().contains(QStringLiteral("import: ") + module),
                 qPrintable(g.newRefusals().join(QLatin1Char('\n'))));
    }

    void aGameWritesNoFileAndReadsOnlyItsOwn()
    {
        Game g;
        write(g.out(QStringLiteral("scenes/other.qml")), "import QQ\nScene { name: \"other\" }\n");
        const QString outside = QUrl::fromLocalFile(g.outside.path()).toString();
        QVERIFY(g.play(QStringLiteral(
            "import QtQml\nimport QQ\nLogic {\n"
            "    property string wrote: \"\"\n    property string saved: \"\"\n    property string copied: \"\"\n"
            "    property bool loaded: true\n    property var listed: ({})\n"
            "    Component.onCompleted: {\n"
            "        wrote = SceneIO.writeLogic(\"%1\", \"made\", \"x\")\n"
            "        saved = SceneIO.save(scene, \"%1/scenes/saved.qml\")\n"
            "        copied = SceneIO.adopt(\"%1/scenes/other.qml\", \"%1/copy\")\n"
            "        loaded = SceneIO.load(\"%1/scenes/other.qml\", null) !== null\n"
            "        listed = SceneIO.project(\"%1\")\n"
            "    }\n}\n").arg(outside).toUtf8()));
        QObject *logic = g.logic();
        QVERIFY2(logic, qPrintable(g.behaviour()->property("error").toString()));
        QCOMPARE(logic->property("wrote").toString(), QString());
        QVERIFY(logic->property("saved").toString().contains(QStringLiteral("cannot write")));
        QCOMPARE(logic->property("copied").toString(), QString());
        QCOMPARE(logic->property("loaded").toBool(), false);
        QVERIFY(logic->property("listed").toMap().isEmpty());
        QVERIFY(!QFile::exists(g.out(QStringLiteral("logic/made.qml"))));
        QVERIFY(!QFile::exists(g.out(QStringLiteral("scenes/saved.qml"))));
        QVERIFY(!QDir(g.out(QStringLiteral("copy"))).exists());
    }

    void aGameLoadsNothingFromOutsideItsPackage()
    {
        Game g;
        write(g.out(QStringLiteral("item.qml")), "import QtQuick\nItem {}\n");
        QImage(4, 4, QImage::Format_RGB32).save(g.out(QStringLiteral("picture.png")));
        QVERIFY(g.play(QStringLiteral(
            "import QtQuick\nimport QQ\nLogic {\n"
            "    property alias loaderStatus: loader.status\n    property alias imageStatus: image.status\n"
            "    Loader { id: loader; source: \"%1\" }\n"
            "    Image { id: image; source: \"%2\" }\n}\n")
                           .arg(g.outUrl(QStringLiteral("item.qml")), g.outUrl(QStringLiteral("picture.png")))
                           .toUtf8()));
        QObject *logic = g.logic();
        QVERIFY2(logic, qPrintable(g.behaviour()->property("error").toString()));
        QTRY_COMPARE(logic->property("loaderStatus").toInt(), 3);  // Loader.Error
        QTRY_COMPARE(logic->property("imageStatus").toInt(), 3);   // Image.Error
        QVERIFY(g.newRefusals().join(QLatin1Char('\n')).contains(QStringLiteral("item.qml")));
    }

    void aRelativePathCannotClimbOutOfThePackage()
    {
        // The same, but written relative to the logic file and climbing out: a relative URL is checked whole, once it
        // is resolved for loading.
        Game g;
        write(g.out(QStringLiteral("item.qml")), "import QtQuick\nItem {}\n");
        QImage(4, 4, QImage::Format_RGB32).save(g.out(QStringLiteral("picture.png")));
        const QString up = QDir(g.in(QStringLiteral("logic"))).relativeFilePath(g.outside.path());
        QVERIFY2(up.startsWith(QStringLiteral("..")), qPrintable(up));
        QVERIFY(g.play(QStringLiteral(
            "import QtQuick\nimport QQ\nLogic {\n"
            "    property alias loaderStatus: loader.status\n    property alias imageStatus: image.status\n"
            "    Loader { id: loader; source: \"%1/item.qml\" }\n"
            "    Image { id: image; source: \"%1/picture.png\" }\n}\n").arg(up).toUtf8()));
        QObject *logic = g.logic();
        QVERIFY2(logic, qPrintable(g.behaviour()->property("error").toString()));
        QTRY_COMPARE(logic->property("loaderStatus").toInt(), 3);  // Loader.Error
        QTRY_COMPARE(logic->property("imageStatus").toInt(), 3);   // Image.Error
    }

    void aGameReachesNoNetwork()
    {
        QTcpServer listener;
        QVERIFY(listener.listen(QHostAddress::LocalHost));
        Game g;
        QVERIFY(g.play(QStringLiteral(
            "import QtQml\nimport QQ\nLogic {\n    property int status: -1\n    property bool done: false\n"
            "    Component.onCompleted: {\n"
            "        const request = new XMLHttpRequest()\n"
            "        request.onreadystatechange = () => { if (request.readyState === XMLHttpRequest.DONE) { status = request.status; done = true } }\n"
            "        request.open(\"GET\", \"http://127.0.0.1:%1/\")\n"
            "        request.send()\n"
            "    }\n}\n").arg(listener.serverPort()).toUtf8()));
        QObject *logic = g.logic();
        QVERIFY2(logic, qPrintable(g.behaviour()->property("error").toString()));
        QTRY_VERIFY(logic->property("done").toBool());
        QTest::qWait(300);
        QVERIFY(!listener.hasPendingConnections());
        QCOMPARE(logic->property("status").toInt(), 0);
        QVERIFY(g.newRefusals().join(QLatin1Char('\n')).contains(QStringLiteral("network request")));
    }

    void aGameOpensNothingOutsideQQ()
    {
        Game g;
        QVERIFY(g.play(QStringLiteral(
            "import QtQml\nimport QQ\nLogic {\n    Component.onCompleted: {\n"
            "        Qt.openUrlExternally(\"%1\")\n"
            "        Qt.openUrlExternally(\"https://example.invalid/\")\n"
            "    }\n}\n").arg(g.outUrl(QStringLiteral("nothing.txt"))).toUtf8()));
        QVERIFY2(g.logic(), qPrintable(g.behaviour()->property("error").toString()));
        // Qt resolves the URL through the engine's interceptor before asking to open it, so each arrives either as
        // itself or as the place refused loads are pointed at: both are refused, and nothing reaches the system.
        const QStringList refused = g.newRefusals();
        QCOMPARE(refused.filter(QStringLiteral("request to open: ")).size(), 2);
        QVERIFY2(refused.join(QLatin1Char('\n')).contains(QStringLiteral("https://example.invalid/")),
                 qPrintable(refused.join(QLatin1Char('\n'))));
    }
};

QTEST_MAIN(TstLockdown)
#include "tst_lockdown.moc"
