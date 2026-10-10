// McpServer: QQ Studio's tools, offered to language models through the Model Context Protocol (DIRECTION P3.3,
// ADR-083 decision 4).
//
// MCP is JSON-RPC 2.0, one message to a line. This class speaks it and knows no transport: `handle()` takes one line and
// answers through a callback, so a test can drive it directly. `listen()` serves it on a local pipe (a Windows named
// pipe, a Unix socket elsewhere) that only the user running the Studio can open; `qq-mcp`, the program an MCP client
// starts, relays its standard input and output to that pipe. No network port is opened.
//
// Requests are answered in order of completion, each with its own id; a tool may take time (`wait` lets frames pass)
// and answers when it is done, without holding up the event loop.

#pragma once

#include <QHash>
#include <QJsonArray>
#include <QJsonObject>
#include <QObject>
#include <QString>

#include <functional>

class QLocalServer;

namespace qq {

class McpServer : public QObject
{
    Q_OBJECT

public:
    /// What a tool hands back: MCP content (text, images) and whether it failed. A failed tool is a result the model
    /// reads and can act on, not a protocol error.
    struct Result {
        QJsonArray content;
        bool isError = false;
        static Result text(const QString &text, bool isError = false);
    };
    using Done = std::function<void(const Result &)>;
    using Handler = std::function<void(const QJsonObject &arguments, const Done &done)>;

    explicit McpServer(QObject *parent = nullptr);

    /// Name, version and what a model should know before it calls anything (sent with `initialize`).
    void setServerInfo(const QString &name, const QString &version, const QString &instructions);

    /// Offer a tool. `name` matches ^[a-zA-Z0-9_-]{1,64}$, as clients require; `inputSchema` is a JSON Schema object.
    void addTool(const QString &name, const QString &description, const QJsonObject &inputSchema, Handler handler);

    /// Every tool offered, in the order added: { name, description, inputSchema }.
    QJsonArray tools() const { return toolList(); }

    /// Call a tool directly, as the Studio's own design loop does: the same tools, without the protocol.
    void call(const QString &name, const QJsonObject &arguments, const Done &done);

    /// One line of JSON-RPC in; `reply` is called with each line to send back (none for a notification).
    void handle(const QByteArray &line, const std::function<void(const QByteArray &)> &reply);

    /// Serve on the local pipe `name`, for this user only. False, with `error()` saying why, if the name is taken (by
    /// another Studio) or cannot be made.
    bool listen(const QString &name);
    QString error() const { return m_error; }

    /// The pipe's name: QQ_MCP_PIPE if set, otherwise one per user.
    static QString defaultPipeName();

    /// The protocol versions this server speaks, newest first.
    static QStringList protocolVersions();

signals:
    /// A client said `initialize`: its name, as it gave it.
    void clientConnected(const QString &client);
    /// A tool was called, from a client or from the Studio itself: its name, and whether it failed.
    void toolCalled(const QString &tool, bool failed);

private:
    struct Tool {
        QString description;
        QJsonObject inputSchema;
        Handler handler;
    };

    QJsonObject initializeResult(const QJsonObject &params);
    QJsonArray toolList() const;

    QString m_name = QStringLiteral("qq");
    QString m_version = QStringLiteral("0");
    QString m_instructions;
    QStringList m_order;
    QHash<QString, Tool> m_tools;
    QLocalServer *m_local = nullptr;
    QString m_error;
};

}  // namespace qq
