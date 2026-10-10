// QQ Studio: the native editor for QQ, the QOR Engine (ADR-083).
//
//   qq-studio [model.gltf] [--project <folder>] [--capture <frame.png>]
//
// Opens a new scene, with the given glTF 2.0 model or the bundled sample, in the given project folder (the QOR Launcher
// passes the project open in Projects). With --capture, the Studio renders until its scene has settled, saves the whole
// window as an image and exits: how a person, a check or the agent (P3.3) sees what the Studio shows. Exit code 0 saved,
// 1 not saved, 2 timed out.
//
// The Studio is its own process, started by the QOR Launcher; it never holds a key, and anything to be signed is
// asked of the launcher's host (ADR-083 decision 5).
//
// It offers its tools to language models (DIRECTION P3.3) on a local pipe only this user can open: an MCP client starts
// qq-mcp, which relays to it (agent/). One Studio at a time serves the pipe; QQ_MCP_PIPE names another.

#include <QCommandLineParser>
#include <QDir>
#include <QGuiApplication>
#include <QImage>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QTimer>
#include <QUrl>
#include <QtQml/qqmlextensionplugin.h>

#include <memory>

#include "gpu.h"
#include "mcpserver.h"
#include "studiotools.h"

Q_IMPORT_QML_PLUGIN(QQPlugin)
Q_IMPORT_QML_PLUGIN(QQ_StudioPlugin)

namespace {

/// Save the window once its scene has settled and a few more frames are drawn; then quit.
void captureWhenSettled(QQuickWindow *window, const QString &file)
{
    auto *frames = new int(0);
    QObject::connect(window, &QQuickWindow::frameSwapped, window, [frames] { ++*frames; });
    auto *poll = new QTimer(window);
    auto *waited = new int(0);
    QObject::connect(poll, &QTimer::timeout, window, [=] {
        ++*waited;
        if (window->property("settled").toBool() && *frames >= 12) {
            poll->stop();
            QCoreApplication::exit(window->grabWindow().save(file) ? 0 : 1);
            return;
        }
        if (*waited > 400) {  // 20 seconds
            poll->stop();
            QCoreApplication::exit(2);
            return;
        }
        window->update();  // a still scene draws once and waits; ask for frames
    });
    poll->start(50);
}

}  // namespace

int main(int argc, char *argv[])
{
    // Before the application exists: the GPU is chosen when the first window is shown.
    const QString gpu = qq::preferHighPerformanceGpu();
    QGuiApplication app(argc, argv);
    // From the start, so the agent can read every QML warning the Studio meets.
    qq::StudioLog::install();
    QGuiApplication::setApplicationName(QStringLiteral("QQ Studio"));
    QGuiApplication::setOrganizationName(QStringLiteral("Demiurge"));
    QQuickStyle::setStyle(QStringLiteral("Material"));

    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral("QQ Studio, the QOR Engine's editor"));
    parser.addHelpOption();
    parser.addPositionalArgument(QStringLiteral("model"), QStringLiteral("A glTF 2.0 model for the new scene."));
    const QCommandLineOption project(QStringLiteral("project"), QStringLiteral("The project folder to work in."),
                                     QStringLiteral("folder"));
    const QCommandLineOption capture(QStringLiteral("capture"),
                                     QStringLiteral("Render, save the window to <file> and exit."),
                                     QStringLiteral("file"));
    parser.addOption(project);
    parser.addOption(capture);
    parser.process(app);

    const QStringList positional = parser.positionalArguments();
    const QUrl model = positional.isEmpty() ? QUrl(QStringLiteral("qrc:/samples/orb.gltf"))
                                            : QUrl::fromLocalFile(QDir().absoluteFilePath(positional.first()));
    const QUrl folder = parser.isSet(project) ? QUrl::fromLocalFile(QDir().absoluteFilePath(parser.value(project)))
                                              : QUrl();

    QQmlApplicationEngine engine;
    engine.setInitialProperties({{QStringLiteral("model"), model}, {QStringLiteral("project"), folder}});
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); },
        Qt::QueuedConnection);
    engine.loadFromModule("QQ.Studio", "Main");

    // Play's physics bodies go before their world, not in whatever order the window's teardown would take.
    QObject::connect(&app, &QCoreApplication::aboutToQuit, &engine, [&engine] {
        for (QObject *root : engine.rootObjects())
            QMetaObject::invokeMethod(root, "release");
        QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    });

    // The agent's tools, on the Studio's window.
    qq::McpServer mcp;
    std::unique_ptr<qq::StudioTools> tools;
    if (!engine.rootObjects().isEmpty()) {
        if (auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
            tools = std::make_unique<qq::StudioTools>(window, &engine, &mcp);
            if (!parser.isSet(capture) && !mcp.listen(qq::McpServer::defaultPipeName()))
                qWarning("Agents cannot connect to this Studio: %s", qPrintable(mcp.error()));
        }
    }

    if (parser.isSet(capture) && !engine.rootObjects().isEmpty()) {
        if (auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first()))
            captureWhenSettled(window, parser.value(capture));
    }
    return QGuiApplication::exec();
}
