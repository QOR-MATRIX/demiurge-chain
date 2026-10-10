// Does the QQ Player play a game on its own, and say how quickly it started? (DIRECTION P3.2)
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
