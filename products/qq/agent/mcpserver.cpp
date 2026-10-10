#include "mcpserver.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QLocalServer>
#include <QLocalSocket>
#include <QPointer>
#include <QRegularExpression>

namespace qq {

namespace {

// JSON-RPC 2.0's error codes.
constexpr int parseError = -32700;
constexpr int invalidRequest = -32600;
constexpr int methodNotFound = -32601;
constexpr int invalidParams = -32602;

QByteArray line(const QJsonObject &message)
{
    return QJsonDocument(message).toJson(QJsonDocument::Compact);
}

QJsonObject response(const QJsonValue &id, const QJsonObject &result)
{
    return {{QStringLiteral("jsonrpc"), QStringLiteral("2.0")}, {QStringLiteral("id"), id},
            {QStringLiteral("result"), result}};
}

QJsonObject failure(const QJsonValue &id, int code, const QString &message)
{
    return {{QStringLiteral("jsonrpc"), QStringLiteral("2.0")}, {QStringLiteral("id"), id},
            {QStringLiteral("error"), QJsonObject{{QStringLiteral("code"), code}, {QStringLiteral("message"), message}}}};
}

QJsonObject toJson(const McpServer::Result &r)
{
    return {{QStringLiteral("content"), r.content}, {QStringLiteral("isError"), r.isError}};
}

}  // namespace

McpServer::Result McpServer::Result::text(const QString &text, bool isError)
{
    Result r;
    r.content.append(QJsonObject{{QStringLiteral("type"), QStringLiteral("text")}, {QStringLiteral("text"), text}});
    r.isError = isError;
    return r;
}

McpServer::McpServer(QObject *parent) : QObject(parent) {}

void McpServer::setServerInfo(const QString &name, const QString &version, const QString &instructions)
{
    m_name = name;
    m_version = version;
    m_instructions = instructions;
}

void McpServer::addTool(const QString &name, const QString &description, const QJsonObject &inputSchema,
                        Handler handler)
{
    static const QRegularExpression valid(QStringLiteral("^[a-zA-Z0-9_-]{1,64}$"));
    Q_ASSERT_X(valid.match(name).hasMatch(), "McpServer::addTool", "a tool name clients will refuse");
    if (!m_tools.contains(name))
        m_order << name;
    m_tools.insert(name, Tool{description, inputSchema, std::move(handler)});
}

void McpServer::call(const QString &name, const QJsonObject &arguments, const Done &done)
{
    const auto tool = m_tools.constFind(name);
    if (tool == m_tools.constEnd()) {
        done(Result::text(QStringLiteral("There is no tool called %1.").arg(name), true));
        return;
    }
    QPointer<McpServer> self(this);
    tool->handler(arguments, [self, name, done](const Result &result) {
        if (self)
            emit self->toolCalled(name, result.isError);
        done(result);
    });
}

QStringList McpServer::protocolVersions()
{
    return {QStringLiteral("2025-06-18"), QStringLiteral("2025-03-26"), QStringLiteral("2024-11-05")};
}

QJsonObject McpServer::initializeResult(const QJsonObject &params)
{
    // The client's version if this server speaks it; otherwise this server's newest, and the client decides.
    const QString asked = params.value(QStringLiteral("protocolVersion")).toString();
    const QString version = protocolVersions().contains(asked) ? asked : protocolVersions().first();
    emit clientConnected(params.value(QStringLiteral("clientInfo")).toObject().value(QStringLiteral("name")).toString());
    QJsonObject result{
        {QStringLiteral("protocolVersion"), version},
        {QStringLiteral("capabilities"),
         QJsonObject{{QStringLiteral("tools"), QJsonObject{{QStringLiteral("listChanged"), false}}}}},
        {QStringLiteral("serverInfo"),
         QJsonObject{{QStringLiteral("name"), m_name}, {QStringLiteral("version"), m_version}}},
    };
    if (!m_instructions.isEmpty())
        result.insert(QStringLiteral("instructions"), m_instructions);
    return result;
}

QJsonArray McpServer::toolList() const
{
    QJsonArray tools;
    for (const QString &name : m_order) {
        const Tool &t = m_tools[name];
        tools.append(QJsonObject{{QStringLiteral("name"), name},
                                 {QStringLiteral("description"), t.description},
                                 {QStringLiteral("inputSchema"), t.inputSchema}});
    }
    return tools;
}

void McpServer::handle(const QByteArray &text, const std::function<void(const QByteArray &)> &reply)
{
    QJsonParseError parsed;
    const QJsonDocument doc = QJsonDocument::fromJson(text, &parsed);
    if (parsed.error != QJsonParseError::NoError) {
        reply(line(failure(QJsonValue::Null, parseError, QStringLiteral("Not JSON: %1").arg(parsed.errorString()))));
        return;
    }
    if (!doc.isObject()) {
        // Batches were dropped from MCP in 2025-06-18; one message to a line.
        reply(line(failure(QJsonValue::Null, invalidRequest, QStringLiteral("One JSON-RPC message to a line."))));
        return;
    }
    const QJsonObject message = doc.object();
    const QString method = message.value(QStringLiteral("method")).toString();
    const bool isRequest = message.contains(QStringLiteral("id"));
    const QJsonValue id = message.value(QStringLiteral("id"));
    const QJsonObject params = message.value(QStringLiteral("params")).toObject();

    if (!isRequest)
        return;  // a notification (initialized, cancelled): nothing is answered
    if (message.value(QStringLiteral("jsonrpc")).toString() != QStringLiteral("2.0") || method.isEmpty()) {
        reply(line(failure(id, invalidRequest, QStringLiteral("A JSON-RPC 2.0 request has a method."))));
        return;
    }

    if (method == QStringLiteral("initialize")) {
        reply(line(response(id, initializeResult(params))));
    } else if (method == QStringLiteral("ping")) {
        reply(line(response(id, {})));
    } else if (method == QStringLiteral("tools/list")) {
        reply(line(response(id, QJsonObject{{QStringLiteral("tools"), toolList()}})));
    } else if (method == QStringLiteral("tools/call")) {
        const QString name = params.value(QStringLiteral("name")).toString();
        if (!m_tools.contains(name)) {
            reply(line(failure(id, invalidParams, QStringLiteral("There is no tool called %1.").arg(name))));
            return;
        }
        call(name, params.value(QStringLiteral("arguments")).toObject(),
             [reply, id](const Result &result) { reply(line(response(id, toJson(result)))); });
    } else {
        reply(line(failure(id, methodNotFound, QStringLiteral("QQ Studio does not answer %1.").arg(method))));
    }
}

QString McpServer::defaultPipeName()
{
    const QString set = qEnvironmentVariable("QQ_MCP_PIPE");
    if (!set.isEmpty())
        return set;
    QString user = qEnvironmentVariable("USERNAME", qEnvironmentVariable("USER", QStringLiteral("user")));
    user.replace(QRegularExpression(QStringLiteral("[^A-Za-z0-9_-]")), QStringLiteral("_"));
    return QStringLiteral("qq-studio-mcp-") + user;
}

bool McpServer::listen(const QString &name)
{
    // Two servers on one Windows pipe name would split the clients between them: if one answers, this one stays out.
    {
        QLocalSocket probe;
        probe.connectToServer(name);
        if (probe.waitForConnected(200)) {
            m_error = QStringLiteral("Another QQ Studio already offers its tools on %1.").arg(name);
            return false;
        }
    }
    QLocalServer::removeServer(name);  // a stale Unix socket file; nothing on Windows
    m_local = new QLocalServer(this);
    m_local->setSocketOptions(QLocalServer::UserAccessOption);
    if (!m_local->listen(name)) {
        m_error = m_local->errorString();
        return false;
    }
    connect(m_local, &QLocalServer::newConnection, this, [this] {
        while (QLocalSocket *socket = m_local->nextPendingConnection()) {
            connect(socket, &QLocalSocket::disconnected, socket, &QObject::deleteLater);
            connect(socket, &QLocalSocket::readyRead, this, [this, socket] {
                while (socket->canReadLine()) {
                    const QByteArray text = socket->readLine().trimmed();
                    if (text.isEmpty())
                        continue;
                    QPointer<QLocalSocket> to(socket);
                    handle(text, [to](const QByteArray &answer) {
                        if (to && to->state() == QLocalSocket::ConnectedState) {
                            to->write(answer + '\n');
                            to->flush();
                        }
                    });
                }
            });
        }
    });
    return true;
}

}  // namespace qq
