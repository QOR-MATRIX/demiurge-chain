// QQ Studio: the native editor for QQ, the QOR Engine (ADR-083).
//
//   qq-studio [model.gltf]
//   qq-studio [model.gltf] --capture frame.png
//
// Opens a world with the given glTF 2.0 model, or with the bundled sample. With --capture, the Studio renders until
// the model has loaded and the frame has settled, saves the whole window as an image and exits: how a person, a check
// or the agent (P3.3) sees what the Studio shows without sitting in front of it. Exit code 0 saved, 2 timed out.
//
// The Studio is its own process, started by the QOR Launcher; it never holds a key, and anything to be signed is
// asked of the launcher's host (ADR-083 decision 5).

#include <QCommandLineParser>
#include <QGuiApplication>
#include <QImage>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QTimer>
#include <QUrl>
#include <QtQml/qqmlextensionplugin.h>

Q_IMPORT_QML_PLUGIN(QQPlugin)

namespace {

/// Save the window once its prop has loaded (or failed) and a few more frames are drawn; then quit.
void captureWhenSettled(QQuickWindow *window, const QString &file)
{
    auto *frames = new int(0);
    QObject::connect(window, &QQuickWindow::frameSwapped, window, [frames] { ++*frames; });
    auto *poll = new QTimer(window);
    auto *waited = new int(0);
    QObject::connect(poll, &QTimer::timeout, window, [=] {
        ++*waited;
        const QObject *prop = window->findChild<QObject *>(QStringLiteral("prop"));
        const bool settled = prop && prop->property("status").toInt() != 0;
        if (settled && *frames >= 12) {
            poll->stop();
            const bool saved = window->grabWindow().save(file);
            QCoreApplication::exit(saved ? 0 : 1);
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
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("QQ Studio"));
    QGuiApplication::setOrganizationName(QStringLiteral("Demiurge"));
    QQuickStyle::setStyle(QStringLiteral("Material"));

    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral("QQ Studio, the QOR Engine's editor"));
    parser.addHelpOption();
    parser.addPositionalArgument(QStringLiteral("model"), QStringLiteral("A glTF 2.0 model to open (.gltf or .glb)."));
    const QCommandLineOption capture(QStringLiteral("capture"),
                                     QStringLiteral("Render, save the window to <file> and exit."),
                                     QStringLiteral("file"));
    parser.addOption(capture);
    parser.process(app);

    const QStringList positional = parser.positionalArguments();
    const QUrl model = positional.isEmpty() ? QUrl(QStringLiteral("qrc:/samples/orb.gltf"))
                                            : QUrl::fromLocalFile(positional.first());

    QQmlApplicationEngine engine;
    engine.setInitialProperties({{QStringLiteral("model"), model}});
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); },
        Qt::QueuedConnection);
    engine.loadFromModule("QQ.Studio", "Main");

    if (parser.isSet(capture) && !engine.rootObjects().isEmpty()) {
        if (auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first()))
            captureWhenSettled(window, parser.value(capture));
    }
    return QGuiApplication::exec();
}
