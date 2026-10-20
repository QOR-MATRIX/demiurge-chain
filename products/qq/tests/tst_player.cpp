// Does the QQ Player play a game on its own, and say how quickly it started? (DIRECTION P3.2) And does it keep a run and
// play it again with no window to the same outcome? (DIRECTION P3.11)
//
// The real executable, started as a person or the launcher starts it: it must play its bundled playground and a scene
// file from disk, report its first frame and frame rate, and show a frame that is the game (sky, ground, lit shapes and
// the player), not an empty window.

#include <QDir>
#include <QFile>
#include <QImage>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QProcess>
#include <QSet>
#include <QTemporaryDir>
#include <QTest>

namespace {

QString fixture(const QString &name)
{
    return QDir(QStringLiteral(QQ_FIXTURES)).filePath(name);
}

/// Run the player with `arguments`; its exit code, or -1 if it did not finish.
int runPlayer(const QStringList &arguments)
{
    QProcess player;
    player.setProgram(QStringLiteral(QQ_PLAYER));
    player.setArguments(arguments);
    player.start();
    if (!player.waitForFinished(60000)) {
        player.kill();
        return -1;
    }
    return player.exitStatus() == QProcess::NormalExit ? player.exitCode() : -1;
}

QJsonObject readJson(const QString &path)
{
    QFile f(path);
    return f.open(QIODevice::ReadOnly) ? QJsonDocument::fromJson(f.readAll()).object() : QJsonObject();
}

}  // namespace

class TstPlayer : public QObject
{
    Q_OBJECT

private slots:
    void thePlaygroundPlaysAndItsStartIsMeasured()
    {
        QTemporaryDir out;
        const QString result = QDir(out.path()).filePath(QStringLiteral("measure.json"));
        QCOMPARE(runPlayer({QStringLiteral("--measure"), result}), 0);
        const QJsonObject m = readJson(result);
        qInfo("%s", QJsonDocument(m).toJson(QJsonDocument::Compact).constData());
        QCOMPARE(m.value(QStringLiteral("problem")).toString(), QString());
        // Every model drawn: a model that failed would still let the game settle, and sooner.
        QCOMPARE(m.value(QStringLiteral("unloaded")).toArray(), QJsonArray());
        QVERIFY(m.value(QStringLiteral("scene")).toString().endsWith(QStringLiteral("playground.qml")));
        // Counted from the process's creation: more than nothing, and well inside the time a person would wait.
        const double first = m.value(QStringLiteral("firstFrameMs")).toDouble();
        QVERIFY2(first > m.value(QStringLiteral("mainMs")).toDouble() && first < 10000, qPrintable(QString::number(first)));
        QVERIFY2(m.value(QStringLiteral("fps")).toDouble() >= 20, qPrintable(QString::number(m.value(QStringLiteral("fps")).toDouble())));
    }

    void itsFirstFrameIsTheGame()
    {
        QTemporaryDir out;
        const QString frame = QDir(out.path()).filePath(QStringLiteral("first.png"));
        QCOMPARE(runPlayer({QStringLiteral("--capture"), frame}), 0);
        const QImage image(frame);
        QVERIFY(!image.isNull());
        image.save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("player-first-frame.png")));
        // A frame with a world in it has many colours; an empty window has one.
        QSet<QRgb> colours;
        for (int y = 0; y < image.height(); y += 4) {
            for (int x = 0; x < image.width(); x += 4)
                colours.insert(image.pixel(x, y) & 0xF0F0F0);
        }
        QVERIFY2(colours.size() > 200, qPrintable(QString::number(colours.size())));
        // The player's orange is in it: the camera found the player.
        int orange = 0;
        for (int y = 0; y < image.height(); y += 2) {
            for (int x = 0; x < image.width(); x += 2) {
                const QColor c = image.pixelColor(x, y);
                orange += c.red() > 170 && c.green() > 50 && c.green() < 150 && c.blue() < 60 && y > image.height() / 3;
            }
        }
        QVERIFY2(orange > 200, qPrintable(QString::number(orange)));
    }

    void aSceneFileFromDiskPlaysToo()
    {
        QTemporaryDir out;
        const QString result = QDir(out.path()).filePath(QStringLiteral("measure.json"));
        QCOMPARE(runPlayer({fixture(QStringLiteral("scenes/sample.qml")), QStringLiteral("--measure"), result}), 0);
        const QJsonObject m = readJson(result);
        QCOMPARE(m.value(QStringLiteral("problem")).toString(), QString());
        QCOMPARE(m.value(QStringLiteral("unloaded")).toArray(), QJsonArray());
        QVERIFY(m.value(QStringLiteral("firstFrameMs")).toDouble() > 0);
    }

    void aTwoDimensionalGamePlaysToo()
    {
        // Ember Climb (P3.10): a level from Tiled, sprites and a 2D player, flat and pixel for pixel.
        const QString sample = QDir(QStringLiteral(QQ_SAMPLES)).filePath(QStringLiteral("ember-climb/scenes/ember-climb.qml"));
        QTemporaryDir out;
        const QString result = QDir(out.path()).filePath(QStringLiteral("measure.json"));
        QCOMPARE(runPlayer({sample, QStringLiteral("--measure"), result}), 0);
        const QJsonObject m = readJson(result);
        QCOMPARE(m.value(QStringLiteral("problem")).toString(), QString());
        QVERIFY(m.value(QStringLiteral("firstFrameMs")).toDouble() > 0);
        QVERIFY2(m.value(QStringLiteral("fps")).toDouble() >= 20, qPrintable(QString::number(m.value(QStringLiteral("fps")).toDouble())));

        const QString frame = QDir(out.path()).filePath(QStringLiteral("first.png"));
        QCOMPARE(runPlayer({sample, QStringLiteral("--capture"), frame}), 0);
        const QImage image(frame);
        QVERIFY(!image.isNull());
        image.save(QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral("player-ember-climb.png")));
        QSet<QRgb> colours;
        for (int y = 0; y < image.height(); y += 2) {
            for (int x = 0; x < image.width(); x += 2)
                colours.insert(image.pixel(x, y));
        }
        QVERIFY2(colours.size() > 40, qPrintable(QString::number(colours.size())));
    }

    void aRunIsRecordedAndReplayedByThePlayer()
    {
        // P3.11, the real executable: a run kept with --record, its seed given with --seed and its length with --ticks,
        // then played again with --replay and no window. The game: a sum of dice thrown every half second.
        QTemporaryDir project;
        const QDir root(project.path());
        root.mkpath(QStringLiteral("scenes"));
        root.mkpath(QStringLiteral("logic"));
        QFile logic(root.filePath(QStringLiteral("logic/dice.qml")));
        QVERIFY(logic.open(QIODevice::WriteOnly));
        logic.write("import QQ\nLogic {\n    property int sum: 0\n    property int rolls: 0\n"
                    "    onStarted: every(0.5, () => { sum += randomInt(1, 6); rolls += 1 })\n"
                    "    outcome: ({ sum: sum, rolls: rolls })\n}\n");
        logic.close();
        QFile scene(root.filePath(QStringLiteral("scenes/dice.qml")));
        QVERIFY(scene.open(QIODevice::WriteOnly));
        scene.write("import QQ\nScene {\n    name: \"dice\"\n    Sun {}\n    Ground {}\n"
                    "    Behaviour { name: \"Dice\"; source: \"../logic/dice.qml\" }\n}\n");
        scene.close();

        const QString kept = root.filePath(QStringLiteral("run.json"));
        QCOMPARE(runPlayer({scene.fileName(), QStringLiteral("--seed"), QStringLiteral("7"), QStringLiteral("--record"), kept,
                            QStringLiteral("--ticks"), QStringLiteral("240")}),
                 0);
        const QJsonObject run = readJson(kept);
        QCOMPARE(run.value(QStringLiteral("seed")).toInt(), 7);
        QCOMPARE(run.value(QStringLiteral("ticks")).toInt(), 240);
        const QJsonObject dice = run.value(QStringLiteral("outcome")).toObject().value(QStringLiteral("Dice")).toObject();
        QCOMPARE(dice.value(QStringLiteral("rolls")).toInt(), 8);  // four seconds, a throw each half second
        QVERIFY(dice.value(QStringLiteral("sum")).toInt() >= 8);

        // Played again, with no window: the same outcome (exit 0). Told another seed, it does not reach it (exit 4).
        QCOMPARE(runPlayer({scene.fileName(), QStringLiteral("--replay"), kept}), 0);
        QJsonObject other = run;
        other.insert(QStringLiteral("seed"), 8);
        const QString changed = root.filePath(QStringLiteral("other.json"));
        QFile out(changed);
        QVERIFY(out.open(QIODevice::WriteOnly));
        out.write(QJsonDocument(other).toJson());
        out.close();
        QCOMPARE(runPlayer({scene.fileName(), QStringLiteral("--replay"), changed}), 4);
        // And a file that is not a run is said (exit 3).
        QCOMPARE(runPlayer({scene.fileName(), QStringLiteral("--replay"), scene.fileName()}), 3);
        // So is a game whose logic no longer builds ("throws" is a word QML keeps for itself): it would replay to an
        // empty outcome, which is no match for anything.
        QVERIFY(logic.open(QIODevice::WriteOnly | QIODevice::Truncate));
        logic.write("import QQ\nLogic {\n    property int throws: 0\n}\n");
        logic.close();
        QCOMPARE(runPlayer({scene.fileName(), QStringLiteral("--replay"), kept}), 3);
    }

    void aSceneThatIsNotThereIsSaidNotHidden()
    {
        QTemporaryDir out;
        const QString result = QDir(out.path()).filePath(QStringLiteral("measure.json"));
        // At once, with the reason: exit code 3, and the problem in the report.
        QCOMPARE(runPlayer({fixture(QStringLiteral("scenes/no-such-scene.qml")), QStringLiteral("--measure"), result}), 3);
        const QString problem = readJson(result).value(QStringLiteral("problem")).toString();
        QVERIFY2(problem.contains(QStringLiteral("no-such-scene")), qPrintable(problem));
    }
};

QTEST_MAIN(TstPlayer)
#include "tst_player.moc"
