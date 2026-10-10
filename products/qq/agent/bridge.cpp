// qq-mcp: QQ Studio's tools for an MCP client (DIRECTION P3.3).
//
// An MCP client (Claude Code, Claude Desktop, any other) starts this program and speaks to it on its standard input
// and output, one JSON-RPC message to a line. It relays every line to the QQ Studio running for this user, over the
// local pipe the Studio serves (McpServer), and every answer back. It opens no network port and holds no key.
//
//   claude mcp add qq -- "%LOCALAPPDATA%\qq-studio\qq-mcp.exe"
//
// With no Studio running, each request is answered with an error saying to open one; the next request tries again,
// so a Studio opened later is found without restarting the client. Nothing but protocol is written to standard output.

#include <QCoreApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalSocket>

#include <cstdio>
#include <iostream>
#include <string>
#include <thread>

#ifdef Q_OS_WIN
#include <fcntl.h>
#include <io.h>
#endif

#include "mcpserver.h"

namespace {

void send(const QByteArray &line)
{
    std::fwrite(line.constData(), 1, size_t(line.size()), stdout);
    std::fputc('\n', stdout);
    std::fflush(stdout);
}

}  // namespace

int main(int argc, char *argv[])
{
#ifdef Q_OS_WIN
    // Bytes as they are: no \r added to what is written, none expected in what is read.
    _setmode(_fileno(stdin), _O_BINARY);
    _setmode(_fileno(stdout), _O_BINARY);
#endif
    QCoreApplication app(argc, argv);
    const QString pipe = qq::McpServer::defaultPipeName();

    QLocalSocket studio;
    auto reach = [&studio, &pipe] {
        if (studio.state() == QLocalSocket::ConnectedState)
            return true;
        studio.abort();
        studio.connectToServer(pipe);
        return studio.waitForConnected(1500);
    };
    reach();

    QObject::connect(&studio, &QLocalSocket::readyRead, &app, [&studio] {
        while (studio.canReadLine()) {
            const QByteArray line = studio.readLine().trimmed();
            if (!line.isEmpty())
                send(line);
        }
    });

    // A line from the client: to the Studio, or, with none to reach, an answer that says so.
    auto relay = [&](const QByteArray &line) {
        if (reach()) {
            studio.write(line + '\n');
            studio.flush();
            return;
        }
        const QJsonObject message = QJsonDocument::fromJson(line).object();
        if (!message.contains(QStringLiteral("id")))
            return;  // a notification needs no answer
        const QJsonObject error{
            {QStringLiteral("code"), -32000},
            {QStringLiteral("message"),
             QStringLiteral("QQ Studio is not running for this user. Open it (the launcher's QQ screen, Open in QQ "
                            "Studio), then try again.")}};
        send(QJsonDocument(QJsonObject{{QStringLiteral("jsonrpc"), QStringLiteral("2.0")},
                                       {QStringLiteral("id"), message.value(QStringLiteral("id"))},
                                       {QStringLiteral("error"), error}})
                 .toJson(QJsonDocument::Compact));
    };

    // Standard input is read on a thread of its own (a console pipe cannot be watched by the event loop on Windows);
    // each line is handed to the event loop, and the end of input ends the program.
    std::thread reader([&app, relay] {
        std::string text;
        while (std::getline(std::cin, text)) {
            QByteArray line = QByteArray::fromStdString(text).trimmed();
            if (line.isEmpty())
                continue;
            QMetaObject::invokeMethod(&app, [relay, line] { relay(line); }, Qt::QueuedConnection);
        }
        QMetaObject::invokeMethod(&app, [] { QCoreApplication::exit(0); }, Qt::QueuedConnection);
    });
    reader.detach();

    return QCoreApplication::exec();
}
