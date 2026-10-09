// Are QQ scene files what P3.1 promises? (DIRECTION P3.1, ADR-083 decision 3)
//
// A scene file is canonical QML: read and written back it is the same bytes, and one changed value is one changed line,
// so Projects shows a creator's change as what it is. These cases drive SceneIO through a real QML engine with the
// runtime module "QQ", against the fixtures: tests/fixtures/scenes/sample.qml (every entity type, written by hand in
// the canonical layout) and tests/fixtures/first-light.qq.json (the 2D preview's starter scene).

#include <QDir>
#include <QFile>
#include <QQmlEngine>
#include <QTemporaryDir>
#include <QTest>
#include <QVector3D>
#include <QtQml/qqmlextensionplugin.h>
#include <QtQuick3D/qquick3dobject.h>

#include "sceneio.h"

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

QObject *entityNamed(QObject *scene, const QString &name)
{
    for (QQuick3DObject *child : qobject_cast<QQuick3DObject *>(scene)->childItems()) {
        if (child->property("name").toString() == name)
            return child;
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
                                     QStringLiteral("Lamp"), QStringLiteral("Prop")}));
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
        QCOMPARE(result.value(QStringLiteral("imported")).toInt(), 3);
        const QStringList skipped = result.value(QStringLiteral("skipped")).toStringList();
        QCOMPARE(skipped.size(), 3);
        QVERIFY(skipped.filter(QStringLiteral("Core: follow")).size() == 1);
        QVERIFY(skipped.filter(QStringLiteral("Core: emitter")).size() == 1);
        QVERIFY(skipped.filter(QStringLiteral("Ring: motion")).size() == 1);

        const QString qml = result.value(QStringLiteral("qml")).toString();
        QObject *scene = io->loadText(qml, QUrl(), nullptr);
        QVERIFY2(scene, qPrintable(io->lastError()));
        QCOMPARE(io->write(scene), qml);  // canonical
        QObject *core = entityNamed(scene, QStringLiteral("Core"));
        QVERIFY(core);
        QCOMPARE(core->property("form").toString(), QStringLiteral("sphere"));
        QCOMPARE(core->property("position").value<QVector3D>(), QVector3D(0, 2.7f, 0));
        QCOMPARE(core->property("emissive").value<QColor>(), QColor(QStringLiteral("#ff6a00")));
        QObject *ring = entityNamed(scene, QStringLiteral("Ring"));
        QCOMPARE(ring->property("form").toString(), QStringLiteral("cube"));
        QCOMPARE(ring->property("position").value<QVector3D>(), QVector3D(-2.6f, 2.7f, 0));
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
        QCOMPARE(io->entities(scene).size(), 5);

        QObject *crystal = io->add(scene, QStringLiteral("Shape"),
                                   {{QStringLiteral("name"), QStringLiteral("Crystal")},
                                    {QStringLiteral("form"), QStringLiteral("cone")},
                                    {QStringLiteral("position"), QVector3D(2, 0.5f, -1)}});
        QVERIFY2(crystal, qPrintable(io->lastError()));
        QCOMPARE(io->entities(scene).size(), 6);
        const QString text = io->write(scene, url);
        QVERIFY2(text.endsWith(QStringLiteral("    Shape {\n        name: \"Crystal\"\n        position: Qt.vector3d(2, 0.5, -1)\n")
                               + QStringLiteral("        eulerRotation: Qt.vector3d(0, 0, 0)\n        scale: Qt.vector3d(1, 1, 1)\n")
                               + QStringLiteral("        form: \"cone\"\n        colour: \"#5ad1ff\"\n        metalness: 0\n")
                               + QStringLiteral("        roughness: 0.4\n        emissive: \"#000000\"\n        emissivePower: 0\n    }\n}\n")),
                 qPrintable(text.right(400)));

        // What is not an entity kind, or not a scene, is refused.
        QVERIFY(!io->add(scene, QStringLiteral("Window"), {}));
        QVERIFY(io->lastError().contains(QStringLiteral("no entity called")));
        QVERIFY(!io->add(crystal, QStringLiteral("Shape"), {}));

        QVERIFY(io->remove(crystal));
        QVERIFY(!io->remove(scene));
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        QCOMPARE(io->entities(scene).size(), 5);
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
};

QTEST_MAIN(TstScene)
#include "tst_scene.moc"
