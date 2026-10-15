#include "launcherlink.h"

#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QtEndian>

#include <cstdio>

namespace qq {

LauncherLink::LauncherLink(QObject *parent) : QObject(parent)
{
    m_reason = tr("Connecting to the launcher…");
    connect(&m_socket, &QLocalSocket::connected, this, [this] {
        send({{QStringLiteral("type"), QStringLiteral("hello")},
              {QStringLiteral("protocol"), Protocol},
              {QStringLiteral("min_protocol"), Protocol},
              {QStringLiteral("product"), QString::fromLatin1(ProductId)},
              {QStringLiteral("version"), QStringLiteral("1.0")},
              {QStringLiteral("token"), QString::fromLatin1(m_token)}});
        // Said once; not kept.
        m_token.fill('\0');
        m_token.clear();
    });
    connect(&m_socket, &QLocalSocket::readyRead, this, &LauncherLink::readFrames);
    connect(&m_socket, &QLocalSocket::errorOccurred, this, [this](QLocalSocket::LocalSocketError) {
        if (!m_welcomed) {
            m_reason = tr("The launcher's channel could not be reached: %1").arg(m_socket.errorString());
            emit changed();
        }
    });
    connect(&m_socket, &QLocalSocket::disconnected, this, [this] {
        if (m_step != Step::Idle)
            fail(QStringLiteral("closed"), tr("The launcher closed the channel before answering."));
        const bool wasWelcomed = m_welcomed;
        m_welcomed = false;
        if (m_reason.isEmpty())
            m_reason = tr("The launcher closed the channel.");
        emit changed();
        if (wasWelcomed)
            emit lost();
    });
}

QByteArray LauncherLink::readToken()
{
    QFile in;
    if (!in.open(stdin, QIODevice::ReadOnly))
        return {};
    return in.readLine(128).trimmed();
}

void LauncherLink::connectTo(const QString &channel, const QByteArray &token)
{
    m_token = token;
    m_socket.connectToServer(channel);
}

void LauncherLink::unavailable(const QString &reason)
{
    m_reason = reason;
    emit changed();
}

void LauncherLink::send(const QJsonObject &frame)
{
    const QByteArray body = QJsonDocument(frame).toJson(QJsonDocument::Compact);
    if (body.size() > FrameLimit)
        return;
    QByteArray length(4, '\0');
    qToBigEndian(quint32(body.size()), length.data());
    m_socket.write(length + body);
    m_socket.flush();
}

void LauncherLink::request(const QString &method, const QJsonObject &params)
{
    m_waitingFor = m_nextId++;
    send({{QStringLiteral("type"), QStringLiteral("request")},
          {QStringLiteral("id"), qint64(m_waitingFor)},
          {QStringLiteral("method"), method},
          {QStringLiteral("params"), params}});
}

void LauncherLink::readFrames()
{
    m_buffer += m_socket.readAll();
    while (m_buffer.size() >= 4) {
        const quint32 length = qFromBigEndian<quint32>(m_buffer.constData());
        if (length > FrameLimit) {
            // A peer breaking the protocol: nothing more is read from it.
            m_reason = tr("The launcher sent a frame over the limit.");
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

void LauncherLink::handle(const QJsonObject &frame)
{
    const QString type = frame.value(QStringLiteral("type")).toString();
    if (type == QLatin1String("welcome")) {
        m_welcomed = true;
        m_reason.clear();
        m_capabilities.clear();
        for (const QJsonValue &c : frame.value(QStringLiteral("capabilities")).toArray())
            m_capabilities << c.toString();
        if (!m_capabilities.contains(QStringLiteral("cartridge.publish")))
            m_reason = tr("This launcher does not offer publishing.");
        emit changed();
        return;
    }
    if (type == QLatin1String("error") && frame.value(QStringLiteral("id")).isNull()) {
        // The handshake refused: the launcher closes the channel after this.
        m_reason = tr("The launcher refused this Studio: %1").arg(frame.value(QStringLiteral("message")).toString());
        emit changed();
        return;
    }
    if (m_step == Step::Idle || quint64(frame.value(QStringLiteral("id")).toInteger()) != m_waitingFor)
        return;
    if (type == QLatin1String("error")) {
        fail(frame.value(QStringLiteral("code")).toString(), frame.value(QStringLiteral("message")).toString());
        return;
    }
    const QJsonObject result = frame.value(QStringLiteral("result")).toObject();
    if (m_step == Step::Committing) {
        m_step = Step::Publishing;
        request(QStringLiteral("cartridge.publish"), {{QStringLiteral("project"), m_project}});
        return;
    }
    finish();
    emit published(result.toVariantMap());
}

void LauncherLink::publish(const QUrl &project, const QString &message)
{
    if (!available() || busy())
        return;
    m_project = QDir::toNativeSeparators(project.isLocalFile() ? project.toLocalFile() : project.toString());
    m_step = Step::Committing;
    emit changed();
    request(QStringLiteral("qontrol.commit"),
            {{QStringLiteral("project"), m_project}, {QStringLiteral("message"), message}});
}

void LauncherLink::finish()
{
    m_step = Step::Idle;
    m_waitingFor = 0;
    emit changed();
}

void LauncherLink::fail(const QString &code, const QString &message)
{
    finish();
    emit failed(code, message);
}

}  // namespace qq
