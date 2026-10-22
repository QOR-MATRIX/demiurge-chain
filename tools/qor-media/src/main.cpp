// QOR Media: the Library's player, viewer and editor (ADR-094; docs/blueprints/library.md section 4).
//
//   qor-media [file] [--launcher-channel <name>]
//
// Started by the QOR Launcher with --launcher-channel and a one-time token on its stdin, it docks inside the launcher's
// window and opens what the launcher asks (medialink.h), and exits when the launcher closes the channel. Started by
// hand with a file, it opens that file in a window of its own.

#include <QCommandLineParser>
#include <QDir>
#include <QFont>
#include <QGuiApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QtQml/qqmlextensionplugin.h>

#include "look.h"
#include "medialink.h"
#include "tokens.h"

// The QOR.Media module is built into this executable (a static library and its plugin).
Q_IMPORT_QML_PLUGIN(QOR_MediaPlugin)

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("QOR Media"));
    QGuiApplication::setOrganizationName(QStringLiteral("Demiurge"));
    QGuiApplication::setApplicationVersion(QStringLiteral("0.1.0"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));

    // Demiurge's one look (ADR-108): the faces QOR Media carries, and reading text in Barlow wherever a control does
    // not say otherwise (a tool tip, a scroll bar's neighbour).
    qormedia::loadFonts();
    QFont reading(QString::fromLatin1(qormedia::tokens::fontBody));
    reading.setPixelSize(qormedia::tokens::sizeBody);
    QGuiApplication::setFont(reading);

    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral("QOR Media: play, view and edit what the Library holds."));
    parser.addHelpOption();
    const QCommandLineOption channel(QStringLiteral("launcher-channel"),
                                     QStringLiteral("The launcher's channel, given by the launcher that started this."),
                                     QStringLiteral("name"));
    parser.addOption(channel);
    parser.addPositionalArgument(QStringLiteral("file"), QStringLiteral("A file to open."));
    parser.process(app);

    auto *link = new qormedia::MediaLink(&app);
    const bool launched = parser.isSet(channel);

    QQmlApplicationEngine engine;
    const QStringList files = parser.positionalArguments();
    engine.setInitialProperties({
        {QStringLiteral("link"), QVariant::fromValue(link)},
        {QStringLiteral("launched"), launched},
        {QStringLiteral("startPath"), files.isEmpty() ? QString() : QDir::toNativeSeparators(QDir(files.first()).absolutePath())},
    });
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); },
                     Qt::QueuedConnection);
    // Less motion when the person asked for it: the launcher's setting, or the system's when started by hand.
    if (auto *theme = engine.singletonInstance<QObject *>("QOR.Media", "Theme"))
        theme->setProperty("reducedMotion", qormedia::prefersReducedMotion());
    engine.loadFromModule("QOR.Media", "Main");

    if (launched) {
        link->connectTo(parser.value(channel), qormedia::MediaLink::readToken());
        // A product ends with its launcher (ADR-046 decision 2).
        QObject::connect(link, &qormedia::MediaLink::lost, &app, &QCoreApplication::quit);
    }
    return QGuiApplication::exec();
}
