// The QOR Launcher's Qt shell (ADR-104): on Android, the launcher; on the desktop, the shell ADR-103's stage 2 grows
// into. It shows what the Rust core answers, and holds no key itself.

#include <QFont>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QStyleHints>
#include <QtQml/qqmlextensionplugin.h>

#include "fonts.h"

// The QOR module is built into this executable (a static library and its plugin).
Q_IMPORT_QML_PLUGIN(QORPlugin)

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("QOR Launcher"));
    QGuiApplication::setOrganizationName(QStringLiteral("Demiurge"));
    QGuiApplication::setApplicationVersion(QStringLiteral("0.1.0"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));

    // Demiurge's faces (ADR-108), and reading text in Barlow wherever a control does not say otherwise.
    qor::loadFonts();
    QGuiApplication::setFont(QFont(QStringLiteral("Barlow")));
    // The look is dark whatever the phone's own setting, so the status bar's clock and icons are drawn light over it.
    QGuiApplication::styleHints()->setColorScheme(Qt::ColorScheme::Dark);

    QQmlApplicationEngine engine;
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); },
                     Qt::QueuedConnection);
    engine.loadFromModule("QOR", "Main");
    return QGuiApplication::exec();
}
