#include "medialink.h"

#include <QFile>
#include <QJsonDocument>
#include <QTimer>
#include <QtEndian>

#include <cstdio>

namespace qormedia {

MediaLink::MediaLink(QObject *parent) : QObject(parent)
{
    connect(&m_socket, &QLocalSocket::connected, this, [this] {
        send({{QStringLiteral("type"), QStringLiteral("hello")},
              {QStringLiteral("protocol"), Protocol},
              {QStringLiteral("min_protocol"), Protocol},
              {QStringLiteral("product"), QString::fromLatin1(ProductId)},
              {QStringLiteral("version"), QStringLiteral("0.1.0")},
              {QStringLiteral("token"), QString::fromLatin1(m_token)}});
        // Said once; not kept.
        m_token.fill('\0');
        m_token.clear();
    });
    connect(&m_socket, &QLocalSocket::readyRead, this, &MediaLink::readFrames);
    connect(&m_socket, &QLocalSocket::disconnected, this, [this] {
        const bool was = m_welcomed;
        m_welcomed = false;
        emit changed();
        if (was)
            emit lost();
    });
}

QByteArray MediaLink::readToken()
{
    QFile in;
    if (!in.open(stdin, QIODevice::ReadOnly))
        return {};
    return in.readLine(128).trimmed();
}

void MediaLink::connectTo(const QString &channel, const QByteArray &token)
{
    m_token = token;
    m_socket.connectToServer(channel);
}

void MediaLink::send(const QJsonObject &frame)
{
    const QByteArray body = QJsonDocument(frame).toJson(QJsonDocument::Compact);
    if (body.size() > FrameLimit)
        return;
    QByteArray length(4, '\0');
    qToBigEndian(quint32(body.size()), length.data());
    m_socket.write(length + body);
    m_socket.flush();
}

void MediaLink::next()
{
    if (!connected())
        return;
    m_waitingFor = m_nextId++;
    const QJsonArray reports = m_reports;
    m_reports = {};
    send({{QStringLiteral("type"), QStringLiteral("request")},
          {QStringLiteral("id"), qint64(m_waitingFor)},
          {QStringLiteral("method"), QStringLiteral("media.next")},
          {QStringLiteral("params"), QJsonObject{{QStringLiteral("reports"), reports}}}});
}

void MediaLink::readFrames()
{
    m_buffer += m_socket.readAll();
    while (m_buffer.size() >= 4) {
        const quint32 length = qFromBigEndian<quint32>(m_buffer.constData());
        if (length > FrameLimit) {
            // A peer breaking the protocol: nothing more is read from it.
            m_socket.abort();
            return;
        }
        if (m_buffer.size() < 4 + qsizetype(length))
            return;
        const QByteArray body = m_buffer.mid(4, length);
        m_buffer.remove(0, 4 + length);
        handle(QJsonDocument::fromJson(body).object());
    }
}

void MediaLink::handle(const QJsonObject &frame)
{
    const QString type = frame.value(QStringLiteral("type")).toString();
    if (type == QLatin1String("welcome")) {
        const QJsonArray capabilities = frame.value(QStringLiteral("capabilities")).toArray();
        m_welcomed = capabilities.contains(QStringLiteral("media.next"));
        emit changed();
        next();
        return;
    }
    if (type == QLatin1String("error") && frame.value(QStringLiteral("id")).isNull())
        return;  // The handshake refused: the launcher closes the channel after this.
    if (quint64(frame.value(QStringLiteral("id")).toInteger()) != m_waitingFor)
        return;
    if (type == QLatin1String("error")) {
        // Refused (a report it could not read, say): ask again shortly rather than spin.
        QTimer::singleShot(1000, this, &MediaLink::next);
        return;
    }
    const QJsonValue command = frame.value(QStringLiteral("result")).toObject().value(QStringLiteral("command"));
    if (command.isObject())
        this->command(command.toObject());
    next();
}

void MediaLink::command(const QJsonObject &c)
{
    const QString type = c.value(QStringLiteral("type")).toString();
    const auto rectOf = [](const QJsonValue &v) {
        const QJsonObject r = v.toObject();
        return QRect(r.value(QStringLiteral("x")).toInt(), r.value(QStringLiteral("y")).toInt(),
                     r.value(QStringLiteral("w")).toInt(), r.value(QStringLiteral("h")).toInt());
    };
    const qint64 parent = c.value(QStringLiteral("parent")).toInteger();
    if (type == QLatin1String("open")) {
        if (c.value(QStringLiteral("rect")).isObject())
            emit placeRequested(rectOf(c.value(QStringLiteral("rect"))), parent);
        emit openRequested(c.value(QStringLiteral("path")).toString(), c.value(QStringLiteral("kind")).toString(),
                           c.value(QStringLiteral("position")).toDouble());
    } else if (type == QLatin1String("place")) {
        emit placeRequested(rectOf(c.value(QStringLiteral("rect"))), parent);
    } else if (type == QLatin1String("hide")) {
        emit hideRequested();
    } else if (type == QLatin1String("theme")) {
        emit themeRequested(c.value(QStringLiteral("colors")).toObject().toVariantMap());
    }
}

void MediaLink::report(const QJsonObject &report)
{
    // Only the newest position of a file matters; keep the queue short.
    if (report.value(QStringLiteral("type")).toString() == QLatin1String("position")) {
        for (qsizetype i = m_reports.size() - 1; i >= 0; --i) {
            const QJsonObject held = m_reports.at(i).toObject();
            if (held.value(QStringLiteral("type")) == report.value(QStringLiteral("type"))
                && held.value(QStringLiteral("path")) == report.value(QStringLiteral("path")))
                m_reports.removeAt(i);
        }
    }
    if (m_reports.size() < 64)
        m_reports.append(report);
}

void MediaLink::reportPosition(const QString &path, double seconds)
{
    if (path.isEmpty() || seconds < 0)
        return;
    report({{QStringLiteral("type"), QStringLiteral("position")},
            {QStringLiteral("path"), path},
            {QStringLiteral("position"), seconds}});
}

void MediaLink::reportLearned(const QString &path, double duration, int width, int height)
{
    if (path.isEmpty())
        return;
    QJsonObject r{{QStringLiteral("type"), QStringLiteral("learned")}, {QStringLiteral("path"), path}};
    if (duration > 0)
        r.insert(QStringLiteral("duration"), duration);
    if (width > 0 && height > 0) {
        r.insert(QStringLiteral("width"), width);
        r.insert(QStringLiteral("height"), height);
    }
    report(r);
}

void MediaLink::reportClosed()
{
    report({{QStringLiteral("type"), QStringLiteral("closed")}});
}

}  // namespace qormedia
