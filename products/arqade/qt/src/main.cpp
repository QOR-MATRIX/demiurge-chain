// ARQADE: the QOR Launcher's arcade (ADR-102; DIRECTION P7.19).
//
//   arqade [--launcher-channel <name>]
//
// Started by the QOR Launcher with --launcher-channel and a one-time token on its stdin, it docks inside the launcher's
// window (arqadelink.h), signs in to ARQADE's server with the launcher's QOR ID (arqadeserver.h), and exits when the
// launcher closes the channel or asks it to. Started by hand, it opens in a window of its own and signs in to nothing.

#include <QCommandLineParser>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QtQml/qqmlextensionplugin.h>
#include <rhi/qrhi.h>
#include <rhi/qrhi_platform.h>

#include "arqadelink.h"
#include "arqadeserver.h"
#include "fonts.h"

#ifdef Q_OS_WIN
#include <dxgi1_6.h>
#include <wrl/client.h>
#endif

// The ARQADE module is built into this executable (a static library and its plugin), and QQ's runtime, which plays the
// games rebuilt in QQ (P7.20).
Q_IMPORT_QML_PLUGIN(ARQADEPlugin)
Q_IMPORT_QML_PLUGIN(QQPlugin)

// The graphics card Windows calls high-performance, where a PC has two (a laptop's discrete card beside its integrated
// one), so ARQADE draws on the faster: Qt otherwise takes the first, usually the integrated. QT_D3D_ADAPTER_INDEX, when
// set, decides instead, and the person's own choice in Windows' graphics settings is what Windows reports here.
static void preferHighPerformanceCard()
{
#ifdef Q_OS_WIN
    if (!qEnvironmentVariableIsEmpty("QT_D3D_ADAPTER_INDEX"))
        return;
    Microsoft::WRL::ComPtr<IDXGIFactory6> factory;
    if (FAILED(CreateDXGIFactory2(0, IID_PPV_ARGS(&factory))))
        return;
    Microsoft::WRL::ComPtr<IDXGIAdapter1> fastest;
    if (FAILED(factory->EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE, IID_PPV_ARGS(&fastest))))
        return;
    DXGI_ADAPTER_DESC1 wanted{};
    if (FAILED(fastest->GetDesc1(&wanted)) || (wanted.Flags & DXGI_ADAPTER_FLAG_SOFTWARE))
        return;
    // Qt counts adapters in DXGI's own order: find the fastest's place in it.
    Microsoft::WRL::ComPtr<IDXGIAdapter1> adapter;
    for (UINT i = 0; factory->EnumAdapters1(i, &adapter) != DXGI_ERROR_NOT_FOUND; ++i) {
        DXGI_ADAPTER_DESC1 desc{};
        const bool same = SUCCEEDED(adapter->GetDesc1(&desc)) && desc.AdapterLuid.LowPart == wanted.AdapterLuid.LowPart
            && desc.AdapterLuid.HighPart == wanted.AdapterLuid.HighPart;
        adapter.Reset();
        if (same) {
            if (i != 0)
                qputenv("QT_D3D_ADAPTER_INDEX", QByteArray::number(i));
            return;
        }
    }
#endif
}

// Direct3D 12 where this PC's graphics card has it (ADR-102 decision 1), tried first so a card without it is not left
// with a black window; otherwise Qt's own choice, Direct3D 11. QSG_RHI_BACKEND, when set, decides instead.
static void chooseGraphics()
{
#ifdef Q_OS_WIN
    if (!qEnvironmentVariableIsEmpty("QSG_RHI_BACKEND"))
        return;
    QRhiD3D12InitParams params;
    if (QRhi::probe(QRhi::D3D12, &params))
        QQuickWindow::setGraphicsApi(QSGRendererInterface::Direct3D12);
#endif
}

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    preferHighPerformanceCard();
    chooseGraphics();
    QGuiApplication::setApplicationName(QStringLiteral("ARQADE"));
    QGuiApplication::setOrganizationName(QStringLiteral("Demiurge"));
    QGuiApplication::setApplicationVersion(QStringLiteral("0.1.0"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));

    arqade::loadFonts();

    QCommandLineParser parser;
    parser.setApplicationDescription(QStringLiteral("ARQADE: the QOR Launcher's arcade."));
    parser.addHelpOption();
    const QCommandLineOption channel(QStringLiteral("launcher-channel"),
                                     QStringLiteral("The launcher's channel, given by the launcher that started this."),
                                     QStringLiteral("name"));
    parser.addOption(channel);
    parser.process(app);

    auto *link = new arqade::ArqadeLink(&app);
    auto *server = new arqade::ArqadeServer(link, arqade::ArqadeServer::defaultBase(), &app);
    const bool launched = parser.isSet(channel);

    QQmlApplicationEngine engine;
    engine.setInitialProperties({
        {QStringLiteral("link"), QVariant::fromValue(link)},
        {QStringLiteral("server"), QVariant::fromValue(server)},
        {QStringLiteral("launched"), launched},
    });
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, [] { QCoreApplication::exit(1); },
                     Qt::QueuedConnection);
    engine.loadFromModule("ARQADE", "Main");

    // A game's physics bodies go before their world, not in whatever order the window's teardown would take.
    if (!engine.rootObjects().isEmpty()) {
        QObject *window = engine.rootObjects().first();
        QObject::connect(&app, &QCoreApplication::aboutToQuit, window, [window] {
            QMetaObject::invokeMethod(window, "release");
            QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
        });
    }

    if (launched) {
        link->connectTo(parser.value(channel), arqade::ArqadeLink::readToken());
        // A product ends with its launcher (ADR-046 decision 2), and when the person signs out of it.
        QObject::connect(link, &arqade::ArqadeLink::lost, &app, &QCoreApplication::quit);
        QObject::connect(link, &arqade::ArqadeLink::quitRequested, &app, &QCoreApplication::quit);
    }
    return QGuiApplication::exec();
}
