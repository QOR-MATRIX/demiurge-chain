// The QOR Launcher's Qt shell (ADR-104): on Android, the launcher; on the desktop, the shell ADR-103's stage 2 grows
// into. It shows what the Rust core answers, and holds no key itself.

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QtQml/qqmlextensionplugin.h>

// The QOR module is built into this executable (a static library and its plugin).
Q_IMPORT_QML_PLUGIN(QORPlugin)

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("QOR Launcher"));
    QGuiApplication::setOrganizationName(QStringLiteral("Demiurge"));
    QGuiApplication::setApplicationVersion(QStringLiteral("0.1.0"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));

    QQmlApplicationEngine engine;
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); },
                     Qt::QueuedConnection);
    engine.loadFromModule("QOR", "Main");
    return QGuiApplication::exec();
}
