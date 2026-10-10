// The QQ Player: plays one QQ scene as a game (DIRECTION P3.2), natively and, built with Qt for WebAssembly, in a browser.
//
//   qq-player [scene.qml] [--capture <frame.png>] [--measure <result.json>]
//
// With no scene it plays the playground bundled inside it. It reports how long the game took to show its first frame
// proper (the scene playing and every model loaded), counted from when the process was created (natively) or the page
// began to load (in a browser), and the frame rate over the next three seconds. The report is printed as one line,
// "QQ-MEASURE {json}"; with --measure it is also written to a file, and the player then exits. --capture saves the
// first frame proper as an image and exits: exit code 0 done, 1 not saved, 2 timed out, 3 the scene could not play.
//
// Every game plays locked down (confinement.h, ADR-087): it loads only its own package and allowed modules, makes no
// network request, writes no file and opens nothing outside. A stranger's game plays only in the browser.

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
    parser.addOption(capture);
    parser.addOption(measureTo);
    parser.process(app);

    const QStringList positional = parser.positionalArguments();
    const QUrl scene = positional.isEmpty() ? QUrl(QStringLiteral("qrc:/playground/scenes/playground.qml"))
                                            : QUrl::fromLocalFile(QDir().absoluteFilePath(positional.first()));

    QQmlApplicationEngine engine;
    // Whatever the game, its logic reaches only its own package (ADR-087 decision 2): before anything is loaded.
    qq::Confinement::apply(&engine, qq::Confinement::packageOf(scene));
    engine.setInitialProperties({{QStringLiteral("scene"), scene}});
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
