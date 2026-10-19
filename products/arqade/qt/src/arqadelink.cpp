#include "arqadelink.h"

#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QTimer>
#include <QtEndian>

#include <cstdio>

namespace arqade {

namespace {
const QString Next = QStringLiteral("arqade.next");
const QString SignIn = QStringLiteral("arqade.signin");
const QString Web = QStringLiteral("arqade.web");
}  // namespace

ArqadeLink::ArqadeLink(QObject *parent) : QObject(parent)
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
    connect(&m_socket, &QLocalSocket::readyRead, this, &ArqadeLink::readFrames);
    connect(&m_socket, &QLocalSocket::disconnected, this, [this] {
        const bool was = m_welcomed;
        m_welcomed = false;
        emit changed();
        if (was)
            emit lost();
    });
}

QByteArray ArqadeLink::readToken()
{
    QFile in;
    if (!in.open(stdin, QIODevice::ReadOnly))
        return {};
    return in.readLine(128).trimmed();
}

void ArqadeLink::connectTo(const QString &channel, const QByteArray &token)
{
    m_token = token;
    m_socket.connectToServer(channel);
}

void ArqadeLink::send(const QJsonObject &frame)
{
    const QByteArray body = QJsonDocument(frame).toJson(QJsonDocument::Compact);
    if (body.size() > FrameLimit)
        return;
    QByteArray length(4, '\0');
    qToBigEndian(quint32(body.size()), length.data());
    m_socket.write(length + body);
    m_socket.flush();
}

void ArqadeLink::ask(const QString &method, const QJsonObject &params)
{
    m_waitingFor = m_nextId++;
    m_waitingMethod = method;
    send({{QStringLiteral("type"), QStringLiteral("request")},
          {QStringLiteral("id"), qint64(m_waitingFor)},
          {QStringLiteral("method"), method},
          {QStringLiteral("params"), params}});
}

void ArqadeLink::next()
{
    if (!connected())
        return;
    // A request waiting goes before the next wait for a command.
    if (!m_pending.isEmpty()) {
        const auto [method, params] = m_pending.takeFirst();
        ask(method, params);
        return;
    }
    ask(Next, {});
}

void ArqadeLink::queue(const QString &method, const QJsonObject &params)
{
    // A newer request of a method replaces one not yet sent.
    m_pending.removeIf([&method](const auto &held) { return held.first == method; });
    m_pending.append({method, params});
    if (connected() && m_waitingFor == 0)
        next();
}

QStringList ArqadeLink::waiting() const
{
    QStringList methods;
    for (const auto &held : m_pending)
        methods.append(held.first);
    return methods;
}

void ArqadeLink::signIn(const QUrl &authorizeUrl)
{
    if (!m_canSignIn) {
        emit signInFailed(QStringLiteral("unsupported_method"),
                          tr("This launcher cannot sign ARQADE in. Update the QOR Launcher."));
        return;
    }
    queue(SignIn, {{QStringLiteral("authorize_url"), authorizeUrl.toString(QUrl::FullyEncoded)}});
}

void ArqadeLink::openWeb()
{
    if (!m_canOpenWeb) {
        emit webFailed(tr("Open ARQADE from the QOR Launcher to play today's versions."));
        return;
    }
    queue(Web, {});
}

void ArqadeLink::readFrames()
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

void ArqadeLink::handle(const QJsonObject &frame)
{
    const QString type = frame.value(QStringLiteral("type")).toString();
    if (type == QLatin1String("welcome")) {
        const QJsonArray capabilities = frame.value(QStringLiteral("capabilities")).toArray();
        m_welcomed = capabilities.contains(Next);
        m_canSignIn = capabilities.contains(SignIn);
        m_canOpenWeb = capabilities.contains(Web);
        emit changed();
        next();
        return;
    }
    if (type == QLatin1String("error") && frame.value(QStringLiteral("id")).isNull())
        return;  // The handshake refused: the launcher closes the channel after this.
    if (quint64(frame.value(QStringLiteral("id")).toInteger()) != m_waitingFor)
        return;
    const QString method = m_waitingMethod;
    m_waitingFor = 0;
    m_waitingMethod.clear();

    if (type == QLatin1String("error")) {
        if (method == SignIn) {
            emit signInFailed(frame.value(QStringLiteral("code")).toString(),
                              frame.value(QStringLiteral("message")).toString());
            next();
        } else if (method == Web) {
            emit webFailed(frame.value(QStringLiteral("message")).toString());
            next();
        } else {
            // Refused: ask again shortly rather than spin.
            QTimer::singleShot(1000, this, &ArqadeLink::next);
        }
        return;
    }
    const QJsonObject result = frame.value(QStringLiteral("result")).toObject();
    if (method == SignIn) {
        const QUrl redirect(result.value(QStringLiteral("redirect")).toString(), QUrl::StrictMode);
        if (redirect.isValid() && !redirect.isRelative())
            emit signedIn(redirect);
        else
            emit signInFailed(QStringLiteral("bad_answer"), QStringLiteral("The launcher's answer had no address."));
    } else if (method == Next) {
        const QJsonValue command = result.value(QStringLiteral("command"));
        if (command.isObject())
            this->command(command.toObject());
    }
    next();
}

void ArqadeLink::command(const QJsonObject &c)
{
    const QString type = c.value(QStringLiteral("type")).toString();
    if (type == QLatin1String("place")) {
        const QJsonObject r = c.value(QStringLiteral("rect")).toObject();
        emit placeRequested(QRect(r.value(QStringLiteral("x")).toInt(), r.value(QStringLiteral("y")).toInt(),
                                  r.value(QStringLiteral("w")).toInt(), r.value(QStringLiteral("h")).toInt()),
                            c.value(QStringLiteral("parent")).toInteger());
    } else if (type == QLatin1String("hide")) {
        emit hideRequested();
    } else if (type == QLatin1String("theme")) {
        emit themeRequested(c.value(QStringLiteral("colors")).toObject().toVariantMap());
    } else if (type == QLatin1String("player")) {
        emit playerChanged(c.value(QStringLiteral("player")).toObject().toVariantMap());
    } else if (type == QLatin1String("prefs")) {
        emit prefsChanged(c.value(QStringLiteral("prefs")).toObject().toVariantMap());
    } else if (type == QLatin1String("quit")) {
        emit quitRequested();
    }
}

}  // namespace arqade
