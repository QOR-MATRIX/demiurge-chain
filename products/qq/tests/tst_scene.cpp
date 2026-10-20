// Are QQ scene files what P3.1 promises? (DIRECTION P3.1, ADR-083 decision 3)
//
// A scene file is canonical QML: read and written back it is the same bytes, and one changed value is one changed line,
// so Projects shows a creator's change as what it is. These cases drive SceneIO through a real QML engine with the
// runtime module "QQ", against the fixtures: tests/fixtures/scenes/sample.qml (every entity type, written by hand in
// the canonical layout), tests/fixtures/first-light.qq.json (the 2D preview's starter scene), and the 2D fixtures that
// tests/fixtures/make_2d.py writes (a Tiled map, its tileset and a sheet of sprite frames, P3.10).

#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQmlEngine>
#include <QTemporaryDir>
#include <QTest>
#include <QVector3D>
#include <QtQml/qqmlextensionplugin.h>
#include <QtQuick3D/qquick3dobject.h>

#include "kinematics.h"
#include "mixer.h"
#include "run.h"
#include "sceneio.h"
#include "tilemap.h"
#include "webvoice.h"

Q_IMPORT_QML_PLUGIN(QQPlugin)

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

/// The entity called `name`, anywhere in the scene (in groups too, P3.15).
QObject *entityNamed(QObject *scene, const QString &name)
{
    for (QQuick3DObject *child : qobject_cast<QQuick3DObject *>(scene)->childItems()) {
        if (!child->property("kind").toString().isEmpty() && child->property("name").toString() == name)
            return child;
        if (QObject *inside = entityNamed(child, name))
            return inside;
    }
    return nullptr;
}

/// The lines of `after` that differ from `before`, when both have the same number of lines.
QStringList changedLines(const QString &before, const QString &after)
{
    const QStringList a = before.split(QLatin1Char('\n'));
    const QStringList b = after.split(QLatin1Char('\n'));
    if (a.size() != b.size())
        return {QStringLiteral("<line count changed: %1 to %2>").arg(a.size()).arg(b.size())};
    QStringList changed;
    for (qsizetype i = 0; i < a.size(); ++i) {
        if (a[i] != b[i])
            changed << b[i];
    }
    return changed;
}

}  // namespace

class TstScene : public QObject
{
    Q_OBJECT

    QQmlEngine *engine = nullptr;
    qq::SceneIO *io = nullptr;

private slots:
    void initTestCase()
    {
        engine = new QQmlEngine(this);
        io = engine->singletonInstance<qq::SceneIO *>("QQ", "SceneIO");
        QVERIFY(io);
    }

    void numbersAreWrittenOneWayOnly()
    {
        QCOMPARE(qq::formatNumber(1.0), QStringLiteral("1"));
        QCOMPARE(qq::formatNumber(0.1 + 0.2), QStringLiteral("0.3"));
        QCOMPARE(qq::formatNumber(12.345678), QStringLiteral("12.3457"));
        QCOMPARE(qq::formatNumber(-0.00001), QStringLiteral("0"));
        QCOMPARE(qq::formatNumber(double(1.15f)), QStringLiteral("1.15"));
        QCOMPARE(qq::formatNumber(-2.5), QStringLiteral("-2.5"));
    }

    void aSceneFileIsReadAndWrittenBackByteForByte()
    {
        const QString path = fixture(QStringLiteral("scenes/sample.qml"));
        QObject *scene = io->load(QUrl::fromLocalFile(path), nullptr);
        QVERIFY2(scene, qPrintable(io->lastError()));
        const QString written = io->write(scene, QUrl::fromLocalFile(path));
        QVERIFY2(written == readAll(path), qPrintable(changedLines(readAll(path), written).join(QLatin1Char('\n'))));
        // Every entity type is in it, in its file order.
        QStringList kinds;
        for (QQuick3DObject *child : qobject_cast<QQuick3DObject *>(scene)->childItems())
            kinds << child->property("kind").toString();
        QCOMPARE(kinds, (QStringList{QStringLiteral("Sun"), QStringLiteral("Ground"), QStringLiteral("Shape"),
                                     QStringLiteral("Lamp"), QStringLiteral("Prop"), QStringLiteral("Player"),
                                     QStringLiteral("Emitter"), QStringLiteral("Sound"), QStringLiteral("Behaviour"),
                                     QStringLiteral("Trigger"), QStringLiteral("ScreenText"), QStringLiteral("ScreenBar"),
                                     QStringLiteral("ScreenImage"), QStringLiteral("ScreenPanel"),
                                     QStringLiteral("ScreenButton"), QStringLiteral("InputAction"),
                                     QStringLiteral("Sprite"), QStringLiteral("TileMap"), QStringLiteral("Player2D"),
                                     QStringLiteral("Music"), QStringLiteral("Spotlight"), QStringLiteral("Group"),
                                     QStringLiteral("Prefab")}));
        // Every kind there is.
        QStringList sorted = kinds;
        sorted.sort();
        QStringList all = io->kinds();
        all.sort();
        QCOMPARE(sorted, all);
        delete scene;
    }

    void changingOneValueChangesOneLine()
    {
        const QString path = fixture(QStringLiteral("scenes/sample.qml"));
        const QUrl url = QUrl::fromLocalFile(path);
        QObject *scene = io->load(url, nullptr);
        QVERIFY(scene);
        const QString before = io->write(scene, url);

        QObject *plinth = entityNamed(scene, QStringLiteral("Plinth"));
        QVERIFY(plinth);
        plinth->setProperty("colour", QColor(QStringLiteral("#ff6a00")));
        QStringList changed = changedLines(before, io->write(scene, url));
        QCOMPARE(changed, QStringList{QStringLiteral("        colour: \"#ff6a00\"")});

        plinth->setProperty("position", QVector3D(0, 0.25f, -1.5f));
        changed = changedLines(before, io->write(scene, url));
        QCOMPARE(changed.size(), 2);
        QVERIFY(changed.contains(QStringLiteral("        position: Qt.vector3d(0, 0.25, -1.5)")));
        delete scene;
    }

    void aSavedSceneKeepsItsModelRelativeAndLoadsAgain()
    {
        QTemporaryDir project;
        QVERIFY(project.isValid());
        const QUrl projectUrl = QUrl::fromLocalFile(project.path());

        QObject *scene = io->load(QUrl::fromLocalFile(fixture(QStringLiteral("scenes/sample.qml"))), nullptr);
        QVERIFY(scene);
        QObject *orb = entityNamed(scene, QStringLiteral("Orb"));
        const QUrl adopted = io->adopt(io->resolvedUrl(orb, orb->property("source").toUrl()), projectUrl);
        QVERIFY2(!adopted.isEmpty(), qPrintable(io->lastError()));
        QCOMPARE(adopted.toLocalFile(), QDir(project.path()).filePath(QStringLiteral("assets/orb.gltf")));
        orb->setProperty("source", adopted);

        const QUrl file = QUrl::fromLocalFile(QDir(project.path()).filePath(QStringLiteral("scenes/sample.qml")));
        QCOMPARE(io->save(scene, file), QString());
        const QString saved = readAll(file.toLocalFile());
        QVERIFY2(saved.contains(QStringLiteral("        source: \"../assets/orb.gltf\"\n")), qPrintable(saved));
        QVERIFY(!saved.contains(project.path()));

        QObject *again = io->load(file, nullptr);
        QVERIFY2(again, qPrintable(io->lastError()));
        QObject *orbAgain = entityNamed(again, QStringLiteral("Orb"));
        QCOMPARE(io->resolvedUrl(orbAgain, orbAgain->property("source").toUrl()), adopted);
        QCOMPARE(io->write(again, file), saved);
        delete again;
        delete scene;
    }

    void adoptingAModelBringsItsFilesAndNeverOverwrites()
    {
        QTemporaryDir outside;
        QTemporaryDir project;
        const QDir out(outside.path());
        // A .gltf that keeps its buffer in a file beside it.
        QFile bin(out.filePath(QStringLiteral("tri.bin")));
        QVERIFY(bin.open(QIODevice::WriteOnly));
        bin.write(QByteArray(36, '\1'));
        bin.close();
        QFile gltf(out.filePath(QStringLiteral("tri.gltf")));
        QVERIFY(gltf.open(QIODevice::WriteOnly));
        gltf.write(R"({"asset":{"version":"2.0"},"buffers":[{"uri":"tri.bin","byteLength":36}]})");
        gltf.close();

        const QUrl projectUrl = QUrl::fromLocalFile(project.path());
        const QUrl first = io->adopt(QUrl::fromLocalFile(gltf.fileName()), projectUrl);
        QVERIFY2(!first.isEmpty(), qPrintable(io->lastError()));
        QVERIFY(QFileInfo::exists(QDir(project.path()).filePath(QStringLiteral("assets/tri.bin"))));
        // The same file again is the same asset.
        QCOMPARE(io->adopt(QUrl::fromLocalFile(gltf.fileName()), projectUrl), first);
        // A different file of the same name is kept beside it.
        QVERIFY(gltf.open(QIODevice::WriteOnly | QIODevice::Append));
        gltf.write(" ");
        gltf.close();
        const QUrl second = io->adopt(QUrl::fromLocalFile(gltf.fileName()), projectUrl);
        QCOMPARE(QFileInfo(second.toLocalFile()).fileName(), QStringLiteral("tri-2.gltf"));
        // A model already in the project stays where it is.
        QCOMPARE(io->adopt(first, projectUrl), first);
    }

    void soundsAndLogicAreAdoptedIntoTheirOwnFolders()
    {
        QTemporaryDir project;
        const QUrl projectUrl = QUrl::fromLocalFile(project.path());
        const QUrl sound = io->adopt(QUrl::fromLocalFile(fixture(QStringLiteral("chime.wav"))), projectUrl);
        QCOMPARE(sound.toLocalFile(), QDir(project.path()).filePath(QStringLiteral("assets/chime.wav")));
        const QUrl logic = io->adopt(QUrl::fromLocalFile(fixture(QStringLiteral("logic/spin.qml"))), projectUrl);
        QCOMPARE(logic.toLocalFile(), QDir(project.path()).filePath(QStringLiteral("logic/spin.qml")));
        QCOMPARE(readAll(logic.toLocalFile()), readAll(fixture(QStringLiteral("logic/spin.qml"))));
    }

    void playingIsACopyAndTheSceneIsNotTouched()
    {
        const QUrl url = QUrl::fromLocalFile(fixture(QStringLiteral("scenes/sample.qml")));
        QObject *scene = io->load(url, nullptr);
        QVERIFY(scene);
        const QString before = io->write(scene, url);
        QObject *copy = io->play(scene, url, nullptr);
        QVERIFY2(copy, qPrintable(io->lastError()));
        QVERIFY(copy != scene);
        QCOMPARE(copy->property("playing").toBool(), true);
        QCOMPARE(scene->property("playing").toBool(), false);
        // Every entity of the copy that changes in play knows it plays; none of the scene's does.
        int live = 0;
        for (const QVariant &e : io->entities(copy))
            live += e.value<QObject *>()->property("live").toBool() ? 1 : 0;
        // The ground, the shape, the model (its clip plays), the player, the sound, the behaviour, the trigger, the five
        // screen parts, the action, the sprite, the tile map, the 2D player, the music, the spotlight (its mark hidden),
        // and the group, the shape in it and the prefab (P3.15); the rest are the same in play.
        QCOMPARE(live, 21);
        for (const QVariant &e : io->entities(scene))
            QVERIFY(!e.value<QObject *>()->property("live").toBool());
        // The copy is the scene, and the scene is as it was; `playing` is not part of either's file.
        QCOMPARE(io->write(copy, url), before);
        QCOMPARE(io->write(scene, url), before);
        QVERIFY(!before.contains(QStringLiteral("playing")));
        QVERIFY(!io->play(entityNamed(scene, QStringLiteral("Orb")), url, nullptr));
        io->discard(copy);
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        delete scene;
    }

    void whatIsNotASceneIsRefusedWithAReason()
    {
        QVERIFY(!io->loadText(QStringLiteral("import QQ\nShape {}\n"), QUrl(), nullptr));
        QVERIFY2(io->lastError().contains(QStringLiteral("not a QQ scene")), qPrintable(io->lastError()));
        QVERIFY(!io->loadText(QStringLiteral("import QQ\nScene { this is not QML"), QUrl(), nullptr));
        QVERIFY(!io->lastError().isEmpty());
        QVERIFY(!io->load(QUrl::fromLocalFile(fixture(QStringLiteral("scenes/no-such-scene.qml"))), nullptr));
        QVERIFY(!io->lastError().isEmpty());
    }

    void aTwoDimensionalSceneIsImportedAsACanonicalOne()
    {
        const QVariantMap result = io->importQqJson(QUrl::fromLocalFile(fixture(QStringLiteral("first-light.qq.json"))));
        QCOMPARE(result.value(QStringLiteral("error")).toString(), QString());
        QCOMPARE(result.value(QStringLiteral("imported")).toInt(), 4);  // three shapes and an emitter
        const QStringList skipped = result.value(QStringLiteral("skipped")).toStringList();
        QCOMPARE(skipped.size(), 2);
        QVERIFY(skipped.filter(QStringLiteral("Core: follow")).size() == 1);
        QVERIFY(skipped.filter(QStringLiteral("Ring: motion")).size() == 1);

        const QString qml = result.value(QStringLiteral("qml")).toString();
        QObject *scene = io->loadText(qml, QUrl(), nullptr);
        QVERIFY2(scene, qPrintable(io->lastError()));
        QCOMPARE(io->write(scene), qml);  // canonical
        // Still a 2D scene (P3.10): flat, 100 pixels to a unit, the view its height of 540 pixels, its backdrop its own.
        QCOMPARE(scene->property("view").toString(), QStringLiteral("2d"));
        QCOMPARE(scene->property("pixelsPerUnit").toDouble(), 100.0);
        QCOMPARE(scene->property("viewHeight").toDouble(), 5.4);
        QCOMPARE(scene->property("pixelArt").toBool(), false);
        QCOMPARE(scene->property("skyTop").value<QColor>(), QColor(QStringLiteral("#07080c")));
        QObject *core = entityNamed(scene, QStringLiteral("Core"));
        QVERIFY(core);
        QCOMPARE(core->property("form").toString(), QStringLiteral("sphere"));
        QCOMPARE(core->property("position").value<QVector3D>(), QVector3D(0, 2.7f, 0));
        QCOMPARE(core->property("emissive").value<QColor>(), QColor(QStringLiteral("#ff6a00")));
        QObject *ring = entityNamed(scene, QStringLiteral("Ring"));
        QCOMPARE(ring->property("form").toString(), QStringLiteral("cube"));
        QCOMPARE(ring->property("position").value<QVector3D>(), QVector3D(-2.6f, 2.7f, 0));
        // The 2D emitter, as a 3D one where the core is: 110 pixels a second is 1.1 metres, and a full circle a burst.
        QObject *sparks = entityNamed(scene, QStringLiteral("Core sparks"));
        QVERIFY(sparks);
        QCOMPARE(sparks->property("kind").toString(), QStringLiteral("Emitter"));
        QCOMPARE(sparks->property("position").value<QVector3D>(), QVector3D(0, 2.7f, 0));
        QCOMPARE(sparks->property("rate").toDouble(), 90.0);
        QCOMPARE(sparks->property("spread").toDouble(), 1.1);
        QCOMPARE(sparks->property("colour").value<QColor>(), QColor(QStringLiteral("#ffb15c")));
        delete scene;
    }

    void aSceneSavedElsewhereStillPointsAtTheSameModel()
    {
        QTemporaryDir other;
        const QUrl from = QUrl::fromLocalFile(fixture(QStringLiteral("scenes/sample.qml")));
        QObject *scene = io->load(from, nullptr);
        QVERIFY(scene);
        const QUrl moved = QUrl::fromLocalFile(QDir(other.path()).filePath(QStringLiteral("deep/down/moved.qml")));
        QCOMPARE(io->save(scene, moved), QString());
        QObject *again = io->load(moved, nullptr);
        QVERIFY2(again, qPrintable(io->lastError()));
        QObject *orb = entityNamed(again, QStringLiteral("Orb"));
        QCOMPARE(io->resolvedUrl(orb, orb->property("source").toUrl()),
                 QUrl::fromLocalFile(fixture(QStringLiteral("orb.gltf"))));
        delete again;
        delete scene;
    }

    void entitiesAreAddedAndRemovedAndTheFileFollows()
    {
        const QUrl url = QUrl::fromLocalFile(fixture(QStringLiteral("scenes/sample.qml")));
        QObject *scene = io->load(url, nullptr);
        QVERIFY(scene);
        QCOMPARE(io->entities(scene).size(), 24);  // the group's shape among them, not the prefab's parts

        QObject *crystal = io->add(scene, QStringLiteral("Shape"),
                                   {{QStringLiteral("name"), QStringLiteral("Crystal")},
                                    {QStringLiteral("form"), QStringLiteral("cone")},
                                    {QStringLiteral("position"), QVector3D(2, 0.5f, -1)}});
        QVERIFY2(crystal, qPrintable(io->lastError()));
        QCOMPARE(io->entities(scene).size(), 25);
        const QString text = io->write(scene, url);
        QVERIFY2(text.endsWith(QStringLiteral("    Shape {\n        name: \"Crystal\"\n        position: Qt.vector3d(2, 0.5, -1)\n")
                               + QStringLiteral("        eulerRotation: Qt.vector3d(0, 0, 0)\n        scale: Qt.vector3d(1, 1, 1)\n")
                               + QStringLiteral("        form: \"cone\"\n        colour: \"#5ad1ff\"\n        metalness: 0\n")
                               + QStringLiteral("        roughness: 0.4\n        emissive: \"#000000\"\n        emissivePower: 0\n")
                               + QStringLiteral("        image: \"\"\n        imageSize: 1\n        normalMap: \"\"\n        crisp: false\n")
                               + QStringLiteral("        body: \"static\"\n        mass: 1\n        bounce: 0.2\n        friction: 0.6\n")
                               + QStringLiteral("    }\n}\n")),
                 qPrintable(text.right(400)));

        // What is not an entity kind, or not a scene, is refused.
        QVERIFY(!io->add(scene, QStringLiteral("Window"), {}));
        QVERIFY(io->lastError().contains(QStringLiteral("no entity called")));
        QVERIFY(!io->add(crystal, QStringLiteral("Shape"), {}));

        QVERIFY(io->remove(crystal));
        QVERIFY(!io->remove(scene));
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        QCOMPARE(io->entities(scene).size(), 24);
        QCOMPARE(io->write(scene, url), readAll(url.toLocalFile()));
        delete scene;
    }

    void aProjectFolderListsItsScenes()
    {
        QTemporaryDir project;
        const QDir root(project.path());
        root.mkpath(QStringLiteral("scenes"));
        for (const char *name : {"b-level.qml", "a-level.qml", "notes.txt"}) {
            QFile f(root.filePath(QStringLiteral("scenes/") + QLatin1String(name)));
            QVERIFY(f.open(QIODevice::WriteOnly));
        }
        QVariantMap info = io->project(QUrl::fromLocalFile(project.path()));
        QCOMPARE(info.value(QStringLiteral("scenes")).toStringList(),
                 (QStringList{QStringLiteral("a-level"), QStringLiteral("b-level")}));
        QCOMPARE(info.value(QStringLiteral("repository")).toBool(), false);
        root.mkdir(QStringLiteral(".git"));
        info = io->project(QUrl::fromLocalFile(project.path()));
        QCOMPARE(info.value(QStringLiteral("repository")).toBool(), true);
    }

    void theBundledSampleCanBeAdoptedIntoAProject()
    {
        QTemporaryDir project;
        const QUrl adopted = io->adopt(QUrl(QStringLiteral("qrc:/fixtures/orb.gltf")),
                                       QUrl::fromLocalFile(project.path()));
        QVERIFY2(!adopted.isEmpty(), qPrintable(io->lastError()));
        QVERIFY(QFileInfo::exists(adopted.toLocalFile()));
    }

    void aFileOfAnotherFormatIsNotImported()
    {
        QTemporaryDir dir;
        QFile f(QDir(dir.path()).filePath(QStringLiteral("two.qq.json")));
        QVERIFY(f.open(QIODevice::WriteOnly));
        f.write(R"({"qq": 2, "entities": []})");
        f.close();
        const QVariantMap result = io->importQqJson(QUrl::fromLocalFile(f.fileName()));
        QVERIFY(result.value(QStringLiteral("error")).toString().contains(QStringLiteral("format 1")));
        QVERIFY(result.value(QStringLiteral("qml")).toString().isEmpty());
    }

    // ── 2D (P3.10, ADR-105) ──

    void aTiledMapIsReadWithItsTilesetAndItsSolidsMerged()
    {
        const QUrl level = QUrl::fromLocalFile(fixture(QStringLiteral("level.tmj")));
        QVariantMap map = io->readTiledMap(level);
        QCOMPARE(map.value(QStringLiteral("error")).toString(), QString());
        QCOMPARE(map.value(QStringLiteral("width")).toInt(), 10);
        QCOMPARE(map.value(QStringLiteral("height")).toInt(), 5);
        QCOMPARE(map.value(QStringLiteral("tileWidth")).toInt(), 16);
        // The tileset is the external one beside the map, and its image is beside that.
        QCOMPARE(map.value(QStringLiteral("image")).toUrl(), QUrl::fromLocalFile(fixture(QStringLiteral("tiles.png"))));
        QCOMPARE(map.value(QStringLiteral("imageWidth")).toInt(), 32);
        QCOMPARE(map.value(QStringLiteral("tileColumns")).toInt(), 2);
        QCOMPARE(map.value(QStringLiteral("tileCount")).toInt(), 4);
        QCOMPARE(map.value(QStringLiteral("firstGid")).toInt(), 1);
        const QVariantList gids = map.value(QStringLiteral("gids")).toList();
        QCOMPARE(gids.size(), 50);
        QCOMPARE(gids.at(9).toLongLong(), 3);                       // the wall's top
        QCOMPARE(gids.at(44).toLongLong(), qint64(0x80000002u));    // the mirrored green, its flag kept
        // The wall, the ledge and the floor: three colliders for sixteen tiles.
        const QVariantList solids = map.value(QStringLiteral("solids")).toList();
        QCOMPARE(solids, (QVariantList{QVariant(QVariantList{9, 0, 1, 4}), QVariant(QVariantList{5, 1, 2, 1}),
                                        QVariant(QVariantList{0, 4, 10, 1})}));

        // Another layer, by its name.
        map = io->readTiledMap(level, QStringLiteral("decor"));
        QCOMPARE(map.value(QStringLiteral("solids")).toList(), (QVariantList{QVariant(QVariantList{2, 2, 1, 1})}));
        map = io->readTiledMap(level, QStringLiteral("clouds"));
        QVERIFY2(map.value(QStringLiteral("error")).toString().contains(QStringLiteral("no tile layer called clouds")),
                 qPrintable(map.value(QStringLiteral("error")).toString()));

        // The image's size, for a sprite's.
        QCOMPARE(io->imageSize(QUrl::fromLocalFile(fixture(QStringLiteral("sheet.png")))), QSize(32, 16));
        QCOMPARE(io->imageSize(QUrl::fromLocalFile(fixture(QStringLiteral("no-such.png")))), QSize());
    }

    void aMapSavedAsBase64WithItsTilesetInsideReadsTheSame()
    {
        // Tiled's other common way: tiles as zlib-compressed Base64, the tileset kept in the map.
        QFile f(fixture(QStringLiteral("level.tmj")));
        QVERIFY(f.open(QIODevice::ReadOnly));
        QJsonObject json = QJsonDocument::fromJson(f.readAll()).object();
        QJsonArray layers = json.value(QStringLiteral("layers")).toArray();
        QJsonObject ground = layers.at(0).toObject();
        QByteArray bytes;
        for (const QJsonValue &v : ground.value(QStringLiteral("data")).toArray()) {
            const auto gid = quint32(v.toDouble());
            for (int i = 0; i < 4; ++i)
                bytes.append(char((gid >> (8 * i)) & 0xff));
        }
        // qCompress puts the size first; Tiled's stream is the zlib stream alone.
        ground.insert(QStringLiteral("data"), QString::fromLatin1(qCompress(bytes).mid(4).toBase64()));
        ground.insert(QStringLiteral("encoding"), QStringLiteral("base64"));
        ground.insert(QStringLiteral("compression"), QStringLiteral("zlib"));
        layers.replace(0, ground);
        json.insert(QStringLiteral("layers"), layers);
        json.insert(QStringLiteral("tilesets"), QJsonArray{QJsonObject{
            {QStringLiteral("firstgid"), 1}, {QStringLiteral("columns"), 2}, {QStringLiteral("image"), QStringLiteral("art/tiles.png")},
            {QStringLiteral("imagewidth"), 32}, {QStringLiteral("imageheight"), 32}, {QStringLiteral("tilecount"), 4},
            {QStringLiteral("tilewidth"), 16}, {QStringLiteral("tileheight"), 16}}});
        QTemporaryDir dir;
        QVERIFY(QDir(dir.path()).mkpath(QStringLiteral("art")));
        QVERIFY(QFile::copy(fixture(QStringLiteral("tiles.png")), QDir(dir.path()).filePath(QStringLiteral("art/tiles.png"))));
        QFile out(QDir(dir.path()).filePath(QStringLiteral("packed.tmj")));
        QVERIFY(out.open(QIODevice::WriteOnly));
        out.write(QJsonDocument(json).toJson());
        out.close();

        const QVariantMap plain = io->readTiledMap(QUrl::fromLocalFile(fixture(QStringLiteral("level.tmj"))));
        const QVariantMap packed = io->readTiledMap(QUrl::fromLocalFile(out.fileName()));
        QCOMPARE(packed.value(QStringLiteral("error")).toString(), QString());
        QCOMPARE(packed.value(QStringLiteral("gids")), plain.value(QStringLiteral("gids")));
        QCOMPARE(packed.value(QStringLiteral("solids")), plain.value(QStringLiteral("solids")));
        QCOMPARE(packed.value(QStringLiteral("image")).toUrl(),
                 QUrl::fromLocalFile(QDir(dir.path()).filePath(QStringLiteral("art/tiles.png"))));
        QCOMPARE(qq::tiledMapNeeds(out.fileName()), QStringList{QStringLiteral("art/tiles.png")});
    }

    void whatAMapCannotBeIsRefusedWithAReason_data()
    {
        QTest::addColumn<QByteArray>("change");
        QTest::addColumn<QString>("reason");
        QTest::newRow("infinite") << QByteArray("\"infinite\": true") << QStringLiteral("Infinite maps");
        QTest::newRow("isometric") << QByteArray("\"orientation\": \"isometric\"") << QStringLiteral("orthogonal");
        QTest::newRow("zstd") << QByteArray("\"compression\": \"zstd\"") << QStringLiteral("zstd");
        QTest::newRow("short") << QByteArray("\"width\": 11") << QStringLiteral("room for 55");
        QTest::newRow("no tileset") << QByteArray("\"tilesets\": []") << QStringLiteral("no tileset");
        QTest::newRow("images") << QByteArray("\"tilesets\": [{\"firstgid\": 1, \"tiles\": []}]")
                                << QStringLiteral("collection of images");
    }

    void whatAMapCannotBeIsRefusedWithAReason()
    {
        QFETCH(QByteArray, change);
        QFETCH(QString, reason);
        QFile f(fixture(QStringLiteral("level.tmj")));
        QVERIFY(f.open(QIODevice::ReadOnly));
        QJsonObject json = QJsonDocument::fromJson(f.readAll()).object();
        const QJsonObject patch = QJsonDocument::fromJson("{" + change + "}").object();
        QVERIFY(!patch.isEmpty());
        for (auto it = patch.begin(); it != patch.end(); ++it) {
            if (it.key() == QStringLiteral("compression")) {
                QJsonArray layers = json.value(QStringLiteral("layers")).toArray();
                QJsonObject ground = layers.at(0).toObject();
                ground.insert(QStringLiteral("encoding"), QStringLiteral("base64"));
                ground.insert(QStringLiteral("data"), QStringLiteral("AAAA"));
                ground.insert(it.key(), it.value());
                layers.replace(0, ground);
                json.insert(QStringLiteral("layers"), layers);
            } else {
                json.insert(it.key(), it.value());
            }
        }
        QTemporaryDir dir;
        QFile out(QDir(dir.path()).filePath(QStringLiteral("bad.tmj")));
        QVERIFY(out.open(QIODevice::WriteOnly));
        out.write(QJsonDocument(json).toJson());
        out.close();
        const QString error = io->readTiledMap(QUrl::fromLocalFile(out.fileName())).value(QStringLiteral("error")).toString();
        QVERIFY2(error.contains(reason), qPrintable(error));
    }

    void solidTilesBecomeTheFewestRectangles()
    {
        // A U: each side is a run joined down the rows, and its foot is one run.
        //   x . . x
        //   x . . x
        //   x x x x
        const QList<int> grid{1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1};
        QCOMPARE(qq::mergedSolids(grid, 4, 3),
                 (QList<QList<int>>{{0, 0, 1, 2}, {3, 0, 1, 2}, {0, 2, 4, 1}}));
        QCOMPARE(qq::mergedSolids(QList<int>(6, 0), 3, 2), QList<QList<int>>{});
        QCOMPARE(qq::mergedSolids(QList<int>(6, 1), 3, 2), (QList<QList<int>>{{0, 0, 3, 2}}));
    }

    void aMapAdoptedIntoAProjectBringsItsTilesetAndImage()
    {
        QTemporaryDir project;
        const QUrl adopted = io->adopt(QUrl::fromLocalFile(fixture(QStringLiteral("level.tmj"))),
                                       QUrl::fromLocalFile(project.path()));
        QVERIFY2(!adopted.isEmpty(), qPrintable(io->lastError()));
        const QDir assets(QDir(project.path()).filePath(QStringLiteral("assets")));
        QVERIFY(QFileInfo::exists(assets.filePath(QStringLiteral("tiles.tsj"))));
        QVERIFY(QFileInfo::exists(assets.filePath(QStringLiteral("tiles.png"))));
        const QVariantMap map = io->readTiledMap(adopted);
        QCOMPARE(map.value(QStringLiteral("error")).toString(), QString());
        QCOMPARE(map.value(QStringLiteral("image")).toUrl(), QUrl::fromLocalFile(assets.filePath(QStringLiteral("tiles.png"))));

        // A map whose tileset is outside its folder could not keep that path, so it is refused, and nothing is copied.
        QTemporaryDir elsewhere;
        const QDir root(elsewhere.path());
        QVERIFY(root.mkpath(QStringLiteral("maps")));
        QVERIFY(QFile::copy(fixture(QStringLiteral("tiles.tsj")), root.filePath(QStringLiteral("tiles.tsj"))));
        QFile f(fixture(QStringLiteral("level.tmj")));
        QVERIFY(f.open(QIODevice::ReadOnly));
        QByteArray text = f.readAll();
        text.replace("\"tiles.tsj\"", "\"../tiles.tsj\"");
        QFile out(root.filePath(QStringLiteral("maps/far.tmj")));
        QVERIFY(out.open(QIODevice::WriteOnly));
        out.write(text);
        out.close();
        QVERIFY(io->adopt(QUrl::fromLocalFile(out.fileName()), QUrl::fromLocalFile(project.path())).isEmpty());
        QVERIFY2(io->lastError().contains(QStringLiteral("outside its folder")), qPrintable(io->lastError()));
        QVERIFY(!QFileInfo::exists(assets.filePath(QStringLiteral("far.tmj"))));
    }

    // ── steps and runs (P3.11, ADR-105) ──

    void theSameSeedGivesTheSameNumbersEverywhere()
    {
        // Pinned against an independent implementation of xoshiro128** seeded by splitmix32 (written in Python for this
        // check): a change here would make every run kept before it replay differently.
        qq::Random random;
        random.reset(42);
        QCOMPARE(random.next(), 0.6606157208001621);
        QCOMPARE(random.next(), 0.1117019639532224);
        QCOMPARE(random.next(), 0.0791056456144914);
        QCOMPARE(random.drawn(), 3);
        random.reset(42);
        QList<int> dice;
        for (int i = 0; i < 10; ++i)
            dice << random.between(1, 6);
        QCOMPARE(dice, (QList<int>{4, 1, 1, 1, 6, 2, 6, 5, 3, 4}));
        // Another seed, other numbers; never outside [0, 1) or the range asked.
        random.reset(43);
        QVERIFY(random.next() != 0.6606157208001621);
        for (int i = 0; i < 10000; ++i) {
            const double v = random.next();
            QVERIFY(v >= 0 && v < 1);
            const int d = random.between(-3, 3);
            QVERIFY(d >= -3 && d <= 3);
        }
        QVERIFY(qq::Random::freshSeed() > 0);
    }

    void aBoxIsStoppedByWhatStandsInItsWay()
    {
        // A floor from x -10 to 10, its top at y 0, and a wall from x 2 to 3, up to y 5.
        const QList<qq::Box> solids{{-10, 10, -1, 0, -1, 1}, {2, 3, 0, 5, -1, 1}};
        // Falling onto the floor: stopped with its foot exactly on the top.
        qq::Moved m = qq::sweep2D(solids, 0, 0.6, 0, 0.5, 0.5, 0, -0.3);
        QCOMPARE(m.y, 0.5);
        QVERIFY(m.down && !m.up && !m.left && !m.right);
        // Resting on it, touching is not overlapping: it walks along freely.
        m = qq::sweep2D(solids, 0, 0.5, 0, 0.5, 0.5, 0.4, 0);
        QCOMPARE(m.x, 0.4);
        QCOMPARE(m.y, 0.5);
        QVERIFY(!m.right && !m.down);
        // Into the wall: stopped with its side on the wall's.
        m = qq::sweep2D(solids, 1.4, 0.5, 0, 0.5, 0.5, 0.3, 0);
        QCOMPARE(m.x, 1.5);
        QVERIFY(m.right);
        // From the other side, and under a ceiling.
        m = qq::sweep2D(solids, 3.6, 0.5, 0, 0.5, 0.5, -0.3, 0);
        QCOMPARE(m.x, 3.5);
        QVERIFY(m.left);
        const QList<qq::Box> roof{{-10, 10, 2, 3, -1, 1}};
        m = qq::sweep2D(roof, 0, 1.4, 0, 0.5, 0.5, 0, 0.2);
        QCOMPARE(m.y, 1.5);
        QVERIFY(m.up);
        // What is at another depth is not in the way.
        m = qq::sweep2D(solids, 1.4, 0.5, 5, 0.5, 0.5, 0.3, 0);
        QCOMPARE(m.x, 1.7);
        QVERIFY(!m.right);
        // Across first, then down: a box falling past a wall's corner keeps to its side.
        m = qq::sweep2D(solids, 1.4, 6, 0, 0.5, 0.5, 0.3, -0.6);
        QCOMPARE(m.x, 1.7);
        QCOMPARE(m.y, 5.5);
        QVERIFY(m.down);

        QVERIFY(qq::overlaps({0, 1, 0, 1, 0, 1}, {0.5, 2, 0.5, 2, 0.5, 2}));
        QVERIFY(!qq::overlaps({0, 1, 0, 1, 0, 1}, {1, 2, 0, 1, 0, 1}));  // touching
    }

    void aRunIsKeptAndReadBack()
    {
        qq::RunRecord run;
        run.setScene(QStringLiteral("climb"));
        run.setSeed(7);
        const QVariantMap still{{QStringLiteral("move"), QVariantList{0.0, 0.0}},
                                {QStringLiteral("look"), QVariantList{0.0, 0.0}},
                                {QStringLiteral("held"), QStringList()},
                                {QStringLiteral("edges"), QVariantList()}};
        QVariantMap right = still;
        right[QStringLiteral("move")] = QVariantList{1.0, 0.0};
        right[QStringLiteral("held")] = QStringList{QStringLiteral("run")};
        right[QStringLiteral("edges")] = QVariantList{QVariant(QVariantList{QStringLiteral("pressed"), QStringLiteral("run")})};
        run.capture(0, still, {});
        run.capture(1, still, {QStringLiteral("Play")});
        run.capture(2, right, {});
        QVariantMap holding = right;
        holding[QStringLiteral("edges")] = QVariantList();
        run.capture(3, holding, {});
        run.setOutcome({{QStringLiteral("Rules"), QVariantMap{{QStringLiteral("gathered"), 2}}}});
        QCOMPARE(run.ticks(), 4);

        // Only what changed is kept, and every tick is given back as it was taken.
        auto check = [](const qq::RunRecord &r) {
            QCOMPARE(r.handsAt(0).value(QStringLiteral("move")).toList(), (QVariantList{0.0, 0.0}));
            QCOMPARE(r.clicksAt(1), QStringList{QStringLiteral("Play")});
            QCOMPARE(r.handsAt(2).value(QStringLiteral("move")).toList(), (QVariantList{1.0, 0.0}));
            QCOMPARE(r.handsAt(2).value(QStringLiteral("edges")).toList().size(), 1);
            QCOMPARE(r.handsAt(3).value(QStringLiteral("held")).toStringList(), QStringList{QStringLiteral("run")});
            QCOMPARE(r.handsAt(3).value(QStringLiteral("edges")).toList().size(), 0);
            QCOMPARE(r.handsAt(250).value(QStringLiteral("move")).toList(), (QVariantList{1.0, 0.0}));
        };
        check(run);
        const QByteArray json = run.toJson();
        QCOMPARE(QJsonDocument::fromJson(json).object().value(QStringLiteral("hands")).toArray().size(), 2);

        qq::RunRecord back;
        QString why;
        QVERIFY2(back.fromJson(json, &why), qPrintable(why));
        QCOMPARE(back.scene(), QStringLiteral("climb"));
        QCOMPARE(back.seed(), 7);
        QCOMPARE(back.ticks(), 4);
        QCOMPARE(back.outcome().value(QStringLiteral("Rules")).toMap().value(QStringLiteral("gathered")).toInt(), 2);
        check(back);
        QCOMPARE(back.toJson(), json);

        QVERIFY(!back.fromJson("{\"qqRun\": 2}", &why));
        QVERIFY2(why.contains(QStringLiteral("format 1")), qPrintable(why));
        QVERIFY(!back.fromJson("not json", &why));
    }

    // ── groups and prefabs (P3.15) ──

    void groupsNestEntitiesAndTheFileKeepsThem()
    {
        QTemporaryDir dir;
        const QUrl url = QUrl::fromLocalFile(QDir(dir.path()).filePath(QStringLiteral("scenes/nest.qml")));
        QObject *scene = io->loadText(QStringLiteral(
            "import QQ\nScene {\n    name: \"nest\"\n"
            "    Group {\n        name: \"Post\"\n        position: Qt.vector3d(5, 0, 0)\n"
            "        Shape { name: \"Pole\"; position: Qt.vector3d(1, 0, 0); scale: Qt.vector3d(0.2, 3, 0.2) }\n"
            "        Group { name: \"Top\"; position: Qt.vector3d(0, 3, 0)\n"
            "            Lamp { name: \"Light\" }\n        }\n    }\n"
            "    Shape { name: \"Floor\" }\n}\n"), url, nullptr);
        QVERIFY2(scene, qPrintable(io->lastError()));
        // Kept by the file: written inside its group, and read back the same bytes.
        const QString text = io->write(scene, url);
        QVERIFY(text.contains(QStringLiteral("    Group {\n        name: \"Post\"\n")));
        QVERIFY(text.contains(QStringLiteral("\n        Shape {\n            name: \"Pole\"\n")));
        QVERIFY(text.contains(QStringLiteral("\n            Lamp {\n                name: \"Light\"\n")));
        QObject *again = io->loadText(text, url, nullptr);
        QCOMPARE(io->write(again, url), text);
        delete again;

        // Listed in file order, each after its group; the scene found from inside.
        QStringList names;
        for (const QVariant &e : io->entities(scene))
            names << e.value<QObject *>()->property("name").toString();
        QCOMPARE(names, (QStringList{QStringLiteral("Post"), QStringLiteral("Pole"), QStringLiteral("Top"),
                                     QStringLiteral("Light"), QStringLiteral("Floor")}));
        QObject *light = entityNamed(scene, QStringLiteral("Light"));
        QCOMPARE(io->sceneOf(light), scene);
        QCOMPARE(light->property("scenePosition").value<QVector3D>(), QVector3D(5, 3, 0));

        // Added into a group; a player cannot be.
        QObject *post = entityNamed(scene, QStringLiteral("Post"));
        QObject *sign = io->add(scene, QStringLiteral("Shape"), {{QStringLiteral("name"), QStringLiteral("Sign")}}, post);
        QVERIFY2(sign, qPrintable(io->lastError()));
        QCOMPARE(qobject_cast<QQuick3DObject *>(sign)->parentItem(), qobject_cast<QQuick3DObject *>(post));
        QVERIFY(!io->add(scene, QStringLiteral("Player"), {}, post));
        QVERIFY(io->lastError().contains(QStringLiteral("cannot be in a group")));

        // Moved between groups where it is in the world: out to the scene, and into the top.
        QObject *pole = entityNamed(scene, QStringLiteral("Pole"));
        QVERIFY(io->reparent(pole, scene));
        QCOMPARE(pole->property("position").value<QVector3D>(), QVector3D(6, 0, 0));
        QObject *top = entityNamed(scene, QStringLiteral("Top"));
        QVERIFY(io->reparent(pole, top));
        QCOMPARE(pole->property("position").value<QVector3D>(), QVector3D(1, -3, 0));
        QCOMPARE(pole->property("scenePosition").value<QVector3D>(), QVector3D(6, 0, 0));
        // A group cannot go into what is in it.
        QVERIFY(!io->reparent(post, top));

        // Removed, a group takes what is in it.
        QVERIFY(io->remove(post));
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        names.clear();
        for (const QVariant &e : io->entities(scene))
            names << e.value<QObject *>()->property("name").toString();
        QCOMPARE(names, QStringList{QStringLiteral("Floor")});
        delete scene;
    }

    void kinematicsMeasureThroughGroupsInDoubles()
    {
        // A wall in a group moved 5 along and turned a quarter about the depth axis: its box, 1 by 2 in the group, is 2
        // by 1 in the world, at (5, 1): a 2D mover from the left stops against it, and an overlap is measured there.
        QObject *scene = io->loadText(QStringLiteral(
            "import QQ\nScene {\n    name: \"walls\"\n    view: \"2d\"\n"
            "    Group {\n        name: \"Turned\"\n        position: Qt.vector3d(5, 1, 0)\n"
            "        eulerRotation: Qt.vector3d(0, 0, 90)\n"
            "        Shape { name: \"Wall\"; scale: Qt.vector3d(1, 2, 1) }\n"
            "        Trigger { name: \"Zone\"; position: Qt.vector3d(0, -3, 0) }\n    }\n"
            "    Trigger { name: \"Probe\"; position: Qt.vector3d(8, 1, 0) }\n}\n"),
            QUrl::fromLocalFile(QStringLiteral("/tmp/walls.qml")), nullptr);
        QVERIFY2(scene, qPrintable(io->lastError()));
        QObject *probe = entityNamed(scene, QStringLiteral("Probe"));
        // A step of 4 (a mover's steps are small: this is where it ends that is measured).
        const QVariantMap moved = io->move2D(scene, probe, 0, 1, 0, 0.5, 0.25, 4, 0);
        // The wall's left side is at 5 - 1 = 4; the mover, half a unit wide, stops at 3.5.
        QCOMPARE(moved.value(QStringLiteral("x")).toDouble(), 3.5);
        QVERIFY(moved.value(QStringLiteral("right")).toBool());
        // The zone, 3 below the group's middle before its turn, is 3 to the right after it: at (8, 1), over the probe.
        QVERIFY(io->overlaps(entityNamed(scene, QStringLiteral("Zone")), probe));
        delete scene;
    }

    void aPrefabIsMadeFromItsFileAndNeverWrittenIntoTheScene()
    {
        QTemporaryDir project;
        const QDir root(project.path());
        root.mkpath(QStringLiteral("prefabs"));
        root.mkpath(QStringLiteral("scenes"));
        const QString prefab = QStringLiteral(
            "import QQ\n\nGroup {\n    name: \"Lamp post\"\n    position: Qt.vector3d(0, 0, 0)\n"
            "    eulerRotation: Qt.vector3d(0, 0, 0)\n    scale: Qt.vector3d(1, 1, 1)\n\n"
            "    Shape { name: \"Pole\"; scale: Qt.vector3d(0.2, 3, 0.2) }\n    Lamp { name: \"Light\"; position: Qt.vector3d(0, 3, 0) }\n}\n");
        {
            QFile f(root.filePath(QStringLiteral("prefabs/lamp-post.qml")));
            QVERIFY(f.open(QIODevice::WriteOnly));
            f.write(prefab.toUtf8());
        }
        const QUrl url = QUrl::fromLocalFile(root.filePath(QStringLiteral("scenes/street.qml")));
        QObject *scene = io->loadText(QStringLiteral(
            "import QQ\nScene {\n    name: \"street\"\n"
            "    Prefab { name: \"West\"; source: \"../prefabs/lamp-post.qml\"; position: Qt.vector3d(-4, 0, 0) }\n"
            "    Prefab { name: \"East\"; source: \"../prefabs/lamp-post.qml\"; position: Qt.vector3d(4, 0, 0) }\n}\n"),
            url, nullptr);
        QVERIFY2(scene, qPrintable(io->lastError()));
        // The scene's own entities are the two prefabs; each is made of the file's parts, where it stands.
        QCOMPARE(io->entities(scene).size(), 2);
        QCOMPARE(io->everyEntity(scene).size(), 8);  // each prefab, its root group, the pole and the light
        QObject *east = entityNamed(scene, QStringLiteral("East"));
        QCOMPARE(east->property("error").toString(), QString());
        QVariant light;
        QMetaObject::invokeMethod(east, "part", Q_RETURN_ARG(QVariant, light), Q_ARG(QVariant, QStringLiteral("Light")));
        QVERIFY(light.value<QObject *>());
        QCOMPARE(light.value<QObject *>()->property("scenePosition").value<QVector3D>(), QVector3D(4, 3, 0));
        // Written, the scene keeps where each stands and its file; never their parts.
        const QString text = io->write(scene, url);
        QVERIFY(text.contains(QStringLiteral("        source: \"../prefabs/lamp-post.qml\"\n")));
        QVERIFY(!text.contains(QStringLiteral("Pole")));
        QVERIFY(!text.contains(QStringLiteral("Lamp {")));
        // A project lists its prefabs.
        QCOMPARE(io->project(QUrl::fromLocalFile(project.path())).value(QStringLiteral("prefabs")).toStringList(),
                 QStringList{QStringLiteral("lamp-post")});

        // A file whose root is not a Group is no prefab, and says so.
        {
            QFile f(root.filePath(QStringLiteral("prefabs/not.qml")));
            QVERIFY(f.open(QIODevice::WriteOnly));
            f.write("import QQ\nShape { name: \"Lonely\" }\n");
        }
        QObject *bad = io->add(scene, QStringLiteral("Prefab"), {{QStringLiteral("source"), QStringLiteral("../prefabs/not.qml")}});
        QVERIFY(bad);
        QVERIFY2(bad->property("error").toString().contains(QStringLiteral("root is not a Group")),
                 qPrintable(bad->property("error").toString()));

        // A group written as a prefab: at the origin, the same bytes as the file a person would keep; never over one.
        QObject *street = io->loadText(QStringLiteral(
            "import QQ\nScene {\n    name: \"street\"\n"
            "    Group {\n        name: \"Lamp post\"\n        position: Qt.vector3d(7, 0, 2)\n"
            "        Shape { name: \"Pole\"; scale: Qt.vector3d(0.2, 3, 0.2) }\n"
            "        Lamp { name: \"Light\"; position: Qt.vector3d(0, 3, 0) }\n    }\n}\n"), url, nullptr);
        QObject *group = entityNamed(street, QStringLiteral("Lamp post"));
        QVERIFY(io->writePrefab(group, QUrl::fromLocalFile(project.path()), QStringLiteral("Lamp post")).isEmpty());
        QVERIFY(io->lastError().contains(QStringLiteral("there already")));
        const QUrl written = io->writePrefab(group, QUrl::fromLocalFile(project.path()), QStringLiteral("Corner Lamp"));
        QCOMPARE(written, QUrl::fromLocalFile(root.filePath(QStringLiteral("prefabs/corner-lamp.qml"))));
        const QString kept = readAll(written.toLocalFile());
        QVERIFY(kept.startsWith(QStringLiteral("import QQ\n\nGroup {\n    name: \"Lamp post\"\n    position: Qt.vector3d(0, 0, 0)\n")));
        QVERIFY(kept.contains(QStringLiteral("        name: \"Pole\"\n")));
        QCOMPARE(group->property("position").value<QVector3D>(), QVector3D(7, 0, 2));  // the group itself stays put

        // Brought into another project, a prefab goes into its prefabs/ folder.
        QTemporaryDir other;
        const QUrl adopted = io->adopt(written, QUrl::fromLocalFile(other.path()));
        QCOMPARE(adopted, QUrl::fromLocalFile(QDir(other.path()).filePath(QStringLiteral("prefabs/corner-lamp.qml"))));
        delete street;
        delete scene;
    }

    // ── sound (P3.13, ADR-105) ──

    void aSoundFilesLengthAndTheMixersLevelsAreRead()
    {
        // chime.wav (make_chime.py) is one second long; what is not a WAV file has no length that can be told.
        qq::Mixer mixer;
        QCOMPARE(mixer.lengthOf(QUrl::fromLocalFile(fixture(QStringLiteral("chime.wav")))), 1.0);
        QCOMPARE(mixer.lengthOf(QUrl::fromLocalFile(fixture(QStringLiteral("orb.gltf")))), 0.0);
        QCOMPARE(qq::wavLength(QByteArray("RIFF\0\0\0\0WAVE", 12)), 0.0);
        // A group never set is at 1; levels are held to 0 to 1; what a sound is multiplied by is its group's level
        // times the master's.
        QCOMPARE(mixer.level(QStringLiteral("music")), 1.0);
        mixer.setLevel(QStringLiteral("music"), 0.5);
        mixer.setLevel(QStringLiteral("master"), 0.5);
        mixer.setLevel(QStringLiteral("ui"), 2);
        QCOMPARE(mixer.level(QStringLiteral("ui")), 1.0);
        QCOMPARE(mixer.gain(QStringLiteral("music")), 0.25);
        QCOMPARE(mixer.gain(QStringLiteral("effects")), 0.5);
        QCOMPARE(mixer.gain(QStringLiteral("master")), 0.5);
        QCOMPARE(mixer.levels().value(QStringLiteral("music")).toDouble(), 0.5);
        // Reset (the Studio's Stop): every group back at 1.
        mixer.reset();
        QCOMPARE(mixer.gain(QStringLiteral("music")), 1.0);
        QCOMPARE(mixer.levels().size(), 1);

        // A file replaced under the same name is measured again: half a second, then two.
        const auto wav = [](quint32 bytes) {
            QByteArray b("RIFF");
            const auto u32 = [&b](quint32 v) {
                for (int i = 0; i < 4; ++i)
                    b.append(char((v >> (8 * i)) & 0xff));
            };
            const auto u16 = [&b](quint16 v) {
                b.append(char(v & 0xff));
                b.append(char(v >> 8));
            };
            u32(36 + bytes);
            b.append("WAVEfmt ");
            u32(16);
            u16(1);     // PCM
            u16(1);     // one channel
            u32(1000);  // samples a second
            u32(1000);  // bytes a second
            u16(1);
            u16(8);
            b.append("data");
            u32(bytes);
            b.append(QByteArray(qsizetype(bytes), char(0x80)));
            return b;
        };
        QTemporaryDir dir;
        const QString path = QDir(dir.path()).filePath(QStringLiteral("hit.wav"));
        const auto write = [&path](const QByteArray &bytes) {
            QFile f(path);
            return f.open(QIODevice::WriteOnly) && f.write(bytes) == bytes.size();
        };
        QVERIFY(write(wav(500)));
        QCOMPARE(mixer.lengthOf(QUrl::fromLocalFile(path)), 0.5);
        QVERIFY(write(wav(2000)));
        QCOMPARE(mixer.lengthOf(QUrl::fromLocalFile(path)), 2.0);

        // A browser's voice is panned from all left to all right, and no further.
        qq::WebVoice voice;
        voice.setPan(3);
        QCOMPARE(voice.pan(), 1.0);
        voice.setPan(-0.25);
        QCOMPARE(voice.pan(), -0.25);
        voice.setVolume(-1);
        QCOMPARE(voice.volume(), 0.0);
    }
};

QTEST_MAIN(TstScene)
#include "tst_scene.moc"
