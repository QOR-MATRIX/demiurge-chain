// The QQ Player: plays one QQ scene as a game (DIRECTION P3.2), natively and, built with Qt for WebAssembly, in a browser.
//
//   qq-player [scene.qml] [--capture <frame.png>] [--measure <result.json>]
//             [--seed <n>] [--record <run.json> [--ticks <n>]] [--replay <run.json>]
//
// With no scene it plays the playground bundled inside it. It reports how long the game took to show its first frame
// proper (the scene playing and every model loaded), counted from when the process was created (natively) or the page
// began to load (in a browser), and the frame rate over the next three seconds. The report is printed as one line,
// "QQ-MEASURE {json}"; with --measure it is also written to a file, and the player then exits. --capture saves the
// first frame proper as an image and exits: exit code 0 done, 1 not saved, 2 timed out, 3 the scene could not play.
//
// Runs (P3.11, run.h): --seed plays with that seed in place of the scene's. --record keeps the run, its seed and what
// each tick took, and writes it when the player closes, or after --ticks ticks, printing "QQ-RUN {json}" (the scene,
// the seed, the ticks and the outcome). --replay plays a kept run again with no window and no physics world, as fast
// as it goes, and prints "QQ-REPLAY {json}": exit code 0 when it reaches the outcome recorded, 4 when it does not, 3 if
// it could not play.
//
// Every game plays locked down (confinement.h, ADR-087): it loads only its own package and allowed modules, makes no
// network request, writes no file and opens nothing outside. A stranger's game plays only in the browser. A run is
// written by the Player itself, never by the game.

#include <QCommandLineParser>
#include <QDateTime>
#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QTimer>
#include <QUrl>
#include <QtQml/qqmlextensionplugin.h>

#include "confinement.h"
#include "gpu.h"
#include "run.h"

#ifdef Q_OS_WIN
#define NOMINMAX
#include <windows.h>
#endif
#ifdef Q_OS_WASM
#include <emscripten.h>
#endif

Q_IMPORT_QML_PLUGIN(QQPlugin)

namespace {

/// Milliseconds since the process was created (natively) or since the page began to load (in a browser): what a person
/// waits through, not just what main() sees.
double sinceStart()
{
#if defined(Q_OS_WASM)
    // The page's clock (main() runs on the page's thread). Not emscripten_get_now(), which in a threaded build counts
    // from 1970 so that every thread shares one clock.
    return EM_ASM_DOUBLE({ return performance.now(); });
#elif defined(Q_OS_WIN)
    FILETIME created, exited, kernel, user;
    if (GetProcessTimes(GetCurrentProcess(), &created, &exited, &kernel, &user)) {
        ULARGE_INTEGER t;
        t.LowPart = created.dwLowDateTime;
        t.HighPart = created.dwHighDateTime;
        // FILETIME counts 100 ns from 1601; the Unix epoch is 11644473600 s later.
        const double createdMs = double(t.QuadPart) / 10000.0 - 11644473600000.0;
        return double(QDateTime::currentMSecsSinceEpoch()) - createdMs;
    }
    return -1;
#else
    return -1;
#endif
}

struct Measure {
    double mainMs = 0;        // process start to main()
    double firstFrameMs = -1;  // process start to the first frame proper
    int frames = 0;
    QElapsedTimer sustained;
};

}  // namespace

int main(int argc, char *argv[])
{
    Measure measure;
    measure.mainMs = sinceStart();

    // Before the application exists: the GPU is chosen when the first window is shown.
    const QString gpu = qq::preferHighPerformanceGpu();
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("QQ Player"));
    QGuiApplication::setOrganizationName(QStringLiteral("Demiurge"));

    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral("QQ Player: plays a QQ scene as a game"));
    parser.addHelpOption();
    parser.addPositionalArgument(QStringLiteral("scene"), QStringLiteral("The scene file to play (scenes/<name>.qml)."));
    const QCommandLineOption capture(QStringLiteral("capture"), QStringLiteral("Save the first frame to <file> and exit."),
                                     QStringLiteral("file"));
    const QCommandLineOption measureTo(QStringLiteral("measure"),
                                       QStringLiteral("Write the start-up measurement to <file> and exit."),
                                       QStringLiteral("file"));
    const QCommandLineOption seedOption(QStringLiteral("seed"), QStringLiteral("Play with seed <n> in place of the scene's."),
                                        QStringLiteral("n"));
    const QCommandLineOption recordTo(QStringLiteral("record"), QStringLiteral("Keep the run and write it to <file>."),
                                      QStringLiteral("file"));
    const QCommandLineOption ticksOption(QStringLiteral("ticks"),
                                         QStringLiteral("With --record, stop after <n> ticks (1/60 s each)."),
                                         QStringLiteral("n"));
    const QCommandLineOption replayFrom(QStringLiteral("replay"),
                                        QStringLiteral("Play the run in <file> again, with no window, and say whether it "
                                                       "reaches the same outcome."),
                                        QStringLiteral("file"));
    parser.addOption(capture);
    parser.addOption(measureTo);
    parser.addOption(seedOption);
    parser.addOption(recordTo);
    parser.addOption(ticksOption);
    parser.addOption(replayFrom);
    parser.process(app);

    const QStringList positional = parser.positionalArguments();
    const QUrl scene = positional.isEmpty() ? QUrl(QStringLiteral("qrc:/playground/scenes/playground.qml"))
                                            : QUrl::fromLocalFile(QDir().absoluteFilePath(positional.first()));

    // A replay: no window, no physics world, as fast as it goes.
    if (parser.isSet(replayFrom)) {
        qq::RunRecord record;
        QFile in(parser.value(replayFrom));
        QString why;
        if (!in.open(QIODevice::ReadOnly) || !record.fromJson(in.readAll(), &why)) {
            qInfo("QQ-REPLAY %s", QJsonDocument(QJsonObject{{QStringLiteral("problem"), why.isEmpty() ? QStringLiteral("The run could not be read.") : why}})
                                      .toJson(QJsonDocument::Compact).constData());
            return 3;
        }
        QQmlEngine headless;
        qq::Confinement::apply(&headless, qq::Confinement::packageOf(scene));
        const QVariantMap result = qq::replayRun(&headless, scene, &record);
        qInfo("QQ-REPLAY %s", QJsonDocument(QJsonObject::fromVariantMap(result)).toJson(QJsonDocument::Compact).constData());
        if (!result.value(QStringLiteral("problem")).toString().isEmpty())
            return 3;
        return result.value(QStringLiteral("matches")).toBool() ? 0 : 4;
    }

    QQmlApplicationEngine engine;
    // Whatever the game, its logic reaches only its own package (ADR-087 decision 2): before anything is loaded.
    qq::Confinement::apply(&engine, qq::Confinement::packageOf(scene));
    qq::RunRecord *record = parser.isSet(recordTo) ? new qq::RunRecord(&app) : nullptr;
    engine.setInitialProperties({{QStringLiteral("scene"), scene},
                                 {QStringLiteral("seed"), parser.value(seedOption).toInt()},
                                 {QStringLiteral("recorder"), QVariant::fromValue(static_cast<QObject *>(record))}});
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); },
        Qt::QueuedConnection);
    engine.loadFromModule("QQ.Player", "Main");
    if (engine.rootObjects().isEmpty())
        return 1;
    auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    if (!window)
        return 1;

    // The game's physics bodies go before their world, not in whatever order the window's teardown would take.
    QObject::connect(&app, &QCoreApplication::aboutToQuit, window, [window] {
        QMetaObject::invokeMethod(window, "release");
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    });

    const QString captureFile = parser.value(capture);
    const QString measureFile = parser.value(measureTo);

    // A record is closed on the run so far and written by the Player: after --ticks ticks, or as the player closes.
    bool written = false;
    auto writeRecord = [&] {
        auto *playing = window->property("playing").value<QObject *>();
        if (!record || written || !playing)
            return;
        written = true;
        QMetaObject::invokeMethod(playing, "finishRecord");
        QFile out(parser.value(recordTo));
        if (out.open(QIODevice::WriteOnly))
            out.write(record->toJson());
        const QJsonObject summary{{QStringLiteral("scene"), record->scene()},
                                  {QStringLiteral("seed"), record->seed()},
                                  {QStringLiteral("ticks"), record->ticks()},
                                  {QStringLiteral("outcome"), QJsonObject::fromVariantMap(record->outcome())}};
        qInfo("QQ-RUN %s", QJsonDocument(summary).toJson(QJsonDocument::Compact).constData());
    };
    if (record) {
        QObject::connect(&app, &QCoreApplication::aboutToQuit, window, writeRecord);
        const int stopAt = parser.value(ticksOption).toInt();
        if (stopAt > 0) {
            auto *watch = new QTimer(window);
            QObject::connect(watch, &QTimer::timeout, window, [window, stopAt, writeRecord] {
                auto *playing = window->property("playing").value<QObject *>();
                if (!playing)
                    return;
                // Not one tick more than asked: the scene's clock stops there.
                playing->setProperty("tickLimit", stopAt);
                if (playing->property("ticks").toInt() >= stopAt) {
                    writeRecord();
                    QCoreApplication::exit(0);
                }
            });
            watch->start(1);
        }
    }

    auto report = [&] {
        const double seconds = measure.sustained.elapsed() / 1000.0;
        const QJsonObject result{{QStringLiteral("scene"), scene.toString()},
                                 {QStringLiteral("mainMs"), qRound(measure.mainMs)},
                                 {QStringLiteral("firstFrameMs"), qRound(measure.firstFrameMs)},
                                 {QStringLiteral("fps"), seconds > 0 ? qRound(measure.frames / seconds * 10) / 10.0 : 0},
                                 {QStringLiteral("problem"), window->property("problem").toString()},
                                 {QStringLiteral("unloaded"),
                                  QJsonArray::fromStringList(window->property("unloaded").toStringList())}};
        const QByteArray line = QJsonDocument(result).toJson(QJsonDocument::Compact);
        qInfo("QQ-MEASURE %s", line.constData());
        if (!measureFile.isEmpty()) {
            QFile out(measureFile);
            if (out.open(QIODevice::WriteOnly))
                out.write(line + '\n');
            QCoreApplication::exit(0);
        }
    };

    const bool once = !captureFile.isEmpty() || !measureFile.isEmpty();
    QObject::connect(window, &QQuickWindow::frameSwapped, window, [&] {
        if (measure.firstFrameMs < 0) {
            // A scene that cannot play is said on the window; asked for one result, the player says so and stops.
            if (once && !window->property("problem").toString().isEmpty()) {
                report();
                QCoreApplication::exit(3);
                return;
            }
            if (!window->property("settled").toBool())
                return;
            measure.firstFrameMs = sinceStart();
            measure.sustained.start();
            if (!captureFile.isEmpty()) {
                // The first frame proper, then a few more so what is saved is what a player sees.
                QTimer::singleShot(250, window, [window, captureFile] {
                    QCoreApplication::exit(window->grabWindow().save(captureFile) ? 0 : 1);
                });
            }
            QTimer::singleShot(3000, window, report);
            return;
        }
        ++measure.frames;
    });

    // Measuring, frames are asked for continuously, so what is counted is what the machine can draw, not only what
    // changed. (In play the game's own animation keeps them coming.)
    if (!captureFile.isEmpty() || !measureFile.isEmpty()) {
        auto *pump = new QTimer(window);
        QObject::connect(pump, &QTimer::timeout, window, [window] { window->update(); });
        pump->start(4);
    }
    QTimer::singleShot(30000, window, [&] {
        if (measure.firstFrameMs < 0 && (!captureFile.isEmpty() || !measureFile.isEmpty()))
            QCoreApplication::exit(2);
    });

    return QGuiApplication::exec();
}
