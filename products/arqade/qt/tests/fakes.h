// Stand-ins for the two things ARQADE talks to, for its tests: the QOR Launcher (a QLocalServer speaking the frames of the
// launcher's product host, host/session.rs) and ARQADE's server (a QTcpServer speaking just enough HTTP/1.1).

#pragma once

#include <QElapsedTimer>
#include <QHash>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalServer>
#include <QLocalSocket>
#include <QPointer>
#include <QTcpServer>
#include <QTcpSocket>
#include <QTest>
#include <QUrl>
#include <QUuid>
#include <QtEndian>

#include <functional>

namespace fakes {

class Launcher
{
public:
    Launcher() { server.listen(QStringLiteral("arqade-test-") + QUuid::createUuid().toString(QUuid::Id128)); }

    QString name() const { return server.serverName(); }

    bool accept()
    {
        if (!server.waitForNewConnection(5000))
            return false;
        peer = server.nextPendingConnection();
        return peer != nullptr;
    }

    void send(const QJsonObject &frame) { sendRaw(QJsonDocument(frame).toJson(QJsonDocument::Compact)); }

    void sendRaw(const QByteArray &body, quint32 claimed = 0)
    {
        QByteArray length(4, '\0');
        qToBigEndian(claimed ? claimed : quint32(body.size()), length.data());
        peer->write(length + body);
        peer->flush();
    }

    // The next frame from ARQADE, waiting for it while the event loop runs; empty after five seconds.
    QJsonObject read(int within = 5000)
    {
        QElapsedTimer waited;
        waited.start();
        while (waited.elapsed() < within) {
            buffer += peer->readAll();
            if (buffer.size() >= 4) {
                const quint32 length = qFromBigEndian<quint32>(buffer.constData());
                if (buffer.size() >= 4 + qsizetype(length)) {
                    const QByteArray body = buffer.mid(4, length);
                    buffer.remove(0, 4 + length);
                    return QJsonDocument::fromJson(body).object();
                }
            }
            QTest::qWait(10);
        }
        return {};
    }

    void welcome(const QJsonArray &capabilities = {QStringLiteral("arqade.next"), QStringLiteral("arqade.signin"),
                                                   QStringLiteral("arqade.web")})
    {
        send({{QStringLiteral("type"), QStringLiteral("welcome")},
              {QStringLiteral("protocol"), 1},
              {QStringLiteral("capabilities"), capabilities},
              {QStringLiteral("session"), QStringLiteral("00")}});
    }

    void result(const QJsonObject &request, const QJsonObject &result)
    {
        send({{QStringLiteral("type"), QStringLiteral("result")},
              {QStringLiteral("id"), request.value(QStringLiteral("id"))},
              {QStringLiteral("result"), result}});
    }

    void command(const QJsonObject &request, const QJsonValue &command)
    {
        result(request, {{QStringLiteral("command"), command}});
    }

    void refuse(const QJsonObject &request, const QString &code, const QString &message)
    {
        send({{QStringLiteral("type"), QStringLiteral("error")},
              {QStringLiteral("id"), request.value(QStringLiteral("id"))},
              {QStringLiteral("code"), code},
              {QStringLiteral("message"), message}});
    }

    QLocalServer server;
    QLocalSocket *peer = nullptr;
    QByteArray buffer;
};

// One HTTP request as the fake server read it.
struct Request
{
    QByteArray method;
    QUrl url;
    QHash<QByteArray, QByteArray> headers;
    /// The body, as long as Content-Length says: a post's JSON.
    QByteArray body;

    QJsonObject json() const { return QJsonDocument::fromJson(body).object(); }

    QByteArray cookie(const QByteArray &name) const
    {
        for (const QByteArray &pair : headers.value("cookie").split(';')) {
            const QByteArray trimmed = pair.trimmed();
            if (trimmed.startsWith(name + '='))
                return trimmed.mid(name.size() + 1);
        }
        return {};
    }
};

struct Response
{
    int status = 200;
    QList<QPair<QByteArray, QByteArray>> headers;
    QByteArray body;
};

class Http : public QObject
{
public:
    using Route = std::function<Response(const Request &)>;

    Http()
    {
        server.listen(QHostAddress::LocalHost);
        QObject::connect(&server, &QTcpServer::newConnection, this, [this] {
            while (QTcpSocket *socket = server.nextPendingConnection()) {
                auto *buffer = new QByteArray;
                QObject::connect(socket, &QTcpSocket::readyRead, socket, [this, socket, buffer] {
                    *buffer += socket->readAll();
                    const qsizetype end = buffer->indexOf("\r\n\r\n");
                    if (end < 0)
                        return;
                    // The body follows the head, as long as Content-Length says: wait for all of it.
                    qsizetype length = 0;
                    for (const QByteArray &line : buffer->left(end).split('\n')) {
                        const QByteArray trimmed = line.trimmed();
                        if (trimmed.toLower().startsWith("content-length:"))
                            length = trimmed.mid(15).trimmed().toLongLong();
                    }
                    if (buffer->size() < end + 4 + length)
                        return;
                    serve(socket, buffer->left(end), buffer->mid(end + 4, length));
                    buffer->clear();
                });
                QObject::connect(socket, &QTcpSocket::disconnected, socket, [socket, buffer] {
                    delete buffer;
                    socket->deleteLater();
                });
            }
        });
    }

    QUrl base() const { return QUrl(QStringLiteral("http://127.0.0.1:%1/").arg(server.serverPort())); }

    QTcpServer server;
    Route route;
    QList<Request> seen;

private:
    void serve(QTcpSocket *socket, const QByteArray &head, const QByteArray &body)
    {
        const QList<QByteArray> lines = head.split('\n');
        const QList<QByteArray> first = lines.value(0).trimmed().split(' ');
        Request request;
        request.method = first.value(0);
        request.url = base().resolved(QUrl(QString::fromLatin1(first.value(1))));
        for (qsizetype i = 1; i < lines.size(); ++i) {
            const QByteArray line = lines.at(i).trimmed();
            const qsizetype colon = line.indexOf(':');
            if (colon > 0)
                request.headers.insert(line.left(colon).trimmed().toLower(), line.mid(colon + 1).trimmed());
        }
        request.body = body;
        seen.append(request);
        const Response response = route ? route(request) : Response{404, {}, {}};
        QByteArray out = "HTTP/1.1 " + QByteArray::number(response.status) + " Fake\r\n";
        for (const auto &[name, value] : response.headers)
            out += name + ": " + value + "\r\n";
        out += "Content-Length: " + QByteArray::number(response.body.size()) + "\r\nConnection: close\r\n\r\n";
        out += response.body;
        socket->write(out);
        socket->disconnectFromHost();
    }
};

}  // namespace fakes
